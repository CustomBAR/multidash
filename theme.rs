use eframe::egui::{self, Color32, Rounding, Style, Visuals};

pub const BG_DARK: Color32 = Color32::from_rgb(18, 19, 23);
pub const BG_PANEL: Color32 = Color32::from_rgb(24, 25, 30);
pub const BG_CARD: Color32 = Color32::from_rgb(28, 29, 35);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(34, 35, 42);
pub const BORDER: Color32 = Color32::from_rgb(52, 54, 64);
pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(255, 107, 28);
pub const ORANGE: Color32 = Color32::from_rgb(255, 107, 28);
pub const ORANGE_HOVER: Color32 = Color32::from_rgb(255, 130, 48);
pub const ORANGE_ACTIVE: Color32 = Color32::from_rgb(230, 90, 18);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 242);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(170, 174, 188);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(130, 134, 150);

pub fn rumble_visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(TEXT_PRIMARY);
    v.panel_fill = BG_PANEL;
    v.window_fill = BG_DARK;
    v.faint_bg_color = BG_CARD;
    v.extreme_bg_color = BG_DARK;
    v.code_bg_color = BG_ELEVATED;
    v.widgets.noninteractive.bg_fill = BG_CARD;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    v.widgets.inactive.bg_fill = BG_ELEVATED;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.hovered.bg_fill = Color32::from_rgb(42, 43, 52);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(90, 92, 110));
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.active.bg_fill = ORANGE_ACTIVE;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, ORANGE);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.open.bg_fill = BG_ELEVATED;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, BORDER_ACTIVE);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(255, 107, 28, 40);
    v.selection.stroke.color = ORANGE;
    v.hyperlink_color = ORANGE;
    v.warn_fg_color = Color32::from_rgb(255, 170, 60);
    v.error_fg_color = Color32::from_rgb(255, 80, 80);
    v
}

pub fn apply_theme(ctx: &egui::Context) {
    ctx.set_visuals(rumble_visuals());
    let mut style = Style::default();
    style.visuals = rumble_visuals();
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(14);
    style.spacing.button_padding = egui::vec2(18.0, 10.0);
    style.spacing.indent = 20.0;
    style.spacing.combo_width = 200.0;
    style.spacing.text_edit_width = 280.0;
    style.spacing.interact_size = egui::vec2(40.0, 24.0);
    ctx.set_style(style);
}
