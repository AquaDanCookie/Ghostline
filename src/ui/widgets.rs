use std::sync::Arc;

use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Frame, Label, Layout,
    Margin, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    WidgetInfo, WidgetType,
};

use crate::icons;
use crate::model::LogLevel;
use crate::theme::Palette;

pub fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(Arc::from("medium")))
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(Arc::from("semibold")))
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn mono_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(Arc::from("mono_medium")))
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        ch(a.r(), b.r()),
        ch(a.g(), b.g()),
        ch(a.b(), b.b()),
        ch(a.a(), b.a()),
    )
}

fn lighten(color: Color32, amount: f32) -> Color32 {
    mix(color, Color32::WHITE, amount)
}

fn darken(color: Color32, amount: f32) -> Color32 {
    mix(color, Color32::BLACK, amount)
}

pub fn label(ui: &mut Ui, text: impl Into<String>, font: FontId, color: Color32) {
    ui.add(Label::new(RichText::new(text.into()).font(font).color(color)).wrap());
}

pub fn eyebrow(ui: &mut Ui, p: &Palette, text: &str) {
    label(ui, text.to_uppercase(), mono_medium(11.0), p.muted);
}

pub fn caption(ui: &mut Ui, p: &Palette, text: impl Into<String>) {
    ui.add(Label::new(RichText::new(text.into()).font(sans(12.5)).color(p.muted)).wrap());
}

pub fn body(ui: &mut Ui, p: &Palette, text: impl Into<String>) {
    label(ui, text, sans(15.0), p.text);
}

pub fn section_title(ui: &mut Ui, p: &Palette, text: &str) {
    ui.add_space(14.0);
    label(ui, text, semibold(20.0), p.text);
    ui.add_space(6.0);
}

pub fn page_header(
    ui: &mut Ui,
    p: &Palette,
    title: &str,
    subtitle: &str,
    add_right: impl FnOnce(&mut Ui),
) {
    ui.add_space(4.0);
    let width = ui.available_width();
    let right_width = 190.0_f32.min(width * 0.4);
    ui.horizontal_top(|ui| {
        let left_width = (width - right_width - 20.0).max(240.0);
        ui.allocate_ui_with_layout(
            Vec2::new(left_width, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_max_width(left_width);
                ui.add(Label::new(RichText::new(title).font(semibold(28.0)).color(p.text)).wrap());
                if !subtitle.is_empty() {
                    ui.add_space(5.0);
                    ui.add(
                        Label::new(RichText::new(subtitle).font(sans(14.0)).color(p.muted)).wrap(),
                    );
                }
            },
        );
        ui.add_space(20.0);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 0.0),
            Layout::right_to_left(Align::Center),
            |ui| add_right(ui),
        );
    });
    ui.add_space(22.0);
}

pub fn page_header_plain(ui: &mut Ui, p: &Palette, title: &str, subtitle: &str) {
    ui.add_space(4.0);
    let width = ui.available_width();
    ui.allocate_ui_with_layout(Vec2::new(width, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_max_width(width);
        ui.add(Label::new(RichText::new(title).font(semibold(28.0)).color(p.text)).wrap());
        if !subtitle.is_empty() {
            ui.add_space(5.0);
            ui.add(Label::new(RichText::new(subtitle).font(sans(14.0)).color(p.muted)).wrap());
        }
    });
    ui.add_space(22.0);
}

pub fn hline(ui: &mut Ui, p: &Palette) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0_f32, p.line),
    );
}

