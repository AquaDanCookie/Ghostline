use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::model::{Protocol, Stats};

#[derive(Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct ImportedConnection {
    pub uuid: String,
}

pub fn nmcli_version() -> String {
    let output = Command::new("nmcli").arg("--version").output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }
        _ => String::new(),
    }
}

pub fn available() -> bool {
    Command::new("nmcli")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn import_profile(
    path: &Path,
    protocol: Protocol,
    display_name: &str,
) -> Result<ImportedConnection, String> {
    let output = run_nmcli(&[
        "connection",
        "import",
        "type",
        protocol.nm_type(),
        "file",
        &path.to_string_lossy(),
    ])?;
    if !output.status.success() {
        return Err(friendly_error(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let uuid = find_uuid(&stdout)
        .or_else(|| find_uuid(&stderr))
        .ok_or_else(|| {
            "NetworkManager imported the file but did not report a connection UUID.".to_owned()
        })?;

    let connection_name = format!("Ghostline: {display_name}");
    let _ = run_nmcli(&[
        "connection",
        "modify",
        "uuid",
        &uuid,
        "connection.id",
        &connection_name,
    ]);

    Ok(ImportedConnection { uuid })
}

pub fn delete_connection(uuid: &str) -> Result<(), String> {
    let output = run_nmcli(&["connection", "delete", "uuid", uuid])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(friendly_error(&output.stderr))
    }
}

pub fn activate(
    uuid: &str,
    credentials: Option<&Credentials>,
    runtime_dir: &Path,
) -> Result<(), String> {
    let mut command = Command::new("nmcli");
    command.args(["--wait", "45", "connection", "up", "uuid", uuid]);
    let mut password_file: Option<PathBuf> = None;

    if let Some(credentials) = credentials {
        if credentials.password.contains('\n') || credentials.password.contains('\r') {
            return Err(
                "The password contains a line break, which NetworkManager cannot accept."
                    .to_owned(),
            );
        }
        if !credentials.username.trim().is_empty() {
            let _ = run_nmcli(&[
                "connection",
                "modify",
                "uuid",
                uuid,
                "+vpn.data",
                &format!("username={}", credentials.username.trim()),
            ]);
        }
        std::fs::create_dir_all(runtime_dir)
            .map_err(|error| format!("Cannot create a private runtime directory: {error}"))?;
        let _ = std::fs::set_permissions(runtime_dir, std::fs::Permissions::from_mode(0o700));
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0);
        let path = runtime_dir.join(format!("passwd-{}-{stamp}.tmp", std::process::id()));
        {
            let mut file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&path)
                .map_err(|error| format!("Cannot create a temporary credentials file: {error}"))?;
            writeln!(file, "vpn.secrets.password:{}", credentials.password)
                .map_err(|error| format!("Cannot write temporary credentials: {error}"))?;
        }
        command.arg("passwd-file").arg(&path);
        password_file = Some(path);
    }

    let output = command
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("Could not run nmcli: {error}"));
    if let Some(path) = password_file {
        let _ = std::fs::remove_file(path);
    }
    let output = output?;

    if output.status.success() {
        Ok(())
    } else {
        Err(friendly_error(&output.stderr))
    }
}

pub fn deactivate(uuid: &str) -> Result<(), String> {
    let output = run_nmcli(&["connection", "down", "uuid", uuid])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(friendly_error(&output.stderr))
    }
}

pub fn active_connections() -> Result<Vec<(String, String, String)>, String> {
    let output = run_nmcli(&[
        "-t",
        "-f",
        "UUID,NAME,TYPE,DEVICE",
        "connection",
        "show",
        "--active",
    ])?;
    if !output.status.success() {
        return Err(friendly_error(&output.stderr));
    }
    let mut result = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let fields = split_terse(line);
        if fields.len() >= 4 {
            let uuid = fields[0].trim().to_owned();
            let name = fields[1].trim().to_owned();
            let kind = fields[2].trim().to_owned();
            let device = fields[3].trim().to_owned();
            if !uuid.is_empty() {
                result.push((uuid, format!("{name} ({kind})"), device));
            }
        }
    }
    Ok(result)
}

