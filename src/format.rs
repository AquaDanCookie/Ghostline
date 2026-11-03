use chrono::{Local, TimeZone};

use crate::model::{LogEntry, LogLevel};

pub fn duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

pub fn bytes(value: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut amount = value as f64;
    let mut unit = 0;
    while amount >= 1024.0 && unit < UNITS.len() - 1 {
        amount /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value} {}", UNITS[unit])
    } else if amount >= 100.0 {
        format!("{amount:.0} {}", UNITS[unit])
    } else {
        format!("{amount:.1} {}", UNITS[unit])
    }
}

pub fn rate(bytes_per_second: f64) -> String {
    if !bytes_per_second.is_finite() || bytes_per_second <= 0.0 {
        return "0 B/s".to_owned();
    }
    format!("{}/s", bytes(bytes_per_second.round() as u64))
}

pub fn date(timestamp: i64) -> String {
    match Local.timestamp_opt(timestamp, 0).single() {
        Some(value) => value.format("%d %b %Y").to_string(),
        None => "Unknown".to_owned(),
    }
}

pub fn time(timestamp: i64) -> String {
    match Local.timestamp_opt(timestamp, 0).single() {
        Some(value) => value.format("%H:%M:%S").to_string(),
        None => "--:--:--".to_owned(),
    }
}

pub fn date_time(timestamp: i64) -> String {
    match Local.timestamp_opt(timestamp, 0).single() {
        Some(value) => value.format("%d %b %Y, %H:%M").to_string(),
        None => "Unknown".to_owned(),
    }
}

pub fn log_line(entry: &LogEntry) -> String {
    format!(
        "{} {} {}",
        date_time(entry.at),
        entry.level.label(),
        entry.message
    )
}

pub fn level_word(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Info => "Info",
        LogLevel::Warn => "Warning",
        LogLevel::Error => "Error",
    }
}

pub fn count(value: usize, singular: &str, plural: &str) -> String {
    if value == 1 {
        format!("1 {singular}")
    } else {
        format!("{value} {plural}")
    }
}

pub fn shorten_path(path: &str, max_chars: usize) -> String {
    if path.chars().count() <= max_chars {
        return path.to_owned();
    }
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path);
    if name.chars().count() + 4 <= max_chars {
        return format!(".../{name}");
    }
    let keep: String = name.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{keep}...")
}

pub fn shorten(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }
    let keep: String = value.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{keep}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations() {
        assert_eq!(duration(0), "00:00");
        assert_eq!(duration(59), "00:59");
        assert_eq!(duration(61), "01:01");
        assert_eq!(duration(3_661), "01:01:01");
        assert_eq!(duration(-4), "00:00");
    }

    #[test]
    fn formats_bytes_and_rates() {
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(1_536), "1.5 KiB");
        assert_eq!(bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(rate(1_024.0), "1.0 KiB/s");
        assert_eq!(rate(0.0), "0 B/s");
    }

    #[test]
    fn shortens_long_paths_without_losing_the_file_name() {
        let short = shorten_path("/home/somebody/profiles/CG_DE_Berlin_01.ovpn", 24);
        assert!(short.ends_with("CG_DE_Berlin_01.ovpn"));
        assert!(short.starts_with("..."));
    }
}
