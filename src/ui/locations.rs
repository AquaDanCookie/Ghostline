use std::collections::BTreeMap;
use std::path::Path;

use egui::{
    Align, CornerRadius, Frame, Label, Layout, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};

use crate::app::{Action, App, Presence};
use crate::format;
use crate::model::Profile;
use crate::theme::Palette;

use super::widgets::{self, Btn, Icon};

pub fn show(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    let p = app.palette;
    let demo = app.demo;
    let nm_ready = app.nm_ready;

    widgets::page_header(
        ui,
        &p,
        "Locations",
        "Saved locations from the CyberGhost CLI and profiles imported into NetworkManager. Select one to use it for the next connection.",
        |ui| {
            if !demo {
                if widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Import profile")
                        .icon(Icon::Import)
                        .height(38.0)
                        .enabled(nm_ready),
                )
                .clicked()
                {
                    actions.push(Action::RequestImport);
                }
            } else {
                widgets::tag(ui, &p, "Demo", p.accent);
            }
        },
    );

    if !nm_ready
        && !demo
        && app
            .state
            .profiles
            .iter()
            .any(|profile| !profile.is_cyberghost())
    {
        widgets::note(
            ui,
            &p,
            "Imported profiles are unavailable",
            "nmcli was not found. CyberGhost network locations in this list still work through the official CLI.",
            p.warn,
        );
        ui.add_space(16.0);
    }

    if app.state.profiles.is_empty() {
        empty_state(app, ui, actions);
        return;
    }

    let selected_id = app.selected_profile.clone();
    let selected = selected_id
        .as_deref()
        .and_then(|id| app.profile(id))
        .cloned();
    let tunnel_id = app.tunnel_profile_id().map(str::to_owned);
    let search = app.location_search.trim().to_lowercase();
    let favorites_only = app.favorites_only;

    let mut filtered: Vec<Profile> = app
        .state
        .profiles
        .iter()
        .filter(|profile| !favorites_only || profile.favorite)
        .filter(|profile| {
            if search.is_empty() {
                return true;
            }
            let country = profile.country_label().to_lowercase();
            let city = profile.city.as_deref().unwrap_or("").to_lowercase();
            profile.name.to_lowercase().contains(&search)
                || country.contains(&search)
                || city.contains(&search)
                || profile.remote_host.to_lowercase().contains(&search)
                || profile.protocol.label().to_lowercase().contains(&search)
        })
        .cloned()
        .collect();
    filtered.sort_by(|a, b| {
        a.sort_country()
            .cmp(&b.sort_country())
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let width = ui.available_width();
    let stacked = width < 840.0;
    let left_width = if stacked {
        width
    } else {
        (width * 0.46).clamp(330.0, 440.0)
    };
    let right_width = if stacked {
        width
    } else {
        width - left_width - 24.0
    };

    if stacked {
        list_panel(
            app,
            ui,
            actions,
            &filtered,
            left_width,
            selected_id.as_deref(),
            tunnel_id.as_deref(),
        );
        ui.add_space(20.0);
        detail_panel(
            app,
            ui,
            actions,
            selected.as_ref(),
            right_width,
            tunnel_id.as_deref(),
        );
    } else {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(left_width);
                list_panel(
                    app,
                    ui,
                    actions,
                    &filtered,
                    left_width,
                    selected_id.as_deref(),
                    tunnel_id.as_deref(),
                );
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.set_width(right_width);
                detail_panel(
                    app,
                    ui,
                    actions,
                    selected.as_ref(),
                    right_width,
                    tunnel_id.as_deref(),
                );
            });
        });
    }

    if !demo {
        ui.add_space(14.0);
        widgets::caption(
            ui,
            &p,
            "You can drop .ovpn or .conf files anywhere in this window.",
        );
    }
}