pub fn connection_uuids() -> Result<Vec<(String, String)>, String> {
    let output = run_nmcli(&["-t", "-f", "UUID,TYPE", "connection", "show"])?;
    if !output.status.success() {
        return Err(friendly_error(&output.stderr));
    }
    let mut result = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let fields = split_terse(line);
        if fields.len() >= 2 {
            let uuid = fields[0].trim().to_owned();
            let kind = fields[1].trim().to_owned();
            if kind == "vpn" || kind == "wireguard" {
                result.push((uuid, kind));
            }
        }
    }
    Ok(result)
}

pub fn vpn_interface() -> Option<String> {
    let root = Path::new("/sys/class/net");
    let entries = std::fs::read_dir(root).ok()?;
    let mut candidates: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            name.starts_with("cyberghost")
                || name.starts_with("tun")
                || name.starts_with("tap")
                || name.starts_with("wg")
                || name.starts_with("vpn")
        })
        .collect();
    candidates.sort_by_key(|name| {
        std::fs::read_to_string(root.join(name).join("operstate"))
            .map(|state| state.trim() != "up")
            .unwrap_or(true)
    });
    candidates.into_iter().next()
}

pub fn stats_for(interface: &str) -> Option<Stats> {
    let base = Path::new("/sys/class/net")
        .join(interface)
        .join("statistics");
    let rx = read_counter(&base.join("rx_bytes"))?;
    let tx = read_counter(&base.join("tx_bytes"))?;
    Some(Stats {
        rx_total: rx,
        tx_total: tx,
        rx_rate: 0.0,
        tx_rate: 0.0,
    })
}

fn read_counter(path: &Path) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

pub fn measure_latency(host: &str, port: u16) -> Result<u32, String> {
    if host.is_empty() || port == 0 {
        return Err("This profile has no usable endpoint.".to_owned());
    }
    let addresses: Vec<_> = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("Could not resolve {host}: {error}"))?
        .collect();
    if addresses.is_empty() {
        return Err(format!("{host} did not resolve to an address."));
    }

    let mut samples = Vec::new();
    let mut last_error = String::new();
    for _ in 0..3 {
        let mut connected = false;
        for address in &addresses {
            let start = Instant::now();
            match TcpStream::connect_timeout(address, Duration::from_millis(1500)) {
                Ok(_) => {
                    samples.push(start.elapsed().as_millis() as u32);
                    connected = true;
                    break;
                }
                Err(error) => {
                    last_error = error.to_string();
                }
            }
        }
        if !connected {
            break;
        }
    }
    if samples.is_empty() {
        return Err(format!("No TCP handshake completed. {last_error}"));
    }
    samples.sort_unstable();
    Ok(samples[samples.len() / 2])
}

pub fn check_exit_ip() -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent("Ghostline/0.1 exit-ip-check")
        .build();
    let response = agent
        .get("https://api.ipify.org?format=json")
        .call()
        .map_err(|error| format!("The check did not complete: {error}"))?;
    let mut body = String::new();
    response
        .into_reader()
        .take(4096)
        .read_to_string(&mut body)
        .map_err(|error| format!("Could not read the response: {error}"))?;
    let value: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| "The check returned an unexpected response.".to_owned())?;
    value
        .get("ip")
        .and_then(|ip| ip.as_str())
        .map(|ip| ip.trim().to_owned())
        .filter(|ip| !ip.is_empty())
        .ok_or_else(|| "The check returned no address.".to_owned())
}

