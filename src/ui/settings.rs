use egui::Ui;

use crate::app::{Action, App, APP_NAME, APP_VERSION};
use crate::format;
use crate::model::{NetworkProtocol, Protocol, ServerType, ThemeMode};
use crate::theme::Palette;

use super::widgets::{self, Btn, Icon};

pub fn show(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    let p = app.palette;
    let settings = app.state.settings.clone();
    let nm_ready = app.nm_ready;
    let nm_version = app.dns.nm_version.clone();
    let dns_servers = if app.dns.servers.is_empty() {
        "No DNS resolvers reported".to_owned()
    } else {
        format::shorten(&app.dns.servers.join(", "), 62)
    };
    let dns_interface = app.dns.interface.clone();
    let data_dir = app.paths.data_dir.to_string_lossy().into_owned();
    let log_path = app.paths.log_file.to_string_lossy().into_owned();
    let log_count = app.logs.len();
    let demo = app.demo;
    let cg_ready = app.cg_ready;

    widgets::page_header_plain(
        ui,
        &p,
        "Settings",
        "Theme, startup behavior, defaults for network locations, and the network details Ghostline can see.",
    );

    if demo {
        widgets::note(
            ui,
            &p,
            "Demo mode",
            "These controls change sample state only. Real settings, logs, and autostart entries are left untouched.",
            p.accent,
        );
        ui.add_space(16.0);
    }

    let width = ui.available_width().min(720.0);
    ui.vertical(|ui| {
            ui.set_width(width);

            widgets::section_title(ui, &p, "Appearance");
            widgets::setting_row(
                ui,
                &p,
                "Theme",
                "System follows the desktop color scheme. Both themes are built and tested.",
                240.0,
                |ui| {
                    if let Some(next) = widgets::segmented(
                        ui,
                        &p,
                        "theme",
                        &[
                            (ThemeMode::System, "System"),
                            (ThemeMode::Dark, "Dark"),
                            (ThemeMode::Light, "Light"),
                        ],
                        settings.theme,
                    ) {
                        if next != settings.theme {
                            actions.push(Action::SetTheme(next));
                        }
                    }
                },
            );
            ui.add_space(12.0);
            widgets::hline(ui, &p);

            widgets::section_title(ui, &p, "Startup and connection");
            widgets::setting_row(
                ui,
                &p,
                "Launch at login",
                "Write an XDG autostart entry for this executable.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "launch-at-login",
                        settings.launch_at_login,
                        "Launch at login",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetLaunchAtLogin(!settings.launch_at_login));
                    }
                },
            );
            widgets::setting_row(
                ui,
                &p,
                "Auto-connect on launch",
                "Connect to the selected location when Ghostline starts. A CyberGhost location may ask for the administrator password.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "auto-connect",
                        settings.auto_connect,
                        "Auto-connect on launch",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetAutoConnect(!settings.auto_connect));
                    }
                },
            );
            widgets::setting_row(
                ui,
                &p,
                "Load the last location",
                "Remember the last selection so Ghostline opens with the same location ready.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "load-last-profile",
                        settings.load_last_profile,
                        "Load the last location",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetLoadLastProfile(!settings.load_last_profile));
                    }
                },
            );
            widgets::setting_row(
                ui,
                &p,
                "Auto-reconnect",
                "Ask the active backend to try again, up to three times, when it reports the tunnel dropped.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "auto-reconnect",
                        settings.auto_reconnect,
                        "Auto-reconnect",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetAutoReconnect(!settings.auto_reconnect));
                    }
                },
            );
            widgets::setting_row(
                ui,
                &p,
                "Disconnect on quit",
                "Stop a tunnel that Ghostline opened when it exits. A tunnel started outside Ghostline is left alone.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "disconnect-on-exit",
                        settings.disconnect_on_exit,
                        "Disconnect on quit",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetDisconnectOnExit(!settings.disconnect_on_exit));
                    }
                },
            );
            ui.add_space(12.0);
            widgets::hline(ui, &p);

            widgets::section_title(ui, &p, "New network selections");
            widgets::caption(
                ui,
                &p,
                "These prefill the Network screen at startup. Each saved location keeps the choices it was saved with.",
            );
            ui.add_space(8.0);
            widgets::setting_row(
                ui,
                &p,
                "Server type",
                "Traffic, streaming, or torrent. Streaming choices also ask for a service before cities load.",
                300.0,
                |ui| {
                    if let Some(next) = widgets::segmented(
                        ui,
                        &p,
                        "default-server-type",
                        &[
                            (ServerType::Traffic, "Traffic"),
                            (ServerType::Streaming, "Streaming"),
                            (ServerType::Torrent, "Torrent"),
                        ],
                        settings.default_server_type,
                    ) {
                        if next != settings.default_server_type {
                            actions.push(Action::SetDefaultServerType(next));
                        }
                    }
                },
            );
            widgets::setting_row(
                ui,
                &p,
                "VPN protocol",
                "OpenVPN or WireGuard for a new CyberGhost location.",
                220.0,
                |ui| {
                    if let Some(next) = widgets::segmented(
                        ui,
                        &p,
                        "default-protocol",
                        &[
                            (Protocol::OpenVpn, "OpenVPN"),
                            (Protocol::WireGuard, "WireGuard"),
                        ],
                        settings.default_protocol,
                    ) {
                        if next != settings.default_protocol {
                            actions.push(Action::SetDefaultProtocol(next));
                        }
                    }
                },
            );
            widgets::setting_row(
                ui,
                &p,
                "Network protocol",
                "UDP or TCP when OpenVPN is selected. WireGuard ignores this choice.",
                190.0,
                |ui| {
                    if let Some(next) = widgets::segmented(
                        ui,
                        &p,
                        "default-network-protocol",
                        &[
                            (NetworkProtocol::Udp, "UDP"),
                            (NetworkProtocol::Tcp, "TCP"),
                        ],
                        settings.default_network_protocol,
                    ) {
                        if next != settings.default_network_protocol {
                            actions.push(Action::SetDefaultNetworkProtocol(next));
                        }
                    }
                },
            );
            ui.add_space(12.0);
            widgets::hline(ui, &p);

            widgets::section_title(ui, &p, "Locations");
            widgets::setting_row(
                ui,
                &p,
                "Compact profile rows",
                "Fit more Locations on screen by tightening each row.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "compact-rows",
                        settings.compact_rows,
                        "Compact profile rows",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetCompactRows(!settings.compact_rows));
                    }
                },
            );
            ui.add_space(12.0);
            widgets::hline(ui, &p);

            widgets::section_title(ui, &p, "Activity and storage");
            widgets::setting_row(
                ui,
                &p,
                "Log retention",
                "Logs older than this are removed when Ghostline starts. Clear the log at any time.",
                262.0,
                |ui| {
                    if let Some(next) = widgets::segmented(
                        ui,
                        &p,
                        "log-retention",
                        &[
                            (7u32, "7 days"),
                            (30, "30 days"),
                            (90, "90 days"),
                            (365, "1 year"),
                        ],
                        settings.log_retention_days,
                    ) {
                        if next != settings.log_retention_days {
                            actions.push(Action::SetLogRetention(next));
                        }
                    }
                },
            );
            ui.add_space(2.0);
            setting_caption(
                ui,
                &p,
                &format!("Log file: {}", format::shorten_path(&log_path, 64)),
            );
            ui.add_space(14.0);
            widgets::setting_row(
                ui,
                &p,
                "Data folder",
                "Settings, logs, and runtime files. Credentials are not stored here.",
                150.0,
                |ui| {
                    if widgets::button(
                        ui,
                        &p,
                        Btn::secondary("Open folder").icon(Icon::Folder),
                    )
                    .clicked()
                    {
                        actions.push(Action::OpenDataFolder);
                    }
                },
            );
            setting_caption(ui, &p, &format::shorten_path(&data_dir, 64));
            ui.add_space(14.0);
            widgets::setting_row(
                ui,
                &p,
                "Activity log",
                &format!(
                    "{} currently shown. This deletes the local log file.",
                    format::count(log_count, "entry", "entries")
                ),
                130.0,
                |ui| {
                    if widgets::button(
                        ui,
                        &p,
                        Btn::ghost_danger("Clear log").icon(Icon::Trash),
                    )
                    .clicked()
                    {
                        actions.push(Action::RequestClearLogs);
                    }
                },
            );
            ui.add_space(12.0);
            widgets::hline(ui, &p);

            widgets::section_title(ui, &p, "Network");
            let cg_summary = if demo {
                "Demo mode simulates the CyberGhost CLI. No cyberghostvpn call is made."
                    .to_owned()
            } else if cg_ready {
                "cyberghostvpn responds. Account sign-in and the server catalogue stay in the official CLI."
                    .to_owned()
            } else {
                "cyberghostvpn was not found. Install the official client before browsing network locations."
                    .to_owned()
            };
            widgets::setting_row(
                ui,
                &p,
                "CyberGhost CLI",
                &cg_summary,
                178.0,
                |ui| {
                    if demo {
                        widgets::tag(ui, &p, "Demo", p.accent);
                    } else {
                        widgets::tag(
                            ui,
                            &p,
                            if cg_ready { "Available" } else { "Missing" },
                            if cg_ready { p.ok } else { p.danger },
                        );
                    }
                    if widgets::button(ui, &p, Btn::secondary("Refresh").height(34.0)).clicked() {
                        actions.push(Action::RefreshNetwork);
                    }
                },
            );
            ui.add_space(4.0);
            let nm_summary: String = if demo {
                "Demo mode simulates NetworkManager. No nmcli call is made.".to_owned()
            } else if nm_ready {
                if nm_version.is_empty() {
                    "nmcli responds. VPN plugins are required for OpenVPN and WireGuard."
                        .to_owned()
                } else {
                    format!("nmcli {nm_version}.")
                }
            } else {
                "nmcli was not found. CyberGhost network locations still work through the official CLI."
                    .to_owned()
            };
            widgets::setting_row(
                ui,
                &p,
                "NetworkManager",
                &nm_summary,
                178.0,
                |ui| {
                    if demo {
                        widgets::tag(ui, &p, "Demo", p.accent);
                    } else {
                        widgets::tag(
                            ui,
                            &p,
                            if nm_ready { "Available" } else { "Missing" },
                            if nm_ready { p.ok } else { p.danger },
                        );
                    }
                    if widgets::button(ui, &p, Btn::secondary("Refresh").height(34.0)).clicked() {
                        actions.push(Action::RefreshNetwork);
                    }
                },
            );
            ui.add_space(4.0);
            widgets::setting_row(
                ui,
                &p,
                "DNS resolvers",
                &format!(
                    "{dns_servers}. Reported on {dns_interface}. Flushing only touches the local resolver cache."
                ),
                126.0,
                |ui| {
                    if widgets::button(ui, &p, Btn::secondary("Flush DNS").height(34.0)).clicked() {
                        actions.push(Action::FlushDns);
                    }
                },
            );
            ui.add_space(4.0);
            widgets::setting_row(
                ui,
                &p,
                "Keep the administrator password",
                "Keep the sudo password in memory for this session so reconnects and disconnects do not ask again. It is never written to disk.",
                52.0,
                |ui| {
                    if widgets::toggle(
                        ui,
                        &p,
                        "keep-sudo-password",
                        settings.keep_sudo_password,
                        "Keep the administrator password",
                    )
                    .clicked()
                    {
                        actions.push(Action::SetKeepSudoPassword(!settings.keep_sudo_password));
                    }
                },
            );
            ui.add_space(12.0);
            widgets::hline(ui, &p);

            widgets::section_title(ui, &p, "Privacy");
            widgets::caption(
                ui,
                &p,
                "These are properties of the code, not preferences. They cannot be turned off because there is nothing to turn off.",
            );
            ui.add_space(10.0);
            privacy_row(
                ui,
                &p,
                "No telemetry",
                "Ghostline has no analytics, crash reporting, remote configuration, or update check. It does not create a device ID or count installations.",
            );
            privacy_row(
                ui,
                &p,
                "Network requests",
                "Ghostline asks cyberghostvpn for country, city, server, and streaming lists when you open Network, and runs the CLI when you connect or disconnect. Those requests use your CyberGhost account and go to CyberGhost. Exit IP checks contact api.ipify.org, and latency checks open a TCP connection to an imported profile endpoint. Both run only when you press them.",
            );
            privacy_row(
                ui,
                &p,
                "Credentials",
                "The administrator password is passed to sudo over stdin. It stays in memory for the session only if you choose, and it is never written to disk. Imported profile credentials use the system keyring through secret-tool, or a private temporary file that Ghostline deletes after the attempt.",
            );
            privacy_row(
                ui,
                &p,
                "Local files",
                "Settings and logs are written under your XDG directories with owner-only permissions. They are never uploaded.",
            );

            widgets::section_title(ui, &p, "About");
            widgets::body(
                ui,
                &p,
                format!(
                    "{APP_NAME} is an independent client for CyberGhost network locations and OpenVPN or WireGuard profile files. It is not affiliated with CyberGhost or its owner."
                ),
            );
            ui.add_space(10.0);
            widgets::metric(ui, &p, "Version", APP_VERSION, None);
            widgets::metric(ui, &p, "License", "MIT", None);
            widgets::metric(ui, &p, "Fonts", "IBM Plex, OFL 1.1", None);
            ui.add_space(8.0);
            widgets::caption(
                ui,
                &p,
                "Country, city, server, and streaming lists come from cyberghostvpn when you open Network. Imported file locations are matched locally. Ghostline does not invent missing servers.",
            );
            widgets::caption(
                ui,
                &p,
                "Fonts are bundled with Ghostline under the SIL Open Font License. The license text is in assets/fonts/OFL.txt.",
            );
    });
}

fn setting_caption(ui: &mut Ui, p: &Palette, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(2.0);
        widgets::label(ui, text, widgets::mono(10.5), p.muted);
    });
}

fn privacy_row(ui: &mut Ui, p: &Palette, title: &str, body: &str) {
    ui.add_space(8.0);
    widgets::label(ui, title, widgets::medium(14.0), p.text);
    ui.add_space(4.0);
    widgets::caption(ui, p, body);
    ui.add_space(12.0);
    widgets::hline(ui, p);
}
