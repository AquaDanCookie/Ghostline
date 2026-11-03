pub mod activity;
pub mod connect;
pub mod locations;
pub mod modals;
pub mod nav;
pub mod network;
pub mod settings;
pub mod toasts;
pub mod widgets;

use egui::{Color32, CornerRadius, Order, Rect, Stroke, StrokeKind, Vec2};

use crate::app::{Action, App};
use crate::model::Screen;

pub fn page(app: &mut App, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .id_salt(("page-scroll", app.screen.title()))
        .show(ui, |ui| {
            let available = ui.available_width();
            let pad = if available >= 1000.0 {
                56.0
            } else if available >= 760.0 {
                40.0
            } else {
                24.0
            };
            let width = (available - pad * 2.0).clamp(320.0, 1040.0);
            ui.add_space(28.0);
            ui.horizontal(|ui| {
                ui.add_space(((available - width) * 0.5).max(0.0));
                ui.vertical(|ui| {
                    ui.set_width(width);
                    match app.screen {
                        Screen::Connect => connect::show(app, ui, actions),
                        Screen::Network => network::show(app, ui, actions),
                        Screen::Locations => locations::show(app, ui, actions),
                        Screen::Activity => activity::show(app, ui, actions),
                        Screen::Settings => settings::show(app, ui, actions),
                    }
                });
            });
            ui.add_space(44.0);
        });
}

pub fn drop_overlay(app: &App, ctx: &egui::Context) {
    if app.modal.is_some() || app.demo {
        return;
    }
    let files = ctx.input(|input| input.raw.hovered_files.clone());
    if files.is_empty() {
        return;
    }
    let count = files.len();
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new("drop-overlay"))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            let painter = ui.painter();
            painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(160));
            let target = Rect::from_min_max(
                screen.min + Vec2::splat(28.0),
                screen.max - Vec2::splat(28.0),
            );
            painter.rect_filled(
                target,
                CornerRadius::same(16),
                app.palette.surface.gamma_multiply(0.96),
            );
            painter.rect_stroke(
                target,
                CornerRadius::same(16),
                Stroke::new(2.0_f32, app.palette.accent),
                StrokeKind::Inside,
            );
            let icon_rect =
                Rect::from_center_size(target.center() - Vec2::new(0.0, 40.0), Vec2::splat(42.0));
            widgets::paint_icon(
                painter,
                icon_rect,
                widgets::Icon::Import,
                app.palette.accent,
                1.6,
            );
            painter.text(
                target.center() + Vec2::new(0.0, 22.0),
                egui::Align2::CENTER_CENTER,
                if count == 1 {
                    "Drop to review this profile".to_owned()
                } else {
                    format!("Drop to review {count} profiles")
                },
                widgets::semibold(22.0),
                app.palette.text,
            );
            painter.text(
                target.center() + Vec2::new(0.0, 50.0),
                egui::Align2::CENTER_CENTER,
                ".ovpn and .conf files",
                widgets::mono(12.5),
                app.palette.muted,
            );
            ui.allocate_space(Vec2::ZERO);
        });
}