pub fn dns_info() -> crate::model::DnsInfo {
    let mut servers: Vec<String> = Vec::new();
    if let Ok(raw) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in raw.lines() {
            let line = line.trim();
            if let Some(server) = line.strip_prefix("nameserver ") {
                let server = server.trim();
                if !server.is_empty() && !servers.iter().any(|value| value.as_str() == server) {
                    servers.push(server.to_owned());
                }
            }
        }
    }

    if let Ok(output) = Command::new("resolvectl").arg("status").output() {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let line = line.trim();
                if let Some(value) = line.strip_prefix("Current DNS Server:") {
                    let value = value.trim();
                    if !value.is_empty() && !servers.iter().any(|server| server.as_str() == value) {
                        servers.insert(0, value.to_owned());
                    }
                } else if let Some(value) = line.strip_prefix("DNS Servers:") {
                    for server in value.split_whitespace() {
                        if !servers.iter().any(|existing| existing.as_str() == server) {
                            servers.push(server.to_owned());
                        }
                    }
                }
            }
        }
    }

    let interface = default_interface().unwrap_or_else(|| "Unknown".to_owned());
    crate::model::DnsInfo {
        servers,
        interface,
        nm_version: nmcli_version(),
    }
}

fn default_interface() -> Option<String> {
    let output = Command::new("ip")
        .args(["route", "get", "1.1.1.1"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut words = text.split_whitespace();
    while let Some(word) = words.next() {
        if word == "dev" {
            return words.next().map(|value| value.to_owned());
        }
    }
    None
}

pub fn flush_dns() -> Result<(), String> {
    for (program, args) in [
        ("resolvectl", vec!["flush-caches"]),
        ("systemd-resolve", vec!["--flush-caches"]),
    ] {
        let output = Command::new(program).args(args).output();
        if let Ok(output) = output {
            if output.status.success() {
                return Ok(());
            }
        }
    }
    Err(
        "Could not flush the DNS cache. The system resolver may not be systemd-resolved."
            .to_owned(),
    )
}

fn run_nmcli(args: &[&str]) -> Result<Output, String> {
    Command::new("nmcli")
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("Could not run nmcli: {error}"))
}

fn split_terse(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == ':' {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(character);
        }
    }
    fields.push(current);
    fields
}

pub fn find_uuid(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    if bytes.len() < 36 {
        return None;
    }
    for start in 0..=bytes.len() - 36 {
        let candidate = &bytes[start..start + 36];
        let valid = candidate.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        });
        let boundary_before = start == 0 || !bytes[start - 1].is_ascii_hexdigit();
        let boundary_after = start + 36 == bytes.len() || !bytes[start + 36].is_ascii_hexdigit();
        if valid && boundary_before && boundary_after {
            return Some(String::from_utf8_lossy(candidate).into_owned());
        }
    }
    None
}

fn friendly_error(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    let text = text.strip_prefix("Error: ").unwrap_or(text);
    if text.is_empty() {
        return "NetworkManager returned an error without a message.".to_owned();
    }
    let lowered = text.to_lowercase();
    if lowered.contains("secrets were required") || lowered.contains("no valid vpn secrets") {
        return "NetworkManager needs credentials for this profile. Connect again and enter the username and password.".to_owned();
    }
    if lowered.contains("could not find vpn plugin") || lowered.contains("vpn plugin") {
        return format!("The NetworkManager VPN plugin is missing. {text}");
    }
    if lowered.contains("not authorized") || lowered.contains("permission") {
        return format!("NetworkManager refused the request. Check your polkit policy. {text}");
    }
    text.replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_uuid_in_nmcli_output() {
        let text = "Connection 'test' (48da4c1b-0bdb-4a6c-8b22-58532b5c667d) successfully added.";
        assert_eq!(
            find_uuid(text).as_deref(),
            Some("48da4c1b-0bdb-4a6c-8b22-58532b5c667d")
        );
    }

    #[test]
    fn does_not_match_uuid_prefix() {
        assert!(find_uuid("48da4c1b-0bdb-4a6c-8b22-58532b5c667dff").is_none());
    }

    #[test]
    fn splits_escaped_terse_fields() {
        let fields = split_terse("uuid:name\\:with\\:colons:vpn:tun0");
        assert_eq!(fields[0], "uuid");
        assert_eq!(fields[1], "name:with:colons");
        assert_eq!(fields[2], "vpn");
    }
}
