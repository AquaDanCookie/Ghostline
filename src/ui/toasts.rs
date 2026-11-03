use std::time::Duration;

use egui::{
    Align, Align2, CornerRadius, Frame, Layout, Margin, Order, Pos2, Rect, Sense, Stroke,
    StrokeKind, Vec2,
};

use crate::app::{Action, App};
use crate::format;

use super::widgets::{self, Icon};

const TOAST_LIFE: Duration = Duration::from_secs(4);
const TOAST_WIDTH: f32 = 360.0;

pub fn show(app: &mut App, ctx: &egui::Context, actions: &mut Vec<Action>) {
    if app.toasts.is_empty() {
        return;
    }
    let p = app.palette;

    egui::Area::new(egui::Id::new("toasts"))
        .order(Order::Foreground)
        .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-22.0, -22.0))
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_width(TOAST_WIDTH);
            for toast in &mut app.toasts {
                let tone = toast.tone();
                let color = tone.color(&p);
                let elapsed = toast.at.elapsed();
                let mut dismissed = false;

                let frame = Frame::new()
                    .fill(p.surface)
                    .stroke(Stroke::new(1.0_f32, p.line))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(14, 11))
                    .show(ui, |ui| {
                        ui.set_width(TOAST_WIDTH - 28.0);
                        ui.horizontal(|ui| {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::new(6.0, 20.0), Sense::hover());
                            ui.painter().circle_filled(
                                Pos2::new(dot_rect.left() + 3.0, dot_rect.center().y),
                                3.5,
                                color,
                            );
                            ui.add_space(5.0);
                            widgets::label(
                                ui,
                                format::level_word(toast.level).to_uppercase(),
                                widgets::mono_medium(9.5),
                                color,
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::icon_button(
                                    ui,
                                    &p,
                                    Icon::Close,
                                    "Dismiss notification",
                                    20.0,
                                )
                                .clicked()
                                {
                                    actions.push(Action::CloseToast(toast.id));
                                    dismissed = true;
                                }
                            });
                        });
                        ui.add_space(3.0);
                        widgets::label(ui, toast.message.clone(), widgets::sans(13.0), p.text);
                    });

                let rect = frame.response.rect;
                let hovered = frame.response.hovered();
                if hovered {
                    toast.at = std::time::Instant::now();
                }
                let progress = if hovered {
                    1.0
                } else {
                    (1.0 - elapsed.as_secs_f32() / TOAST_LIFE.as_secs_f32()).clamp(0.0, 1.0)
                };
                if progress > 0.0 {
                    let track = Rect::from_min_size(
                        Pos2::new(rect.left() + 1.0, rect.bottom() - 2.0),
                        Vec2::new((rect.width() - 2.0) * progress, 2.0),
                    );
                    ui.painter().rect_filled(
                        track,
                        CornerRadius::same(1),
                        color.gamma_multiply(0.75),
                    );
                }
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(10),
                    Stroke::new(1.0_f32, p.line),
                    StrokeKind::Inside,
                );

                if !dismissed && !hovered && elapsed >= TOAST_LIFE {
                    actions.push(Action::CloseToast(toast.id));
                }
                ui.add_space(8.0);
            }
        });
}
