use std::path::Path;

use egui::{
    Align, CornerRadius, Frame, Label, Layout, Margin, RichText, Sense, Stroke, StrokeKind, Ui,
    Vec2,
};

use crate::app::{Action, App, IpState, Presence};
use crate::format;
use crate::model::{Profile, TunnelState};

use super::widgets::{self, Btn, Icon};

pub fn show(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    let p = app.palette;
    let external = matches!(app.tunnel, TunnelState::External { .. });

    ui.horizontal(|ui| {
        widgets::eyebrow(ui, &p, "Tunnel");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.demo {
                widgets::tag(ui, &p, "Demo", p.accent);
            } else {
                if widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Import profile")
                        .icon(Icon::Import)
                        .height(36.0),
                )
                .clicked()
                {
                    actions.push(Action::RequestImport);
                }
            }
        });
    });
    ui.add_space(14.0);

    if !app.nm_ready && !app.demo {
        widgets::note(
            ui,
            &p,
            "Imported profiles are unavailable",
            "nmcli was not found. CyberGhost network locations can still work through the official CLI.",
            p.warn,
        );
        ui.add_space(16.0);
    }

    let tunnel_id = app.tunnel_profile_id().map(str::to_owned);
    let profile = if external {
        None
    } else {
        tunnel_id
            .as_deref()
            .and_then(|id| app.profile(id))
            .or_else(|| app.current_profile())
            .cloned()
    };

    let width = ui.available_width();
    if width < 790.0 {
        hero(app, ui, actions, profile.as_ref(), external, width);
        ui.add_space(20.0);
        session(app, ui, actions, profile.as_ref(), external, width);
    } else {
        let left = (width - 24.0) * 0.575;
        let right = width - left - 24.0;
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(Vec2::new(left, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left);
                hero(app, ui, actions, profile.as_ref(), external, left);
            });
            ui.add_space(24.0);
            ui.allocate_ui_with_layout(Vec2::new(right, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(right);
                session(app, ui, actions, profile.as_ref(), external, right);
            });
        });
    }
}

