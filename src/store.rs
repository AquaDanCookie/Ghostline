use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use chrono::Local;
use directories::ProjectDirs;

use crate::model::{AppState, LogEntry, LogLevel, Profile, STATE_VERSION};

#[derive(Clone, Debug)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_file: PathBuf,
    pub log_file: PathBuf,
    pub autostart_file: PathBuf,
    pub runtime_dir: PathBuf,
}

impl Paths {
    pub fn discover() -> Self {
        let project = ProjectDirs::from("io", "ghostline", "Ghostline").unwrap_or_else(|| {
            ProjectDirs::from("io", "ghostline", "ghostline").expect("home directory")
        });
        let config_dir = project.config_dir().to_path_buf();
        let data_dir = project.data_dir().to_path_buf();
        let state_file = config_dir.join("state.json");
        let log_file = data_dir.join("logs").join("ghostline.log");
        let autostart_dir = directories::BaseDirs::new()
            .map(|base| base.config_dir().join("autostart"))
            .unwrap_or_else(|| config_dir.join("autostart"));
        let autostart_file = autostart_dir.join("ghostline.desktop");
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("ghostline");

        Self {
            config_dir,
            data_dir,
            state_file,
            log_file,
            autostart_file,
            runtime_dir,
        }
    }

    pub fn ensure(&self) -> Result<(), String> {
        for dir in [
            &self.config_dir,
            &self.data_dir,
            self.log_file.parent().unwrap_or(&self.data_dir),
            &self.runtime_dir,
        ] {
            fs::create_dir_all(dir)
                .map_err(|error| format!("Cannot create {}: {error}", dir.display()))?;
            let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
        }
        Ok(())
    }
}

fn keep_after_load(profile: &Profile) -> bool {
    !profile.id.is_empty() && (profile.is_cyberghost() || !profile.nm_uuid.is_empty())
}

pub fn load_state(paths: &Paths) -> (AppState, Option<String>) {
    let raw = match fs::read_to_string(&paths.state_file) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (AppState::default(), None)
        }
        Err(error) => {
            return (
                AppState::default(),
                Some(format!("Could not read saved state: {error}")),
            )
        }
    };

    match serde_json::from_str::<AppState>(&raw) {
        Ok(mut state) => {
            if state.version > STATE_VERSION {
                return (
                    AppState::default(),
                    Some("Saved state was written by a newer Ghostline version. It was left untouched.".to_owned()),
                );
            }
            state.version = STATE_VERSION;
            state.profiles.retain(keep_after_load);
            (state, None)
        }
        Err(error) => {
            let backup = paths.state_file.with_extension("json.bad");
            if fs::copy(&paths.state_file, &backup).is_ok() {
                let _ = fs::set_permissions(&backup, fs::Permissions::from_mode(0o600));
            }
            (
                AppState::default(),
                Some(format!(
                    "Saved state was unreadable ({error}). A copy was kept at {}.",
                    backup.display()
                )),
            )
        }
    }
}

pub fn save_state(paths: &Paths, state: &AppState) -> Result<(), String> {
    paths.ensure()?;
    let json = serde_json::to_vec_pretty(state)
        .map_err(|error| format!("Cannot serialize settings: {error}"))?;
    let temporary = paths.state_file.with_extension("json.tmp");
    {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| format!("Cannot write {}: {error}", temporary.display()))?;
        file.write_all(&json)
            .map_err(|error| format!("Cannot write settings: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Cannot flush settings: {error}"))?;
    }
    fs::rename(&temporary, &paths.state_file)
        .map_err(|error| format!("Cannot replace settings: {error}"))?;
    let _ = fs::set_permissions(&paths.state_file, fs::Permissions::from_mode(0o600));
    Ok(())
}