fn empty_state(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    let p = app.palette;
    let width = ui.available_width();
    let frame = widgets::card(ui, &p, 28.0, |ui| {
        ui.set_width((width - 56.0).max(240.0));
        ui.add(
            Label::new(
                RichText::new("No saved locations")
                    .font(widgets::semibold(26.0))
                    .color(p.text),
            )
            .wrap(),
        );
        ui.add_space(7.0);
        ui.add(
            Label::new(
                RichText::new(
                    "Browse the CyberGhost network through the official CLI, or import an OpenVPN or WireGuard profile. Saved locations, profile files, and credentials stay on this machine.",
                )
                .font(widgets::sans(14.0))
                .color(p.muted),
            )
            .wrap(),
        );
        ui.add_space(20.0);
        widgets::route_visual(
            ui,
            &p,
            "This device",
            "No location",
            Some("Choose a network location or import a profile"),
            false,
            false,
            false,
        );
        ui.add_space(18.0);
        ui.horizontal_wrapped(|ui| {
            if widgets::button(
                ui,
                &p,
                Btn::primary("Browse network")
                    .icon(Icon::Globe)
                    .min_width(190.0),
            )
            .clicked()
            {
                actions.push(Action::Navigate(crate::model::Screen::Network));
            }
            if !app.demo
                && widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Import profile")
                        .icon(Icon::Import)
                        .min_width(180.0)
                        .enabled(app.nm_ready),
                )
                .clicked()
            {
                actions.push(Action::RequestImport);
            }
        });
        if app.demo {
            widgets::caption(ui, &p, "Demo mode does not include an import action.");
        } else if !app.nm_ready {
            widgets::caption(
                ui,
                &p,
                "NetworkManager is not available, so imported profiles cannot be added yet.",
            );
        } else {
            widgets::caption(
                ui,
                &p,
                "You can also drop .ovpn or .conf files anywhere in this window.",
            );
        }
    });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
}

fn list_panel(
    app: &mut App,
    ui: &mut Ui,
    actions: &mut Vec<Action>,
    filtered: &[Profile],
    width: f32,
    selected_id: Option<&str>,
    tunnel_id: Option<&str>,
) {
    let p = app.palette;

    ui.horizontal(|ui| {
        let toggle_width = 148.0;
        let search_width = (width - toggle_width).max(180.0);
        widgets::search_input(
            ui,
            &p,
            "location-search",
            &mut app.location_search,
            "Search name, country, or endpoint",
            search_width,
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::toggle(
                ui,
                &p,
                "favorites-only",
                app.favorites_only,
                "Show favorites only",
            )
            .clicked()
            {
                app.favorites_only = !app.favorites_only;
            }
            widgets::label(ui, "Favorites", widgets::sans(12.5), p.muted);
        });
    });
    ui.add_space(10.0);

    let frame = Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(12))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.add_space(13.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                widgets::label(
                    ui,
                    format!("{} of {}", filtered.len(), app.state.profiles.len()),
                    widgets::mono_medium(11.0),
                    p.muted,
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(16.0);
                    if app.favorites_only {
                        widgets::tag(ui, &p, "Favorites", p.accent);
                    }
                });
            });
            ui.add_space(11.0);
            widgets::hline(ui, &p);

            if filtered.is_empty() {
                ui.add_space(26.0);
                ui.vertical_centered(|ui| {
                    widgets::label(ui, "No profiles match", widgets::medium(15.0), p.text);
                    ui.add_space(4.0);
                    widgets::caption(ui, &p, "Try a different search, or clear the filters.");
                    ui.add_space(12.0);
                    if widgets::button(ui, &p, Btn::secondary("Clear filters")).clicked() {
                        app.location_search.clear();
                        app.favorites_only = false;
                    }
                });
                ui.add_space(26.0);
                return;
            }

            let mut grouped: BTreeMap<String, Vec<Profile>> = BTreeMap::new();
            for profile in filtered {
                grouped
                    .entry(profile.country_label().to_owned())
                    .or_default()
                    .push(profile.clone());
            }

            let compact = app.state.settings.compact_rows;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .min_scrolled_height(420.0)
                .max_height(610.0)
                .show(ui, |ui| {
                    for (country, profiles) in grouped {
                        country_header(ui, &p, &country, profiles.len());
                        for profile in profiles {
                            let selected = selected_id == Some(profile.id.as_str());
                            let active = tunnel_id == Some(profile.id.as_str());
                            let row =
                                profile_row(ui, &p, &profile, selected, active, compact, width);
                            if row.star_clicked {
                                actions.push(Action::ToggleFavorite(profile.id.clone()));
                            } else if row.clicked {
                                actions.push(Action::SelectProfile(profile.id.clone()));
                            }
                        }
                    }
                    ui.add_space(6.0);
                });
        });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
}