pub fn card<R>(
    ui: &mut Ui,
    p: &Palette,
    padding: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(padding as i8))
        .show(ui, add_contents)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Power,
    Route,
    Pin,
    Globe,
    Activity,
    Settings,
    Import,
    Copy,
    Folder,
    Trash,
    Close,
    Eye,
    EyeOff,
    Star,
    StarFilled,
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32, width: f32) {
    match icon {
        Icon::Power => icons::power(painter, rect, color, width),
        Icon::Route => icons::route(painter, rect, color, width),
        Icon::Pin => icons::pin(painter, rect, color, width),
        Icon::Globe => icons::globe(painter, rect, color, width),
        Icon::Activity => icons::log(painter, rect, color, width),
        Icon::Settings => icons::cog(painter, rect, color, width),
        Icon::Import => icons::import(painter, rect, color, width),
        Icon::Copy => icons::copy(painter, rect, color, width),
        Icon::Folder => icons::folder(painter, rect, color, width),
        Icon::Trash => icons::trash(painter, rect, color, width),
        Icon::Close => icons::close(painter, rect, color, width),
        Icon::Eye => icons::eye(painter, rect, color, width),
        Icon::EyeOff => icons::eye_off(painter, rect, color, width),
        Icon::Star => icons::star(painter, rect, color, width, false),
        Icon::StarFilled => icons::star(painter, rect, color, width, true),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtnKind {
    Primary,
    Secondary,
    Ghost,
    GhostDanger,
    Danger,
}

pub struct Btn<'a> {
    label: &'a str,
    kind: BtnKind,
    icon: Option<Icon>,
    min_width: f32,
    height: f32,
    enabled: bool,
}

impl<'a> Btn<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            kind: BtnKind::Secondary,
            icon: None,
            min_width: 0.0,
            height: 40.0,
            enabled: true,
        }
    }

    pub fn primary(label: &'a str) -> Self {
        Self {
            kind: BtnKind::Primary,
            height: 44.0,
            ..Self::new(label)
        }
    }

    pub fn secondary(label: &'a str) -> Self {
        Self::new(label)
    }

    pub fn ghost(label: &'a str) -> Self {
        Self {
            kind: BtnKind::Ghost,
            ..Self::new(label)
        }
    }

    pub fn ghost_danger(label: &'a str) -> Self {
        Self {
            kind: BtnKind::GhostDanger,
            ..Self::new(label)
        }
    }

    pub fn danger(label: &'a str) -> Self {
        Self {
            kind: BtnKind::Danger,
            height: 44.0,
            ..Self::new(label)
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width.max(0.0);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(28.0);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

pub fn button(ui: &mut Ui, p: &Palette, button: Btn<'_>) -> Response {
    let enabled = button.enabled && ui.is_enabled();
    let font = medium(if button.kind == BtnKind::Primary {
        15.0
    } else {
        14.5
    });
    let text_color = match (button.kind, enabled) {
        (_, false) => p.muted,
        (BtnKind::Primary, true) => p.accent_ink,
        (BtnKind::Danger, true) => p.accent_ink,
        (BtnKind::GhostDanger, true) => p.danger,
        (BtnKind::Ghost, true) => p.text,
        (BtnKind::Secondary, true) => p.text,
    };
    let galley = ui
        .painter()
        .layout_no_wrap(button.label.to_owned(), font, text_color);
    let icon_space = if button.icon.is_some() {
        18.0 + 8.0
    } else {
        0.0
    };
    let horizontal_padding = if button.kind == BtnKind::Ghost || button.kind == BtnKind::GhostDanger
    {
        10.0
    } else {
        16.0
    };
    let desired_width =
        (galley.size().x + icon_space + horizontal_padding * 2.0).max(button.min_width);
    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(desired_width, button.height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), hovered, 0.12);
    let pressed = enabled && response.is_pointer_button_down_on();

    let fill = match button.kind {
        BtnKind::Primary if enabled => {
            let base = if pressed {
                darken(p.accent, 0.08)
            } else {
                lighten(p.accent, hover_t * 0.06)
            };
            base
        }
        BtnKind::Danger if enabled => {
            if pressed {
                darken(p.danger, 0.08)
            } else {
                lighten(p.danger, hover_t * 0.06)
            }
        }
        BtnKind::Secondary if enabled => mix(Color32::TRANSPARENT, p.surface_alt, hover_t),
        BtnKind::Ghost | BtnKind::GhostDanger if enabled => {
            mix(Color32::TRANSPARENT, p.surface_alt, hover_t * 0.8)
        }
        BtnKind::Primary
        | BtnKind::Danger
        | BtnKind::Secondary
        | BtnKind::Ghost
        | BtnKind::GhostDanger => Color32::TRANSPARENT,
    };
    if fill.a() > 0 {
        ui.painter().rect_filled(rect, CornerRadius::same(10), fill);
    }
    if !enabled {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(10),
            Stroke::new(1.0_f32, p.line),
            StrokeKind::Inside,
        );
    } else if matches!(button.kind, BtnKind::Secondary) {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(10),
            Stroke::new(1.0_f32, p.line),
            StrokeKind::Inside,
        );
    }

    let content_width = galley.size().x + icon_space;
    let mut x = rect.center().x - content_width / 2.0;
    if let Some(icon) = button.icon {
        let icon_rect =
            Rect::from_center_size(Pos2::new(x + 9.0, rect.center().y), Vec2::splat(17.0));
        paint_icon(ui.painter(), icon_rect, icon, text_color, 1.6);
        x += 18.0 + 8.0;
    }
    ui.painter().galley(
        Pos2::new(x, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );

    if response.has_focus() {
        let ring = if matches!(button.kind, BtnKind::Primary | BtnKind::Danger) {
            text_color
        } else {
            p.accent
        };
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            CornerRadius::same(10),
            Stroke::new(2.0_f32, ring),
            StrokeKind::Inside,
        );
    }

    if enabled {
        response = response.on_hover_cursor(CursorIcon::PointingHand);
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, button.label));
    response
}