fn hero(
    app: &mut App,
    ui: &mut Ui,
    actions: &mut Vec<Action>,
    profile: Option<&Profile>,
    external: bool,
    width: f32,
) {
    let p = app.palette;
    let (title, subtitle) = hero_copy(app, profile, external);
    ui.add(
        egui::Label::new(
            egui::RichText::new(title)
                .font(widgets::semibold(32.0))
                .color(p.text),
        )
        .wrap(),
    );
    ui.add_space(7.0);
    ui.add(
        egui::Label::new(
            egui::RichText::new(subtitle)
                .font(widgets::sans(14.5))
                .color(p.muted),
        )
        .wrap(),
    );
    ui.add_space(18.0);

    if external {
        widgets::route_visual(
            ui,
            &p,
            "This device",
            "External tunnel",
            Some("Opened outside Ghostline"),
            true,
            false,
            false,
        );
    } else if let Some(profile) = profile {
        let running = matches!(
            &app.tunnel,
            TunnelState::Connected { profile_id, .. }
                | TunnelState::Connecting { profile_id }
                | TunnelState::Reconnecting { profile_id, .. }
                if profile_id == &profile.id
        ) || matches!(app.tunnel, TunnelState::Disconnecting);
        if running {
            let connected = matches!(
                &app.tunnel,
                TunnelState::Connected { profile_id, .. } if profile_id == &profile.id
            );
            let busy = matches!(
                &app.tunnel,
                TunnelState::Connecting { profile_id }
                    | TunnelState::Reconnecting { profile_id, .. }
                    if profile_id == &profile.id
            ) || matches!(app.tunnel, TunnelState::Disconnecting);
            let failed = matches!(&app.tunnel, TunnelState::Error { .. });
            let location = profile_location(profile);
            let detail = format!("{location}, {}", profile.network_label());
            widgets::route_visual(
                ui,
                &p,
                "This device",
                &profile.name,
                Some(&detail),
                connected,
                busy,
                failed,
            );
        } else {
            ready_panel(ui, &p, profile, width);
        }
    } else {
        nothing_panel(ui, &p);
    }

    ui.add_space(18.0);

    if external {
        if widgets::button(
            ui,
            &p,
            Btn::secondary("Disconnect")
                .icon(Icon::Power)
                .min_width(width.min(280.0)),
        )
        .clicked()
        {
            actions.push(Action::Disconnect);
        }
        ui.add_space(10.0);
        widgets::caption(
            ui,
            &p,
            "Ghostline did not open this tunnel. Disconnect stops it through the CyberGhost CLI.",
        );
        return;
    }

    if let Some(profile) = profile {
        let presence = app.connection_presence(profile);
        let source_available = if profile.is_cyberghost() {
            app.demo || app.cg_ready
        } else {
            app.demo || Path::new(&profile.source_path).is_file()
        };
        let failed = matches!(&app.tunnel, TunnelState::Error { .. });
        let action_width = width.min(280.0);

        match &app.tunnel {
            TunnelState::Connected { profile_id, .. } if profile_id == &profile.id => {
                if widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Disconnect")
                        .icon(Icon::Power)
                        .min_width(action_width),
                )
                .clicked()
                {
                    actions.push(Action::Disconnect);
                }
            }
            TunnelState::Connecting { profile_id }
            | TunnelState::Reconnecting { profile_id, .. }
                if profile_id == &profile.id =>
            {
                if widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Cancel")
                        .icon(Icon::Close)
                        .min_width(action_width),
                )
                .clicked()
                {
                    actions.push(Action::CancelConnect);
                }
            }
            TunnelState::Disconnecting => {
                widgets::button(
                    ui,
                    &p,
                    Btn::primary("Disconnecting")
                        .icon(Icon::Power)
                        .min_width(action_width)
                        .enabled(false),
                );
            }
            _ => {
                if presence == Presence::Missing {
                    if source_available {
                        if widgets::button(
                            ui,
                            &p,
                            Btn::primary("Recreate connection")
                                .icon(Icon::Route)
                                .min_width(action_width),
                        )
                        .clicked()
                        {
                            actions.push(Action::RecreateConnection(profile.id.clone()));
                        }
                    } else {
                        if widgets::button(
                            ui,
                            &p,
                            Btn::primary("Import profile")
                                .icon(Icon::Import)
                                .min_width(action_width),
                        )
                        .clicked()
                        {
                            actions.push(Action::RequestImport);
                        }
                        ui.add_space(8.0);
                        widgets::caption(
                            ui,
                            &p,
                            "The original file is missing. Import the profile again to restore the connection.",
                        );
                    }
                } else if presence == Presence::Unavailable {
                    widgets::button(
                        ui,
                        &p,
                        Btn::primary("Connect")
                            .icon(Icon::Power)
                            .min_width(action_width)
                            .enabled(false),
                    );
                    ui.add_space(8.0);
                    widgets::caption(
                        ui,
                        &p,
                        if profile.is_cyberghost() {
                            "The official CyberGhost CLI is not available. Install it and sign in there before connecting."
                        } else {
                            "NetworkManager is not available. Install nmcli and its VPN plugins before connecting."
                        },
                    );
                } else {
                    if widgets::button(
                        ui,
                        &p,
                        Btn::primary(if failed { "Try again" } else { "Connect" })
                            .icon(Icon::Power)
                            .min_width(action_width),
                    )
                    .clicked()
                    {
                        actions.push(Action::Connect(profile.id.clone()));
                    }
                }
            }
        }

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            widgets::tag(ui, &p, &profile.network_label(), p.muted);
            widgets::label(ui, profile_location(profile), widgets::sans(13.0), p.muted);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::button(ui, &p, Btn::ghost("Change location").height(30.0)).clicked() {
                    actions.push(Action::Navigate(if profile.is_cyberghost() {
                        crate::model::Screen::Network
                    } else {
                        crate::model::Screen::Locations
                    }));
                }
                if profile.requires_auth
                    && widgets::button(ui, &p, Btn::ghost("Credentials").height(30.0)).clicked()
                {
                    actions.push(Action::EditCredentials(profile.id.clone()));
                }
            });
        });

        if let TunnelState::Error { message, .. } = &app.tunnel {
            ui.add_space(16.0);
            widgets::note(
                ui,
                &p,
                if profile.is_cyberghost() {
                    "The CyberGhost CLI reported"
                } else {
                    "NetworkManager reported"
                },
                message,
                p.danger,
            );
        }
    } else {
        ui.horizontal_wrapped(|ui| {
            if widgets::button(
                ui,
                &p,
                Btn::primary("Browse network")
                    .icon(Icon::Globe)
                    .min_width(176.0),
            )
            .clicked()
            {
                actions.push(Action::Navigate(crate::model::Screen::Network));
            }
            if widgets::button(
                ui,
                &p,
                Btn::secondary("Import profile")
                    .icon(Icon::Import)
                    .min_width(168.0)
                    .enabled(!app.demo),
            )
            .clicked()
            {
                actions.push(Action::RequestImport);
            }
        });
        ui.add_space(10.0);
        if app.demo {
            widgets::caption(ui, &p, "Demo mode does not include an import action.");
        } else {
            widgets::caption(
                ui,
                &p,
                "You can also drop .ovpn or .conf files anywhere in this window.",
            );
        }
    }
}

