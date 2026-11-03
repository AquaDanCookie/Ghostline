use egui::{
    Align, CornerRadius, Frame, Label, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};

use crate::app::{Action, App, NetworkBrowser};
use crate::format;
use crate::model::{LoadState, NetworkProtocol, Protocol, ServerType};
use crate::theme::Palette;

use super::widgets::{self, Btn, Icon};

pub fn show(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    app.ensure_network_loaded();
    let p = app.palette;
    let demo = app.demo;
    let cg_ready = app.cg_ready;
    let connected = app.tunnel.is_connected();
    let busy = app.is_busy();

    widgets::page_header(
        ui,
        &p,
        "Network",
        "Browse the CyberGhost catalogue through the official CLI. Account sign-in stays in the CLI, not in Ghostline.",
        |ui| {
            if widgets::button(
                ui,
                &p,
                Btn::secondary("Refresh")
                    .icon(Icon::Route)
                    .height(38.0)
                    .enabled(!app.network.countries_state.is_loading()),
            )
            .clicked()
            {
                actions.push(Action::RefreshCgCountries);
            }
            ui.add_space(8.0);
            if demo {
                widgets::tag(ui, &p, "Demo", p.accent);
            } else if cg_ready {
                widgets::tag(ui, &p, "CLI ready", p.ok);
            } else {
                widgets::tag(ui, &p, "CLI missing", p.danger);
            }
        },
    );

    if demo {
        widgets::note(
            ui,
            &p,
            "Demo catalogue",
            "These countries, cities, services, and loads are sample values. No request is made and no traffic is routed.",
            p.accent,
        );
        ui.add_space(14.0);
    } else if !cg_ready {
        widgets::note(
            ui,
            &p,
            "CyberGhost CLI not found",
            "Install the official Linux client and sign in with sudo cyberghostvpn --configure. Live catalogue browsing and CyberGhost connections stay unavailable until the command is on PATH. The reference countries below are for navigation only.",
            p.danger,
        );
        ui.add_space(14.0);
    }

    if cg_ready {
        if let Some(error) = app.network.countries_state.error() {
            widgets::note(
                ui,
                &p,
                "Catalogue unavailable",
                &if app.network.countries_fallback {
                    format!(
                        "{error} Country names come from a bundled reference so you can still identify the right country code. City and server lists stay empty until the CLI can answer."
                    )
                } else {
                    format!("{error} The list below is the last successful response.")
                },
                p.warn,
            );
            ui.add_space(14.0);
        }
    }

    controls(&mut app.network, ui, &p, actions);
    ui.add_space(16.0);

    let width = ui.available_width();
    if width >= 820.0 {
        let gap = 16.0;
        let country_width = (width * 0.31).clamp(240.0, 330.0);
        let city_width = (width * 0.28).clamp(210.0, 310.0);
        let server_width = width - country_width - city_width - gap * 2.0;
        ui.horizontal_top(|ui| {
            country_panel(&mut app.network, ui, &p, country_width, actions);
            ui.add_space(gap);
            city_panel(&mut app.network, ui, &p, city_width, actions);
            ui.add_space(gap);
            server_panel(&mut app.network, ui, &p, server_width, actions);
        });
    } else {
        country_panel(&mut app.network, ui, &p, width, actions);
        ui.add_space(14.0);
        city_panel(&mut app.network, ui, &p, width, actions);
        ui.add_space(14.0);
        server_panel(&mut app.network, ui, &p, width, actions);
    }

    ui.add_space(18.0);
    selection_summary(
        &app.network,
        ui,
        &p,
        demo,
        cg_ready,
        connected,
        busy,
        actions,
    );
}

