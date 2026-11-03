use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;

use crate::cg::{self, City, Country, Server, StreamingService};
use crate::model::{
    DnsInfo, Latency, LogEntry, LogLevel, Profile, ProfileKind, Stats, TunnelState,
};
use crate::nm::{self, Credentials};
use crate::profile::ParsedProfile;
use crate::store::{self, Paths};

#[derive(Clone, Debug)]
pub enum Command {
    Connect {
        profile: Profile,
        credentials: Option<Credentials>,
        sudo_password: Option<String>,
    },
    CancelConnect,
    Disconnect {
        sudo_password: Option<String>,
    },
    Import {
        path: PathBuf,
        parsed: ParsedProfile,
    },
    Remove(Profile),
    Repair {
        profile: Profile,
        parsed: ParsedProfile,
    },
    MeasureLatency {
        profile_id: String,
        host: String,
        port: u16,
    },
    CheckIp,
    RefreshInventory,
    RefreshDns,
    FlushDns,
    SetAutoReconnect(bool),
    LoadCountries {
        server_type: crate::model::ServerType,
    },
    LoadStreamingServices {
        country_code: String,
    },
    LoadCities {
        server_type: crate::model::ServerType,
        country_code: String,
        streaming_service: Option<String>,
    },
    LoadServers {
        server_type: crate::model::ServerType,
        country_code: String,
        city: String,
        streaming_service: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub enum Event {
    Tunnel(TunnelState),
    Stats(Stats),
    Log(LogEntry),
    Notice {
        level: LogLevel,
        message: String,
    },
    Imported(Profile),
    ImportFailed {
        path: String,
        error: String,
    },
    Removed(String),
    Repaired(Profile),
    Latency {
        profile_id: String,
        result: Result<Latency, String>,
    },
    IpChecked(Result<String, String>),
    Inventory(Vec<String>),
    Dns(DnsInfo),
    Countries {
        server_type: crate::model::ServerType,
        result: Result<Vec<Country>, String>,
    },
    StreamingServices {
        country_code: String,
        result: Result<Vec<StreamingService>, String>,
    },
    Cities {
        server_type: crate::model::ServerType,
        country_code: String,
        streaming_service: Option<String>,
        result: Result<Vec<City>, String>,
    },
    Servers {
        server_type: crate::model::ServerType,
        country_code: String,
        city: String,
        streaming_service: Option<String>,
        result: Result<Vec<Server>, String>,
    },
}

pub struct Backend {
    commands: Sender<Command>,
    events: Receiver<Event>,
}

impl Backend {
    pub fn spawn(paths: Paths, demo: bool, auto_reconnect: bool) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        thread::Builder::new()
            .name("ghostline-backend".to_owned())
            .spawn(move || {
                let mut worker = Worker::new(paths, demo, auto_reconnect, event_tx);
                worker.run(command_rx);
            })
            .expect("backend thread");
        Self {
            commands: command_tx,
            events: event_rx,
        }
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    pub fn try_recv(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

struct ActivationResult {
    generation: u64,
    profile: Profile,
    result: Result<(), String>,
}

struct Worker {
    paths: Paths,
    demo: bool,
    events: Sender<Event>,
    state: TunnelState,
    activation: Option<Receiver<ActivationResult>>,
    activation_started: Option<Instant>,
    generation: u64,
    last_profile: Option<Profile>,
    last_credentials: Option<Credentials>,
    last_sudo_password: Option<String>,
    connected_at: i64,
    active_interface: Option<String>,
    last_stats: Option<(Instant, u64, u64)>,
    last_stats_poll: Instant,
    last_active_poll: Instant,
    missing_polls: u8,
    reconnect_attempts: u8,
    auto_reconnect: bool,
    demo_started: Option<Instant>,
}

impl Worker {
    fn new(paths: Paths, demo: bool, auto_reconnect: bool, events: Sender<Event>) -> Self {
        Self {
            paths,
            demo,
            events,
            state: TunnelState::Disconnected,
            activation: None,
            activation_started: None,
            generation: 0,
            last_profile: None,
            last_credentials: None,
            last_sudo_password: None,
            connected_at: 0,
            active_interface: None,
            last_stats: None,
            last_stats_poll: Instant::now(),
            last_active_poll: Instant::now(),
            missing_polls: 0,
            reconnect_attempts: 0,
            auto_reconnect,
            demo_started: None,
        }
    }

    fn run(&mut self, commands: Receiver<Command>) {
        self.emit(Event::Tunnel(TunnelState::Disconnected));
        if !self.demo && !nm::available() {
            let message =
                "nmcli was not found. Imported profiles are unavailable; CyberGhost network locations still work through the official CLI."
                    .to_owned();
            self.log(LogLevel::Warn, &message);
            self.emit(Event::Notice {
                level: LogLevel::Warn,
                message,
            });
            self.emit(Event::Inventory(Vec::new()));
        } else {
            self.refresh_inventory();
        }
        self.refresh_dns();
        self.detect_external_tunnel();

        loop {
            match commands.recv_timeout(Duration::from_millis(250)) {
                Ok(command) => self.handle(command),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            self.tick();
        }
    }

    fn handle(&mut self, command: Command) {
        match command {
            Command::Connect {
                profile,
                credentials,
                sudo_password,
            } => self.connect(profile, credentials, sudo_password),
            Command::CancelConnect => self.cancel(),
            Command::Disconnect { sudo_password } => self.disconnect(sudo_password),
            Command::Import { path, parsed } => self.import(path, parsed),
            Command::Remove(profile) => self.remove(profile),
            Command::Repair { profile, parsed } => self.repair(profile, parsed),
            Command::MeasureLatency {
                profile_id,
                host,
                port,
            } => self.measure_latency(profile_id, host, port),
            Command::CheckIp => self.check_ip(),
            Command::RefreshInventory => self.refresh_inventory(),
            Command::RefreshDns => self.refresh_dns(),
            Command::FlushDns => self.flush_dns(),
            Command::SetAutoReconnect(enabled) => {
                self.auto_reconnect = enabled;
            }
            Command::LoadCountries { server_type } => self.load_countries(server_type),
            Command::LoadStreamingServices { country_code } => {
                self.load_streaming_services(country_code)
            }
            Command::LoadCities {
                server_type,
                country_code,
                streaming_service,
            } => self.load_cities(server_type, country_code, streaming_service),
            Command::LoadServers {
                server_type,
                country_code,
                city,
                streaming_service,
            } => self.load_servers(server_type, country_code, city, streaming_service),
        }
    }

    fn connect(
        &mut self,
        profile: Profile,
        credentials: Option<Credentials>,
        sudo_password: Option<String>,
    ) {
        if self.demo {
            self.last_profile = Some(profile.clone());
            self.connected_at = now();
            self.active_interface = Some("demo0".to_owned());
            self.last_stats = None;
            self.demo_started = Some(Instant::now());
            self.reconnect_attempts = 0;
            self.set_state(TunnelState::Connecting {
                profile_id: profile.id.clone(),
            });
            self.start_activation(profile, credentials, sudo_password, Duration::ZERO);
            return;
        }

        if self.state.is_connected() || self.state.is_busy() {
            self.log(
                LogLevel::Warn,
                "A tunnel is already active; disconnect before connecting again.",
            );
            return;
        }

        self.reconnect_attempts = 0;
        self.last_profile = Some(profile.clone());
        self.last_credentials = credentials.clone();
        self.last_sudo_password = sudo_password.clone();
        self.set_state(TunnelState::Connecting {
            profile_id: profile.id.clone(),
        });
        let route = if profile.is_cyberghost() {
            format!(
                "{} through the CyberGhost CLI ({})",
                profile.name,
                profile.network_label()
            )
        } else {
            format!(
                "{} ({}, {})",
                profile.name,
                profile.protocol.label(),
                profile.endpoint()
            )
        };
        self.log(LogLevel::Info, &format!("Connecting to {route}."));
        self.start_activation(profile, credentials, sudo_password, Duration::ZERO);
    }

    fn start_activation(
        &mut self,
        profile: Profile,
        credentials: Option<Credentials>,
        sudo_password: Option<String>,
        delay: Duration,
    ) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        let (sender, receiver) = mpsc::channel();
        self.activation = Some(receiver);
        self.activation_started = Some(Instant::now());
        let paths = self.paths.clone();
        let demo = self.demo;
        thread::Builder::new()
            .name("ghostline-activate".to_owned())
            .spawn(move || {
                if !delay.is_zero() {
                    thread::sleep(delay);
                }
                let result = if demo {
                    thread::sleep(Duration::from_millis(1100));
                    Ok(())
                } else if profile.is_cyberghost() {
                    let mut outcome = cg::connect(&profile, sudo_password.as_deref());
                    if outcome.is_ok() {
                        let mut up = false;
                        for _ in 0..40 {
                            match cg::status() {
                                Ok(true) => {
                                    up = true;
                                    break;
                                }
                                Ok(false) => thread::sleep(Duration::from_millis(500)),
                                Err(error) => {
                                    outcome = Err(error);
                                    break;
                                }
                            }
                        }
                        if outcome.is_ok() && !up {
                            outcome = Err(
                                "The CyberGhost CLI finished, but no tunnel was detected. Check the Activity log and the official CLI."
                                    .to_owned(),
                            );
                        }
                    }
                    outcome
                } else {
                    nm::activate(&profile.nm_uuid, credentials.as_ref(), &paths.runtime_dir)
                };
                let _ = sender.send(ActivationResult {
                    generation,
                    profile,
                    result,
                });
            })
            .ok();
    }

    fn cancel(&mut self) {
        if !self.state.is_busy() {
            return;
        }
        self.generation = self.generation.wrapping_add(1);
        self.activation = None;
        let cyber = self
            .last_profile
            .as_ref()
            .map(Profile::is_cyberghost)
            .unwrap_or(false);
        if cyber {
            if !self.demo {
                if let Err(error) = cg::stop(self.last_sudo_password.as_deref()) {
                    self.log(
                        LogLevel::Warn,
                        &format!("Could not stop the CyberGhost CLI attempt: {error}"),
                    );
                }
            }
        } else if let Some(profile) = self.last_profile.clone() {
            if !self.demo {
                let _ = nm::deactivate(&profile.nm_uuid);
            }
        }
        self.active_interface = None;
        self.last_stats = None;
        self.set_state(TunnelState::Disconnected);
        self.log(LogLevel::Info, "Connection attempt cancelled.");
    }

    fn disconnect(&mut self, sudo_password: Option<String>) {
        let external = matches!(self.state, TunnelState::External { .. });
        let Some(profile) = self.last_profile.clone() else {
            if external {
                self.set_state(TunnelState::Disconnecting);
                let result = if self.demo {
                    Ok(())
                } else {
                    cg::stop(
                        sudo_password
                            .as_deref()
                            .or(self.last_sudo_password.as_deref()),
                    )
                };
                self.active_interface = None;
                self.last_stats = None;
                match result {
                    Ok(()) => {
                        self.set_state(TunnelState::Disconnected);
                        self.log(LogLevel::Info, "Stopped the active CyberGhost CLI tunnel.");
                    }
                    Err(error) => {
                        self.set_state(TunnelState::Error {
                            profile_id: None,
                            message: error.clone(),
                        });
                        self.log(LogLevel::Error, &error);
                    }
                }
            } else {
                self.set_state(TunnelState::Disconnected);
            }
            return;
        };
        if sudo_password.is_some() {
            self.last_sudo_password = sudo_password.clone();
        }
        self.set_state(TunnelState::Disconnecting);
        self.generation = self.generation.wrapping_add(1);
        self.activation = None;
        let result = if self.demo {
            Ok(())
        } else if profile.is_cyberghost() {
            cg::stop(
                sudo_password
                    .as_deref()
                    .or(self.last_sudo_password.as_deref()),
            )
        } else {
            nm::deactivate(&profile.nm_uuid)
        };
        self.active_interface = None;
        self.last_stats = None;
        self.missing_polls = 0;
        match result {
            Ok(()) => {
                self.set_state(TunnelState::Disconnected);
                self.log(
                    LogLevel::Info,
                    &format!("Disconnected from {}.", profile.name),
                );
            }
            Err(error) => {
                self.set_state(TunnelState::Error {
                    profile_id: Some(profile.id.clone()),
                    message: error.clone(),
                });
                self.log(LogLevel::Error, &error);
            }
        }
    }

    fn import(&mut self, path: PathBuf, parsed: ParsedProfile) {
        if self.demo {
            self.emit(Event::Notice {
                level: LogLevel::Warn,
                message: "Import is disabled while demo data is shown.".to_owned(),
            });
            return;
        }
        let path_string = path.to_string_lossy().into_owned();
        match nm::import_profile(&path, parsed.protocol, &parsed.name) {
            Ok(imported) => {
                let profile = Profile {
                    id: stable_id(&path_string, &parsed.name, &parsed.remote_host),
                    name: parsed.name.clone(),
                    kind: ProfileKind::Imported,
                    country_code: parsed.country_code.clone(),
                    country: parsed.country.clone(),
                    city: parsed.city.clone(),
                    protocol: parsed.protocol,
                    server_type: crate::model::ServerType::Traffic,
                    network_protocol: crate::model::NetworkProtocol::Udp,
                    streaming_service: None,
                    server_instance: None,
                    server_load: None,
                    remote_host: parsed.remote_host.clone(),
                    remote_port: parsed.remote_port,
                    requires_auth: parsed.requires_auth,
                    source_path: path_string,
                    nm_uuid: imported.uuid,
                    imported_at: now(),
                    favorite: false,
                    latency: None,
                    saved_username: None,
                };
                self.log(
                    LogLevel::Info,
                    &format!("Imported {} as {}.", profile.name, profile.protocol.label()),
                );
                self.emit(Event::Imported(profile));
                self.refresh_inventory();
            }
            Err(error) => {
                self.log(
                    LogLevel::Error,
                    &format!("Import failed for {path_string}: {error}"),
                );
                self.emit(Event::ImportFailed {
                    path: path_string,
                    error,
                });
            }
        }
    }

    fn repair(&mut self, profile: Profile, parsed: ParsedProfile) {
        if profile.is_cyberghost() {
            self.emit(Event::Notice {
                level: LogLevel::Warn,
                message: "CyberGhost network locations do not have a NetworkManager connection to recreate.".to_owned(),
            });
            return;
        }
        if self
            .state
            .profile_id()
            .map(|id| id == profile.id)
            .unwrap_or(false)
            || self.state.is_busy()
        {
            self.emit(Event::Notice {
                level: LogLevel::Warn,
                message: "Finish the active connection before recreating this profile.".to_owned(),
            });
            return;
        }
        if self.demo {
            self.emit(Event::Notice {
                level: LogLevel::Warn,
                message: "Recreating NetworkManager connections is disabled in demo mode."
                    .to_owned(),
            });
            return;
        }
        let path = PathBuf::from(&profile.source_path);
        if !path.is_file() {
            let error = format!(
                "The original file {} is no longer available. Import it again.",
                profile.source_path
            );
            self.log(LogLevel::Error, &error);
            self.emit(Event::Notice {
                level: LogLevel::Error,
                message: error,
            });
            return;
        }
        match nm::import_profile(&path, parsed.protocol, &profile.name) {
            Ok(imported) => {
                let old_uuid = profile.nm_uuid.clone();
                let was_saved = profile.saved_username.is_some();
                let mut updated = profile;
                updated.name = parsed.name;
                updated.country_code = parsed.country_code;
                updated.country = parsed.country;
                updated.city = parsed.city;
                updated.protocol = parsed.protocol;
                updated.remote_host = parsed.remote_host;
                updated.remote_port = parsed.remote_port;
                updated.requires_auth = parsed.requires_auth;
                updated.nm_uuid = imported.uuid.clone();
                updated.imported_at = now();
                updated.latency = None;
                if !updated.requires_auth {
                    updated.saved_username = None;
                    if was_saved {
                        let _ = store::secret_clear(&updated.id);
                    }
                }
                if old_uuid != imported.uuid {
                    if let Err(error) = nm::delete_connection(&old_uuid) {
                        self.log(
                            LogLevel::Warn,
                            &format!(
                                "The previous NetworkManager connection could not be deleted: {error}"
                            ),
                        );
                    }
                }
                self.log(
                    LogLevel::Info,
                    &format!(
                        "Recreated the NetworkManager connection for {}.",
                        updated.name
                    ),
                );
                self.emit(Event::Repaired(updated));
                self.refresh_inventory();
            }
            Err(error) => {
                self.log(LogLevel::Error, &format!("Repair failed: {error}"));
                self.emit(Event::Notice {
                    level: LogLevel::Error,
                    message: error,
                });
            }
        }
    }

    fn remove(&mut self, profile: Profile) {
        if self
            .state
            .profile_id()
            .map(|id| id == profile.id)
            .unwrap_or(false)
        {
            self.disconnect(None);
        }
        let result = if self.demo || profile.is_cyberghost() {
            Ok(())
        } else {
            nm::delete_connection(&profile.nm_uuid)
        };
        match result {
            Ok(()) => {
                let _ = store::secret_clear(&profile.id);
                self.log(
                    LogLevel::Info,
                    &format!(
                        "Removed {}{}.",
                        profile.name,
                        if profile.is_cyberghost() {
                            ""
                        } else {
                            " and its NetworkManager connection"
                        }
                    ),
                );
                self.emit(Event::Removed(profile.id));
                self.refresh_inventory();
            }
            Err(error) => {
                self.log(
                    LogLevel::Error,
                    &format!("Could not remove {}: {error}", profile.name),
                );
                self.emit(Event::Notice {
                    level: LogLevel::Error,
                    message: error,
                });
            }
        }
    }

    fn measure_latency(&mut self, profile_id: String, host: String, port: u16) {
        if host.is_empty() || port == 0 {
            self.emit(Event::Latency {
                profile_id,
                result: Err(
                    "This location does not expose a direct endpoint for a latency check."
                        .to_owned(),
                ),
            });
            return;
        }
        if self.demo {
            let events = self.events.clone();
            let ms = demo_latency(&profile_id);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(450));
                let _ = events.send(Event::Latency {
                    profile_id,
                    result: Ok(Latency {
                        ms,
                        measured_at: now(),
                    }),
                });
            });
            return;
        }
        let events = self.events.clone();
        thread::spawn(move || {
            let result = nm::measure_latency(&host, port).map(|ms| Latency {
                ms,
                measured_at: now(),
            });
            let _ = events.send(Event::Latency { profile_id, result });
        });
    }

    fn check_ip(&mut self) {
        let events = self.events.clone();
        if self.demo {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(500));
                let _ = events.send(Event::IpChecked(Ok("203.0.113.42".to_owned())));
            });
            return;
        }
        thread::spawn(move || {
            let _ = events.send(Event::IpChecked(nm::check_exit_ip()));
        });
    }

    fn refresh_inventory(&mut self) {
        if self.demo {
            self.emit(Event::Inventory(
                ["demo-de", "demo-nl", "demo-us", "demo-jp"]
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
            ));
            return;
        }
        match nm::connection_uuids() {
            Ok(connections) => {
                let uuids = connections.into_iter().map(|(uuid, _)| uuid).collect();
                self.emit(Event::Inventory(uuids));
            }
            Err(error) => {
                self.log(
                    LogLevel::Warn,
                    &format!("Could not read NetworkManager connections: {error}"),
                );
                self.emit(Event::Inventory(Vec::new()));
            }
        }
    }

    fn refresh_dns(&mut self) {
        if self.demo {
            self.emit(Event::Dns(DnsInfo {
                servers: vec!["192.0.2.53".to_owned(), "192.0.2.54".to_owned()],
                interface: "demo0".to_owned(),
                nm_version: String::new(),
            }));
            return;
        }
        let info = nm::dns_info();
        self.emit(Event::Dns(info));
    }

    fn detect_external_tunnel(&mut self) {
        if self.demo || !cg::available() {
            return;
        }
        match cg::status() {
            Ok(true) => {
                self.connected_at = now();
                self.active_interface = nm::vpn_interface().or_else(|| Some("tun".to_owned()));
                self.last_stats = None;
                self.set_state(TunnelState::External {
                    iface: self
                        .active_interface
                        .clone()
                        .unwrap_or_else(|| "tun".to_owned()),
                    since: self.connected_at,
                });
                self.log(
                    LogLevel::Info,
                    "A VPN tunnel is already active outside Ghostline. The status check will keep watching it.",
                );
            }
            Ok(false) => {}
            Err(error) => {
                self.log(
                    LogLevel::Warn,
                    &format!("Could not check the CyberGhost CLI status: {error}"),
                );
            }
        }
    }

    fn load_countries(&mut self, server_type: crate::model::ServerType) {
        if self.demo {
            self.emit(Event::Countries {
                server_type,
                result: Ok(Vec::new()),
            });
            return;
        }
        let events = self.events.clone();
        thread::Builder::new()
            .name("ghostline-countries".to_owned())
            .spawn(move || {
                let result = cg::list_countries(server_type);
                let _ = events.send(Event::Countries {
                    server_type,
                    result,
                });
            })
            .ok();
    }

    fn load_streaming_services(&mut self, country_code: String) {
        let events = self.events.clone();
        thread::Builder::new()
            .name("ghostline-streaming".to_owned())
            .spawn(move || {
                let result = cg::list_streaming_services(&country_code);
                let _ = events.send(Event::StreamingServices {
                    country_code,
                    result,
                });
            })
            .ok();
    }

    fn load_cities(
        &mut self,
        server_type: crate::model::ServerType,
        country_code: String,
        streaming_service: Option<String>,
    ) {
        let events = self.events.clone();
        thread::Builder::new()
            .name("ghostline-cities".to_owned())
            .spawn(move || {
                let result =
                    cg::list_cities(server_type, &country_code, streaming_service.as_deref());
                let _ = events.send(Event::Cities {
                    server_type,
                    country_code,
                    streaming_service,
                    result,
                });
            })
            .ok();
    }

    fn load_servers(
        &mut self,
        server_type: crate::model::ServerType,
        country_code: String,
        city: String,
        streaming_service: Option<String>,
    ) {
        let events = self.events.clone();
        thread::Builder::new()
            .name("ghostline-servers".to_owned())
            .spawn(move || {
                let result = cg::list_servers(
                    server_type,
                    &country_code,
                    &city,
                    streaming_service.as_deref(),
                );
                let _ = events.send(Event::Servers {
                    server_type,
                    country_code,
                    city,
                    streaming_service,
                    result,
                });
            })
            .ok();
    }

    fn flush_dns(&mut self) {
        if self.demo {
            self.log(LogLevel::Info, "Demo mode simulated a DNS cache flush.");
            self.emit(Event::Notice {
                level: LogLevel::Info,
                message: "DNS cache flush simulated. No system resolver was touched.".to_owned(),
            });
            return;
        }
        match nm::flush_dns() {
            Ok(()) => {
                self.log(LogLevel::Info, "Flushed the system DNS cache.");
                self.emit(Event::Notice {
                    level: LogLevel::Info,
                    message: "DNS cache flushed.".to_owned(),
                });
            }
            Err(error) => {
                self.log(LogLevel::Error, &error);
                self.emit(Event::Notice {
                    level: LogLevel::Error,
                    message: error,
                });
            }
        }
    }

    fn tick(&mut self) {
        self.poll_activation();
        self.poll_connection();
        self.poll_stats();
    }

    fn poll_activation(&mut self) {
        let Some(receiver) = &self.activation else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.activation = None;
                self.activation_started = None;
                if result.generation != self.generation {
                    return;
                }
                if self.demo {
                    if matches!(self.state, TunnelState::Connecting { .. }) {
                        self.connected_at = now();
                        self.last_stats = None;
                        self.set_state(TunnelState::Connected {
                            profile_id: result.profile.id.clone(),
                            iface: "demo0".to_owned(),
                            since: self.connected_at,
                        });
                        self.log(
                            LogLevel::Info,
                            "Demo tunnel is up. No traffic is being routed.",
                        );
                    }
                    return;
                }
                match result.result {
                    Ok(()) => {
                        let interface = if result.profile.is_cyberghost() {
                            nm::vpn_interface()
                                .or_else(|| Some("cyberghost".to_owned()))
                                .unwrap_or_else(|| "tun".to_owned())
                        } else {
                            nm::active_connections()
                                .ok()
                                .and_then(|connections| {
                                    connections
                                        .into_iter()
                                        .find(|(uuid, _, _)| uuid == &result.profile.nm_uuid)
                                        .map(|(_, _, device)| device)
                                })
                                .filter(|device| !device.is_empty())
                                .or_else(nm::vpn_interface)
                                .unwrap_or_else(|| "tun".to_owned())
                        };
                        self.connected_at = now();
                        self.active_interface = Some(interface.clone());
                        self.last_stats = None;
                        self.missing_polls = 0;
                        self.reconnect_attempts = 0;
                        self.set_state(TunnelState::Connected {
                            profile_id: result.profile.id.clone(),
                            iface: interface,
                            since: self.connected_at,
                        });
                        self.log(
                            LogLevel::Info,
                            &format!("Tunnel is up through {}.", result.profile.name),
                        );
                    }
                    Err(error) => {
                        if self.reconnect_attempts > 0 && self.reconnect_attempts < 3 {
                            self.schedule_reconnect(error);
                        } else {
                            self.log(LogLevel::Error, &error);
                            self.set_state(TunnelState::Error {
                                profile_id: Some(result.profile.id.clone()),
                                message: error,
                            });
                        }
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                if let Some(started) = self.activation_started {
                    let limit = if self
                        .last_profile
                        .as_ref()
                        .map(Profile::is_cyberghost)
                        .unwrap_or(false)
                    {
                        Duration::from_secs(120)
                    } else {
                        Duration::from_secs(70)
                    };
                    if started.elapsed() > limit {
                        self.activation = None;
                        self.activation_started = None;
                        let message = if limit.as_secs() == 70 {
                            "NetworkManager did not finish the connection attempt in 70 seconds."
                                .to_owned()
                        } else {
                            "The CyberGhost CLI did not finish the connection attempt in 120 seconds."
                                .to_owned()
                        };
                        self.log(LogLevel::Error, &message);
                        self.set_state(TunnelState::Error {
                            profile_id: self
                                .last_profile
                                .as_ref()
                                .map(|profile| profile.id.clone()),
                            message,
                        });
                    }
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.activation = None;
                self.activation_started = None;
            }
        }
    }

    fn poll_connection(&mut self) {
        if !self.state.is_connected() {
            return;
        }
        if self.last_active_poll.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.last_active_poll = Instant::now();

        if let TunnelState::External { iface, since } = self.state.clone() {
            match cg::status() {
                Ok(false) => {
                    self.missing_polls = self.missing_polls.saturating_add(1);
                    if self.missing_polls >= 2 {
                        self.missing_polls = 0;
                        self.active_interface = None;
                        self.last_stats = None;
                        self.set_state(TunnelState::Disconnected);
                        self.log(
                            LogLevel::Info,
                            "The tunnel started outside Ghostline is no longer active.",
                        );
                    }
                }
                Ok(true) => {
                    self.missing_polls = 0;
                    if let Some(current) = nm::vpn_interface() {
                        if current != iface {
                            self.active_interface = Some(current.clone());
                            self.last_stats = None;
                            self.set_state(TunnelState::External {
                                iface: current,
                                since,
                            });
                        }
                    }
                }
                Err(_) => {}
            }
            return;
        }

        let Some(profile) = self.last_profile.clone() else {
            return;
        };
        let found = if self.demo {
            Some("demo0".to_owned())
        } else if profile.is_cyberghost() {
            match cg::status() {
                Ok(true) => nm::vpn_interface().or_else(|| Some("cyberghost".to_owned())),
                Ok(false) => None,
                Err(error) => {
                    self.missing_polls = self.missing_polls.saturating_add(1);
                    if self.missing_polls == 3 {
                        self.log(
                            LogLevel::Warn,
                            &format!("Could not read the CyberGhost tunnel status: {error}"),
                        );
                    }
                    return;
                }
            }
        } else {
            nm::active_connections().ok().and_then(|connections| {
                connections
                    .into_iter()
                    .find(|(uuid, _, _)| uuid == &profile.nm_uuid)
                    .map(|(_, _, device)| device)
            })
        };

        match found {
            Some(device) => {
                self.missing_polls = 0;
                if !device.is_empty() {
                    if self.active_interface.as_deref() != Some(device.as_str()) {
                        self.active_interface = Some(device.clone());
                        self.last_stats = None;
                        if let TunnelState::Connected {
                            profile_id, since, ..
                        } = self.state.clone()
                        {
                            self.set_state(TunnelState::Connected {
                                profile_id,
                                iface: device,
                                since,
                            });
                        }
                    }
                }
            }
            None => {
                self.missing_polls = self.missing_polls.saturating_add(1);
                if self.missing_polls >= 2 {
                    self.missing_polls = 0;
                    self.active_interface = None;
                    self.last_stats = None;
                    self.log(
                        LogLevel::Warn,
                        if profile.is_cyberghost() {
                            "The CyberGhost CLI reports the tunnel is no longer active."
                        } else {
                            "NetworkManager reports the tunnel is no longer active."
                        },
                    );
                    if self.auto_reconnect && self.reconnect_attempts < 3 {
                        self.schedule_reconnect(
                            "The tunnel dropped. Ghostline is trying again.".to_owned(),
                        );
                    } else {
                        self.set_state(TunnelState::Error {
                            profile_id: Some(profile.id.clone()),
                            message: if profile.is_cyberghost() {
                                "The tunnel dropped. The CyberGhost CLI no longer reports it as active."
                                    .to_owned()
                            } else {
                                "The tunnel dropped. NetworkManager no longer reports it as active."
                                    .to_owned()
                            },
                        });
                    }
                }
            }
        }
    }

    fn schedule_reconnect(&mut self, _context: String) {
        let Some(profile) = self.last_profile.clone() else {
            return;
        };
        self.reconnect_attempts = self.reconnect_attempts.saturating_add(1);
        let attempt = self.reconnect_attempts;
        self.set_state(TunnelState::Reconnecting {
            profile_id: profile.id.clone(),
            attempt,
        });
        self.log(
            LogLevel::Warn,
            &format!("Tunnel dropped; reconnect attempt {attempt} of 3."),
        );
        let delay = Duration::from_secs(1 << (attempt.saturating_sub(1)));
        self.start_activation(
            profile,
            self.last_credentials.clone(),
            self.last_sudo_password.clone(),
            delay,
        );
    }

    fn poll_stats(&mut self) {
        if !self.state.is_connected() {
            return;
        }
        if self.last_stats_poll.elapsed() < Duration::from_millis(800) {
            return;
        }
        self.last_stats_poll = Instant::now();

        let stats = if self.demo {
            let started = self.demo_started.unwrap_or_else(Instant::now);
            let seconds = started.elapsed().as_secs_f64();
            Some(Stats {
                rx_total: (seconds * 384_000.0) as u64,
                tx_total: (seconds * 42_000.0) as u64,
                rx_rate: 384_000.0,
                tx_rate: 42_000.0,
            })
        } else {
            self.active_interface.as_deref().and_then(nm::stats_for)
        };

        let Some(mut stats) = stats else {
            return;
        };
        if let Some((last_at, last_rx, last_tx)) = self.last_stats {
            let elapsed = last_at.elapsed().as_secs_f64();
            if elapsed > 0.0 && stats.rx_total >= last_rx && stats.tx_total >= last_tx {
                stats.rx_rate = (stats.rx_total - last_rx) as f64 / elapsed;
                stats.tx_rate = (stats.tx_total - last_tx) as f64 / elapsed;
            }
        }
        self.last_stats = Some((Instant::now(), stats.rx_total, stats.tx_total));
        self.emit(Event::Stats(stats));
    }

    fn set_state(&mut self, state: TunnelState) {
        self.state = state.clone();
        self.emit(Event::Tunnel(state));
    }

    fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }

    fn log(&self, level: LogLevel, message: &str) {
        store::append_log(&self.paths, level, message);
        self.emit(Event::Log(LogEntry {
            at: now(),
            level,
            message: message.to_owned(),
        }));
    }
}

pub fn stable_id(path: &str, name: &str, remote: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!("{path}|{name}|{remote}").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("p_{hash:016x}")
}

fn now() -> i64 {
    Local::now().timestamp()
}

fn demo_latency(profile_id: &str) -> u32 {
    match profile_id {
        "demo-de" => 24,
        "demo-nl" => 31,
        "demo-us" => 92,
        "demo-jp" => 168,
        _ => 52,
    }
}
