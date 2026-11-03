use std::path::Path;

use crate::countries;
use crate::model::Protocol;

const MAX_PROFILE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct ParsedProfile {
    pub name: String,
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub protocol: Protocol,
    pub remote_host: String,
    pub remote_port: u16,
    pub requires_auth: bool,
    pub warnings: Vec<String>,
}

pub fn parse_path(path: &Path) -> Result<ParsedProfile, String> {
    let metadata = std::fs::metadata(path).map_err(|error| format!("Cannot read file: {error}"))?;
    if !metadata.is_file() {
        return Err("That path is not a file.".to_owned());
    }
    if metadata.len() > MAX_PROFILE_BYTES {
        return Err("The profile is larger than 2 MB.".to_owned());
    }
    let content =
        std::fs::read_to_string(path).map_err(|error| format!("Cannot read file: {error}"))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Profile");
    parse_text(stem, &content, &extension)
}

pub fn parse_text(stem: &str, content: &str, extension: &str) -> Result<ParsedProfile, String> {
    let content = content.trim_start_matches('\u{feff}');
    let looks_wireguard = content.contains("[Interface]") && content.contains("[Peer]");
    let wireguard = extension.eq_ignore_ascii_case("conf") || looks_wireguard;

    if wireguard {
        parse_wireguard(stem, content)
    } else {
        parse_openvpn(stem, content)
    }
}

fn parse_openvpn(stem: &str, content: &str) -> Result<ParsedProfile, String> {
    let mut remote: Option<(String, u16)> = None;
    let mut extra_remotes = 0usize;
    let mut requires_auth = false;
    let mut warnings = Vec::new();
    let mut in_block = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('<') && !line.starts_with("</") {
            in_block = true;
            continue;
        }
        if line.starts_with("</") {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }

        let mut parts = line.split_whitespace();
        let directive = parts.next().unwrap_or_default().to_ascii_lowercase();
        match directive.as_str() {
            "remote" => {
                let Some(host) = parts.next() else {
                    continue;
                };
                if host.starts_with('$') {
                    continue;
                }
                let port = parts
                    .next()
                    .and_then(|value| value.parse::<u16>().ok())
                    .unwrap_or(1194);
                if remote.is_none() {
                    remote = Some((host.to_owned(), port));
                } else {
                    extra_remotes += 1;
                }
            }
            "auth-user-pass" => {
                requires_auth = true;
            }
            _ => {}
        }
    }

    let Some((host, port)) = remote else {
        return Err("No remote server was declared in this OpenVPN profile.".to_owned());
    };
    if extra_remotes > 0 {
        warnings.push(format!(
            "{extra_remotes} additional remote entries ignored; Ghostline uses the first one."
        ));
    }
    let (country_code, country, city) = location_parts(stem);
    let name = display_name(stem, city.as_deref());

    Ok(ParsedProfile {
        name,
        country_code,
        country,
        city,
        protocol: Protocol::OpenVpn,
        remote_host: host,
        remote_port: port,
        requires_auth,
        warnings,
    })
}

fn parse_wireguard(stem: &str, content: &str) -> Result<ParsedProfile, String> {
    let mut endpoint: Option<(String, u16)> = None;
    let mut has_private_key = false;
    let mut warnings = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        match key.as_str() {
            "privatekey" => has_private_key = true,
            "endpoint" => {
                if endpoint.is_none() {
                    endpoint = parse_endpoint(value);
                }
            }
            _ => {}
        }
    }

    if !has_private_key {
        return Err(
            "This WireGuard profile has no PrivateKey in its Interface section.".to_owned(),
        );
    }
    let Some((host, port)) = endpoint else {
        return Err("No Endpoint was declared in this WireGuard profile.".to_owned());
    };
    if host.is_empty() {
        return Err("The WireGuard endpoint is empty.".to_owned());
    }
    if port == 0 {
        warnings.push("The endpoint has no port; 51820 is shown instead.".to_owned());
    }

    let (country_code, country, city) = location_parts(stem);
    let name = display_name(stem, city.as_deref());

    Ok(ParsedProfile {
        name,
        country_code,
        country,
        city,
        protocol: Protocol::WireGuard,
        remote_host: host,
        remote_port: if port == 0 { 51820 } else { port },
        requires_auth: false,
        warnings,
    })
}

fn parse_endpoint(value: &str) -> Option<(String, u16)> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(rest) = value.strip_prefix('[') {
        let (host, tail) = rest.split_once(']')?;
        let port = tail
            .trim_start_matches(':')
            .parse::<u16>()
            .ok()
            .unwrap_or(0);
        return Some((host.to_owned(), port));
    }
    if let Some((host, port)) = value.rsplit_once(':') {
        if port.chars().all(|c| c.is_ascii_digit()) {
            return Some((host.to_owned(), port.parse::<u16>().unwrap_or(0)));
        }
    }
    Some((value.to_owned(), 0))
}

