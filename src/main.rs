mod app;
mod backend;
mod cg;
mod countries;
mod format;
mod icons;
mod model;
mod nm;
mod profile;
mod store;
mod theme;
mod ui;

use eframe::egui;

use crate::app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Ghostline")
            .with_app_id("io.ghostline.Ghostline")
            .with_inner_size(App::initial_window_size())
            .with_min_inner_size([1000.0, 660.0])
            .with_icon(icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Ghostline",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

fn icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../ghostline.png"))
        .expect("ghostline.png must be a valid PNG")
}