fn controls(network: &mut NetworkBrowser, ui: &mut Ui, p: &Palette, actions: &mut Vec<Action>) {
    widgets::card(ui, p, 16.0, |ui| {
        ui.set_width((ui.available_width() - 4.0).max(240.0));
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                widgets::eyebrow(ui, p, "Server type");
                ui.add_space(6.0);
                if let Some(next) = widgets::segmented(
                    ui,
                    p,
                    "cg-server-type",
                    &[
                        (ServerType::Traffic, "Traffic"),
                        (ServerType::Streaming, "Streaming"),
                        (ServerType::Torrent, "Torrent"),
                    ],
                    network.server_type,
                ) {
                    if next != network.server_type {
                        actions.push(Action::SetCgServerType(next));
                    }
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                widgets::eyebrow(ui, p, "VPN protocol");
                ui.add_space(6.0);
                if let Some(next) = widgets::segmented(
                    ui,
                    p,
                    "cg-vpn-protocol",
                    &[
                        (Protocol::OpenVpn, "OpenVPN"),
                        (Protocol::WireGuard, "WireGuard"),
                    ],
                    network.vpn_service,
                ) {
                    if next != network.vpn_service {
                        actions.push(Action::SetCgVpnService(next));
                    }
                }
            });
            if network.vpn_service == Protocol::OpenVpn {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    widgets::eyebrow(ui, p, "Network protocol");
                    ui.add_space(6.0);
                    if let Some(next) = widgets::segmented(
                        ui,
                        p,
                        "cg-network-protocol",
                        &[(NetworkProtocol::Udp, "UDP"), (NetworkProtocol::Tcp, "TCP")],
                        network.network_protocol,
                    ) {
                        if next != network.network_protocol {
                            actions.push(Action::SetCgNetworkProtocol(next));
                        }
                    }
                });
            }
        });
        ui.add_space(10.0);
        widgets::caption(
            ui,
            p,
            "The CLI applies these choices when Ghostline saves or opens a location. WireGuard ignores the UDP and TCP choice.",
        );
    });
}

