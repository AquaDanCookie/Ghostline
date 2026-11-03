use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use chrono::Local;
use eframe::egui;

use crate::backend::{Backend, Command as BackendCommand, Event};
use crate::cg::{self, City, Country, Server, StreamingService};
use crate::model::{
    AppState, DnsInfo, Latency, LoadState, LogEntry, LogLevel, NetworkProtocol, Profile,
    ProfileKind, Protocol, Screen, ServerType, Stats, ThemeMode, TunnelState,
};
use crate::nm::{self, Credentials};
use crate::profile::{self, ParsedProfile};
use crate::store::{self, Paths};
use crate::theme::{self, Palette, Tone};

pub const APP_NAME: &str = "Ghostline";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Info,
    Warnings,
    Errors,
}

impl LogFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Info => "Info",
            Self::Warnings => "Warnings",
            Self::Errors => "Errors",
        }
    }

    pub fn matches(self, level: LogLevel) -> bool {
        match self {
            Self::All => true,
            Self::Info => level == LogLevel::Info,
            Self::Warnings => level == LogLevel::Warn,
            Self::Errors => level == LogLevel::Error,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum IpState {
    #[default]
    Idle,
    Checking,
    Done(String),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub level: LogLevel,
    pub message: String,
    pub at: Instant,
}

impl Toast {
    pub fn tone(&self) -> Tone {
        match self.level {
            LogLevel::Info => Tone::Accent,
            LogLevel::Warn => Tone::Warning,
            LogLevel::Error => Tone::Danger,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Checking,
    Present,
    Missing,
    Unavailable,
}

#[derive(Clone, Debug)]
pub struct StatusView {
    pub label: &'static str,
    pub tone: Tone,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct ImportReview {
    pub path: PathBuf,
    pub parsed: ParsedProfile,
    pub duplicate: bool,
}

#[derive(Clone, Debug)]
pub struct CredentialsPrompt {
    pub profile_id: String,
    pub username: String,
    pub password: String,
    pub remember: bool,
    pub show_password: bool,
    pub focus_requested: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RemovePrompt {
    pub profile_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SudoIntent {
    Connect(String),
    Disconnect,
}

#[derive(Clone, Debug)]
pub struct SudoPrompt {
    pub intent: SudoIntent,
    pub password: String,
    pub remember: bool,
    pub show_password: bool,
    pub focus_requested: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ModalState {
    Import(ImportReview),
    ImportError { path: String, error: String },
    Credentials(CredentialsPrompt),
    Sudo(SudoPrompt),
    Remove(RemovePrompt),
    ClearLogs,
}

#[derive(Clone, Debug)]
pub struct NetworkBrowser {
    pub server_type: ServerType,
    pub vpn_service: Protocol,
    pub network_protocol: NetworkProtocol,
    pub countries: Vec<Country>,
    pub countries_state: LoadState,
    pub countries_fallback: bool,
    pub country: Option<Country>,
    pub country_search: String,
    pub streaming_services: Vec<StreamingService>,
    pub streaming_state: LoadState,
    pub streaming_service: Option<String>,
    pub cities: Vec<City>,
    pub cities_state: LoadState,
    pub city: Option<City>,
    pub city_search: String,
    pub servers: Vec<Server>,
    pub servers_state: LoadState,
    pub server: Option<Server>,
    pub server_search: String,
    pub loaded_once: bool,
}

impl NetworkBrowser {
    pub fn new(state: &AppState) -> Self {
        Self {
            server_type: state.settings.default_server_type,
            vpn_service: state.settings.default_protocol,
            network_protocol: state.settings.default_network_protocol,
            countries: Vec::new(),
            countries_state: LoadState::Idle,
            countries_fallback: false,
            country: None,
            country_search: String::new(),
            streaming_services: Vec::new(),
            streaming_state: LoadState::Idle,
            streaming_service: None,
            cities: Vec::new(),
            cities_state: LoadState::Idle,
            city: None,
            city_search: String::new(),
            servers: Vec::new(),
            servers_state: LoadState::Idle,
            server: None,
            server_search: String::new(),
            loaded_once: false,
        }
    }

    pub fn reset_downstream(&mut self) {
        self.country = None;
        self.streaming_services.clear();
        self.streaming_state = LoadState::Idle;
        self.streaming_service = None;
        self.cities.clear();
        self.cities_state = LoadState::Idle;
        self.city = None;
        self.servers.clear();
        self.servers_state = LoadState::Idle;
        self.server = None;
    }
}

#[derive(Clone, Debug)]
struct PendingImport {
    path: PathBuf,
    parsed: ParsedProfile,
}

#[derive(Clone, Debug)]
pub enum Action {
    Navigate(Screen),
    RequestImport,
    ConfirmImport,
    CloseModal,
    Connect(String),
    Disconnect,
    CancelConnect,
    SelectProfile(String),
    ToggleFavorite(String),
    MeasureLatency(String),
    EditCredentials(String),
    RecreateConnection(String),
    RemoveProfile(String),
    ConfirmRemove,
    Copy(String),
    CheckIp,
    RefreshNetwork,
    FlushDns,
    RequestClearLogs,
    ConfirmClearLogs,
    OpenDataFolder,
    SetTheme(ThemeMode),
    SetLaunchAtLogin(bool),
    SetAutoConnect(bool),
    SetAutoReconnect(bool),
    SetDisconnectOnExit(bool),
    SetCompactRows(bool),
    SetLogRetention(u32),
    SubmitCredentials,
    SubmitSudo,
    CloseToast(u64),
    ToggleConnection,
    RefreshCgCountries,
    SelectCgCountry(String),
    SelectCgStreaming(String),
    SelectCgCity(String),
    SelectCgServer(String),
    SetCgServerType(ServerType),
    SetCgVpnService(Protocol),
    SetCgNetworkProtocol(NetworkProtocol),
    SaveCgLocation,
    ConnectCgLocation,
    SetLoadLastProfile(bool),
    SetKeepSudoPassword(bool),
    SetDefaultProtocol(Protocol),
    SetDefaultNetworkProtocol(NetworkProtocol),
    SetDefaultServerType(ServerType),
}

pub struct App {
    pub paths: Paths,
    pub state: AppState,
    backend: Backend,
    pub demo: bool,
    pub screen: Screen,
    pub dark: bool,
    pub palette: Palette,
    pub tunnel: TunnelState,
    pub stats: Stats,
    pub stats_seen: bool,
    pub logs: Vec<LogEntry>,
    pub log_filter: LogFilter,
    pub log_search: String,
    pub location_search: String,
    pub favorites_only: bool,
    pub selected_profile: Option<String>,
    pub dns: DnsInfo,
    pub inventory: HashSet<String>,
    pub inventory_ready: bool,
    pub nm_ready: bool,
    pub cg_ready: bool,
    pub network: NetworkBrowser,
    pub sudo_password: Option<String>,
    pub ip: IpState,
    pub latency_pending: HashSet<String>,
    pub toasts: Vec<Toast>,
    pub modal: Option<ModalState>,
    pending_imports: Vec<PendingImport>,
    toast_seq: u64,
    dirty: bool,
    last_saved: Instant,
    last_window: [f32; 2],
    last_title: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup_fonts(&cc.egui_ctx);

        let demo = demo_enabled();
        let (paths, mut state, logs, startup_warning) = if demo {
            let paths = demo_paths();
            let (mut state, logs) = demo_state();
            if let Ok(theme_value) = std::env::var("GHOSTLINE_DEMO_THEME") {
                state.settings.theme = if theme_value.eq_ignore_ascii_case("light") {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                };
            } else {
                state.settings.theme = ThemeMode::Dark;
            }
            (paths, state, logs, None)
        } else {
            let paths = Paths::discover();
            let (state, warning) = store::load_state(&paths);
            let logs = store::load_log_entries(&paths, 1_000);
            (paths, state, logs, warning)
        };

        let mut startup_toasts: Vec<(LogLevel, String)> = Vec::new();
        if let Err(error) = paths.ensure() {
            startup_toasts.push((LogLevel::Error, error));
        }
        if !demo {
            if let Some(warning) = startup_warning {
                startup_toasts.push((LogLevel::Warn, warning));
            }
            store::prune_log(&paths, state.settings.log_retention_days);
            state.settings.launch_at_login = store::autostart_enabled(&paths);
        }

        let dark = match state.settings.theme {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => theme::system_prefers_dark(),
        };
        theme::apply(&cc.egui_ctx, dark);
        let palette = theme::palette(dark);

        if state.last_profile_id.is_none() {
            if state.settings.load_last_profile {
                state.last_profile_id = state.profiles.first().map(|profile| profile.id.clone());
            }
        } else if !state.settings.load_last_profile {
            state.last_profile_id = None;
        }
        let selected_profile = if state.settings.load_last_profile {
            state
                .last_profile_id
                .clone()
                .or_else(|| state.profiles.first().map(|profile| profile.id.clone()))
        } else {
            None
        };
        let mut network = NetworkBrowser::new(&state);
        if demo {
            demo_network(&mut network);
        }

        let cg_ready = demo || cg::available();
        let backend = Backend::spawn(paths.clone(), demo, state.settings.auto_reconnect);

        let screen = if demo {
            std::env::var("GHOSTLINE_DEMO_SCREEN")
                .ok()
                .and_then(|value| match value.to_ascii_lowercase().as_str() {
                    "connect" => Some(Screen::Connect),
                    "network" => Some(Screen::Network),
                    "locations" => Some(Screen::Locations),
                    "activity" => Some(Screen::Activity),
                    "settings" => Some(Screen::Settings),
                    _ => None,
                })
                .unwrap_or(Screen::Connect)
        } else {
            Screen::Connect
        };

        let last_window = state
            .window_size
            .map(|size| clamp_window_size(size))
            .unwrap_or([1240.0, 800.0]);

        let mut app = Self {
            paths,
            state,
            backend,
            demo,
            screen,
            dark,
            palette,
            tunnel: TunnelState::Disconnected,
            stats: Stats::default(),
            stats_seen: false,
            logs,
            log_filter: LogFilter::All,
            log_search: String::new(),
            location_search: String::new(),
            favorites_only: false,
            selected_profile,
            dns: DnsInfo::default(),
            inventory: HashSet::new(),
            inventory_ready: false,
            nm_ready: demo || nm::available(),
            cg_ready,
            network,
            sudo_password: None,
            ip: IpState::Idle,
            latency_pending: HashSet::new(),
            toasts: Vec::new(),
            modal: None,
            pending_imports: Vec::new(),
            toast_seq: 0,
            dirty: false,
            last_saved: Instant::now(),
            last_window,
            last_title: String::new(),
        };

        for (level, message) in startup_toasts {
            app.toast(level, message);
        }
        if demo {
            app.record_log(
                LogLevel::Info,
                "Demo mode is active. These profiles are sample data. No traffic is routed.",
            );
            if std::env::var("GHOSTLINE_DEMO_AUTOCONNECT").as_deref() == Ok("1") {
                if let Some(profile) = app.current_profile().cloned() {
                    app.send_connect(profile, None, None);
                }
            }
        } else {
            app.maybe_auto_connect();
        }
        if app.screen == Screen::Network {
            app.ensure_network_loaded();
        }
        app
    }

    pub fn initial_window_size() -> egui::Vec2 {
        if demo_enabled() {
            return egui::vec2(1240.0, 800.0);
        }
        let paths = Paths::discover();
        let (state, _) = store::load_state(&paths);
        let size = state
            .window_size
            .map(clamp_window_size)
            .unwrap_or([1240.0, 800.0]);
        egui::vec2(size[0], size[1])
    }

    pub fn profile(&self, id: &str) -> Option<&Profile> {
        self.state.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn current_profile(&self) -> Option<&Profile> {
        if let Some(id) = self.state.last_profile_id.as_deref() {
            if let Some(profile) = self.profile(id) {
                return Some(profile);
            }
        }
        if self.state.settings.load_last_profile {
            self.state.profiles.first()
        } else {
            None
        }
    }

    pub fn tunnel_profile_id(&self) -> Option<&str> {
        self.tunnel.profile_id()
    }

    pub fn status(&self) -> StatusView {
        let profile_name = |id: &str| {
            self.profile(id)
                .map(|profile| profile.name.clone())
                .unwrap_or_else(|| "Unknown profile".to_owned())
        };
        match &self.tunnel {
            TunnelState::Disconnected => StatusView {
                label: "Disconnected",
                tone: Tone::Neutral,
                detail: self
                    .current_profile()
                    .map(|profile| format!("Ready for {}", profile.name))
                    .unwrap_or_else(|| "No profile selected".to_owned()),
            },
            TunnelState::Connecting { profile_id } => StatusView {
                label: "Connecting",
                tone: Tone::Accent,
                detail: profile_name(profile_id),
            },
            TunnelState::Connected { profile_id, .. } => StatusView {
                label: "Connected",
                tone: Tone::Ok,
                detail: profile_name(profile_id),
            },
            TunnelState::External { .. } => StatusView {
                label: "Connected",
                tone: Tone::Ok,
                detail: "Active outside Ghostline".to_owned(),
            },
            TunnelState::Disconnecting => StatusView {
                label: "Disconnecting",
                tone: Tone::Neutral,
                detail: self
                    .current_profile()
                    .map(|profile| profile.name.clone())
                    .unwrap_or_else(|| "Tunnel".to_owned()),
            },
            TunnelState::Reconnecting {
                profile_id,
                attempt,
            } => StatusView {
                label: "Reconnecting",
                tone: Tone::Accent,
                detail: format!("Attempt {attempt} of 3, {}", profile_name(profile_id)),
            },
            TunnelState::Error {
                profile_id,
                message,
            } => StatusView {
                label: "Connection failed",
                tone: Tone::Danger,
                detail: profile_id
                    .as_deref()
                    .map(profile_name)
                    .map(|name| format!("{name}: {message}"))
                    .unwrap_or_else(|| message.clone()),
            },
        }
    }

    pub fn connection_presence(&self, profile: &Profile) -> Presence {
        if self.demo {
            return Presence::Present;
        }
        if profile.is_cyberghost() {
            return if self.cg_ready {
                Presence::Present
            } else {
                Presence::Unavailable
            };
        }
        if !self.nm_ready {
            return Presence::Unavailable;
        }
        if !self.inventory_ready {
            return Presence::Checking;
        }
        if self.inventory.contains(&profile.nm_uuid) {
            Presence::Present
        } else {
            Presence::Missing
        }
    }

    pub fn is_busy(&self) -> bool {
        self.tunnel.is_busy()
    }

    pub fn session_seconds(&self) -> Option<i64> {
        if let TunnelState::Connected { since, .. } | TunnelState::External { since, .. } =
            self.tunnel
        {
            Some((Local::now().timestamp() - since).max(0))
        } else {
            None
        }
    }

    pub fn open_modal_for_import(&mut self, paths: Vec<PathBuf>) {
        self.queue_imports(paths);
    }

    pub fn save_now(&mut self) {
        if self.demo {
            return;
        }
        if let Err(error) = store::save_state(&self.paths, &self.state) {
            self.toast(LogLevel::Error, error);
        }
        self.dirty = false;
        self.last_saved = Instant::now();
    }

    pub fn update_title(&mut self, ctx: &egui::Context) {
        let title = match &self.tunnel {
            TunnelState::Connected { .. } | TunnelState::External { .. } => {
                format!("{APP_NAME} - Connected")
            }
            TunnelState::Connecting { .. } | TunnelState::Reconnecting { .. } => {
                format!("{APP_NAME} - Connecting")
            }
            TunnelState::Disconnecting => format!("{APP_NAME} - Disconnecting"),
            TunnelState::Error { .. } => format!("{APP_NAME} - Connection failed"),
            TunnelState::Disconnected => {
                if self.demo {
                    format!("{APP_NAME} - Demo")
                } else {
                    APP_NAME.to_owned()
                }
            }
        };
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events(ctx);
        self.poll_window(ctx);
        self.update_title(ctx);

        let dropped = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect::<Vec<_>>()
        });
        if !dropped.is_empty() {
            self.open_modal_for_import(dropped);
        }

        let mut actions = Vec::new();
        self.handle_shortcuts(ctx, &mut actions);

        let palette = self.palette;
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(224.0)
            .frame(egui::Frame::new().fill(palette.bg))
            .show(ctx, |ui| {
                crate::ui::nav::show(self, ui, &mut actions);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.bg))
            .show(ctx, |ui| {
                crate::ui::page(self, ui, &mut actions);
            });

        crate::ui::toasts::show(self, ctx, &mut actions);
        crate::ui::modals::show(self, ctx, &mut actions);
        crate::ui::drop_overlay(self, ctx);

        for action in actions {
            self.apply(action, ctx);
        }

        if !self.demo && self.dirty && self.last_saved.elapsed() >= Duration::from_millis(700) {
            self.save_now();
        }

        let repaint = if self.tunnel.is_busy() || matches!(self.ip, IpState::Checking) {
            Duration::from_millis(40)
        } else if !self.toasts.is_empty() || self.tunnel.is_connected() {
            Duration::from_millis(120)
        } else {
            Duration::from_millis(450)
        };
        ctx.request_repaint_after(repaint);
    }

    fn apply(&mut self, action: Action, ctx: &egui::Context) {
        match action {
            Action::Navigate(screen) => {
                self.screen = screen;
                if screen == Screen::Locations && self.selected_profile.is_none() {
                    self.selected_profile = self.state.last_profile_id.clone().or_else(|| {
                        self.state
                            .profiles
                            .first()
                            .map(|profile| profile.id.clone())
                    });
                }
            }
            Action::RequestImport => self.browse_import(),
            Action::ConfirmImport => self.confirm_import(),
            Action::CloseModal => self.close_modal(true),
            Action::Connect(id) => {
                self.select_profile(&id);
                self.begin_connect(&id);
            }
            Action::Disconnect => self.request_disconnect(),
            Action::CancelConnect => self.backend.send(BackendCommand::CancelConnect),
            Action::SelectProfile(id) => self.select_profile(&id),
            Action::ToggleFavorite(id) => {
                if let Some(profile) = self
                    .state
                    .profiles
                    .iter_mut()
                    .find(|profile| profile.id == id)
                {
                    profile.favorite = !profile.favorite;
                    self.mark_dirty();
                }
            }
            Action::MeasureLatency(id) => self.measure_latency(&id),
            Action::EditCredentials(id) => self.open_credentials(&id, None),
            Action::RecreateConnection(id) => self.recreate_connection(&id),
            Action::RemoveProfile(id) => {
                self.modal = Some(ModalState::Remove(RemovePrompt { profile_id: id }));
            }
            Action::ConfirmRemove => self.confirm_remove(),
            Action::Copy(text) => {
                ctx.copy_text(text);
                self.toast(LogLevel::Info, "Copied to the clipboard.");
            }
            Action::CheckIp => {
                if self.tunnel.is_connected() && !matches!(self.ip, IpState::Checking) {
                    self.ip = IpState::Checking;
                    self.backend.send(BackendCommand::CheckIp);
                }
            }
            Action::RefreshNetwork => {
                self.nm_ready = self.demo || nm::available();
                self.cg_ready = self.demo || cg::available();
                self.inventory_ready = false;
                self.backend.send(BackendCommand::RefreshInventory);
                self.backend.send(BackendCommand::RefreshDns);
                if self.screen == Screen::Network {
                    self.refresh_cg_countries();
                }
            }
            Action::FlushDns => self.backend.send(BackendCommand::FlushDns),
            Action::RequestClearLogs => self.modal = Some(ModalState::ClearLogs),
            Action::ConfirmClearLogs => {
                store::clear_logs(&self.paths);
                self.logs.clear();
                self.modal = None;
                self.toast(LogLevel::Info, "Activity log cleared.");
            }
            Action::OpenDataFolder => self.open_data_folder(),
            Action::SetTheme(mode) => {
                self.state.settings.theme = mode;
                self.dark = match mode {
                    ThemeMode::Dark => true,
                    ThemeMode::Light => false,
                    ThemeMode::System => theme::system_prefers_dark(),
                };
                self.palette = theme::palette(self.dark);
                theme::apply(ctx, self.dark);
                self.mark_dirty();
            }
            Action::SetLaunchAtLogin(enabled) => self.set_launch_at_login(enabled),
            Action::SetAutoConnect(enabled) => {
                self.state.settings.auto_connect = enabled;
                self.mark_dirty();
            }
            Action::SetAutoReconnect(enabled) => {
                self.state.settings.auto_reconnect = enabled;
                self.backend.send(BackendCommand::SetAutoReconnect(enabled));
                self.mark_dirty();
            }
            Action::SetDisconnectOnExit(enabled) => {
                self.state.settings.disconnect_on_exit = enabled;
                self.mark_dirty();
            }
            Action::SetCompactRows(enabled) => {
                self.state.settings.compact_rows = enabled;
                self.mark_dirty();
            }
            Action::SetLogRetention(days) => {
                self.state.settings.log_retention_days = days;
                store::prune_log(&self.paths, days);
                self.mark_dirty();
            }
            Action::SubmitCredentials => self.submit_credentials(),
            Action::SubmitSudo => self.submit_sudo(),
            Action::CloseToast(id) => self.toasts.retain(|toast| toast.id != id),
            Action::ToggleConnection => self.toggle_connection(),
            Action::RefreshCgCountries => self.refresh_cg_countries(),
            Action::SelectCgCountry(code) => self.select_cg_country(&code),
            Action::SelectCgStreaming(service) => self.select_cg_streaming(&service),
            Action::SelectCgCity(city) => self.select_cg_city(&city),
            Action::SelectCgServer(server) => self.select_cg_server(&server),
            Action::SetCgServerType(server_type) => self.set_cg_server_type(server_type),
            Action::SetCgVpnService(protocol) => {
                self.network.vpn_service = protocol;
                self.mark_dirty();
            }
            Action::SetCgNetworkProtocol(network_protocol) => {
                self.network.network_protocol = network_protocol;
                self.mark_dirty();
            }
            Action::SaveCgLocation => self.save_cg_location(),
            Action::ConnectCgLocation => self.connect_cg_location(),
            Action::SetLoadLastProfile(enabled) => {
                self.state.settings.load_last_profile = enabled;
                if !enabled {
                    self.state.last_profile_id = None;
                    self.selected_profile = None;
                }
                self.mark_dirty();
            }
            Action::SetKeepSudoPassword(enabled) => {
                self.state.settings.keep_sudo_password = enabled;
                if !enabled {
                    self.sudo_password = None;
                }
                self.mark_dirty();
            }
            Action::SetDefaultProtocol(protocol) => {
                self.state.settings.default_protocol = protocol;
                self.mark_dirty();
            }
            Action::SetDefaultNetworkProtocol(network_protocol) => {
                self.state.settings.default_network_protocol = network_protocol;
                self.mark_dirty();
            }
            Action::SetDefaultServerType(server_type) => {
                self.state.settings.default_server_type = server_type;
                self.mark_dirty();
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        if self.modal.is_some() {
            return;
        }
        let (command, keys) = ctx.input(|input| {
            (
                input.modifiers.command && !input.modifiers.alt,
                [
                    input.key_pressed(egui::Key::Num1),
                    input.key_pressed(egui::Key::Num2),
                    input.key_pressed(egui::Key::Num3),
                    input.key_pressed(egui::Key::Num4),
                    input.key_pressed(egui::Key::Num5),
                    input.key_pressed(egui::Key::O),
                    input.key_pressed(egui::Key::Enter),
                ],
            )
        });
        if !command {
            return;
        }
        if keys[0] {
            actions.push(Action::Navigate(Screen::Connect));
        } else if keys[1] {
            actions.push(Action::Navigate(Screen::Network));
        } else if keys[2] {
            actions.push(Action::Navigate(Screen::Locations));
        } else if keys[3] {
            actions.push(Action::Navigate(Screen::Activity));
        } else if keys[4] {
            actions.push(Action::Navigate(Screen::Settings));
        } else if keys[5] {
            actions.push(Action::RequestImport);
        } else if keys[6] {
            actions.push(Action::ToggleConnection);
        }
    }

    fn toggle_connection(&mut self) {
        match &self.tunnel {
            TunnelState::Connected { profile_id, .. } => {
                if self
                    .state
                    .last_profile_id
                    .as_deref()
                    .map(|id| id == profile_id)
                    .unwrap_or(true)
                {
                    self.request_disconnect();
                }
            }
            TunnelState::External { .. } => self.request_disconnect(),
            TunnelState::Connecting { .. } | TunnelState::Reconnecting { .. } => {
                self.backend.send(BackendCommand::CancelConnect);
            }
            TunnelState::Disconnecting => {}
            TunnelState::Disconnected | TunnelState::Error { .. } => {
                if let Some(profile) = self.current_profile().cloned() {
                    self.begin_connect(&profile.id);
                }
            }
        }
    }

    fn select_profile(&mut self, id: &str) {
        if !self.state.profiles.iter().any(|profile| profile.id == id) {
            return;
        }
        let changed = self.selected_profile.as_deref() != Some(id)
            || self.state.last_profile_id.as_deref() != Some(id);
        self.selected_profile = Some(id.to_owned());
        self.state.last_profile_id = Some(id.to_owned());
        if changed {
            self.mark_dirty();
        }
    }

    fn begin_connect(&mut self, id: &str) {
        self.begin_connect_with_sudo(id, None);
    }

    fn begin_connect_with_sudo(&mut self, id: &str, sudo_password: Option<String>) {
        let Some(profile) = self.profile(id).cloned() else {
            return;
        };
        if self.tunnel.is_busy() {
            self.toast(
                LogLevel::Warn,
                "Wait for the current tunnel attempt to finish.",
            );
            return;
        }
        if self.tunnel.is_connected() {
            self.toast(LogLevel::Info, "Disconnect the active tunnel first.");
            return;
        }
        if profile.is_cyberghost() {
            if !self.cg_ready {
                self.toast(
                    LogLevel::Error,
                    "cyberghostvpn was not found. Install the official CLI before connecting.",
                );
                return;
            }
            if !self.demo
                && sudo_password.is_none()
                && self.sudo_password.is_none()
                && !cg::sudo_ready()
            {
                self.open_sudo(SudoIntent::Connect(profile.id.clone()));
                return;
            }
            let password = sudo_password.or_else(|| self.sudo_password.clone());
            self.send_connect(profile, None, password);
            return;
        }
        if !self.nm_ready {
            self.toast(
                LogLevel::Error,
                "NetworkManager is not available. Install nmcli and its VPN plugins.",
            );
            return;
        }
        if profile.requires_auth {
            if let Some(username) = profile.saved_username.clone() {
                match store::secret_lookup(&profile.id) {
                    Ok(Some(password)) => {
                        let credentials = Credentials { username, password };
                        self.send_connect(profile, Some(credentials), None);
                        return;
                    }
                    Ok(None) => {
                        self.open_credentials(&profile.id, Some(username));
                        return;
                    }
                    Err(error) => {
                        self.open_credentials(&profile.id, Some(username));
                        if let Some(ModalState::Credentials(prompt)) = self.modal.as_mut() {
                            prompt.error = Some(error);
                        }
                        return;
                    }
                }
            }
            self.open_credentials(&profile.id, None);
            return;
        }
        self.send_connect(profile, None, None);
    }

    fn send_connect(
        &mut self,
        profile: Profile,
        credentials: Option<Credentials>,
        sudo_password: Option<String>,
    ) {
        self.screen = Screen::Connect;
        self.selected_profile = Some(profile.id.clone());
        self.state.last_profile_id = Some(profile.id.clone());
        self.ip = IpState::Idle;
        self.stats = Stats::default();
        self.stats_seen = false;
        self.backend.send(BackendCommand::Connect {
            profile,
            credentials,
            sudo_password,
        });
        self.mark_dirty();
    }

    fn request_disconnect(&mut self) {
        let cyber = matches!(self.tunnel, TunnelState::External { .. })
            || self
                .tunnel
                .profile_id()
                .and_then(|id| self.profile(id))
                .map(Profile::is_cyberghost)
                .unwrap_or(false);
        if !cyber {
            self.send_disconnect(None);
            return;
        }
        if !self.demo && self.sudo_password.is_none() && !cg::sudo_ready() {
            self.open_sudo(SudoIntent::Disconnect);
            return;
        }
        self.send_disconnect(self.sudo_password.clone());
    }

    fn send_disconnect(&mut self, sudo_password: Option<String>) {
        self.backend
            .send(BackendCommand::Disconnect { sudo_password });
    }

    fn open_sudo(&mut self, intent: SudoIntent) {
        self.modal = Some(ModalState::Sudo(SudoPrompt {
            intent,
            password: String::new(),
            remember: self.state.settings.keep_sudo_password,
            show_password: false,
            focus_requested: true,
            error: None,
        }));
    }

    fn submit_sudo(&mut self) {
        let Some(ModalState::Sudo(prompt)) = self.modal.take() else {
            return;
        };
        if prompt.password.is_empty() {
            self.modal = Some(ModalState::Sudo(SudoPrompt {
                error: Some("Enter the administrator password.".to_owned()),
                focus_requested: true,
                ..prompt
            }));
            return;
        }
        if prompt.remember {
            self.sudo_password = Some(prompt.password.clone());
        } else {
            self.sudo_password = None;
        }
        match prompt.intent {
            SudoIntent::Connect(id) => {
                self.begin_connect_with_sudo(&id, Some(prompt.password));
            }
            SudoIntent::Disconnect => {
                self.send_disconnect(Some(prompt.password));
            }
        }
    }

    fn open_credentials(&mut self, id: &str, username: Option<String>) {
        let Some(profile) = self.profile(id) else {
            return;
        };
        if !profile.requires_auth {
            self.begin_connect(id);
            return;
        }
        let username = username
            .or_else(|| profile.saved_username.clone())
            .unwrap_or_default();
        self.modal = Some(ModalState::Credentials(CredentialsPrompt {
            profile_id: id.to_owned(),
            username,
            password: String::new(),
            remember: false,
            show_password: false,
            focus_requested: true,
            error: None,
        }));
    }

    fn submit_credentials(&mut self) {
        let Some(ModalState::Credentials(prompt)) = self.modal.take() else {
            return;
        };
        let username = prompt.username.trim().to_owned();
        if username.is_empty() || prompt.password.is_empty() {
            self.modal = Some(ModalState::Credentials(CredentialsPrompt {
                error: Some("Enter both the username and the password.".to_owned()),
                focus_requested: true,
                ..prompt
            }));
            return;
        }

        let Some(mut profile) = self.profile(&prompt.profile_id).cloned() else {
            return;
        };

        if !self.demo {
            if prompt.remember {
                match store::secret_store(&profile.id, &prompt.password) {
                    Ok(()) => {
                        profile.saved_username = Some(username.clone());
                        self.toast(LogLevel::Info, "Password saved in the system keyring.");
                    }
                    Err(error) => {
                        let _ = store::secret_clear(&profile.id);
                        profile.saved_username = None;
                        self.toast(
                            LogLevel::Warn,
                            format!("{error} The password is used for this session only."),
                        );
                    }
                }
            } else {
                if profile.saved_username.is_some() {
                    let _ = store::secret_clear(&profile.id);
                }
                profile.saved_username = None;
            }
            if let Some(stored) = self
                .state
                .profiles
                .iter_mut()
                .find(|stored| stored.id == profile.id)
            {
                stored.saved_username = profile.saved_username.clone();
            }
        }

        let credentials = Credentials {
            username,
            password: prompt.password,
        };
        self.send_connect(profile, Some(credentials), None);
        self.mark_dirty();
    }

    fn measure_latency(&mut self, id: &str) {
        let Some(profile) = self.profile(id) else {
            return;
        };
        let host = profile.remote_host.clone();
        let port = profile.remote_port;
        if host.is_empty() || port == 0 {
            self.toast(LogLevel::Error, "This profile has no usable endpoint.");
            return;
        }
        if self.latency_pending.contains(id) {
            return;
        }
        self.latency_pending.insert(id.to_owned());
        self.backend.send(BackendCommand::MeasureLatency {
            profile_id: id.to_owned(),
            host,
            port,
        });
    }

    fn recreate_connection(&mut self, id: &str) {
        let Some(profile) = self.profile(id).cloned() else {
            return;
        };
        let path = PathBuf::from(&profile.source_path);
        if !path.is_file() {
            self.toast(
                LogLevel::Error,
                format!(
                    "The original file is no longer available at {}.",
                    profile.source_path
                ),
            );
            return;
        }
        match profile::parse_path(&path) {
            Ok(parsed) => {
                self.backend
                    .send(BackendCommand::Repair { profile, parsed });
            }
            Err(error) => self.toast(
                LogLevel::Error,
                format!("Could not read the original profile: {error}"),
            ),
        }
    }

    fn confirm_remove(&mut self) {
        let Some(ModalState::Remove(prompt)) = self.modal.take() else {
            return;
        };
        let Some(profile) = self.profile(&prompt.profile_id).cloned() else {
            return;
        };
        self.backend.send(BackendCommand::Remove(profile));
    }

    fn browse_import(&mut self) {
        if self.demo {
            self.toast(
                LogLevel::Warn,
                "Import is disabled while demo data is shown.",
            );
            return;
        }
        if !self.nm_ready {
            self.toast(
                LogLevel::Error,
                "NetworkManager is not available. Install nmcli before importing.",
            );
            return;
        }
        let picked = rfd::FileDialog::new()
            .set_title("Import a VPN profile")
            .add_filter("VPN profiles", &["ovpn", "conf"])
            .add_filter("OpenVPN profiles", &["ovpn"])
            .add_filter("WireGuard profiles", &["conf"])
            .pick_files();
        if let Some(paths) = picked {
            self.queue_imports(paths);
        }
    }

    fn queue_imports(&mut self, paths: Vec<PathBuf>) {
        if self.demo {
            self.toast(
                LogLevel::Warn,
                "Import is disabled while demo data is shown.",
            );
            return;
        }
        for path in paths {
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if extension != "ovpn" && extension != "conf" {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("that file");
                self.toast(
                    LogLevel::Warn,
                    format!("Skipped {name}. Only .ovpn and .conf profiles are supported."),
                );
                continue;
            }
            match profile::parse_path(&path) {
                Ok(parsed) => self.pending_imports.push(PendingImport { path, parsed }),
                Err(error) => {
                    let message = format!("Could not read {}: {error}", path.display());
                    self.record_log(LogLevel::Error, &message);
                    if self.modal.is_none() {
                        self.modal = Some(ModalState::ImportError {
                            path: path.display().to_string(),
                            error,
                        });
                    } else {
                        self.toast(LogLevel::Error, message);
                    }
                }
            }
        }
        if self.modal.is_none() {
            self.open_next_import();
        }
    }

    fn open_next_import(&mut self) {
        if self.pending_imports.is_empty() {
            return;
        }
        let pending = self.pending_imports.remove(0);
        let source = pending.path.to_string_lossy().into_owned();
        let duplicate = self
            .state
            .profiles
            .iter()
            .any(|profile| profile.source_path == source);
        self.modal = Some(ModalState::Import(ImportReview {
            path: pending.path,
            parsed: pending.parsed,
            duplicate,
        }));
    }

    fn confirm_import(&mut self) {
        let Some(ModalState::Import(review)) = self.modal.take() else {
            return;
        };
        if review.duplicate {
            let source = review.path.to_string_lossy().into_owned();
            if let Some(existing) = self
                .state
                .profiles
                .iter()
                .find(|profile| profile.source_path == source)
                .cloned()
            {
                if self.tunnel.is_busy() || self.tunnel.is_connected() {
                    self.toast(
                        if self.tunnel.is_busy() {
                            LogLevel::Warn
                        } else {
                            LogLevel::Info
                        },
                        "Finish the active connection before recreating this profile.",
                    );
                    self.modal = Some(ModalState::Import(review));
                    return;
                }
                self.backend.send(BackendCommand::Repair {
                    profile: existing,
                    parsed: review.parsed,
                });
                self.open_next_import();
                return;
            }
        }
        self.backend.send(BackendCommand::Import {
            path: review.path,
            parsed: review.parsed,
        });
        self.open_next_import();
    }

    fn close_modal(&mut self, advance_imports: bool) {
        let was_import = matches!(
            self.modal,
            Some(ModalState::Import(_) | ModalState::ImportError { .. })
        );
        self.modal = None;
        if was_import && advance_imports {
            self.open_next_import();
        }
    }

    fn set_launch_at_login(&mut self, enabled: bool) {
        let executable = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ghostline"));
        match store::set_autostart(&self.paths, enabled, &executable) {
            Ok(()) => {
                self.state.settings.launch_at_login = enabled;
                self.mark_dirty();
            }
            Err(error) => self.toast(LogLevel::Error, error),
        }
    }

    fn open_data_folder(&mut self) {
        if let Err(error) = Command::new("xdg-open")
            .arg(&self.paths.data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            self.toast(
                LogLevel::Error,
                format!("Could not open the data folder: {error}"),
            );
        }
    }

    fn maybe_auto_connect(&mut self) {
        if self.demo || !self.state.settings.auto_connect {
            return;
        }
        let Some(profile) = self.current_profile().cloned() else {
            return;
        };
        if profile.is_cyberghost() {
            if self.cg_ready {
                self.begin_connect(&profile.id);
            }
            return;
        }
        if !self.nm_ready {
            return;
        }
        if profile.requires_auth {
            let Some(username) = profile.saved_username.clone() else {
                self.toast(
                    LogLevel::Warn,
                    format!(
                        "Auto-connect skipped for {}. Save the password in the system keyring first.",
                        profile.name
                    ),
                );
                return;
            };
            match store::secret_lookup(&profile.id) {
                Ok(Some(password)) => {
                    self.send_connect(profile, Some(Credentials { username, password }), None);
                }
                _ => self.toast(
                    LogLevel::Warn,
                    format!(
                        "Auto-connect skipped for {}. The saved password could not be read.",
                        profile.name
                    ),
                ),
            }
        } else {
            self.send_connect(profile, None, None);
        }
    }

    pub fn ensure_network_loaded(&mut self) {
        if self.network.loaded_once {
            return;
        }
        self.network.loaded_once = true;
        if self.demo {
            demo_network(&mut self.network);
        } else {
            self.refresh_cg_countries();
        }
    }

    pub fn refresh_cg_countries(&mut self) {
        self.network.loaded_once = true;
        self.cg_ready = self.demo || cg::available();
        if self.demo {
            demo_network(&mut self.network);
            return;
        }
        self.network.countries_state = LoadState::Loading;
        self.network.countries_fallback = false;
        if !self.cg_ready {
            self.network.countries = cg::fallback_countries();
            self.network.countries_fallback = true;
            self.network.countries_state =
                LoadState::Failed("cyberghostvpn was not found.".to_owned());
            return;
        }
        self.backend.send(BackendCommand::LoadCountries {
            server_type: self.network.server_type,
        });
    }

    fn set_cg_server_type(&mut self, server_type: ServerType) {
        if self.network.server_type == server_type {
            return;
        }
        self.network.server_type = server_type;
        self.network.reset_downstream();
        self.refresh_cg_countries();
    }

    fn select_cg_country(&mut self, code: &str) {
        let Some(country) = self
            .network
            .countries
            .iter()
            .find(|country| country.code == code)
            .cloned()
        else {
            return;
        };
        if self
            .network
            .country
            .as_ref()
            .map(|selected| selected.code == code)
            .unwrap_or(false)
        {
            return;
        }
        self.network.reset_downstream();
        self.network.country = Some(country.clone());
        self.network.city_search.clear();
        self.network.server_search.clear();
        if self.demo {
            demo_fill_country(&mut self.network);
            return;
        }
        if self.network.server_type == ServerType::Streaming {
            self.network.streaming_state = LoadState::Loading;
            self.backend.send(BackendCommand::LoadStreamingServices {
                country_code: country.code.clone(),
            });
        } else {
            self.load_cg_cities(&country.code);
        }
    }

    fn select_cg_streaming(&mut self, service: &str) {
        let Some(selected) = self
            .network
            .streaming_services
            .iter()
            .find(|item| item.service == service)
            .cloned()
        else {
            return;
        };
        self.network.streaming_service = Some(selected.service);
        self.network.city = None;
        self.network.servers.clear();
        self.network.server = None;
        self.network.servers_state = LoadState::Idle;
        if self.demo {
            demo_fill_cities(&mut self.network);
            return;
        }
        if let Some(country) = self.network.country.clone() {
            self.load_cg_cities(&country.code);
        }
    }

    fn select_cg_city(&mut self, name: &str) {
        if name.is_empty() {
            self.network.city = None;
            self.network.servers.clear();
            self.network.server = None;
            self.network.servers_state = LoadState::Idle;
            return;
        }
        let already = self
            .network
            .city
            .as_ref()
            .map(|city| city.name == name)
            .unwrap_or(false);
        if already {
            self.network.city = None;
            self.network.servers.clear();
            self.network.server = None;
            self.network.servers_state = LoadState::Idle;
            return;
        }
        let Some(city) = self
            .network
            .cities
            .iter()
            .find(|city| city.name == name)
            .cloned()
        else {
            return;
        };
        self.network.city = Some(city.clone());
        self.network.server = None;
        self.network.servers.clear();
        self.network.servers_state = LoadState::Loading;
        if self.demo {
            demo_fill_servers(&mut self.network);
            return;
        }
        if let Some(country) = self.network.country.clone() {
            self.backend.send(BackendCommand::LoadServers {
                server_type: self.network.server_type,
                country_code: country.code,
                city: city.name,
                streaming_service: self.network.streaming_service.clone(),
            });
        }
    }

    fn select_cg_server(&mut self, instance: &str) {
        if instance.is_empty() {
            self.network.server = None;
            return;
        }
        let already = self
            .network
            .server
            .as_ref()
            .map(|server| server.instance == instance)
            .unwrap_or(false);
        if already {
            self.network.server = None;
            return;
        }
        if let Some(server) = self
            .network
            .servers
            .iter()
            .find(|server| server.instance == instance)
            .cloned()
        {
            self.network.server = Some(server);
        }
    }

    fn load_cg_cities(&mut self, country_code: &str) {
        self.network.cities.clear();
        self.network.cities_state = LoadState::Loading;
        self.network.city = None;
        self.network.servers.clear();
        self.network.server = None;
        self.network.servers_state = LoadState::Idle;
        self.backend.send(BackendCommand::LoadCities {
            server_type: self.network.server_type,
            country_code: country_code.to_owned(),
            streaming_service: self.network.streaming_service.clone(),
        });
    }

    fn cg_profile_from_selection(&self) -> Result<Profile, String> {
        let country = self
            .network
            .country
            .clone()
            .ok_or_else(|| "Choose a country first.".to_owned())?;
        if self.network.server_type == ServerType::Streaming
            && self.network.streaming_service.is_none()
        {
            return Err("Choose a streaming service first.".to_owned());
        }
        let city = self.network.city.clone();
        let server = self.network.server.clone();
        let name = if let Some(server) = &server {
            server.name.clone()
        } else if let Some(city) = &city {
            city.name.clone()
        } else {
            country.name.clone()
        };
        let id = cg::profile_id(
            self.network.server_type,
            &country.code,
            city.as_ref().map(|city| city.name.as_str()),
            server.as_ref().map(|server| server.instance.as_str()),
            self.network.vpn_service,
            self.network.network_protocol,
            self.network.streaming_service.as_deref(),
        );
        Ok(Profile {
            id,
            name,
            kind: ProfileKind::CyberGhost,
            country_code: Some(country.code),
            country: Some(country.name),
            city: city.map(|city| city.name),
            protocol: self.network.vpn_service,
            server_type: self.network.server_type,
            network_protocol: self.network.network_protocol,
            streaming_service: self.network.streaming_service.clone(),
            server_instance: server.as_ref().map(|server| server.instance.clone()),
            server_load: server.map(|server| server.load),
            remote_host: String::new(),
            remote_port: 0,
            requires_auth: false,
            source_path: String::new(),
            nm_uuid: String::new(),
            imported_at: Local::now().timestamp(),
            favorite: false,
            latency: None,
            saved_username: None,
        })
    }

    fn upsert_cg_profile(&mut self, mut profile: Profile) -> String {
        if let Some(existing) = self
            .state
            .profiles
            .iter_mut()
            .find(|existing| existing.id == profile.id)
        {
            profile.imported_at = existing.imported_at;
            profile.favorite = existing.favorite;
            profile.latency = existing.latency.clone();
            *existing = profile.clone();
        } else {
            self.state.profiles.push(profile.clone());
        }
        self.select_profile(&profile.id);
        self.mark_dirty();
        profile.name
    }

    fn save_cg_location(&mut self) {
        match self.cg_profile_from_selection() {
            Ok(profile) => {
                let name = self.upsert_cg_profile(profile);
                self.toast(LogLevel::Info, format!("Saved {name} to Locations."));
            }
            Err(error) => self.toast(LogLevel::Warn, error),
        }
    }

    fn connect_cg_location(&mut self) {
        match self.cg_profile_from_selection() {
            Ok(profile) => {
                let id = profile.id.clone();
                self.upsert_cg_profile(profile);
                self.begin_connect(&id);
            }
            Err(error) => self.toast(LogLevel::Warn, error),
        }
    }

    fn poll_window(&mut self, ctx: &egui::Context) {
        if self.demo {
            return;
        }
        let (inner, maximized) = ctx.input(|input| {
            (
                input.viewport().inner_rect,
                input.viewport().maximized.unwrap_or(false),
            )
        });
        let Some(rect) = inner else {
            return;
        };
        if maximized || rect.width() < 100.0 || rect.height() < 100.0 {
            return;
        }
        let size = [rect.width(), rect.height()];
        if (size[0] - self.last_window[0]).abs() > 2.0
            || (size[1] - self.last_window[1]).abs() > 2.0
        {
            self.last_window = size;
            self.state.window_size = Some(size);
            self.mark_dirty();
        }
    }

    fn mark_dirty(&mut self) {
        if !self.demo {
            self.dirty = true;
        }
    }

    fn toast(&mut self, level: LogLevel, message: impl Into<String>) {
        let message = message.into();
        if let Some(existing) = self
            .toasts
            .iter_mut()
            .find(|toast| toast.level == level && toast.message == message)
        {
            existing.at = Instant::now();
            return;
        }
        self.toast_seq = self.toast_seq.wrapping_add(1);
        self.toasts.push(Toast {
            id: self.toast_seq,
            level,
            message,
            at: Instant::now(),
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    fn record_log(&mut self, level: LogLevel, message: &str) {
        let entry = LogEntry {
            at: Local::now().timestamp(),
            level,
            message: message.to_owned(),
        };
        self.logs.push(entry);
        if self.logs.len() > 1_000 {
            let excess = self.logs.len() - 1_000;
            self.logs.drain(0..excess);
        }
        store::append_log(&self.paths, level, message);
    }

    fn drain_events(&mut self, ctx: &egui::Context) {
        while let Some(event) = self.backend.try_recv() {
            match event {
                Event::Tunnel(state) => {
                    let previous = std::mem::replace(&mut self.tunnel, state.clone());
                    if previous != state {
                        match &state {
                            TunnelState::Connected { .. } | TunnelState::External { .. } => {
                                self.ip = IpState::Idle;
                                self.stats = Stats::default();
                                self.stats_seen = false;
                            }
                            TunnelState::Disconnected
                            | TunnelState::Connecting { .. }
                            | TunnelState::Reconnecting { .. }
                            | TunnelState::Disconnecting => {
                                self.ip = IpState::Idle;
                                self.stats = Stats::default();
                                self.stats_seen = false;
                            }
                            TunnelState::Error { message, .. } => {
                                self.ip = IpState::Idle;
                                self.stats = Stats::default();
                                self.stats_seen = false;
                                self.toast(LogLevel::Error, message.clone());
                            }
                        }
                    }
                }
                Event::Stats(stats) => {
                    self.stats = stats;
                    self.stats_seen = true;
                }
                Event::Log(entry) => {
                    self.logs.push(entry);
                    if self.logs.len() > 1_000 {
                        let excess = self.logs.len() - 1_000;
                        self.logs.drain(0..excess);
                    }
                }
                Event::Notice { level, message } => {
                    if message.contains("nmcli was not found") {
                        self.nm_ready = false;
                    }
                    if message.contains("cyberghostvpn was not found") {
                        self.cg_ready = false;
                    }
                    self.toast(level, message);
                }
                Event::Imported(profile) => {
                    let name = profile.name.clone();
                    let id = profile.id.clone();
                    if let Some(existing) = self
                        .state
                        .profiles
                        .iter_mut()
                        .find(|existing| existing.id == id)
                    {
                        *existing = profile;
                    } else {
                        self.state.profiles.push(profile);
                    }
                    self.select_profile(&id);
                    self.mark_dirty();
                    self.toast(LogLevel::Info, format!("Imported {name}."));
                }
                Event::ImportFailed { path, error } => {
                    let name = Path::new(&path)
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("the profile");
                    self.toast(LogLevel::Error, format!("Could not import {name}: {error}"));
                }
                Event::Removed(id) => {
                    self.state.profiles.retain(|profile| profile.id != id);
                    if self.selected_profile.as_deref() == Some(&id) {
                        self.selected_profile = None;
                    }
                    if self.state.last_profile_id.as_deref() == Some(&id) {
                        self.state.last_profile_id = None;
                    }
                    let next = self
                        .state
                        .profiles
                        .first()
                        .map(|profile| profile.id.clone());
                    if self.selected_profile.is_none() {
                        self.selected_profile = next.clone();
                    }
                    if self.state.last_profile_id.is_none() {
                        self.state.last_profile_id = next;
                    }
                    self.mark_dirty();
                    self.toast(LogLevel::Info, "Profile removed.");
                }
                Event::Repaired(profile) => {
                    let id = profile.id.clone();
                    if let Some(existing) = self
                        .state
                        .profiles
                        .iter_mut()
                        .find(|existing| existing.id == id)
                    {
                        *existing = profile;
                    }
                    self.mark_dirty();
                    self.toast(LogLevel::Info, "NetworkManager connection recreated.");
                }
                Event::Latency { profile_id, result } => {
                    self.latency_pending.remove(&profile_id);
                    let name = self
                        .profile(&profile_id)
                        .map(|profile| profile.name.clone())
                        .unwrap_or_else(|| "Profile".to_owned());
                    match result {
                        Ok(latency) => {
                            if let Some(profile) = self
                                .state
                                .profiles
                                .iter_mut()
                                .find(|profile| profile.id == profile_id)
                            {
                                profile.latency = Some(latency.clone());
                            }
                            self.mark_dirty();
                            self.toast(
                                LogLevel::Info,
                                format!("Latency to {name}: {} ms.", latency.ms),
                            );
                        }
                        Err(error) => {
                            self.toast(
                                LogLevel::Warn,
                                format!("Latency check for {name} failed: {error}"),
                            );
                        }
                    }
                }
                Event::IpChecked(result) => match result {
                    Ok(ip) => {
                        let connected = self.tunnel.is_connected();
                        self.ip = IpState::Done(ip.clone());
                        if connected {
                            self.toast(LogLevel::Info, format!("Exit IP: {ip}."));
                        }
                    }
                    Err(error) => {
                        self.ip = IpState::Error(error.clone());
                        self.toast(LogLevel::Warn, error);
                    }
                },
                Event::Inventory(uuids) => {
                    self.inventory = uuids.into_iter().collect();
                    self.inventory_ready = true;
                }
                Event::Dns(info) => {
                    if self.demo && info.nm_version.is_empty() {
                        self.dns = DnsInfo {
                            nm_version: "Demo data".to_owned(),
                            ..info
                        };
                    } else {
                        self.dns = info;
                    }
                }
                Event::Countries {
                    server_type,
                    result,
                } => {
                    if server_type != self.network.server_type {
                        continue;
                    }
                    match result {
                        Ok(mut countries) => {
                            countries.sort_by(|a, b| a.name.cmp(&b.name));
                            self.network.countries = countries;
                            self.network.countries_state = LoadState::Ready;
                            self.network.countries_fallback = false;
                            if let Some(selected) = self.network.country.clone() {
                                if let Some(found) = self
                                    .network
                                    .countries
                                    .iter()
                                    .find(|country| country.code == selected.code)
                                    .cloned()
                                {
                                    self.network.country = Some(found);
                                } else {
                                    self.network.reset_downstream();
                                }
                            }
                        }
                        Err(error) => {
                            self.network.countries_state = LoadState::Failed(error);
                            if self.network.countries.is_empty() {
                                self.network.countries = cg::fallback_countries();
                                self.network.countries_fallback = true;
                            }
                        }
                    }
                }
                Event::StreamingServices {
                    country_code,
                    result,
                } => {
                    let matches_country = self
                        .network
                        .country
                        .as_ref()
                        .map(|country| country.code == country_code)
                        .unwrap_or(false);
                    if !matches_country || self.network.server_type != ServerType::Streaming {
                        continue;
                    }
                    match result {
                        Ok(mut services) => {
                            services.sort_by(|a, b| a.service.cmp(&b.service));
                            self.network.streaming_services = services;
                            self.network.streaming_state = LoadState::Ready;
                            if let Some(selected) = self.network.streaming_service.clone() {
                                if !self
                                    .network
                                    .streaming_services
                                    .iter()
                                    .any(|service| service.service == selected)
                                {
                                    self.network.streaming_service = None;
                                }
                            }
                        }
                        Err(error) => {
                            self.network.streaming_services.clear();
                            self.network.streaming_service = None;
                            self.network.streaming_state = LoadState::Failed(error);
                        }
                    }
                }
                Event::Cities {
                    server_type,
                    country_code,
                    streaming_service,
                    result,
                } => {
                    let matches_country = self
                        .network
                        .country
                        .as_ref()
                        .map(|country| country.code == country_code)
                        .unwrap_or(false);
                    if server_type != self.network.server_type
                        || !matches_country
                        || streaming_service != self.network.streaming_service
                    {
                        continue;
                    }
                    match result {
                        Ok(mut cities) => {
                            cities.sort_by(|a, b| a.name.cmp(&b.name));
                            self.network.cities = cities;
                            self.network.cities_state = LoadState::Ready;
                            if let Some(selected) = self.network.city.clone() {
                                if let Some(found) = self
                                    .network
                                    .cities
                                    .iter()
                                    .find(|city| city.name == selected.name)
                                    .cloned()
                                {
                                    self.network.city = Some(found);
                                } else {
                                    self.network.city = None;
                                    self.network.servers.clear();
                                    self.network.server = None;
                                    self.network.servers_state = LoadState::Idle;
                                }
                            }
                        }
                        Err(error) => {
                            self.network.cities.clear();
                            self.network.city = None;
                            self.network.servers.clear();
                            self.network.server = None;
                            self.network.servers_state = LoadState::Idle;
                            self.network.cities_state = LoadState::Failed(error);
                        }
                    }
                }
                Event::Servers {
                    server_type,
                    country_code,
                    city,
                    streaming_service,
                    result,
                } => {
                    let matches_country = self
                        .network
                        .country
                        .as_ref()
                        .map(|country| country.code == country_code)
                        .unwrap_or(false);
                    let matches_city = self
                        .network
                        .city
                        .as_ref()
                        .map(|selected| selected.name == city)
                        .unwrap_or(false);
                    if server_type != self.network.server_type
                        || !matches_country
                        || !matches_city
                        || streaming_service != self.network.streaming_service
                    {
                        continue;
                    }
                    match result {
                        Ok(servers) => {
                            self.network.servers = servers;
                            self.network.servers_state = LoadState::Ready;
                            if let Some(selected) = self.network.server.clone() {
                                if let Some(found) = self
                                    .network
                                    .servers
                                    .iter()
                                    .find(|server| server.instance == selected.instance)
                                    .cloned()
                                {
                                    self.network.server = Some(found);
                                } else {
                                    self.network.server = None;
                                }
                            }
                        }
                        Err(error) => {
                            self.network.servers.clear();
                            self.network.server = None;
                            self.network.servers_state = LoadState::Failed(error);
                        }
                    }
                }
            }
        }
        if self.demo {
            // Demo events are emitted immediately; keep the session clock moving.
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        App::update(self, ctx, frame);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.demo && self.state.settings.disconnect_on_exit {
            let profile = self
                .tunnel_profile_id()
                .and_then(|id| self.profile(id))
                .cloned();
            if let Some(profile) = profile {
                if profile.is_cyberghost() {
                    let _ = cg::stop(self.sudo_password.as_deref());
                } else {
                    let _ = crate::nm::deactivate(&profile.nm_uuid);
                }
            }
        }
        self.save_now();
    }
}

fn demo_enabled() -> bool {
    matches!(
        std::env::var("GHOSTLINE_DEMO").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn clamp_window_size(size: [f32; 2]) -> [f32; 2] {
    [size[0].clamp(1000.0, 3840.0), size[1].clamp(660.0, 2160.0)]
}

fn demo_paths() -> Paths {
    let base = std::env::temp_dir().join("ghostline-demo");
    let config = base.join("config");
    let data = base.join("data");
    Paths {
        config_dir: config.clone(),
        data_dir: data.clone(),
        state_file: config.join("state.json"),
        log_file: data.join("logs").join("ghostline.log"),
        autostart_file: config.join("autostart").join("ghostline.desktop"),
        runtime_dir: base.join("runtime"),
    }
}

fn demo_state() -> (AppState, Vec<LogEntry>) {
    let now = Local::now().timestamp();
    let mut state = AppState::default();
    state.settings.theme = ThemeMode::Dark;
    state.settings.auto_reconnect = true;
    state.profiles = vec![
        demo_profile(
            "demo-de",
            "Berlin 01",
            "DE",
            "Germany",
            "Berlin",
            Protocol::OpenVpn,
            "de-berlin-01.demo.invalid",
            443,
            false,
            Some(24),
            true,
            now,
        ),
        demo_profile(
            "demo-nl",
            "Amsterdam 01",
            "NL",
            "Netherlands",
            "Amsterdam",
            Protocol::WireGuard,
            "nl-ams-01.demo.invalid",
            51820,
            false,
            Some(31),
            true,
            now,
        ),
        demo_profile(
            "demo-us",
            "New York 01",
            "US",
            "United States",
            "New York",
            Protocol::OpenVpn,
            "us-nyc-01.demo.invalid",
            443,
            true,
            None,
            false,
            now,
        ),
        demo_profile(
            "demo-jp",
            "Tokyo 01",
            "JP",
            "Japan",
            "Tokyo",
            Protocol::OpenVpn,
            "jp-tyo-01.demo.invalid",
            443,
            false,
            Some(168),
            false,
            now,
        ),
    ];
    state.last_profile_id = Some("demo-de".to_owned());
    let logs = vec![
        LogEntry {
            at: now - 120,
            level: LogLevel::Info,
            message: "Demo data loaded. No traffic is routed.".to_owned(),
        },
        LogEntry {
            at: now - 60,
            level: LogLevel::Info,
            message: "Select a location, then connect to preview the session view.".to_owned(),
        },
    ];
    (state, logs)
}

fn demo_network(browser: &mut NetworkBrowser) {
    let previous_country = browser.country.as_ref().map(|country| country.code.clone());
    let previous_city = browser.city.as_ref().map(|city| city.name.clone());
    let previous_server = browser
        .server
        .as_ref()
        .map(|server| server.instance.clone());

    browser.countries = demo_countries();
    browser.countries_state = LoadState::Ready;
    browser.countries_fallback = false;
    browser.loaded_once = true;

    let code = previous_country
        .filter(|code| {
            browser
                .countries
                .iter()
                .any(|country| &country.code == code)
        })
        .unwrap_or_else(|| "DE".to_owned());
    browser.country = browser
        .countries
        .iter()
        .find(|country| country.code == code)
        .cloned();

    browser.streaming_services.clear();
    browser.streaming_service = None;
    browser.streaming_state = LoadState::Idle;
    browser.cities.clear();
    browser.cities_state = LoadState::Idle;
    browser.city = None;
    browser.servers.clear();
    browser.servers_state = LoadState::Idle;
    browser.server = None;

    if browser.server_type == ServerType::Streaming {
        demo_fill_streaming(browser);
    }
    demo_fill_cities(browser);
    if let Some(city) = previous_city {
        if let Some(found) = browser
            .cities
            .iter()
            .find(|item| item.name == city)
            .cloned()
        {
            browser.city = Some(found);
        }
    }
    demo_fill_servers(browser);
    if let Some(server) = previous_server {
        if browser.servers.iter().any(|item| item.instance == server) {
            browser.server = browser
                .servers
                .iter()
                .find(|item| item.instance == server)
                .cloned();
        }
    }
}

fn demo_fill_country(browser: &mut NetworkBrowser) {
    if browser.server_type == ServerType::Streaming {
        demo_fill_streaming(browser);
    }
    demo_fill_cities(browser);
}

fn demo_fill_streaming(browser: &mut NetworkBrowser) {
    let Some(country) = browser.country.as_ref() else {
        return;
    };
    browser.streaming_services = demo_streaming_services(&country.code);
    browser.streaming_state = LoadState::Ready;
    browser.streaming_service = browser
        .streaming_services
        .first()
        .map(|service| service.service.clone());
}

fn demo_fill_cities(browser: &mut NetworkBrowser) {
    let Some(country) = browser.country.as_ref() else {
        return;
    };
    browser.cities = demo_cities(&country.code);
    browser.cities_state = LoadState::Ready;
    browser.city = browser.cities.first().cloned();
    browser.servers.clear();
    browser.server = None;
    browser.servers_state = LoadState::Idle;
    demo_fill_servers(browser);
}

fn demo_fill_servers(browser: &mut NetworkBrowser) {
    let Some(country) = browser.country.as_ref() else {
        return;
    };
    let Some(city) = browser.city.as_ref() else {
        browser.servers.clear();
        browser.servers_state = LoadState::Idle;
        browser.server = None;
        return;
    };
    browser.servers = demo_servers(&country.code, &city.name);
    browser.servers_state = LoadState::Ready;
    browser.server = browser.servers.first().cloned();
}

fn demo_countries() -> Vec<Country> {
    [
        ("DE", "Germany"),
        ("JP", "Japan"),
        ("NL", "Netherlands"),
        ("GB", "United Kingdom"),
        ("US", "United States"),
    ]
    .iter()
    .enumerate()
    .map(|(index, (code, name))| Country {
        id: index as u32 + 1,
        name: (*name).to_owned(),
        code: (*code).to_owned(),
    })
    .collect()
}

fn demo_cities(country_code: &str) -> Vec<City> {
    let names: &[&str] = match country_code {
        "DE" => &["Berlin", "Frankfurt"],
        "JP" => &["Osaka", "Tokyo"],
        "NL" => &["Amsterdam", "Rotterdam"],
        "GB" => &["London", "Manchester"],
        "US" => &["Chicago", "New York"],
        _ => &["Sample city"],
    };
    names
        .iter()
        .enumerate()
        .map(|(index, name)| City {
            id: index as u32 + 1,
            name: (*name).to_owned(),
            instances: ["6", "11", "18"][index % 3].to_owned(),
            load: ["18", "34", "51"][index % 3].to_owned(),
        })
        .collect()
}

fn demo_servers(country_code: &str, city: &str) -> Vec<Server> {
    ["14", "39", "63"]
        .iter()
        .enumerate()
        .map(|(index, load)| Server {
            id: index as u32 + 1,
            name: format!("{city} sample {:02}", index + 1),
            instance: format!(
                "demo-{}-{}-s{:02}",
                country_code.to_lowercase(),
                demo_slug(city),
                index + 1
            ),
            load: (*load).to_owned(),
        })
        .collect()
}

fn demo_streaming_services(country_code: &str) -> Vec<StreamingService> {
    ["Sample stream A", "Sample stream B"]
        .iter()
        .enumerate()
        .map(|(index, service)| StreamingService {
            id: index as u32 + 1,
            service: (*service).to_owned(),
            country_code: country_code.to_owned(),
        })
        .collect()
}

fn demo_slug(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn demo_profile(
    id: &str,
    name: &str,
    code: &str,
    country: &str,
    city: &str,
    protocol: Protocol,
    host: &str,
    port: u16,
    requires_auth: bool,
    latency: Option<u32>,
    favorite: bool,
    now: i64,
) -> Profile {
    Profile {
        id: id.to_owned(),
        name: name.to_owned(),
        kind: ProfileKind::Imported,
        country_code: Some(code.to_owned()),
        country: Some(country.to_owned()),
        city: Some(city.to_owned()),
        protocol,
        server_type: ServerType::Traffic,
        network_protocol: NetworkProtocol::Udp,
        streaming_service: None,
        server_instance: None,
        server_load: None,
        remote_host: host.to_owned(),
        remote_port: port,
        requires_auth,
        source_path: format!("demo://{id}"),
        nm_uuid: id.to_owned(),
        imported_at: now - 3 * 86_400,
        favorite,
        latency: latency.map(|ms| Latency {
            ms,
            measured_at: now - 3_600,
        }),
        saved_username: None,
    }
}
