mod app;
mod config;
mod engine;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([580.0, 740.0])
            .with_min_inner_size([480.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MultiDash",
        options,
        Box::new(|_cc| Ok(Box::new(app::MultiDash::new()))),
    )
}
