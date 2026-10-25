use eframe::egui::{self, Color32, Style, Visuals};

pub fn rumble_visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(Color32::from_rgb(235, 235, 235));
    v.panel_fill = Color32::from_rgb(16, 18, 22);
    v.window_fill = Color32::from_rgb(16, 18, 22);
    v.faint_bg_color = Color32::from_rgb(24, 26, 32);
    v.extreme_bg_color = Color32::from_rgb(12, 14, 18);
    v.code_bg_color = Color32::from_rgb(28, 30, 36);
    v.widgets.noninteractive.bg_fill = Color32::from_rgb(24, 26, 32);
    v.widgets.inactive.bg_fill = Color32::from_rgb(32, 34, 42);
    v.widgets.hovered.bg_fill = Color32::from_rgb(40, 42, 52);
    v.widgets.active.bg_fill = Color32::from_rgb(52, 54, 66);
    v.widgets.open.bg_fill = Color32::from_rgb(40, 42, 52);
    v.selection.bg_fill = Color32::from_rgb(90, 100, 220);
    v.selection.stroke.color = Color32::from_rgb(140, 150, 255);
    v.hyperlink_color = Color32::from_rgb(110, 150, 255);
    v.warn_fg_color = Color32::from_rgb(255, 190, 80);
    v.error_fg_color = Color32::from_rgb(255, 90, 90);
    v
}

pub fn apply_theme(ctx: &egui::Context) {
    ctx.set_visuals(rumble_visuals());
    let mut style = Style::default();
    style.visuals = rumble_visuals();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.indent = 18.0;
    ctx.set_style(style);
}