fn ready_panel(ui: &mut Ui, p: &crate::theme::Palette, profile: &Profile, width: f32) {
    let frame = Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width((width - 32.0).max(220.0));
            ui.horizontal_top(|ui| {
                let (tile, _) = ui.allocate_exact_size(Vec2::splat(42.0), Sense::hover());
                ui.painter()
                    .rect_filled(tile, CornerRadius::same(9), p.surface_alt);
                ui.painter().rect_stroke(
                    tile,
                    CornerRadius::same(9),
                    Stroke::new(1.0_f32, p.line),
                    StrokeKind::Inside,
                );
                ui.painter().text(
                    tile.center(),
                    egui::Align2::CENTER_CENTER,
                    profile.country_code.as_deref().unwrap_or("--"),
                    widgets::mono_medium(14.0),
                    p.accent,
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add(
                        Label::new(
                            RichText::new(format::shorten(&profile.name, 34))
                                .font(widgets::medium(15.5))
                                .color(p.text),
                        )
                        .wrap(),
                    );
                    ui.add_space(2.0);
                    widgets::caption(ui, p, profile_location(profile));
                });
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    widgets::tag(ui, p, "Ready", p.muted);
                });
            });
            ui.add_space(13.0);
            widgets::hline(ui, p);
            ui.add_space(11.0);
            widgets::metric(ui, p, "Transport", &profile.network_label(), None);
            widgets::metric(
                ui,
                p,
                "Endpoint",
                &format::shorten(&profile.endpoint(), 30),
                None,
            );
            widgets::metric(
                ui,
                p,
                "Source",
                if profile.is_cyberghost() {
                    "CyberGhost CLI"
                } else {
                    "Imported profile"
                },
                None,
            );
        });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
}

fn nothing_panel(ui: &mut Ui, p: &crate::theme::Palette) {
    let frame = Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_width((ui.available_width() - 4.0).max(220.0));
            widgets::label(ui, "No tunnel is open", widgets::medium(16.5), p.text);
            ui.add_space(4.0);
            widgets::caption(
                ui,
                p,
                "Choose a CyberGhost location in Network, or bring in an .ovpn or .conf profile. Nothing leaves this machine until you connect.",
            );
        });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
}

fn hero_copy(app: &App, profile: Option<&Profile>, external: bool) -> (&'static str, String) {
    if external {
        return (
            "Active tunnel",
            "A VPN tunnel is already open outside Ghostline. Ghostline can watch it and stop it through the CyberGhost CLI.".to_owned(),
        );
    }
    match &app.tunnel {
        TunnelState::Disconnected => {
            if let Some(profile) = profile {
                if profile.is_cyberghost() {
                    (
                        "Not connected",
                        format!(
                            "{} is ready. The CyberGhost CLI opens the tunnel when you connect.",
                            profile.name
                        ),
                    )
                } else {
                    (
                        "Not connected",
                        format!(
                            "{} is selected. NetworkManager opens the tunnel when you connect.",
                            profile.name
                        ),
                    )
                }
            } else {
                (
                    "Nothing connected",
                    "Choose a CyberGhost network location or import a profile. Nothing is sent until you connect.".to_owned(),
                )
            }
        }
        TunnelState::Connecting { profile_id } => {
            let name = app
                .profile(profile_id)
                .map(|profile| profile.name.as_str())
                .unwrap_or("the selected location");
            if app
                .profile(profile_id)
                .map(Profile::is_cyberghost)
                .unwrap_or(false)
            {
                (
                    "Connecting",
                    format!("The CyberGhost CLI is opening {name}."),
                )
            } else {
                (
                    "Connecting",
                    format!("NetworkManager is bringing up {name}."),
                )
            }
        }
        TunnelState::Connected { profile_id, .. } => {
            let name = app
                .profile(profile_id)
                .map(|profile| profile.name.as_str())
                .unwrap_or("the selected location");
            ("Connected", format!("The tunnel is up through {name}."))
        }
        TunnelState::External { .. } => (
            "Active tunnel",
            "A VPN tunnel is already open outside Ghostline.".to_owned(),
        ),
        TunnelState::Disconnecting => {
            let cyber = app
                .tunnel_profile_id()
                .and_then(|id| app.profile(id))
                .map(Profile::is_cyberghost)
                .unwrap_or(false);
            (
                "Disconnecting",
                if cyber {
                    "The CyberGhost CLI is closing the tunnel.".to_owned()
                } else {
                    "NetworkManager is closing the tunnel.".to_owned()
                },
            )
        }
        TunnelState::Reconnecting { attempt, .. } => (
            "Reconnecting",
            format!("The tunnel dropped. Ghostline is trying again, attempt {attempt} of 3."),
        ),
        TunnelState::Error { .. } => (
            "Connection failed",
            "The last attempt did not open a tunnel. Read the message, then try again.".to_owned(),
        ),
    }
}

