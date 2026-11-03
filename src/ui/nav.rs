use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, Sense, Stroke, StrokeKind,
    Ui, UiBuilder, Vec2, WidgetInfo, WidgetType,
};

use crate::app::{Action, App, APP_VERSION};
use crate::model::Screen;
use crate::theme::Palette;

use super::widgets::{self, paint_icon};

pub fn show(app: &mut App, ui: &mut Ui, actions: &mut Vec<Action>) {
    let p = app.palette;
    let panel = ui.max_rect();
    ui.painter().vline(
        panel.right() - 0.5,
        panel.y_range(),
        Stroke::new(1.0_f32, p.line),
    );

    let content = Rect::from_min_max(
        panel.min + Vec2::new(20.0, 24.0),
        panel.max - Vec2::new(16.0, 18.0),
    );
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(content)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.set_width(content.width());
            ui.set_height(content.height());
            wordmark(ui, &p, app.demo);
            ui.add_space(30.0);

            for screen in Screen::ALL {
                let active = app.screen == screen;
                if nav_row(ui, &p, screen, active) {
                    actions.push(Action::Navigate(screen));
                }
                ui.add_space(3.0);
            }

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                widgets::label(
                    ui,
                    format!("Ghostline {APP_VERSION}"),
                    widgets::mono(10.5),
                    p.muted,
                );
                ui.add_space(2.0);
                widgets::label(ui, "Independent project", widgets::sans(11.0), p.muted);
                ui.add_space(18.0);
                status_block(ui, app);
            });
        },
    );
}

fn wordmark(ui: &mut Ui, p: &Palette, demo: bool) {
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), Sense::hover());
        paint_icon(ui.painter(), icon_rect, widgets::Icon::Route, p.accent, 1.5);
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.add_space(1.0);
            widgets::label(ui, "Ghostline", widgets::semibold(19.0), p.text);
            widgets::label(ui, "Independent client", widgets::sans(11.0), p.muted);
        });
    });
    if demo {
        ui.add_space(10.0);
        widgets::tag(ui, p, "Demo", p.accent);
    }
}

fn nav_row(ui: &mut Ui, p: &Palette, screen: Screen, active: bool) -> bool {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 40.0), Sense::click());
    let hovered = response.hovered();
    let id = response.id;
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(id.with("nav-hover"), hovered, 0.12);
    if !active && hover_t > 0.0 {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(8),
            widgets::mix(Color32::TRANSPARENT, p.surface_alt, hover_t),
        );
    }
    if active {
        ui.painter().rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.center().y - 9.0),
                Vec2::new(2.0, 18.0),
            ),
            CornerRadius::same(1),
            p.accent,
        );
    }
    let foreground = if active {
        p.accent
    } else if hovered {
        p.text
    } else {
        p.muted
    };
    let icon_rect = Rect::from_center_size(
        Pos2::new(rect.left() + 21.0, rect.center().y),
        Vec2::new(18.0, 18.0),
    );
    paint_icon(
        ui.painter(),
        icon_rect,
        widgets::nav_icon(screen),
        foreground,
        1.5,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 42.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        screen.title(),
        widgets::medium(14.5),
        foreground,
    );
    let hint = format!("Ctrl+{}", widgets::screen_shortcut(screen));
    ui.painter().text(
        Pos2::new(rect.right() - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        hint,
        widgets::mono_medium(10.0),
        if active {
            p.accent.gamma_multiply(0.75)
        } else {
            p.muted.gamma_multiply(0.65)
        },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            CornerRadius::same(8),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    }
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, active, screen.title()));
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn status_block(ui: &mut Ui, app: &App) {
    let p = app.palette;
    let status = app.status();
    let color = status.tone.color(&p);
    let width = ui.available_width();
    let detail_font = widgets::sans(12.0);
    let (detail, truncated) = fit_line(
        ui,
        &status.detail,
        detail_font.clone(),
        p.muted,
        (width - 28.0).max(40.0),
    );
    let height = 64.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(10), p.surface);
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );

    let row_y = rect.top() + 22.0;
    let dot = Pos2::new(rect.left() + 17.0, row_y);
    if status.label == "Connected" {
        let time = ui.input(|input| input.time);
        let amount =
            (0.16 + 0.16 * (time * std::f64::consts::PI).sin().abs() as f32).clamp(0.0, 0.4);
        painter.circle_stroke(dot, 8.0, Stroke::new(2.0_f32, color.gamma_multiply(amount)));
    }
    painter.circle_filled(dot, 3.5, color);
    painter.text(
        Pos2::new(rect.left() + 30.0, row_y),
        Align2::LEFT_CENTER,
        status.label,
        widgets::medium(13.0),
        color,
    );
    painter.text(
        Pos2::new(rect.left() + 14.0, rect.top() + 46.0),
        Align2::LEFT_CENTER,
        detail,
        detail_font,
        p.muted,
    );

    if truncated {
        response.on_hover_text(&status.detail);
    }
}

fn fit_line(ui: &Ui, text: &str, font: FontId, color: Color32, max_width: f32) -> (String, bool) {
    let painter = ui.painter();
    if painter
        .layout_no_wrap(text.to_owned(), font.clone(), color)
        .size()
        .x
        <= max_width
    {
        return (text.to_owned(), false);
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let candidate = format!("{}...", chars.iter().collect::<String>());
        if painter
            .layout_no_wrap(candidate.clone(), font.clone(), color)
            .size()
            .x
            <= max_width
        {
            return (candidate, true);
        }
    }
    ("...".to_owned(), true)
}