pub fn icon_button(ui: &mut Ui, p: &Palette, icon: Icon, tooltip: &str, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let hovered = response.hovered();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), hovered, 0.12);
    if hover_t > 0.0 {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(8),
            mix(Color32::TRANSPARENT, p.surface_alt, hover_t),
        );
    }
    let color = if hovered { p.text } else { p.muted };
    paint_icon(
        ui.painter(),
        Rect::from_center_size(rect.center(), Vec2::splat(size * 0.52)),
        icon,
        color,
        1.6,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            CornerRadius::same(8),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    }
    let response = response.on_hover_text(tooltip);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, tooltip));
    response.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn toggle(ui: &mut Ui, p: &Palette, _id_salt: &str, on: bool, label: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(38.0, 22.0), Sense::click());
    let t = ui.ctx().animate_bool_with_time(response.id, on, 0.16);
    let track = Color32::from_rgba_unmultiplied(
        mix(p.surface_alt, p.accent, t).r(),
        mix(p.surface_alt, p.accent, t).g(),
        mix(p.surface_alt, p.accent, t).b(),
        255,
    );
    ui.painter()
        .rect_filled(rect, CornerRadius::same(11), track);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(11),
        Stroke::new(1.0_f32, mix(p.line, p.accent, t)),
        StrokeKind::Inside,
    );
    let thumb_x = rect.left() + 11.0 + (rect.width() - 22.0) * t;
    ui.painter().circle_filled(
        Pos2::new(thumb_x, rect.center().y),
        9.0,
        mix(p.muted, p.accent_ink, t),
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            CornerRadius::same(12),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Outside,
        );
    }
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, on, label));
    response.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn segmented<T: Copy + PartialEq>(
    ui: &mut Ui,
    p: &Palette,
    id_salt: &str,
    options: &[(T, &str)],
    selected: T,
) -> Option<T> {
    if options.is_empty() {
        return None;
    }
    let font = medium(13.5);
    let widths: Vec<f32> = options
        .iter()
        .map(|(_, label)| {
            let text = ui
                .painter()
                .layout_no_wrap((*label).to_owned(), font.clone(), p.muted);
            (text.size().x + 28.0).max(58.0)
        })
        .collect();
    let total: f32 = widths.iter().sum();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total, 34.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), p.surface_alt);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );

    let mut chosen = None;
    let mut x = rect.left();
    for (index, (value, label)) in options.iter().enumerate() {
        let segment = Rect::from_min_size(
            Pos2::new(x, rect.top()),
            Vec2::new(widths[index], rect.height()),
        );
        let id = ui.make_persistent_id((id_salt, index));
        let response = ui.interact(segment, id, Sense::click());
        let is_selected = *value == selected;
        let hovered = response.hovered();
        let inner = segment.shrink(2.5);
        if is_selected {
            ui.painter()
                .rect_filled(inner, CornerRadius::same(8), p.surface);
            ui.painter().rect_stroke(
                inner,
                CornerRadius::same(8),
                Stroke::new(1.0_f32, p.line),
                StrokeKind::Inside,
            );
        } else if hovered {
            ui.painter()
                .rect_filled(inner, CornerRadius::same(8), p.surface.gamma_multiply(0.35));
        }
        let color = if is_selected {
            p.accent
        } else if hovered {
            p.text
        } else {
            p.muted
        };
        ui.painter().text(
            segment.center(),
            Align2::CENTER_CENTER,
            *label,
            font.clone(),
            color,
        );
        if response.has_focus() {
            ui.painter().rect_stroke(
                inner,
                CornerRadius::same(8),
                Stroke::new(2.0_f32, p.accent),
                StrokeKind::Inside,
            );
        }
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::RadioButton, true, is_selected, *label)
        });
        if response.clicked() {
            chosen = Some(*value);
        }
        x += widths[index];
    }
    chosen
}