struct RowResult {
    clicked: bool,
    star_clicked: bool,
}

fn country_header(ui: &mut Ui, p: &Palette, country: &str, count: usize) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, p.bg);
    ui.painter().text(
        egui::Pos2::new(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        country.to_uppercase(),
        widgets::mono_medium(10.5),
        p.muted,
    );
    ui.painter().text(
        egui::Pos2::new(rect.right() - 16.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        format::count(count, "profile", "profiles"),
        widgets::mono(10.5),
        p.muted,
    );
}

fn profile_row(
    ui: &mut Ui,
    p: &Palette,
    profile: &Profile,
    selected: bool,
    active: bool,
    compact: bool,
    width: f32,
) -> RowResult {
    let height = if compact { 46.0 } else { 56.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let star_rect = egui::Rect::from_center_size(
        egui::Pos2::new(rect.right() - 20.0, rect.center().y),
        Vec2::splat(28.0),
    );
    let row_rect = egui::Rect::from_min_max(
        rect.min,
        egui::Pos2::new(star_rect.left() - 4.0, rect.max.y),
    );
    let row_id = ui.make_persistent_id(("profile-row", profile.id.as_str()));
    let row_response = ui.interact(row_rect, row_id, Sense::click());
    let star_response = ui.interact(star_rect, row_id.with("star"), Sense::click());
    let hovered = row_response.hovered() || star_response.hovered();
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, p.surface_alt);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
            CornerRadius::ZERO,
            p.accent,
        );
    } else if hovered {
        ui.painter().rect_filled(
            rect,
            CornerRadius::ZERO,
            widgets::mix(p.surface, p.surface_alt, 0.55),
        );
    }

    let name_color = if selected { p.text } else { p.text };
    let name_pos = egui::Pos2::new(rect.left() + 16.0, rect.top() + height * 0.32);
    ui.painter().text(
        name_pos,
        egui::Align2::LEFT_CENTER,
        format::shorten(&profile.name, 30),
        widgets::medium(14.5),
        name_color,
    );

    let mut facts: Vec<String> = Vec::new();
    if let Some(city) = profile.city.as_deref() {
        facts.push(city.to_owned());
    }
    facts.push(profile.network_label());
    if let Some(latency) = &profile.latency {
        facts.push(format!("{} ms", latency.ms));
    }
    let facts = facts.join("  /  ");
    ui.painter().text(
        egui::Pos2::new(rect.left() + 16.0, rect.top() + height * 0.72),
        egui::Align2::LEFT_CENTER,
        format::shorten(&facts, 46),
        widgets::sans(11.5),
        p.muted,
    );

    if active {
        ui.painter().circle_filled(
            egui::Pos2::new(star_rect.left() - 55.0, rect.center().y),
            3.0,
            p.ok,
        );
        ui.painter().text(
            egui::Pos2::new(star_rect.left() - 48.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Active",
            widgets::mono_medium(10.0),
            p.ok,
        );
    }

    if star_response.hovered() {
        ui.painter()
            .rect_filled(star_rect, CornerRadius::same(8), p.surface);
    }
    widgets::paint_icon(
        ui.painter(),
        egui::Rect::from_center_size(star_rect.center(), Vec2::splat(16.0)),
        if profile.favorite {
            Icon::StarFilled
        } else {
            Icon::Star
        },
        if profile.favorite {
            p.accent
        } else if star_response.hovered() {
            p.text
        } else {
            p.muted
        },
        1.4,
    );
    if star_response.has_focus() {
        ui.painter().rect_stroke(
            star_rect,
            CornerRadius::same(8),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    }
    if row_response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            CornerRadius::ZERO,
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    }

    row_response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            true,
            selected,
            profile.name.clone(),
        )
    });
    star_response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            true,
            profile.favorite,
            format!("Favorite {}", profile.name),
        )
    });
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let star_response = star_response.on_hover_text(if profile.favorite {
        "Remove from favorites"
    } else {
        "Add to favorites"
    });

    RowResult {
        clicked: row_response.clicked(),
        star_clicked: star_response.clicked(),
    }
}