fn session(
    app: &mut App,
    ui: &mut Ui,
    actions: &mut Vec<Action>,
    profile: Option<&Profile>,
    external: bool,
    width: f32,
) {
    let p = app.palette;
    let status = app.status();
    let connected = app.tunnel.is_connected();
    let busy = app.tunnel.is_busy();

    if !connected && !busy {
        idle_session(app, ui, profile, width);
        return;
    }

    let transport = if app.demo {
        "Demo"
    } else if external {
        "External"
    } else if profile.map(Profile::is_cyberghost).unwrap_or(false) {
        "CyberGhost CLI"
    } else {
        "NetworkManager"
    };

    widgets::card(ui, &p, 22.0, |ui| {
        ui.set_width((width - 44.0).max(220.0));
        ui.horizontal(|ui| {
            widgets::label(ui, "Session", widgets::medium(15.0), p.text);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::tag(ui, &p, transport, if app.demo { p.accent } else { p.muted });
            });
        });
        ui.add_space(14.0);
        widgets::hline(ui, &p);
        ui.add_space(13.0);

        ui.horizontal(|ui| {
            widgets::label(ui, "Status", widgets::sans(13.0), p.muted);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::status(
                    ui,
                    &p,
                    status.label,
                    status.tone.color(&p),
                    status.label == "Connected",
                );
            });
        });

        match profile {
            Some(profile) => {
                let endpoint = profile.endpoint();
                widgets::metric(ui, &p, "Profile", &format::shorten(&profile.name, 24), None);
                widgets::metric(ui, &p, "Protocol", &profile.network_label(), None);
                widgets::metric(ui, &p, "Endpoint", &format::shorten(&endpoint, 28), None);
            }
            None if external => {
                widgets::metric(ui, &p, "Profile", "Outside Ghostline", None);
                widgets::metric(ui, &p, "Transport", "Not exposed here", None);
                widgets::metric(ui, &p, "Endpoint", "Not exposed here", None);
            }
            None => {
                widgets::metric(ui, &p, "Profile", "None", None);
            }
        }

        let (duration_label, duration, interface) = match &app.tunnel {
            TunnelState::Connected { iface, .. } => (
                "Duration",
                app.session_seconds()
                    .map(format::duration)
                    .unwrap_or_else(|| "00:00".to_owned()),
                iface.clone(),
            ),
            TunnelState::External { iface, .. } => (
                "Observed for",
                app.session_seconds()
                    .map(format::duration)
                    .unwrap_or_else(|| "00:00".to_owned()),
                iface.clone(),
            ),
            TunnelState::Connecting { .. } | TunnelState::Reconnecting { .. } => {
                ("Duration", "Starting".to_owned(), "--".to_owned())
            }
            TunnelState::Disconnecting => ("Duration", "Closing".to_owned(), "--".to_owned()),
            _ => ("Duration", "Not active".to_owned(), "--".to_owned()),
        };
        widgets::metric(ui, &p, duration_label, &duration, None);
        widgets::metric(ui, &p, "Interface", &interface, None);
        if external {
            ui.add_space(2.0);
            widgets::caption(
                ui,
                &p,
                "The tunnel predates Ghostline, so its full uptime is unknown.",
            );
        }

        if connected && app.stats_seen {
            widgets::metric(
                ui,
                &p,
                "Received",
                &format::bytes(app.stats.rx_total),
                Some(&format::rate(app.stats.rx_rate)),
            );
            widgets::metric(
                ui,
                &p,
                "Sent",
                &format::bytes(app.stats.tx_total),
                Some(&format::rate(app.stats.tx_rate)),
            );
        } else {
            widgets::metric(
                ui,
                &p,
                "Received",
                if connected { "Waiting" } else { "--" },
                None,
            );
            widgets::metric(
                ui,
                &p,
                "Sent",
                if connected { "Waiting" } else { "--" },
                None,
            );
        }

        let ip_value = match &app.ip {
            IpState::Idle => "Not checked".to_owned(),
            IpState::Checking => "Checking".to_owned(),
            IpState::Done(ip) => ip.clone(),
            IpState::Error(_) => "Check failed".to_owned(),
        };
        widgets::metric(ui, &p, "Exit IP", &ip_value, None);

        if let IpState::Error(error) = &app.ip {
            ui.add_space(4.0);
            widgets::caption(ui, &p, format::shorten(error, 110));
        }

        if connected {
            ui.add_space(14.0);
            widgets::hline(ui, &p);
            ui.add_space(13.0);
            let checking = matches!(app.ip, IpState::Checking);
            if widgets::button(
                ui,
                &p,
                Btn::secondary(if checking {
                    "Checking exit IP"
                } else {
                    "Check exit IP"
                })
                .min_width(ui.available_width())
                .enabled(!checking),
            )
            .clicked()
            {
                actions.push(Action::CheckIp);
            }
            ui.add_space(6.0);
            if app.demo {
                widgets::caption(ui, &p, "Simulated in demo mode. No request is sent.");
            } else if external {
                widgets::caption(
                    ui,
                    &p,
                    "Contact api.ipify.org. It runs only when you press the button.",
                );
            } else {
                widgets::caption(
                    ui,
                    &p,
                    "Contacts api.ipify.org. It runs only when you press the button.",
                );
            }
        }
    });
}

