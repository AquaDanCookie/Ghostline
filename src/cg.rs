use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::countries;
use crate::model::{NetworkProtocol, Profile, Protocol, ServerType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Country {
    pub id: u32,
    pub name: String,
    pub code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct City {
    pub id: u32,
    pub name: String,
    pub instances: String,
    pub load: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    pub id: u32,
    pub name: String,
    pub instance: String,
    pub load: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamingService {
    pub id: u32,
    pub service: String,
    pub country_code: String,
}

pub fn available() -> bool {
    executable().is_some()
}

pub fn executable() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join("cyberghostvpn");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let fallback = Path::new("/usr/bin/cyberghostvpn");
    fallback.is_file().then(|| fallback.to_path_buf())
}

pub fn fallback_countries() -> Vec<Country> {
    let mut list: Vec<Country> = countries::all()
        .iter()
        .enumerate()
        .map(|(index, (code, name))| Country {
            id: index as u32 + 1,
            name: (*name).to_owned(),
            code: (*code).to_owned(),
        })
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

pub fn list_countries(server_type: ServerType) -> Result<Vec<Country>, String> {
    // Ask the CLI for the selected catalogue so the country list reflects
    // where that server type can actually connect.
    let flag = if server_type == ServerType::Torrent {
        "--torrent"
    } else {
        "--traffic"
    };
    let output = run_public(&[flag.to_owned(), "--country-code".to_owned()])?;
    parse_countries(&output)
}

pub fn list_streaming_services(country_code: &str) -> Result<Vec<StreamingService>, String> {
    let output = run_public(&[
        "--streaming".to_owned(),
        "--country-code".to_owned(),
        country_code.to_owned(),
    ])?;
    parse_streaming_services(&output)
}

pub fn list_cities(
    server_type: ServerType,
    country_code: &str,
    streaming_service: Option<&str>,
) -> Result<Vec<City>, String> {
    let mut args = Vec::new();
    if server_type == ServerType::Streaming {
        let service = streaming_service
            .filter(|service| !service.is_empty())
            .ok_or_else(|| "Choose a streaming service first.".to_owned())?;
        args.push("--streaming".to_owned());
        args.push(service.to_owned());
    } else {
        args.push(server_flag(server_type).to_owned());
    }
    args.push("--country-code".to_owned());
    args.push(country_code.to_owned());
    let output = run_public(&args)?;
    parse_cities(&output)
}

pub fn list_servers(
    server_type: ServerType,
    country_code: &str,
    city: &str,
    streaming_service: Option<&str>,
) -> Result<Vec<Server>, String> {
    let mut args = Vec::new();
    if server_type == ServerType::Streaming {
        let service = streaming_service
            .filter(|service| !service.is_empty())
            .ok_or_else(|| "Choose a streaming service first.".to_owned())?;
        args.push("--streaming".to_owned());
        args.push(service.to_owned());
    } else {
        args.push(server_flag(server_type).to_owned());
    }
    args.push("--country-code".to_owned());
    args.push(country_code.to_owned());
    args.push("--city".to_owned());
    args.push(city.to_owned());
    let output = run_public(&args)?;
    parse_servers(&output)
}

pub fn connect(profile: &Profile, sudo_password: Option<&str>) -> Result<(), String> {
    let args = connect_args(profile)?;
    run_privileged(&args, sudo_password)
}

pub fn stop(sudo_password: Option<&str>) -> Result<(), String> {
    run_privileged(&["--stop".to_owned()], sudo_password)
}

pub fn status() -> Result<bool, String> {
    if Path::new("/sys/class/net/cyberghost").exists() {
        return Ok(true);
    }
    if !available() {
        return Err("cyberghostvpn was not found.".to_owned());
    }
    let output = run_public(&["--status".to_owned()])?;
    let text = combined(&output);
    if let Some(active) = parse_status(&text) {
        return Ok(active);
    }
    if text.trim().is_empty() {
        return Err("The CyberGhost CLI returned no status.".to_owned());
    }
    Err(first_error(&text).unwrap_or_else(|| text.trim().to_owned()))
}

fn parse_status(text: &str) -> Option<bool> {
    let text = text.to_lowercase();
    if text.contains("no vpn connection")
        || text.contains("no wireguard connection")
        || text.contains("not connected")
    {
        return Some(false);
    }
    if text.contains("vpn connection found") || text.contains("wireguard connection found") {
        return Some(true);
    }
    None
}

pub fn sudo_ready() -> bool {
    if !available() {
        return false;
    }
    Command::new("sudo")
        .args(["-n", "true"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn connect_args(profile: &Profile) -> Result<Vec<String>, String> {
    if !profile.is_cyberghost() {
        return Err("This is not a CyberGhost network location.".to_owned());
    }
    let country_code = profile
        .country_code
        .as_deref()
        .filter(|code| !code.is_empty())
        .ok_or_else(|| "Choose a country first.".to_owned())?;

    let mut args = Vec::new();
    match profile.server_type {
        ServerType::Traffic => args.push("--traffic".to_owned()),
        ServerType::Torrent => args.push("--torrent".to_owned()),
        ServerType::Streaming => {
            let service = profile
                .streaming_service
                .as_deref()
                .filter(|service| !service.is_empty())
                .ok_or_else(|| "Choose a streaming service first.".to_owned())?;
            args.push("--streaming".to_owned());
            args.push(service.to_owned());
        }
    }
    args.push("--country-code".to_owned());
    args.push(country_code.to_owned());
    if let Some(city) = profile.city.as_deref().filter(|city| !city.is_empty()) {
        args.push("--city".to_owned());
        args.push(city.to_owned());
    }
    if let Some(server) = profile
        .server_instance
        .as_deref()
        .filter(|server| !server.is_empty())
    {
        args.push("--server".to_owned());
        args.push(server.to_owned());
    }
    if profile.protocol == Protocol::OpenVpn {
        args.push("--connection".to_owned());
        args.push(profile.network_protocol.cli_value().to_owned());
        args.push("--openvpn".to_owned());
    } else {
        args.push("--wireguard".to_owned());
    }
    args.push("--connect".to_owned());
    Ok(args)
}

pub fn profile_id(
    server_type: ServerType,
    country_code: &str,
    city: Option<&str>,
    server: Option<&str>,
    protocol: Protocol,
    network_protocol: NetworkProtocol,
    streaming_service: Option<&str>,
) -> String {
    let label = format!(
        "{:?}|{}|{}|{}|{}|{}|{}",
        server_type,
        country_code,
        city.unwrap_or_default(),
        server.unwrap_or_default(),
        protocol.label(),
        network_protocol.label(),
        streaming_service.unwrap_or_default()
    );
    let mut hash = 0xcbf29ce484222325u64;
    for byte in label.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("cg_{hash:016x}")
}

pub fn server_flag(server_type: ServerType) -> &'static str {
    match server_type {
        ServerType::Traffic => "--traffic",
        ServerType::Streaming => "--streaming",
        ServerType::Torrent => "--torrent",
    }
}

fn run_public(args: &[String]) -> Result<Output, String> {
    let executable = executable().ok_or_else(|| "cyberghostvpn was not found.".to_owned())?;
    Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Could not run cyberghostvpn: {error}"))
}

fn run_privileged(args: &[String], password: Option<&str>) -> Result<(), String> {
    let executable = executable().ok_or_else(|| "cyberghostvpn was not found.".to_owned())?;
    let mut command = Command::new("sudo");
    if password.is_some() {
        command.arg("-S");
    } else {
        command.arg("-n");
    }
    command
        .arg("--")
        .arg(executable)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not run sudo: {error}"))?;
    if let Some(password) = password {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(password.as_bytes())
                .and_then(|_| stdin.write_all(b"\n"))
                .map_err(|error| format!("Could not pass the password to sudo: {error}"))?;
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("Could not wait for sudo: {error}"))?;
    let text = combined(&output);
    if !output.status.success() {
        return Err(error_message(&text, output.status.code()));
    }
    if let Some(error) = first_error(&text) {
        return Err(error);
    }
    Ok(())
}

fn parse_countries(output: &Output) -> Result<Vec<Country>, String> {
    let mut list = Vec::new();
    for line in table_lines(output) {
        let parts = columns(&line);
        if parts.len() != 3 {
            continue;
        }
        let Some(id) = parts[0].parse::<u32>().ok() else {
            continue;
        };
        let name = parts[1].trim();
        let code = parts[2].trim();
        if name.is_empty() || code.is_empty() {
            continue;
        }
        list.push(Country {
            id,
            name: name.to_owned(),
            code: code.to_owned(),
        });
    }
    finish_list(output, list)
}

fn parse_streaming_services(output: &Output) -> Result<Vec<StreamingService>, String> {
    let mut list = Vec::new();
    for line in table_lines(output) {
        let parts = columns(&line);
        if parts.len() != 3 {
            continue;
        }
        let Some(id) = parts[0].parse::<u32>().ok() else {
            continue;
        };
        let service = parts[1].trim();
        let country_code = parts[2].trim();
        if service.is_empty() || country_code.is_empty() {
            continue;
        }
        list.push(StreamingService {
            id,
            service: service.to_owned(),
            country_code: country_code.to_owned(),
        });
    }
    list.sort_by(|a, b| a.service.cmp(&b.service));
    finish_list(output, list)
}

fn parse_cities(output: &Output) -> Result<Vec<City>, String> {
    let mut list = Vec::new();
    for line in table_lines(output) {
        let parts = columns(&line);
        if parts.len() != 4 {
            continue;
        }
        let Some(id) = parts[0].parse::<u32>().ok() else {
            continue;
        };
        let name = parts[1].trim();
        if name.is_empty() {
            continue;
        }
        list.push(City {
            id,
            name: name.to_owned(),
            instances: parts[2].trim().to_owned(),
            load: parts[3].trim().to_owned(),
        });
    }
    list.sort_by(|a, b| a.name.cmp(&b.name));
    finish_list(output, list)
}

fn parse_servers(output: &Output) -> Result<Vec<Server>, String> {
    let mut list = Vec::new();
    for line in table_lines(output) {
        let parts = columns(&line);
        if parts.len() != 4 {
            continue;
        }
        let Some(id) = parts[0].parse::<u32>().ok() else {
            continue;
        };
        let name = parts[1].trim();
        if name.is_empty() {
            continue;
        }
        list.push(Server {
            id,
            name: name.to_owned(),
            instance: parts[2].trim().to_owned(),
            load: parts[3].trim().to_owned(),
        });
    }
    list.sort_by(|a, b| {
        load_value(&a.load)
            .cmp(&load_value(&b.load))
            .then_with(|| a.name.cmp(&b.name))
    });
    finish_list(output, list)
}

fn finish_list<T>(output: &Output, list: Vec<T>) -> Result<Vec<T>, String> {
    if list.is_empty() {
        if let Some(error) = first_error(&combined(output)) {
            return Err(error);
        }
    }
    Ok(list)
}

fn table_lines(output: &Output) -> Vec<String> {
    combined(output)
        .lines()
        .filter(|line| line.contains('|'))
        .map(str::to_owned)
        .collect()
}

fn columns(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|value| value.trim().to_owned())
        .collect()
}

fn combined(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn first_error(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    if !(lower.contains("error")
        || lower.contains("cannot fetch")
        || lower.contains("failed")
        || lower.contains("not found")
        || lower.contains("not installed"))
    {
        return None;
    }
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_lowercase();
            !line.is_empty()
                && (lower.contains("error")
                    || lower.contains("cannot")
                    || lower.contains("failed")
                    || lower.contains("not found")
                    || lower.contains("not installed"))
                && !line.starts_with('|')
        })
        .unwrap_or_else(|| text.trim());
    Some(truncate(line, 320))
}

fn error_message(text: &str, code: Option<i32>) -> String {
    if let Some(error) = first_error(text) {
        return error;
    }
    if text.trim().is_empty() {
        return match code {
            Some(code) => format!("The CyberGhost CLI exited with code {code}."),
            None => "The CyberGhost CLI failed.".to_owned(),
        };
    }
    truncate(text.trim(), 320)
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut end = 0;
    for (index, _) in text.char_indices().take(limit) {
        end = index;
    }
    format!("{}...", &text[..end])
}

fn load_value(load: &str) -> u32 {
    load.trim_end_matches('%')
        .trim()
        .parse()
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(stdout: &str) -> Output {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            Output {
                status: std::process::ExitStatus::from_raw(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = stdout;
            unimplemented!()
        }
    }

    #[test]
    fn parses_country_table() {
        let parsed = parse_countries(&output(
            "| id | name | code |\n|----|------|------|\n| 1 | Germany | DE |\n| 2 | Japan | JP |\n",
        ))
        .unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].code, "DE");
        assert_eq!(parsed[1].name, "Japan");
    }

    #[test]
    fn parses_city_and_server_tables() {
        let cities = parse_cities(&output(
            "| id | name | instance | load |\n| 4 | Berlin | 110 | 22 |\n| 5 | Munich | 27 | 8 |\n",
        ))
        .unwrap();
        assert_eq!(cities[0].name, "Berlin");
        assert_eq!(cities[1].instances, "27");

        let servers = parse_servers(&output(
            "| id | name | instance | load |\n| 1 | Berlin 01 | de-berlin-s01 | 71 |\n| 2 | Berlin 02 | de-berlin-s02 | 12 |\n",
        ))
        .unwrap();
        assert_eq!(servers[0].instance, "de-berlin-s02");
        assert_eq!(servers[1].load, "71");
    }

    #[test]
    fn reports_cli_errors() {
        let error = parse_countries(&output(
            "Cannot fetch user data. Error code \"401\", please try again later!\n",
        ))
        .unwrap_err();
        assert!(error.contains("401"));
    }

    #[test]
    fn parses_tunnel_status_without_false_positives() {
        assert_eq!(parse_status("VPN connection found.\n"), Some(true));
        assert_eq!(parse_status("No VPN connection found.\n"), Some(false));
        assert_eq!(parse_status("No VPN connections found.\n"), Some(false));
        assert_eq!(parse_status("Wireguard connection found.\n"), Some(true));
        assert_eq!(
            parse_status("No Wireguard connection found.\n"),
            Some(false)
        );
        assert_eq!(parse_status("Unexpected output"), None);
    }

    #[test]
    fn builds_openvpn_connect_arguments() {
        let profile = Profile {
            kind: crate::model::ProfileKind::CyberGhost,
            country_code: Some("DE".to_owned()),
            city: Some("Berlin".to_owned()),
            server_instance: Some("de-berlin-s01".to_owned()),
            server_type: ServerType::Traffic,
            network_protocol: NetworkProtocol::Tcp,
            protocol: Protocol::OpenVpn,
            ..Profile::default()
        };
        let args = connect_args(&profile).unwrap();
        assert_eq!(
            args,
            [
                "--traffic",
                "--country-code",
                "DE",
                "--city",
                "Berlin",
                "--server",
                "de-berlin-s01",
                "--connection",
                "tcp",
                "--openvpn",
                "--connect"
            ]
        );
    }

    #[test]
    fn streaming_requires_a_service() {
        let profile = Profile {
            kind: crate::model::ProfileKind::CyberGhost,
            country_code: Some("US".to_owned()),
            server_type: ServerType::Streaming,
            ..Profile::default()
        };
        assert!(connect_args(&profile).unwrap_err().contains("streaming"));
    }

    #[test]
    fn builds_streaming_wireguard_arguments() {
        let profile = Profile {
            kind: crate::model::ProfileKind::CyberGhost,
            country_code: Some("US".to_owned()),
            city: Some("New York".to_owned()),
            server_instance: Some("newyork-s404-i62".to_owned()),
            server_type: ServerType::Streaming,
            streaming_service: Some("Fandango US".to_owned()),
            network_protocol: NetworkProtocol::Udp,
            protocol: Protocol::WireGuard,
            ..Profile::default()
        };
        let args = connect_args(&profile).unwrap();
        assert_eq!(
            args,
            [
                "--streaming",
                "Fandango US",
                "--country-code",
                "US",
                "--city",
                "New York",
                "--server",
                "newyork-s404-i62",
                "--wireguard",
                "--connect"
            ]
        );
    }

    #[test]
    fn profile_ids_follow_streaming_service_and_protocol() {
        let base = profile_id(
            ServerType::Streaming,
            "US",
            Some("New York"),
            Some("newyork-s404-i62"),
            Protocol::OpenVpn,
            NetworkProtocol::Udp,
            Some("Fandango US"),
        );
        let other_service = profile_id(
            ServerType::Streaming,
            "US",
            Some("New York"),
            Some("newyork-s404-i62"),
            Protocol::OpenVpn,
            NetworkProtocol::Udp,
            Some("Netflix US"),
        );
        let wireguard = profile_id(
            ServerType::Streaming,
            "US",
            Some("New York"),
            Some("newyork-s404-i62"),
            Protocol::WireGuard,
            NetworkProtocol::Udp,
            Some("Fandango US"),
        );
        assert_ne!(base, other_service);
        assert_ne!(base, wireguard);
        assert!(base.starts_with("cg_"));
    }

    #[test]
    fn fallback_country_codes_are_unique() {
        let list = fallback_countries();
        let mut codes: Vec<&str> = list.iter().map(|country| country.code.as_str()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), list.len());
        assert!(list.iter().any(|country| country.code == "DE"));
    }
}
