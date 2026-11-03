use std::sync::Arc;

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Accent,
    Ok,
    Warning,
    Danger,
}

impl Tone {
    pub fn color(self, palette: &Palette) -> Color32 {
        match self {
            Self::Neutral => palette.muted,
            Self::Accent => palette.accent,
            Self::Ok => palette.ok,
            Self::Warning => palette.warn,
            Self::Danger => palette.danger,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub line: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_ink: Color32,
    pub ok: Color32,
    pub danger: Color32,
    pub warn: Color32,
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: Color32::from_rgb(0x0F, 0x12, 0x16),
            surface: Color32::from_rgb(0x17, 0x1C, 0x21),
            surface_alt: Color32::from_rgb(0x22, 0x2A, 0x31),
            line: Color32::from_rgb(0x2C, 0x34, 0x3B),
            text: Color32::from_rgb(0xF2, 0xEF, 0xE6),
            muted: Color32::from_rgb(0xA6, 0xAD, 0xA9),
            accent: Color32::from_rgb(0xE9, 0xB4, 0x4C),
            accent_ink: Color32::from_rgb(0x17, 0x13, 0x0A),
            ok: Color32::from_rgb(0x82, 0xB7, 0x8D),
            danger: Color32::from_rgb(0xD9, 0x77, 0x5C),
            warn: Color32::from_rgb(0xE0, 0xA3, 0x5A),
        }
    } else {
        Palette {
            bg: Color32::from_rgb(0xF2, 0xEF, 0xE6),
            surface: Color32::from_rgb(0xFB, 0xF9, 0xF3),
            surface_alt: Color32::from_rgb(0xE8, 0xE3, 0xD7),
            line: Color32::from_rgb(0xD6, 0xCF, 0xC0),
            text: Color32::from_rgb(0x17, 0x1A, 0x1D),
            muted: Color32::from_rgb(0x5C, 0x62, 0x5F),
            accent: Color32::from_rgb(0x98, 0x60, 0x00),
            accent_ink: Color32::from_rgb(0xFF, 0xF8, 0xE6),
            ok: Color32::from_rgb(0x25, 0x6B, 0x3B),
            danger: Color32::from_rgb(0xA3, 0x3B, 0x21),
            warn: Color32::from_rgb(0x8A, 0x5A, 0x00),
        }
    }
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "plex_sans".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "plex_sans_medium".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/IBMPlexSans-Medium.ttf")).into(),
    );
    fonts.font_data.insert(
        "plex_sans_semibold".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/IBMPlexSans-SemiBold.ttf")).into(),
    );
    fonts.font_data.insert(
        "plex_mono".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "plex_mono_medium".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf")).into(),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "plex_sans".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "plex_mono".to_owned());
    fonts.families.insert(
        FontFamily::Name(Arc::from("medium")),
        vec!["plex_sans_medium".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(Arc::from("semibold")),
        vec!["plex_sans_semibold".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(Arc::from("mono_medium")),
        vec!["plex_mono_medium".to_owned()],
    );
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    let p = palette(dark);
    let mut style = (*ctx.style()).clone();
    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.override_text_color = Some(p.text);
    visuals.panel_fill = p.bg;
    visuals.window_fill = p.surface;
    visuals.extreme_bg_color = p.surface_alt;
    visuals.faint_bg_color = p.surface_alt;
    visuals.code_bg_color = p.surface_alt;
    visuals.window_stroke = Stroke::new(1.0_f32, p.line);
    visuals.window_corner_radius = egui::CornerRadius::same(14);
    visuals.menu_corner_radius = egui::CornerRadius::same(10);
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    visuals.selection.bg_fill = p.accent.gamma_multiply(0.34);
    visuals.selection.stroke = Stroke::new(1.0_f32, p.accent);
    visuals.hyperlink_color = p.accent;
    visuals.warn_fg_color = p.warn;
    visuals.error_fg_color = p.danger;
    visuals.striped = false;
    visuals.button_frame = true;
    visuals.slider_trailing_fill = true;

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = p.surface;
    widgets.noninteractive.weak_bg_fill = p.surface;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.muted);
    widgets.noninteractive.corner_radius = egui::CornerRadius::same(10);

    widgets.inactive.bg_fill = p.surface_alt;
    widgets.inactive.weak_bg_fill = p.surface_alt;
    widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.line);
    widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    widgets.inactive.expansion = 0.0;

    widgets.hovered.bg_fill = p.line;
    widgets.hovered.weak_bg_fill = p.line;
    widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.muted);
    widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
    widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    widgets.hovered.expansion = 0.0;

    widgets.active.bg_fill = p.accent;
    widgets.active.weak_bg_fill = p.accent;
    widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    widgets.active.fg_stroke = Stroke::new(1.0_f32, p.accent_ink);
    widgets.active.corner_radius = egui::CornerRadius::same(8);
    widgets.active.expansion = 0.0;

    widgets.open.bg_fill = p.surface_alt;
    widgets.open.bg_stroke = Stroke::new(1.0_f32, p.line);
    widgets.open.fg_stroke = Stroke::new(1.0_f32, p.text);
    widgets.open.corner_radius = egui::CornerRadius::same(8);

    style.visuals = visuals;
    style.animation_time = 0.18;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.interact_size.y = 36.0;
    style.spacing.menu_margin = egui::Margin::same(8);
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(15.0, FontFamily::Name(Arc::from("medium"))),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(22.0, FontFamily::Name(Arc::from("semibold"))),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(13.5, FontFamily::Monospace),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.5, FontFamily::Proportional),
    );
    ctx.set_style(style);
}

pub fn system_prefers_dark() -> bool {
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if value.contains("prefer-dark") {
                return true;
            }
            if value.contains("prefer-light") || value.contains("default") {
                return false;
            }
        }
    }
    if let Ok(value) = std::env::var("GTK_THEME") {
        if value.to_lowercase().contains("dark") {
            return true;
        }
    }
    if let Ok(value) = std::env::var("COLORSCHEME") {
        if value.to_lowercase().contains("dark") {
            return true;
        }
    }
    true
}

#[cfg(test)]
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let la = luminance(a);
    let lb = luminance(b);
    let (light, dark) = if la > lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

#[cfg(test)]
fn luminance(color: Color32) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_contrast_passes_aa_in_both_themes() {
        for dark in [true, false] {
            let p = palette(dark);
            assert!(
                contrast_ratio(p.text, p.bg) >= 4.5,
                "text on bg, dark={dark}"
            );
            assert!(
                contrast_ratio(p.text, p.surface) >= 4.5,
                "text on surface, dark={dark}"
            );
            assert!(
                contrast_ratio(p.muted, p.surface) >= 4.5,
                "muted on surface, dark={dark}"
            );
            assert!(
                contrast_ratio(p.accent_ink, p.accent) >= 4.5,
                "accent ink on accent, dark={dark}"
            );
            assert!(contrast_ratio(p.ok, p.surface) >= 4.5, "ok, dark={dark}");
            assert!(
                contrast_ratio(p.danger, p.surface) >= 4.5,
                "danger, dark={dark}"
            );
        }
    }
}
