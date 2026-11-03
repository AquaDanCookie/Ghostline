use egui::{Align, CornerRadius, Layout, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::app::{Action, App, LogFilter};
use crate::format;
use crate::model::LogEntry;
use crate::theme::Palette;

use super::widgets::{self, Btn, Icon};

pub fn show(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    let p = app.palette;
    let total = app.logs.len();
    let filter = app.log_filter;
    let search = app.log_search.clone();
    let retention = app.state.settings.log_retention_days;
    let log_path = app.paths.log_file.to_string_lossy().into_owned();

    let visible: Vec<LogEntry> = app
        .logs
        .iter()
        .rev()
        .filter(|entry| filter.matches(entry.level))
        .filter(|entry| log_matches(entry, &search))
        .cloned()
        .collect();
    let copy_text = visible
        .iter()
        .rev()
        .map(format::log_line)
        .collect::<Vec<_>>()
        .join("\n");

    widgets::page_header(
        ui,
        &p,
        "Activity",
        "Connection events, profile changes, and errors. Everything here stays on this machine.",
        |ui| {
            if widgets::button(
                ui,
                &p,
                Btn::secondary("Copy log").icon(Icon::Copy).height(38.0),
            )
            .clicked()
                && !copy_text.is_empty()
            {
                actions.push(Action::Copy(copy_text.clone()));
            }
        },
    );

    let width = ui.available_width();
    let frame = egui::Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(12))
        .show(ui, |ui| {
            ui.set_width(width);

            ui.add_space(14.0);
            ui.horizontal_wrapped(|ui| {
                ui.add_space(16.0);
                let search_width = (width - 300.0).clamp(240.0, 360.0);
                widgets::search_input(
                    ui,
                    &p,
                    "activity-search",
                    &mut app.log_search,
                    "Search log entries",
                    search_width,
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(16.0);
                    if let Some(next) = widgets::segmented(
                        ui,
                        &p,
                        "activity-filter",
                        &[
                            (LogFilter::All, LogFilter::All.label()),
                            (LogFilter::Info, LogFilter::Info.label()),
                            (LogFilter::Warnings, LogFilter::Warnings.label()),
                            (LogFilter::Errors, LogFilter::Errors.label()),
                        ],
                        app.log_filter,
                    ) {
                        app.log_filter = next;
                    }
                });
            });
            ui.add_space(12.0);
            widgets::hline(ui, &p);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                widgets::label(
                    ui,
                    format!(
                        "{} of {}",
                        format::count(visible.len(), "entry", "entries"),
                        format::count(total, "entry", "entries")
                    ),
                    widgets::mono_medium(10.5),
                    p.muted,
                );
                ui.add_space(10.0);
                widgets::label(ui, "Newest first", widgets::sans(11.5), p.muted);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(12.0);
                    if widgets::button(ui, &p, Btn::ghost_danger("Clear log").icon(Icon::Trash))
                        .clicked()
                    {
                        actions.push(Action::RequestClearLogs);
                    }
                });
            });
            ui.add_space(8.0);
            widgets::hline(ui, &p);

            if visible.is_empty() {
                empty_state(app, ui, actions, &p, total);
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .min_scrolled_height(300.0)
                    .max_height(520.0)
                    .show(ui, |ui| {
                        let row_width = ui.available_width();
                        for entry in &visible {
                            if log_row(ui, &p, entry, row_width) {
                                actions.push(Action::Copy(format::log_line(entry)));
                            }
                        }
                        ui.add_space(2.0);
                    });
            }
        });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );

    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        widgets::caption(
            ui,
            &p,
            format!(
                "Log file: {}. Retention: {}.",
                format::shorten_path(&log_path, 58),
                match retention {
                    0 => "kept until cleared".to_owned(),
                    1 => "1 day".to_owned(),
                    days => format!("{days} days"),
                }
            ),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::button(ui, &p, Btn::ghost("Open data folder").height(30.0)).clicked() {
                actions.push(Action::OpenDataFolder);
            }
        });
    });
}

fn empty_state(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>, p: &Palette, total: usize) {
    let _ = actions;
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        widgets::label(
            ui,
            if total == 0 {
                "No activity yet"
            } else {
                "No entries match"
            },
            widgets::medium(16.0),
            p.text,
        );
        ui.add_space(5.0);
        widgets::caption(
            ui,
            p,
            if total == 0 {
                "Connection attempts, profile imports, and errors appear here. Nothing is sent anywhere."
            } else {
                "Change the filter or clear the search to see the rest of the log."
            },
        );
        ui.add_space(14.0);
        if total > 0
            && widgets::button(ui, p, Btn::secondary("Clear filters")).clicked()
        {
            app.log_filter = LogFilter::All;
            app.log_search.clear();
        }
    });
    ui.add_space(40.0);
}

fn log_row(ui: &mut Ui, p: &Palette, entry: &LogEntry, width: f32) -> bool {
    let time_width = 70.0;
    let level_width = 62.0;
    let padding = 16.0;
    let copy_width = 24.0;
    let message_width =
        (width - padding * 2.0 - time_width - level_width - copy_width - 16.0).max(120.0);
    let galley = ui.painter().layout(
        entry.message.clone(),
        widgets::sans(13.0),
        p.text,
        message_width,
    );
    let height = (galley.size().y + 20.0).max(38.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let hovered = response.hovered();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("row-hover"), hovered, 0.1);
    if hover_t > 0.0 {
        ui.painter().rect_filled(
            rect,
            CornerRadius::ZERO,
            widgets::mix(p.surface, p.surface_alt, hover_t * 0.7),
        );
    }

    let top = rect.top() + 10.0;
    let level_color = widgets::log_level_color(p, entry.level);
    ui.painter().text(
        egui::Pos2::new(rect.left() + padding, top),
        egui::Align2::LEFT_TOP,
        format::time(entry.at),
        widgets::mono(11.0),
        p.muted,
    );
    ui.painter().text(
        egui::Pos2::new(rect.left() + padding + time_width, top + 1.0),
        egui::Align2::LEFT_TOP,
        entry.level.label(),
        widgets::mono_medium(10.0),
        level_color,
    );
    ui.painter().galley(
        egui::Pos2::new(rect.left() + padding + time_width + level_width, top),
        galley,
        p.text,
    );

    let icon_center = egui::Pos2::new(rect.right() - padding - 6.0, top + 7.0);
    let icon_color = if hovered {
        p.text
    } else {
        p.muted.gamma_multiply(0.55)
    };
    widgets::paint_icon(
        ui.painter(),
        Rect::from_center_size(icon_center, Vec2::splat(14.0)),
        Icon::Copy,
        icon_color,
        1.4,
    );

    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            CornerRadius::same(6),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    }
    ui.painter().hline(
        (rect.left() + padding)..=(rect.right() - padding),
        rect.bottom() - 0.5,
        Stroke::new(1.0_f32, p.line.gamma_multiply(0.55)),
    );

    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Copy this entry");
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            format!(
                "Copy {} entry from {}",
                format::level_word(entry.level),
                format::time(entry.at)
            ),
        )
    });
    response.clicked()
}

fn log_matches(entry: &LogEntry, search: &str) -> bool {
    let search = search.trim().to_lowercase();
    if search.is_empty() {
        return true;
    }
    entry.message.to_lowercase().contains(&search)
        || entry.level.label().to_lowercase().contains(&search)
        || format::date_time(entry.at).to_lowercase().contains(&search)
}