fn country_panel(
    network: &mut NetworkBrowser,
    ui: &mut Ui,
    p: &Palette,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let search = network.country_search.trim().to_lowercase();
    let filtered: Vec<_> = network
        .countries
        .iter()
        .filter(|country| {
            search.is_empty()
                || country.name.to_lowercase().contains(&search)
                || country.code.to_lowercase().contains(&search)
        })
        .cloned()
        .collect();

    panel(ui, p, width, "Country", |ui| {
        widgets::search_input(
            ui,
            p,
            "cg-country-search",
            &mut network.country_search,
            "Search countries",
            ui.available_width(),
        );
        ui.add_space(8.0);
        if network.countries_state.is_loading() && network.countries.is_empty() {
            loading(ui, p, "Loading countries");
        } else if filtered.is_empty() {
            empty(ui, p, "No countries match", "Try a different search.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("cg-country-list")
                .auto_shrink([false, false])
                .min_scrolled_height(236.0)
                .max_height(236.0)
                .show(ui, |ui| {
                    for country in &filtered {
                        let selected = network
                            .country
                            .as_ref()
                            .map(|selected| selected.code == country.code)
                            .unwrap_or(false);
                        if choice_row(
                            ui,
                            p,
                            &format!("cg-country-{}", country.code),
                            &country.name,
                            None,
                            Some(&country.code),
                            selected,
                        ) {
                            actions.push(Action::SelectCgCountry(country.code.clone()));
                        }
                    }
                });
        }
        ui.add_space(6.0);
        widgets::hline(ui, p);
        ui.add_space(8.0);
        let footer = if network.countries_fallback {
            if search.is_empty() {
                format!("{} reference countries", network.countries.len())
            } else {
                format!(
                    "{} of {} reference countries",
                    filtered.len(),
                    network.countries.len()
                )
            }
        } else if search.is_empty() {
            format::count(network.countries.len(), "country", "countries")
        } else {
            format!(
                "{} of {}",
                filtered.len(),
                format::count(network.countries.len(), "country", "countries")
            )
        };
        widgets::label(ui, footer, widgets::mono(10.5), p.muted);
    });
}

fn city_panel(
    network: &mut NetworkBrowser,
    ui: &mut Ui,
    p: &Palette,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let Some(country) = network.country.clone() else {
        panel(ui, p, width, "City", |ui| {
            empty(
                ui,
                p,
                "Choose a country",
                "Cities and streaming services appear after a country is selected.",
            );
        });
        return;
    };

    panel(ui, p, width, "City", |ui| {
        if network.server_type == ServerType::Streaming {
            widgets::eyebrow(ui, p, "Streaming service");
            ui.add_space(6.0);
            match &network.streaming_state {
                LoadState::Loading => loading(ui, p, "Loading services"),
                LoadState::Failed(error) => {
                    empty(ui, p, "Services unavailable", &format::shorten(error, 130));
                }
                LoadState::Idle => empty(
                    ui,
                    p,
                    "No services loaded",
                    "Choose the country again to request its streaming services.",
                ),
                LoadState::Ready => {
                    egui::ScrollArea::vertical()
                        .id_salt("cg-streaming-list")
                        .auto_shrink([false, false])
                        .min_scrolled_height(176.0)
                        .max_height(176.0)
                        .show(ui, |ui| {
                            for service in &network.streaming_services {
                                let selected = network.streaming_service.as_deref()
                                    == Some(service.service.as_str());
                                if choice_row(
                                    ui,
                                    p,
                                    &format!("cg-streaming-{}", service.service),
                                    &service.service,
                                    None,
                                    None,
                                    selected,
                                ) {
                                    actions
                                        .push(Action::SelectCgStreaming(service.service.clone()));
                                }
                            }
                        });
                    if network.streaming_services.is_empty() {
                        empty(
                            ui,
                            p,
                            "No streaming services",
                            "The CLI returned no services for this country.",
                        );
                    }
                }
            }
            ui.add_space(8.0);
            widgets::hline(ui, p);
            ui.add_space(8.0);
        }

        let ready_for_cities =
            network.server_type != ServerType::Streaming || network.streaming_service.is_some();
        if !ready_for_cities {
            empty(
                ui,
                p,
                "Choose a service",
                "Cities depend on the streaming service you select.",
            );
            return;
        }

        widgets::search_input(
            ui,
            p,
            "cg-city-search",
            &mut network.city_search,
            "Search cities",
            ui.available_width(),
        );
        ui.add_space(8.0);

        let search = network.city_search.trim().to_lowercase();
        let filtered: Vec<_> = network
            .cities
            .iter()
            .filter(|city| search.is_empty() || city.name.to_lowercase().contains(&search))
            .cloned()
            .collect();

        if network.cities_state.is_loading() && network.cities.is_empty() {
            loading(ui, p, "Loading cities");
        } else if let Some(error) = network.cities_state.error() {
            empty(ui, p, "Cities unavailable", &format::shorten(error, 130));
        } else if filtered.is_empty() {
            empty(ui, p, "No cities match", "Try a different search.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("cg-city-list")
                .auto_shrink([false, false])
                .min_scrolled_height(236.0)
                .max_height(236.0)
                .show(ui, |ui| {
                    if choice_row(
                        ui,
                        p,
                        "cg-city-auto",
                        "Any city",
                        Some(&format!("Let the CLI choose in {}", country.name)),
                        None,
                        network.city.is_none(),
                    ) {
                        actions.push(Action::SelectCgCity(String::new()));
                    }
                    for city in &filtered {
                        let selected = network
                            .city
                            .as_ref()
                            .map(|selected| selected.name == city.name)
                            .unwrap_or(false);
                        let detail = city
                            .instances
                            .parse::<usize>()
                            .ok()
                            .map(|count| format::count(count, "server", "servers"));
                        if choice_row(
                            ui,
                            p,
                            &format!("cg-city-{}", city.name),
                            &city.name,
                            detail.as_deref(),
                            Some(&city.load),
                            selected,
                        ) {
                            actions.push(Action::SelectCgCity(city.name.clone()));
                        }
                    }
                });
        }
        ui.add_space(6.0);
        widgets::hline(ui, p);
        ui.add_space(8.0);
        widgets::label(
            ui,
            if search.is_empty() {
                format::count(network.cities.len(), "city", "cities")
            } else {
                format!(
                    "{} of {}",
                    filtered.len(),
                    format::count(network.cities.len(), "city", "cities")
                )
            },
            widgets::mono(10.5),
            p.muted,
        );
    });
}

fn server_panel(
    network: &mut NetworkBrowser,
    ui: &mut Ui,
    p: &Palette,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let Some(city) = network.city.clone() else {
        panel(ui, p, width, "Server", |ui| {
            if let Some(country) = network.country.as_ref() {
                empty(
                    ui,
                    p,
                    "Automatic server",
                    &format!(
                        "The CLI will choose a city and server in {}. Select a city to browse specific instances.",
                        country.name
                    ),
                );
            } else {
                empty(
                    ui,
                    p,
                    "Choose a city",
                    "Servers appear for a city. You can also connect with automatic city selection.",
                );
            }
        });
        return;
    };

    panel(ui, p, width, "Server", |ui| {
        widgets::search_input(
            ui,
            p,
            "cg-server-search",
            &mut network.server_search,
            "Search servers",
            ui.available_width(),
        );
        ui.add_space(8.0);

        let search = network.server_search.trim().to_lowercase();
        let filtered: Vec<_> = network
            .servers
            .iter()
            .filter(|server| {
                search.is_empty()
                    || server.name.to_lowercase().contains(&search)
                    || server.instance.to_lowercase().contains(&search)
            })
            .cloned()
            .collect();

        if network.servers_state.is_loading() && network.servers.is_empty() {
            loading(ui, p, "Loading servers");
        } else if let Some(error) = network.servers_state.error() {
            empty(ui, p, "Servers unavailable", &format::shorten(error, 130));
        } else if filtered.is_empty() {
            empty(ui, p, "No servers match", "Try a different search.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("cg-server-list")
                .auto_shrink([false, false])
                .min_scrolled_height(236.0)
                .max_height(236.0)
                .show(ui, |ui| {
                    if choice_row(
                        ui,
                        p,
                        "cg-server-auto",
                        "Any server",
                        Some(&format!("Automatic choice in {}", city.name)),
                        None,
                        network.server.is_none(),
                    ) {
                        actions.push(Action::SelectCgServer(String::new()));
                    }
                    for server in &filtered {
                        let selected = network
                            .server
                            .as_ref()
                            .map(|selected| selected.instance == server.instance)
                            .unwrap_or(false);
                        if choice_row(
                            ui,
                            p,
                            &format!("cg-server-{}", server.instance),
                            &server.name,
                            Some(&server.instance),
                            Some(&format!("{}%", server.load.trim_end_matches('%'))),
                            selected,
                        ) {
                            actions.push(Action::SelectCgServer(server.instance.clone()));
                        }
                    }
                });
        }
        ui.add_space(6.0);
        widgets::hline(ui, p);
        ui.add_space(8.0);
        widgets::label(
            ui,
            if search.is_empty() {
                format::count(network.servers.len(), "server", "servers")
            } else {
                format!(
                    "{} of {}",
                    filtered.len(),
                    format::count(network.servers.len(), "server", "servers")
                )
            },
            widgets::mono(10.5),
            p.muted,
        );
    });
}

fn selection_summary(
    network: &NetworkBrowser,
    ui: &mut Ui,
    p: &Palette,
    demo: bool,
    cg_ready: bool,
    connected: bool,
    busy: bool,
    actions: &mut Vec<Action>,
) {
    widgets::card(ui, p, 22.0, |ui| {
        ui.set_width((ui.available_width() - 4.0).max(240.0));
        let Some(country) = network.country.as_ref() else {
            widgets::eyebrow(ui, p, "Selection");
            ui.add_space(5.0);
            widgets::label(
                ui,
                "Choose a country to begin",
                widgets::semibold(21.0),
                p.text,
            );
            ui.add_space(4.0);
            widgets::caption(
                ui,
                p,
                "The summary follows your choices and shows exactly what the CLI will receive.",
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                widgets::button(
                    ui,
                    p,
                    Btn::secondary("Save to Locations")
                        .min_width(172.0)
                        .enabled(false),
                );
                ui.add_space(6.0);
                widgets::button(
                    ui,
                    p,
                    Btn::primary("Connect")
                        .icon(Icon::Power)
                        .min_width(140.0)
                        .enabled(false),
                );
            });
            return;
        };

        let name = if let Some(server) = &network.server {
            server.name.clone()
        } else if let Some(city) = &network.city {
            city.name.clone()
        } else {
            country.name.clone()
        };
        let protocol = if network.vpn_service == Protocol::OpenVpn {
            format!("OpenVPN {}", network.network_protocol.label())
        } else {
            "WireGuard".to_owned()
        };
        let mut detail = country.name.clone();
        if let Some(city) = &network.city {
            detail.push_str(", ");
            detail.push_str(&city.name);
        }
        if let Some(service) = &network.streaming_service {
            detail.push_str(" / ");
            detail.push_str(service);
        }

        ui.horizontal_top(|ui| {
            let left = (ui.available_width() - 210.0).max(200.0);
            ui.allocate_ui_with_layout(Vec2::new(left, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_max_width(left);
                widgets::eyebrow(ui, p, "Selection");
                ui.add_space(5.0);
                ui.add(
                    Label::new(
                        RichText::new(&name)
                            .font(widgets::semibold(21.0))
                            .color(p.text),
                    )
                    .wrap(),
                );
                ui.add_space(3.0);
                widgets::caption(ui, p, &detail);
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if connected {
                    widgets::tag(ui, p, "Tunnel active", p.ok);
                } else if busy {
                    widgets::tag(ui, p, "Working", p.accent);
                }
                widgets::tag(ui, p, network.server_type.label(), p.muted);
                widgets::tag(ui, p, &protocol, p.muted);
            });
        });

        ui.add_space(14.0);
        widgets::hline(ui, p);
        ui.add_space(12.0);
        let load_label = network
            .server
            .as_ref()
            .map(|server| format!("{}% load", server.load.trim_end_matches('%')));
        widgets::metric(
            ui,
            p,
            "Server",
            network
                .server
                .as_ref()
                .map(|server| server.instance.as_str())
                .unwrap_or("Automatic"),
            load_label.as_deref(),
        );
        let city_value = network
            .city
            .as_ref()
            .map(|city| city.name.as_str())
            .unwrap_or("Automatic");
        widgets::metric(ui, p, "City", city_value, None);
        if let Some(service) = &network.streaming_service {
            widgets::metric(ui, p, "Streaming", service, None);
        }

        ui.add_space(14.0);
        let reference_only = network.countries_fallback;
        let selection_ready = network.country.is_some()
            && (network.server_type != ServerType::Streaming
                || network.streaming_service.is_some());
        let can_save = cg_ready && !reference_only && selection_ready;
        let can_connect = can_save && !connected && !busy;
        ui.horizontal_wrapped(|ui| {
            if widgets::button(
                ui,
                p,
                Btn::secondary("Save to Locations")
                    .icon(Icon::Pin)
                    .min_width(172.0)
                    .enabled(can_save),
            )
            .clicked()
            {
                actions.push(Action::SaveCgLocation);
            }
            if widgets::button(
                ui,
                p,
                Btn::primary("Connect")
                    .icon(Icon::Power)
                    .min_width(140.0)
                    .enabled(can_connect),
            )
            .clicked()
            {
                actions.push(Action::ConnectCgLocation);
            }
            if connected {
                ui.add_space(4.0);
                widgets::caption(
                    ui,
                    p,
                    "Disconnect the active tunnel before opening another location.",
                );
            } else if busy {
                ui.add_space(4.0);
                widgets::caption(ui, p, "Wait for the current attempt to finish.");
            } else if !cg_ready {
                ui.add_space(4.0);
                widgets::caption(
                    ui,
                    p,
                    "The official CyberGhost CLI is required before a connection can start.",
                );
            } else if reference_only {
                ui.add_space(4.0);
                widgets::caption(
                    ui,
                    p,
                    "Reference countries are for navigation only. Connect after the CLI returns live catalogue data.",
                );
            } else if demo {
                ui.add_space(4.0);
                widgets::caption(ui, p, "Demo mode simulates the connection.");
            }
        });
    });
}

fn panel(ui: &mut Ui, p: &Palette, width: f32, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 800.0),
        Layout::top_down(Align::Min),
        |ui| {
            ui.set_width(width);
            ui.set_max_width(width);
            let frame = Frame::new()
                .fill(p.surface)
                .stroke(Stroke::new(1.0_f32, p.line))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width((width - 28.0).max(160.0));
                    ui.horizontal(|ui| {
                        widgets::label(ui, title, widgets::medium(15.0), p.text);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            widgets::label(ui, "Catalogue", widgets::mono(9.5), p.muted);
                        });
                    });
                    ui.add_space(10.0);
                    add_contents(ui);
                });
            ui.painter().rect_stroke(
                frame.response.rect,
                CornerRadius::same(12),
                Stroke::new(1.0_f32, p.line),
                StrokeKind::Inside,
            );
        },
    );
}