pub fn append_log(paths: &Paths, level: LogLevel, message: &str) {
    let _ = paths.ensure();
    rotate_log_if_needed(paths);
    let stamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!(
        "{stamp} {:<5} {}\n",
        level.label(),
        message.replace('\n', " ")
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(&paths.log_file)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

fn rotate_log_if_needed(paths: &Paths) {
    let Ok(metadata) = fs::metadata(&paths.log_file) else {
        return;
    };
    if metadata.len() < 1024 * 1024 {
        return;
    }
    let previous = paths.log_file.with_extension("log.1");
    let _ = fs::remove_file(&previous);
    let _ = fs::rename(&paths.log_file, previous);
}

pub fn clear_logs(paths: &Paths) {
    let _ = fs::remove_file(&paths.log_file);
    let _ = fs::remove_file(paths.log_file.with_extension("log.1"));
}

pub fn load_log_entries(paths: &Paths, limit: usize) -> Vec<LogEntry> {
    let Ok(raw) = fs::read_to_string(&paths.log_file) else {
        return Vec::new();
    };
    let mut entries: Vec<LogEntry> = raw.lines().filter_map(parse_log_line).collect();
    if entries.len() > limit {
        entries.drain(0..entries.len() - limit);
    }
    entries
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let (date, rest) = line.split_once(' ')?;
    let (time, rest) = rest.split_once(' ')?;
    let (level, message) = rest.split_once(' ')?;
    let level = match level {
        "INFO" => LogLevel::Info,
        "WARN" => LogLevel::Warn,
        "ERROR" => LogLevel::Error,
        _ => return None,
    };
    let naive =
        chrono::NaiveDateTime::parse_from_str(&format!("{date} {time}"), "%Y-%m-%d %H:%M:%S")
            .ok()?;
    Some(LogEntry {
        at: naive.and_local_timezone(Local).single()?.timestamp(),
        level,
        message: message.trim_start().to_owned(),
    })
}

pub fn prune_log(paths: &Paths, retention_days: u32) {
    if retention_days == 0 {
        return;
    }
    let Ok(metadata) = fs::metadata(&paths.log_file) else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(age) = modified.elapsed() else {
        return;
    };
    if age.as_secs() > u64::from(retention_days) * 86_400 {
        clear_logs(paths);
    }
}

pub fn autostart_enabled(paths: &Paths) -> bool {
    paths.autostart_file.is_file()
}

pub fn set_autostart(paths: &Paths, enabled: bool, executable: &Path) -> Result<(), String> {
    if enabled {
        let parent = paths
            .autostart_file
            .parent()
            .ok_or_else(|| "Autostart path has no parent directory.".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Cannot create autostart directory: {error}"))?;
        let executable = executable
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let desktop = format!(
            "[Desktop Entry]\nType=Application\nName=Ghostline\nComment=Connect CyberGhost profiles through NetworkManager\nExec=\"{executable}\"\nIcon=network-vpn\nTerminal=false\nStartupNotify=false\nCategories=Network;VPN;\nX-GNOME-Autostart-enabled=true\n"
        );
        fs::write(&paths.autostart_file, desktop)
            .map_err(|error| format!("Cannot write autostart entry: {error}"))?;
        let _ = fs::set_permissions(&paths.autostart_file, fs::Permissions::from_mode(0o644));
    } else {
        match fs::remove_file(&paths.autostart_file) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Cannot remove autostart entry: {error}")),
        }
    }
    Ok(())
}

pub fn secret_store(profile_id: &str, password: &str) -> Result<(), String> {
    let mut child = Command::new("secret-tool")
        .args([
            "store",
            "--label",
            &format!("Ghostline profile {profile_id}"),
            "service",
            "ghostline",
            "profile",
            profile_id,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("System keyring is unavailable: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(password.as_bytes())
            .map_err(|error| format!("Cannot write to the system keyring: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("System keyring failed: {error}"))?;
    if !output.status.success() {
        return Err(command_error("secret-tool store", &output.stderr));
    }
    Ok(())
}

pub fn secret_lookup(profile_id: &str) -> Result<Option<String>, String> {
    let output = Command::new("secret-tool")
        .args(["lookup", "service", "ghostline", "profile", profile_id])
        .output()
        .map_err(|error| format!("System keyring is unavailable: {error}"))?;
    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    } else {
        Ok(None)
    }
}

pub fn secret_clear(profile_id: &str) -> Result<(), String> {
    let output = Command::new("secret-tool")
        .args(["clear", "service", "ghostline", "profile", profile_id])
        .output()
        .map_err(|error| format!("System keyring is unavailable: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Ok(())
    }
}

fn command_error(command: &str, stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    if text.is_empty() {
        format!("{command} failed.")
    } else {
        format!("{command} failed: {text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_line_round_trip() {
        let line = "2026-10-21 21:00:00 INFO  connected to test";
        let entry = parse_log_line(line).unwrap();
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "connected to test");
    }

    #[test]
    fn cyberghost_profiles_survive_without_an_nm_uuid() {
        let mut cg = Profile {
            id: "cg_test".to_owned(),
            kind: crate::model::ProfileKind::CyberGhost,
            ..Profile::default()
        };
        assert!(keep_after_load(&cg));

        cg.id.clear();
        assert!(!keep_after_load(&cg));

        let imported = Profile {
            id: "imported".to_owned(),
            ..Profile::default()
        };
        assert!(!keep_after_load(&imported));
    }

    #[test]
    fn saved_cyberghost_profile_loads_after_restart() {
        let root =
            std::env::temp_dir().join(format!("ghostline-store-test-{}", std::process::id()));
        let config = root.join("config");
        let data = root.join("data");
        let paths = Paths {
            config_dir: config.clone(),
            data_dir: data.clone(),
            state_file: config.join("state.json"),
            log_file: data.join("logs/ghostline.log"),
            autostart_file: config.join("autostart/ghostline.desktop"),
            runtime_dir: root.join("run"),
        };
        let mut state = AppState::default();
        state.profiles.push(Profile {
            id: "cg_test".to_owned(),
            name: "Test location".to_owned(),
            kind: crate::model::ProfileKind::CyberGhost,
            ..Profile::default()
        });

        save_state(&paths, &state).expect("save state");
        let (loaded, warning) = load_state(&paths);
        assert!(warning.is_none());
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].id, "cg_test");

        let _ = fs::remove_dir_all(root);
    }
}