pub fn text_input(
    ui: &mut Ui,
    p: &Palette,
    id_salt: &str,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    let id = ui.make_persistent_id(("text-input", id_salt));
    let frame = Frame::new()
        .fill(p.surface_alt)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_width((width - 24.0).max(40.0));
            ui.add(
                TextEdit::singleline(value)
                    .id(id)
                    .hint_text(RichText::new(hint).font(sans(14.5)).color(p.muted))
                    .font(sans(14.5))
                    .text_color(p.text)
                    .frame(false)
                    .desired_width(ui.available_width())
                    .vertical_align(Align::Center),
            )
        });
    paint_input_outline(
        ui,
        p,
        frame.response.rect,
        frame.inner.has_focus(),
        frame.inner.hovered(),
    );
    frame.inner
}

pub fn search_input(
    ui: &mut Ui,
    p: &Palette,
    id_salt: &str,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    let id = ui.make_persistent_id(("search-input", id_salt));
    let frame = Frame::new()
        .fill(p.surface_alt)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_width((width - 24.0).max(40.0));
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                icons::search(ui.painter(), icon_rect, p.muted, 1.5);
                ui.add_space(4.0);
                let clear_width = if value.is_empty() { 0.0 } else { 24.0 };
                let edit_width = (ui.available_width() - clear_width - 4.0).max(30.0);
                let response = ui.add(
                    TextEdit::singleline(value)
                        .id(id)
                        .hint_text(RichText::new(hint).font(sans(14.0)).color(p.muted))
                        .font(sans(14.0))
                        .text_color(p.text)
                        .frame(false)
                        .desired_width(edit_width)
                        .vertical_align(Align::Center),
                );
                if !value.is_empty() {
                    if icon_button(ui, p, Icon::Close, "Clear search", 22.0).clicked() {
                        value.clear();
                    }
                }
                response
            })
            .inner
        });
    paint_input_outline(
        ui,
        p,
        frame.response.rect,
        frame.inner.has_focus(),
        frame.inner.hovered(),
    );
    frame.inner
}

pub fn password_input(
    ui: &mut Ui,
    p: &Palette,
    id_salt: &str,
    value: &mut String,
    hint: &str,
    width: f32,
    show: &mut bool,
) -> Response {
    let id = ui.make_persistent_id(("password-input", id_salt));
    let frame = Frame::new()
        .fill(p.surface_alt)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_width((width - 24.0).max(40.0));
            ui.horizontal(|ui| {
                let edit_width = (ui.available_width() - 28.0).max(40.0);
                let response = ui.add(
                    TextEdit::singleline(value)
                        .id(id)
                        .password(!*show)
                        .hint_text(RichText::new(hint).font(sans(14.5)).color(p.muted))
                        .font(sans(14.5))
                        .text_color(p.text)
                        .frame(false)
                        .desired_width(edit_width)
                        .vertical_align(Align::Center),
                );
                let icon = if *show { Icon::EyeOff } else { Icon::Eye };
                let tooltip = if *show {
                    "Hide password"
                } else {
                    "Show password"
                };
                if icon_button(ui, p, icon, tooltip, 22.0).clicked() {
                    *show = !*show;
                    ui.memory_mut(|memory| memory.request_focus(id));
                }
                response
            })
            .inner
        });
    paint_input_outline(
        ui,
        p,
        frame.response.rect,
        frame.inner.has_focus(),
        frame.inner.hovered(),
    );
    frame.inner
}