fn choice_row(
    ui: &mut Ui,
    p: &Palette,
    id: &str,
    title: &str,
    subtitle: Option<&str>,
    trailing: Option<&str>,
    selected: bool,
) -> bool {
    let width = ui.available_width();
    let height = if subtitle.is_some() { 48.0 } else { 40.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let id = ui.make_persistent_id(("cg-row", id));
    let response = ui.interact(rect, id, Sense::click());
    let hovered = response.hovered();
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(7), p.surface_alt);
        ui.painter().rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.center().y - 10.0),
                Vec2::new(2.0, 20.0),
            ),
            CornerRadius::same(1),
            p.accent,
        );
    } else if hovered {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(7),
            widgets::mix(p.surface, p.surface_alt, 0.55),
        );
    }

    let right_space = trailing
        .map(|value| value.chars().count() as f32 * 7.5 + 14.0)
        .unwrap_or(8.0);
    let title_width = (width - 26.0 - right_space).max(60.0);
    let title_text = format::shorten(title, (title_width / 7.0) as usize);
    let title_pos = if subtitle.is_some() {
        Pos2::new(rect.left() + 10.0, rect.top() + height * 0.33)
    } else {
        Pos2::new(rect.left() + 10.0, rect.center().y)
    };
    ui.painter().text(
        title_pos,
        egui::Align2::LEFT_CENTER,
        title_text,
        widgets::medium(13.5),
        if selected { p.accent } else { p.text },
    );
    if let Some(subtitle) = subtitle {
        ui.painter().text(
            Pos2::new(rect.left() + 10.0, rect.top() + height * 0.72),
            egui::Align2::LEFT_CENTER,
            format::shorten(subtitle, (title_width / 6.0) as usize),
            widgets::mono(10.0),
            p.muted,
        );
    }
    if let Some(trailing) = trailing {
        ui.painter().text(
            Pos2::new(rect.right() - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            trailing,
            widgets::mono_medium(10.5),
            if selected { p.accent } else { p.muted },
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            CornerRadius::same(7),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, title)
    });
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn loading(ui: &mut Ui, p: &Palette, text: &str) {
    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.add(egui::Spinner::new().size(16.0).color(p.accent));
        ui.add_space(7.0);
        widgets::label(ui, text, widgets::sans(13.0), p.muted);
    });
    ui.add_space(18.0);
}

fn empty(ui: &mut Ui, p: &Palette, title: &str, body: &str) {
    ui.add_space(14.0);
    widgets::label(ui, title, widgets::medium(13.5), p.text);
    ui.add_space(3.0);
    widgets::caption(ui, p, body);
    ui.add_space(14.0);
}
