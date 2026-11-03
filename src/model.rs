use serde::{Deserialize, Serialize};

pub const STATE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    OpenVpn,
    WireGuard,
}

impl Protocol {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenVpn => "OpenVPN",
            Self::WireGuard => "WireGuard",
        }
    }

    pub fn nm_type(self) -> &'static str {
        match self {
            Self::OpenVpn => "openvpn",
            Self::WireGuard => "wireguard",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    #[default]
    Imported,
    CyberGhost,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
    #[default]
    Traffic,
    Streaming,
    Torrent,
}

impl ServerType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Traffic => "Traffic",
            Self::Streaming => "Streaming",
            Self::Torrent => "Torrent",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkProtocol {
    #[default]
    Udp,
    Tcp,
}

impl NetworkProtocol {
    pub fn label(self) -> &'static str {
        match self {
            Self::Udp => "UDP",
            Self::Tcp => "TCP",
        }
    }

    pub fn cli_value(self) -> &'static str {
        match self {
            Self::Udp => "udp",
            Self::Tcp => "tcp",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LoadState {
    #[default]
    Idle,
    Loading,
    Ready,
    Failed(String),
}

impl LoadState {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Failed(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub theme: ThemeMode,
    pub launch_at_login: bool,
    pub auto_connect: bool,
    pub auto_reconnect: bool,
    pub disconnect_on_exit: bool,
    pub compact_rows: bool,
    pub log_retention_days: u32,
    pub load_last_profile: bool,
    pub keep_sudo_password: bool,
    pub default_protocol: Protocol,
    pub default_network_protocol: NetworkProtocol,
    pub default_server_type: ServerType,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            launch_at_login: false,
            auto_connect: false,
            auto_reconnect: false,
            disconnect_on_exit: false,
            compact_rows: false,
            log_retention_days: 30,
            load_last_profile: true,
            keep_sudo_password: false,
            default_protocol: Protocol::OpenVpn,
            default_network_protocol: NetworkProtocol::Udp,
            default_server_type: ServerType::Traffic,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Latency {
    pub ms: u32,
    pub measured_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub kind: ProfileKind,
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub protocol: Protocol,
    pub server_type: ServerType,
    pub network_protocol: NetworkProtocol,
    pub streaming_service: Option<String>,
    pub server_instance: Option<String>,
    pub server_load: Option<String>,
    pub remote_host: String,
    pub remote_port: u16,
    pub requires_auth: bool,
    pub source_path: String,
    pub nm_uuid: String,
    pub imported_at: i64,
    pub favorite: bool,
    pub latency: Option<Latency>,
    pub saved_username: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: ProfileKind::Imported,
            country_code: None,
            country: None,
            city: None,
            protocol: Protocol::OpenVpn,
            server_type: ServerType::Traffic,
            network_protocol: NetworkProtocol::Udp,
            streaming_service: None,
            server_instance: None,
            server_load: None,
            remote_host: String::new(),
            remote_port: 0,
            requires_auth: false,
            source_path: String::new(),
            nm_uuid: String::new(),
            imported_at: 0,
            favorite: false,
            latency: None,
            saved_username: None,
        }
    }
}

impl Profile {
    pub fn endpoint(&self) -> String {
        if self.kind == ProfileKind::CyberGhost {
            if let Some(server) = &self.server_instance {
                return server.clone();
            }
            if let Some(city) = &self.city {
                return format!("Automatic in {city}");
            }
            return "Automatic".to_owned();
        }
        if self.remote_host.is_empty() {
            return "Not declared".to_owned();
        }
        format!("{}:{}", self.remote_host, self.remote_port)
    }

    pub fn is_cyberghost(&self) -> bool {
        self.kind == ProfileKind::CyberGhost
    }

    pub fn network_label(&self) -> String {
        if self.is_cyberghost() && self.protocol == Protocol::OpenVpn {
            format!(
                "{} {}",
                self.protocol.label(),
                self.network_protocol.label()
            )
        } else {
            self.protocol.label().to_owned()
        }
    }

    pub fn country_label(&self) -> &str {
        self.country
            .as_deref()
            .or(self.country_code.as_deref())
            .unwrap_or("Unclassified")
    }

    pub fn sort_country(&self) -> String {
        self.country_label().to_lowercase()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppState {
    pub version: u32,
    pub profiles: Vec<Profile>,
    pub settings: Settings,
    pub last_profile_id: Option<String>,
    pub window_size: Option<[f32; 2]>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            profiles: Vec::new(),
            settings: Settings::default(),
            last_profile_id: None,
            window_size: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TunnelState {
    Disconnected,
    Connecting {
        profile_id: String,
    },
    Connected {
        profile_id: String,
        iface: String,
        since: i64,
    },
    External {
        iface: String,
        since: i64,
    },
    Disconnecting,
    Reconnecting {
        profile_id: String,
        attempt: u8,
    },
    Error {
        profile_id: Option<String>,
        message: String,
    },
}

impl TunnelState {
    pub fn profile_id(&self) -> Option<&str> {
        match self {
            Self::Disconnected | Self::Disconnecting => None,
            Self::Connecting { profile_id }
            | Self::Reconnecting { profile_id, .. }
            | Self::Connected { profile_id, .. } => Some(profile_id),
            Self::External { .. } => None,
            Self::Error { profile_id, .. } => profile_id.as_deref(),
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            Self::Connecting { .. } | Self::Disconnecting | Self::Reconnecting { .. }
        )
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. } | Self::External { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub at: i64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub rx_total: u64,
    pub tx_total: u64,
    pub rx_rate: f64,
    pub tx_rate: f64,
}

#[derive(Clone, Debug)]
pub struct DnsInfo {
    pub servers: Vec<String>,
    pub interface: String,
    pub nm_version: String,
}

impl Default for DnsInfo {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            interface: "Unknown".to_owned(),
            nm_version: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Connect,
    Network,
    Locations,
    Activity,
    Settings,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Self::Connect => "Connect",
            Self::Network => "Network",
            Self::Locations => "Locations",
            Self::Activity => "Activity",
            Self::Settings => "Settings",
        }
    }

    pub const ALL: [Self; 5] = [
        Self::Connect,
        Self::Network,
        Self::Locations,
        Self::Activity,
        Self::Settings,
    ];
}