fn detail_panel(
    app: &mut App,
    ui: &mut Ui,
    actions: &mut Vec<Action>,
    profile: Option<&Profile>,
    width: f32,
    tunnel_id: Option<&str>,
) {
    let Some(profile) = profile else {
        let p = app.palette;
        let frame = widgets::card(ui, &p, 28.0, |ui| {
            ui.set_width((width - 56.0).max(220.0));
            ui.add_space(18.0);
            for (index, line) in [
                "Select a profile from the list.",
                "The detail panel shows its endpoint, protocol, and actions.",
            ]
            .iter()
            .enumerate()
            {
                widgets::label(
                    ui,
                    *line,
                    if index == 0 {
                        widgets::medium(16.0)
                    } else {
                        widgets::sans(13.5)
                    },
                    if index == 0 { p.text } else { p.muted },
                );
                ui.add_space(5.0);
            }
            ui.add_space(16.0);
            widgets::route_visual(
                ui,
                &p,
                "This device",
                "No selection",
                None,
                false,
                false,
                false,
            );
        });
        ui.painter().rect_stroke(
            frame.response.rect,
            CornerRadius::same(12),
            Stroke::new(1.0_f32, p.line),
            StrokeKind::Inside,
        );
        return;
    };

    let p = app.palette;
    let presence = app.connection_presence(profile);
    let cyber = profile.is_cyberghost();
    let connected_here = tunnel_id == Some(profile.id.as_str()) && app.tunnel.is_connected();
    let busy_here = tunnel_id == Some(profile.id.as_str()) && app.tunnel.is_busy();
    let other_active = app.tunnel.is_connected() && !connected_here;
    let other_busy = app.tunnel.is_busy() && !busy_here;
    let source_available = if cyber {
        app.demo || app.cg_ready
    } else {
        app.demo || Path::new(&profile.source_path).is_file()
    };
    let source_path = profile.source_path.clone();
    let endpoint = profile.endpoint();
    let latency_pending = app.latency_pending.contains(&profile.id);

    let frame = widgets::card(ui, &p, 24.0, |ui| {
        ui.set_width((width - 48.0).max(220.0));
        ui.horizontal_top(|ui| {
            let (tile, _) = ui.allocate_exact_size(Vec2::splat(48.0), Sense::hover());
            ui.painter()
                .rect_filled(tile, CornerRadius::same(10), p.surface_alt);
            ui.painter().rect_stroke(
                tile,
                CornerRadius::same(10),
                Stroke::new(1.0_f32, p.line),
                StrokeKind::Inside,
            );
            ui.painter().text(
                tile.center(),
                egui::Align2::CENTER_CENTER,
                profile.country_code.as_deref().unwrap_or("--"),
                widgets::mono_medium(15.0),
                p.accent,
            );
            ui.add_space(12.0);
            let left_width = (ui.available_width() - 150.0).max(160.0);
            ui.allocate_ui_with_layout(
                Vec2::new(left_width, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_max_width(left_width);
                    ui.add(
                        Label::new(
                            RichText::new(&profile.name)
                                .font(widgets::semibold(22.0))
                                .color(p.text),
                        )
                        .wrap(),
                    );
                    let location = match (profile.country.as_deref(), profile.city.as_deref()) {
                        (Some(country), Some(city)) => format!("{country}, {city}"),
                        (Some(country), None) => country.to_owned(),
                        _ => "Unclassified".to_owned(),
                    };
                    ui.add_space(3.0);
                    ui.add(
                        Label::new(
                            RichText::new(location)
                                .font(widgets::sans(13.0))
                                .color(p.muted),
                        )
                        .wrap(),
                    );
                },
            );
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if connected_here {
                    widgets::tag(ui, &p, "Active", p.ok);
                }
                presence_tag(ui, &p, presence, cyber);
            });
        });
        ui.add_space(18.0);
        widgets::hline(ui, &p);
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            widgets::label(ui, "Endpoint", widgets::sans(13.0), p.muted);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::icon_button(ui, &p, Icon::Copy, "Copy endpoint", 24.0).clicked() {
                    actions.push(Action::Copy(endpoint.clone()));
                }
                widgets::label(
                    ui,
                    format::shorten(&endpoint, 34),
                    widgets::mono_medium(12.0),
                    p.text,
                );
            });
        });
        ui.add_space(4.0);
        widgets::metric(ui, &p, "Protocol", &profile.network_label(), None);
        if cyber {
            widgets::metric(ui, &p, "Server type", profile.server_type.label(), None);
            if let Some(service) = &profile.streaming_service {
                widgets::metric(ui, &p, "Streaming", service, None);
            }
            if let Some(load) = &profile.server_load {
                widgets::metric(
                    ui,
                    &p,
                    "Server load",
                    &format!("{}%", load.trim_end_matches('%')),
                    None,
                );
            }
            widgets::metric(ui, &p, "Added", &format::date(profile.imported_at), None);
        } else {
            widgets::metric(
                ui,
                &p,
                "Authentication",
                if profile.requires_auth {
                    "Username and password"
                } else {
                    "Not required"
                },
                None,
            );
            widgets::metric(ui, &p, "Imported", &format::date(profile.imported_at), None);
            let latency_value = if latency_pending {
                "Measuring".to_owned()
            } else {
                profile
                    .latency
                    .as_ref()
                    .map(|latency| format!("{} ms", latency.ms))
                    .unwrap_or_else(|| "Not measured".to_owned())
            };
            widgets::metric(ui, &p, "Latency", &latency_value, None);
            if let Some(latency) = &profile.latency {
                ui.add_space(1.0);
                widgets::caption(
                    ui,
                    &p,
                    format!("Last measured {}.", format::date_time(latency.measured_at)),
                );
            }
            if app.demo {
                ui.add_space(1.0);
                widgets::caption(
                    ui,
                    &p,
                    "Latency values are simulated in demo mode. No network request is made.",
                );
            }
        }

        ui.add_space(12.0);
        widgets::label(
            ui,
            if cyber { "Source" } else { "Source file" },
            widgets::sans(12.0),
            p.muted,
        );
        if cyber {
            ui.add(
                Label::new(
                    RichText::new("CyberGhost CLI network catalogue")
                        .font(widgets::mono(11.0))
                        .color(if source_available { p.muted } else { p.danger }),
                )
                .wrap(),
            );
            ui.add_space(5.0);
            widgets::caption(
                ui,
                &p,
                "Account credentials and live server lists stay in the official CLI.",
            );
        } else {
            let source_display = if app.demo {
                "Sample profile. No file is read."
            } else {
                &source_path
            };
            ui.add(
                Label::new(
                    RichText::new(format::shorten_path(
                        source_display,
                        if width < 460.0 { 42 } else { 58 },
                    ))
                    .font(widgets::mono(11.0))
                    .color(if source_available { p.muted } else { p.danger }),
                )
                .wrap(),
            );
        }

        if !app.demo && !source_available {
            ui.add_space(12.0);
            widgets::note(
                ui,
                &p,
                if cyber {
                    "CyberGhost CLI missing"
                } else {
                    "Source file missing"
                },
                if cyber {
                    "The official CyberGhost CLI is not available, so this saved location cannot connect."
                } else {
                    "The original profile is no longer at this path. Import it again to recreate the NetworkManager connection."
                },
                p.danger,
            );
        }
        if presence == Presence::Missing {
            ui.add_space(12.0);
            widgets::note(
                ui,
                &p,
                "NetworkManager connection missing",
                if source_available {
                    "Recreate the connection from the original profile, or import the file again."
                } else {
                    "The original profile is missing. Import the file again before reconnecting."
                },
                p.warn,
            );
        }

        ui.add_space(16.0);
        ui.horizontal_wrapped(|ui| {
            if connected_here {
                if widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Disconnect")
                        .icon(Icon::Power)
                        .min_width(148.0),
                )
                .clicked()
                {
                    actions.push(Action::Disconnect);
                }
            } else if busy_here {
                if widgets::button(
                    ui,
                    &p,
                    Btn::secondary("Cancel").icon(Icon::Close).min_width(148.0),
                )
                .clicked()
                {
                    actions.push(Action::CancelConnect);
                }
            } else if presence == Presence::Missing {
                if source_available {
                    if widgets::button(
                        ui,
                        &p,
                        Btn::primary("Recreate connection")
                            .icon(Icon::Route)
                            .min_width(176.0),
                    )
                    .clicked()
                    {
                        actions.push(Action::RecreateConnection(profile.id.clone()));
                    }
                } else if !app.demo
                    && widgets::button(
                        ui,
                        &p,
                        Btn::primary("Import profile")
                            .icon(Icon::Import)
                            .min_width(164.0)
                            .enabled(app.nm_ready),
                    )
                    .clicked()
                {
                    actions.push(Action::RequestImport);
                }
            } else if app.tunnel.is_busy() || other_active || presence == Presence::Unavailable {
                widgets::button(
                    ui,
                    &p,
                    Btn::primary("Connect")
                        .icon(Icon::Power)
                        .min_width(140.0)
                        .enabled(false),
                );
            } else if widgets::button(
                ui,
                &p,
                Btn::primary("Connect").icon(Icon::Power).min_width(140.0),
            )
            .clicked()
            {
                actions.push(Action::Connect(profile.id.clone()));
            }

            if !cyber
                && widgets::button(
                    ui,
                    &p,
                    Btn::secondary(if latency_pending {
                        "Measuring"
                    } else {
                        "Measure latency"
                    })
                    .min_width(148.0)
                    .enabled(!latency_pending),
                )
                .clicked()
            {
                actions.push(Action::MeasureLatency(profile.id.clone()));
            }

            if widgets::button(
                ui,
                &p,
                Btn::ghost(if profile.favorite {
                    "Favorited"
                } else {
                    "Favorite"
                })
                .icon(if profile.favorite {
                    Icon::StarFilled
                } else {
                    Icon::Star
                })
                .height(40.0),
            )
            .clicked()
            {
                actions.push(Action::ToggleFavorite(profile.id.clone()));
            }

            if profile.requires_auth
                && widgets::button(ui, &p, Btn::ghost("Credentials").height(40.0)).clicked()
            {
                actions.push(Action::EditCredentials(profile.id.clone()));
            }
        });

        if other_active {
            ui.add_space(6.0);
            widgets::caption(
                ui,
                &p,
                "Disconnect the active tunnel before connecting to another profile.",
            );
        } else if other_busy {
            ui.add_space(6.0);
            widgets::caption(ui, &p, "Wait for the current connection attempt to finish.");
        } else if presence == Presence::Unavailable {
            ui.add_space(6.0);
            widgets::caption(
                ui,
                &p,
                if cyber {
                    "The official CyberGhost CLI is not available, so this saved location cannot connect yet."
                } else {
                    "NetworkManager is not available, so this profile cannot connect yet."
                },
            );
        }

        ui.add_space(16.0);
        widgets::hline(ui, &p);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let remove_width = 150.0;
            let text_width = (ui.available_width() - remove_width - 12.0).max(120.0);
            ui.allocate_ui_with_layout(
                Vec2::new(text_width, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_max_width(text_width);
                    widgets::caption(
                        ui,
                        &p,
                        if cyber {
                            "Removes the saved location. The CyberGhost CLI and your account stay as they are."
                        } else {
                            "Removes the profile and its NetworkManager connection."
                        },
                    );
                },
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::button(ui, &p, Btn::ghost_danger("Remove profile").height(34.0))
                    .clicked()
                {
                    actions.push(Action::RemoveProfile(profile.id.clone()));
                }
            });
        });
    });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
}

fn presence_tag(ui: &mut Ui, p: &Palette, presence: Presence, cyber: bool) {
    match presence {
        Presence::Present => widgets::tag(
            ui,
            p,
            if cyber { "CyberGhost" } else { "Imported" },
            p.muted,
        ),
        Presence::Missing => widgets::tag(ui, p, "Missing", p.danger),
        Presence::Checking => widgets::tag(ui, p, "Checking", p.warn),
        Presence::Unavailable => widgets::tag(ui, p, "Unavailable", p.danger),
    }
}
