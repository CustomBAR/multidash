mod app;
mod config;
mod engine;
mod theme;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([660.0, 820.0])
            .with_min_inner_size([560.0, 620.0])
            .with_app_id("multidash"),
        ..Default::default()
    };
    eframe::run_native(
        "MultiDash",
        options,
        Box::new(|cc| {
            theme::apply_theme(&cc.egui_ctx);
            let _ = image::open("assets/RumbleDash Logo.png");
            Ok(Box::new(app::MultiDash::new()))
        }),
    )
}