fn idle_session(app: &mut App, ui: &mut Ui, profile: Option<&Profile>, width: f32) {
    let p = app.palette;
    let status = app.status();
    widgets::card(ui, &p, 22.0, |ui| {
        ui.set_width((width - 44.0).max(220.0));
        ui.horizontal(|ui| {
            widgets::label(ui, "Session", widgets::medium(15.0), p.text);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::tag(
                    ui,
                    &p,
                    if app.demo { "Demo" } else { "Idle" },
                    if app.demo { p.accent } else { p.muted },
                );
            });
        });
        ui.add_space(14.0);
        widgets::hline(ui, &p);
        ui.add_space(13.0);
        ui.horizontal(|ui| {
            widgets::label(ui, "Status", widgets::sans(13.0), p.muted);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::status(ui, &p, status.label, status.tone.color(&p), false);
            });
        });
        ui.add_space(12.0);
        widgets::hline(ui, &p);
        ui.add_space(13.0);

        if let Some(profile) = profile {
            let location = profile_location(profile);
            widgets::metric(
                ui,
                &p,
                "Selected",
                &format::shorten(&profile.name, 26),
                None,
            );
            widgets::metric(ui, &p, "Location", &location, None);
            widgets::metric(
                ui,
                &p,
                "Transport",
                &if profile.is_cyberghost() {
                    "CyberGhost CLI".to_owned()
                } else {
                    "NetworkManager".to_owned()
                },
                None,
            );
            ui.add_space(10.0);
            widgets::caption(
                ui,
                &p,
                "Session details appear here after the tunnel opens.",
            );
        } else {
            widgets::label(ui, "No location selected", widgets::medium(15.0), p.text);
            ui.add_space(4.0);
            widgets::caption(
                ui,
                &p,
                "Choose a CyberGhost location in Network, or import a profile in Locations.",
            );
        }

        if app.demo {
            ui.add_space(10.0);
            widgets::caption(
                ui,
                &p,
                "Demo mode uses sample values. No traffic is routed.",
            );
        }
    });
}

fn profile_location(profile: &Profile) -> String {
    match (profile.country.as_deref(), profile.city.as_deref()) {
        (Some(country), Some(city)) => {
            if country == city {
                country.to_owned()
            } else {
                format!("{country}, {city}")
            }
        }
        (Some(country), None) => country.to_owned(),
        _ => profile.country_label().to_owned(),
    }
}