fn location_parts(stem: &str) -> (Option<String>, Option<String>, Option<String>) {
    let (code, name) = match countries::find_in_text(stem) {
        Some((code, name)) => (Some(code.to_owned()), Some(name.to_owned())),
        None => (None, None),
    };

    let mut tokens: Vec<String> = stem
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_owned())
        .collect();

    tokens.retain(|token| {
        let lower = token.to_ascii_lowercase();
        !matches!(
            lower.as_str(),
            "cg" | "cyberghost"
                | "ghostline"
                | "vpn"
                | "openvpn"
                | "wireguard"
                | "config"
                | "profile"
                | "client"
        )
    });

    if let Some(code) = &code {
        tokens.retain(|token| !token.eq_ignore_ascii_case(code));
    }
    if let Some(country) = &name {
        let country_words: Vec<String> = country
            .split_whitespace()
            .map(|word| word.to_ascii_lowercase())
            .collect();
        tokens.retain(|token| {
            !country_words
                .iter()
                .any(|word| token.eq_ignore_ascii_case(word))
        });
    }

    let trailing_number = tokens
        .iter()
        .rev()
        .find(|token| token.chars().all(|c| c.is_ascii_digit()))
        .cloned();
    tokens.retain(|token| !token.chars().all(|c| c.is_ascii_digit()));

    let city = if tokens.is_empty() {
        None
    } else {
        Some(title_case(&tokens.join(" ")))
    };
    let _ = trailing_number;

    (code, name, city)
}

fn display_name(stem: &str, city: Option<&str>) -> String {
    let trailing_number = stem
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .rev()
        .find(|token| token.chars().all(|c| c.is_ascii_digit()))
        .map(|token| token.to_owned());

    if let Some(city) = city {
        if let Some(number) = trailing_number {
            return format!("{city} {number}");
        }
        return city.to_owned();
    }

    let cleaned: Vec<&str> = stem
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .filter(|token| {
            !matches!(
                token.to_ascii_lowercase().as_str(),
                "cg" | "cyberghost"
                    | "ghostline"
                    | "vpn"
                    | "openvpn"
                    | "wireguard"
                    | "config"
                    | "profile"
            )
        })
        .collect();
    if cleaned.is_empty() {
        "Imported profile".to_owned()
    } else {
        title_case(&cleaned.join(" "))
    }
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let rest: String = chars.collect();
                    format!("{}{}", first.to_uppercase(), rest.to_lowercase())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPENVPN: &str = r#"
client
dev tun
proto udp
remote de-berlin-01.example.net 1194
auth-user-pass
nobind
persist-key
persist-tun
<ca>
-----BEGIN CERTIFICATE-----
not a real certificate
-----END CERTIFICATE-----
</ca>
"#;

    const WIREGUARD: &str = r#"
[Interface]
PrivateKey = aaaa
Address = 10.0.0.2/32
DNS = 10.0.0.1

[Peer]
PublicKey = bbbb
AllowedIPs = 0.0.0.0/0, ::/0
Endpoint = nl-ams-01.example.net:51820
"#;

    #[test]
    fn parses_openvpn_profile() {
        let parsed = parse_text("CG_DE_Berlin_01", OPENVPN, "ovpn").unwrap();
        assert_eq!(parsed.protocol, Protocol::OpenVpn);
        assert_eq!(parsed.remote_host, "de-berlin-01.example.net");
        assert_eq!(parsed.remote_port, 1194);
        assert!(parsed.requires_auth);
        assert_eq!(parsed.country_code.as_deref(), Some("DE"));
        assert_eq!(parsed.country.as_deref(), Some("Germany"));
        assert_eq!(parsed.city.as_deref(), Some("Berlin"));
        assert_eq!(parsed.name, "Berlin 01");
    }

    #[test]
    fn parses_wireguard_profile() {
        let parsed = parse_text("NL_Amsterdam_02", WIREGUARD, "conf").unwrap();
        assert_eq!(parsed.protocol, Protocol::WireGuard);
        assert_eq!(parsed.remote_host, "nl-ams-01.example.net");
        assert_eq!(parsed.remote_port, 51820);
        assert!(!parsed.requires_auth);
        assert_eq!(parsed.country_code.as_deref(), Some("NL"));
        assert_eq!(parsed.city.as_deref(), Some("Amsterdam"));
        assert_eq!(parsed.name, "Amsterdam 02");
    }

    #[test]
    fn rejects_missing_remote() {
        let error = parse_text("empty", "client\ndev tun\n", "ovpn").unwrap_err();
        assert!(error.contains("remote"));
    }

    #[test]
    fn ignores_directives_inside_inline_blocks() {
        let content =
            "client\nremote real.example.net 443\n<ca>\nremote fake.example.net 1\n</ca>\n";
        let parsed = parse_text("Test", content, "ovpn").unwrap();
        assert_eq!(parsed.remote_host, "real.example.net");
        assert_eq!(parsed.remote_port, 443);
    }

    #[test]
    fn handles_ipv6_wireguard_endpoint() {
        let content = WIREGUARD.replace("nl-ams-01.example.net:51820", "[2001:db8::1]:51820");
        let parsed = parse_text("Test", &content, "conf").unwrap();
        assert_eq!(parsed.remote_host, "2001:db8::1");
        assert_eq!(parsed.remote_port, 51820);
    }
}