fn paint_input_outline(ui: &mut Ui, p: &Palette, rect: Rect, focused: bool, hovered: bool) {
    if focused {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(10),
            Stroke::new(2.0_f32, p.accent),
            StrokeKind::Inside,
        );
    } else if hovered {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(10),
            Stroke::new(1.0_f32, p.muted),
            StrokeKind::Inside,
        );
    }
}

pub fn tag(ui: &mut Ui, p: &Palette, text: &str, color: Color32) {
    let text = text.to_uppercase();
    let font = mono_medium(10.5);
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), color);
    let size = Vec2::new(galley.size().x + 14.0, 20.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(6), p.surface_alt);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, color);
}

pub fn status(ui: &mut Ui, _p: &Palette, text: &str, color: Color32, pulse: bool) -> Response {
    let font = medium(13.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), color);
    let width = 8.0 + 7.0 + 7.0 + galley.size().x;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 20.0), Sense::hover());
    let center = Pos2::new(rect.left() + 4.0, rect.center().y);
    if pulse {
        let time = ui.input(|input| input.time);
        let amount =
            (0.16 + 0.16 * (time * std::f64::consts::PI).sin().abs() as f32).clamp(0.0, 0.4);
        ui.painter().circle_stroke(
            center,
            8.0,
            Stroke::new(2.0_f32, color.gamma_multiply(amount)),
        );
    }
    ui.painter().circle_filled(center, 3.5, color);
    ui.painter().text(
        Pos2::new(rect.left() + 15.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
    response
}

pub fn note(ui: &mut Ui, p: &Palette, title: &str, message: &str, color: Color32) {
    let frame = Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width((ui.available_width() - 8.0).max(120.0));
            ui.horizontal_top(|ui| {
                let (rule, _) = ui.allocate_exact_size(Vec2::new(3.0, 34.0), Sense::hover());
                ui.painter().rect_filled(rule, CornerRadius::same(2), color);
                ui.add_space(9.0);
                ui.vertical(|ui| {
                    label(ui, title, medium(13.5), color);
                    ui.add_space(2.0);
                    caption(ui, p, message);
                });
            });
        });
    ui.painter().rect_stroke(
        frame.response.rect,
        CornerRadius::same(10),
        Stroke::new(1.0_f32, p.line),
        StrokeKind::Inside,
    );
}

pub fn metric(ui: &mut Ui, p: &Palette, title: &str, value: &str, secondary: Option<&str>) {
    ui.horizontal(|ui| {
        label(ui, title, sans(13.0), p.muted);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            label(ui, value, mono_medium(12.5), p.text);
            if let Some(secondary) = secondary {
                label(ui, secondary, mono(11.5), p.muted);
                ui.add_space(8.0);
            }
        });
    });
}

pub fn log_level_color(p: &Palette, level: LogLevel) -> Color32 {
    match level {
        LogLevel::Info => p.muted,
        LogLevel::Warn => p.warn,
        LogLevel::Error => p.danger,
    }
}

pub fn setting_row(
    ui: &mut Ui,
    p: &Palette,
    title: &str,
    description: &str,
    control_width: f32,
    add_control: impl FnOnce(&mut Ui),
) {
    let total = ui.available_width();
    let control_width = control_width.min(total * 0.55);
    let label_width = (total - control_width - 36.0).max(150.0);
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(label_width, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(label_width);
                label(ui, title, medium(14.5), p.text);
                if !description.is_empty() {
                    ui.add_space(4.0);
                    ui.add(
                        Label::new(RichText::new(description).font(sans(13.0)).color(p.muted))
                            .wrap(),
                    );
                }
            },
        );
        ui.add_space(28.0);
        let remaining = ui.available_width().max(control_width);
        ui.allocate_ui_with_layout(
            Vec2::new(remaining, 40.0),
            Layout::right_to_left(Align::Center),
            |ui| add_control(ui),
        );
    });
    ui.add_space(12.0);
}

pub fn route_visual(
    ui: &mut Ui,
    p: &Palette,
    left_label: &str,
    right_label: &str,
    right_detail: Option<&str>,
    active: bool,
    busy: bool,
    failed: bool,
) {
    let width = ui.available_width().max(240.0);
    let height = 112.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter().clone();
    let start = Pos2::new(rect.left() + 26.0, rect.top() + 66.0);
    let end = Pos2::new(rect.right() - 26.0, rect.top() + 40.0);
    let c1 = Pos2::new(rect.left() + width * 0.32, rect.top() + 8.0);
    let c2 = Pos2::new(rect.left() + width * 0.68, rect.top() + 102.0);
    let color = if failed {
        p.danger
    } else if active || busy {
        p.accent
    } else {
        p.muted
    };
    let line_color = if active || busy || failed {
        color.gamma_multiply(0.9)
    } else {
        p.line
    };
    let points: Vec<Pos2> = (0..=48)
        .map(|index| cubic(start, c1, c2, end, index as f32 / 48.0))
        .collect();
    painter.add(egui::Shape::line(points, Stroke::new(2.0_f32, line_color)));

    painter.circle_filled(start, 4.5, p.muted);
    painter.circle_stroke(start, 10.0, Stroke::new(1.0_f32, p.line));

    let destination_color = if failed {
        p.danger
    } else if active {
        p.ok
    } else if busy {
        p.accent
    } else {
        p.muted
    };
    if active {
        let time = ui.input(|input| input.time);
        let amount =
            (0.18 + 0.18 * (time * std::f64::consts::PI).sin().abs() as f32).clamp(0.0, 0.4);
        painter.circle_stroke(
            end,
            15.0,
            Stroke::new(2.0_f32, destination_color.gamma_multiply(amount)),
        );
    }
    painter.circle_filled(end, 7.0, destination_color);
    painter.circle_stroke(end, 7.0, Stroke::new(1.5_f32, line_color));

    if busy {
        let time = ui.input(|input| input.time);
        let t = ((time * 0.62) % 1.0) as f32;
        let dot = cubic(start, c1, c2, end, t);
        painter.circle_filled(dot, 4.0, p.accent);
        painter.circle_stroke(
            dot,
            7.0,
            Stroke::new(1.0_f32, p.accent.gamma_multiply(0.35)),
        );
    }

    ui.horizontal_top(|ui| {
        let half = (width - 8.0) / 2.0;
        ui.add_space(18.0);
        ui.allocate_ui_with_layout(
            Vec2::new(half - 18.0, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                caption(ui, p, left_label);
            },
        );
        ui.allocate_ui_with_layout(
            Vec2::new((width - half - 18.0).max(80.0), 0.0),
            Layout::top_down(Align::Max),
            |ui| {
                ui.add(
                    Label::new(
                        RichText::new(right_label)
                            .font(medium(14.0))
                            .color(if failed { p.danger } else { p.text }),
                    )
                    .wrap(),
                );
                if let Some(detail) = right_detail {
                    ui.add_space(1.0);
                    ui.add(
                        Label::new(RichText::new(detail).font(mono(11.5)).color(p.muted)).wrap(),
                    );
                }
            },
        );
        ui.add_space(18.0);
    });
}

fn cubic(start: Pos2, c1: Pos2, c2: Pos2, end: Pos2, t: f32) -> Pos2 {
    let one = 1.0 - t;
    Pos2::new(
        start.x * one * one * one
            + 3.0 * c1.x * one * one * t
            + 3.0 * c2.x * one * t * t
            + end.x * t * t * t,
        start.y * one * one * one
            + 3.0 * c1.y * one * one * t
            + 3.0 * c2.y * one * t * t
            + end.y * t * t * t,
    )
}

pub fn nav_icon(screen: crate::model::Screen) -> Icon {
    match screen {
        crate::model::Screen::Connect => Icon::Power,
        crate::model::Screen::Network => Icon::Globe,
        crate::model::Screen::Locations => Icon::Pin,
        crate::model::Screen::Activity => Icon::Activity,
        crate::model::Screen::Settings => Icon::Settings,
    }
}

pub fn screen_shortcut(screen: crate::model::Screen) -> &'static str {
    match screen {
        crate::model::Screen::Connect => "1",
        crate::model::Screen::Network => "2",
        crate::model::Screen::Locations => "3",
        crate::model::Screen::Activity => "4",
        crate::model::Screen::Settings => "5",
    }
}
