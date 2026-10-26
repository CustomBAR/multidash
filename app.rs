use crate::{
    config::{random_key, Config},
    engine::{Engine, Status},
    theme,
};
use eframe::egui::{self, Color32, RichText, TextureHandle};
use std::time::Duration;

pub struct MultiDash {
    cfg: Config,
    engine: Engine,
    local_ip: String,
    show_keys: bool,
    logo_tex: Option<TextureHandle>,
}

fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(46.0, 26.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let bg = if *on { theme::ORANGE } else { theme::BG_ELEVATED };
        let stroke = if *on { theme::ORANGE } else { theme::BORDER };
        ui.painter().rect(rect, egui::Rounding::same(999), bg, egui::Stroke::new(1.0, stroke));
        let circle_pos = if *on { rect.right_center() - egui::vec2(13.0, 0.0) } else { rect.left_center() + egui::vec2(13.0, 0.0) };
        ui.painter().circle_filled(circle_pos, 10.0, Color32::WHITE);
    }
    response
}

impl MultiDash {
    pub fn new() -> Self {
        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".into());
        Self {
            cfg: Config::load(),
            engine: Engine::new(),
            local_ip,
            show_keys: false,
            logo_tex: None,
        }
    }

    fn load_logo(ctx: &egui::Context, logo_tex: &mut Option<TextureHandle>) {
        if logo_tex.is_some() {
            return;
        }
        if let Ok(img) = image::open("assets/RumbleDash Logo.png") {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            *logo_tex = Some(ctx.load_texture("multidash_logo", color_image, egui::TextureOptions::LINEAR));
        }
    }
}

impl eframe::App for MultiDash {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(250));
        Self::load_logo(ctx, &mut self.logo_tex);

        let (status, stats, log) = {
            let s = self.engine.shared.lock().unwrap();
            (s.status, s.stats.clone(), s.log.iter().cloned().collect::<Vec<_>>())
        };
        let locked = status != Status::Stopped;
        let show_keys = self.show_keys;
        let mut changed = false;

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if let Some(tex) = &self.logo_tex {
                    ui.add(egui::Image::new((tex.id(), egui::vec2(40.0, 40.0))).fit_to_exact_size(egui::vec2(40.0, 40.0)));
                    ui.add_space(12.0);
                }
                ui.vertical(|ui| {
                    ui.label(RichText::new("MULTIDASH").size(18.0).strong().color(theme::TEXT_PRIMARY));
                    ui.label(RichText::new("Multi-Stream Relay").size(11.0).color(theme::TEXT_MUTED));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (txt, col) = match status {
                        Status::Stopped => ("IDLE", theme::TEXT_MUTED),
                        Status::Listening => ("WAITING FOR OBS", theme::ORANGE),
                        Status::Live => ("LIVE", Color32::from_rgb(255, 72, 40)),
                    };
                    egui::Frame::none()
                        .fill(col.gamma_multiply(0.16))
                        .rounding(egui::Rounding::same(999))
                        .stroke(egui::Stroke::new(1.0, col.gamma_multiply(0.6)))
                        .inner_margin(egui::Margin::symmetric(14, 6))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("● {txt}")).color(col).strong().size(11.0));
                        });
                });
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ---- Ingest -------------------------------------------------
                let ingest_frame = egui::Frame::none()
                    .fill(theme::BG_CARD)
                    .rounding(egui::Rounding::same(14))
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(egui::Margin::same(18));
                ingest_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("INGEST").size(11.0).color(theme::TEXT_MUTED).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Add this to OBS → Settings → Stream → Custom").size(11.0).color(theme::TEXT_MUTED));
                        });
                    });
                    ui.add_space(12.0);
                    let server = format!("rtmp://{}:{}/live", self.local_ip, self.cfg.ingest_port);
                    egui::Grid::new("ingest_grid").num_columns(2).spacing([14.0, 12.0]).show(ui, |ui| {
                        ui.label(RichText::new("Server").color(theme::TEXT_SECONDARY));
                        ui.horizontal(|ui| {
                            let mut server_txt = server.clone();
                            ui.add_sized([400.0, 30.0], egui::TextEdit::singleline(&mut server_txt).font(egui::TextStyle::Monospace));
                            if ui.add(egui::Button::new("Copy")).clicked() {
                                ui.ctx().copy_text(server.clone());
                            }
                        });
                        ui.end_row();
                        ui.label(RichText::new("Stream Key").color(theme::TEXT_SECONDARY));
                        ui.horizontal(|ui| {
                            ui.add_sized([340.0, 30.0], egui::TextEdit::singleline(&mut self.cfg.ingest_key).font(egui::TextStyle::Monospace));
                            if ui.add(egui::Button::new("Copy")).clicked() {
                                ui.ctx().copy_text(self.cfg.ingest_key.clone());
                            }
                            if ui.add_enabled(!locked, egui::Button::new("Generate")).clicked() {
                                self.cfg.ingest_key = random_key();
                                changed = true;
                            }
                        });
                        ui.end_row();
                        ui.label(RichText::new("Port").color(theme::TEXT_SECONDARY));
                        changed |= ui.add_enabled(!locked, egui::DragValue::new(&mut self.cfg.ingest_port).range(1024..=65535)).changed();
                        ui.end_row();
                    });
                });

                ui.add_space(14.0);

                // ---- Platform slots ----------------------------------------
                ui.horizontal(|ui| {
                    ui.label(RichText::new("DESTINATIONS").size(11.0).color(theme::TEXT_MUTED).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Show keys").color(theme::TEXT_SECONDARY));
                        ui.add_space(6.0);
                        let mut show = self.show_keys;
                        if toggle_switch(ui, &mut show).changed() {
                            self.show_keys = show;
                        }
                    });
                });
                ui.add_space(10.0);
                for (i, slot) in self.cfg.slots.iter_mut().enumerate() {
                    let enabled = slot.enabled;
                    let slot_frame = egui::Frame::none()
                        .fill(theme::BG_CARD)
                        .rounding(egui::Rounding::same(14))
                        .stroke(egui::Stroke::new(1.0, if enabled { theme::BORDER_ACTIVE } else { theme::BORDER }))
                        .inner_margin(egui::Margin::same(18));
                    slot_frame.show(ui, |ui| {
                        ui.add_enabled_ui(!locked, |ui| {
                            ui.horizontal(|ui| {
                                let mut en = slot.enabled;
                                if toggle_switch(ui, &mut en).changed() {
                                    slot.enabled = en;
                                    changed = true;
                                }
                                ui.add_space(10.0);
                                let name_label = if slot.name.trim().is_empty() { format!("Destination {}", i + 1) } else { slot.name.clone() };
                                ui.label(RichText::new(name_label).strong().color(theme::TEXT_PRIMARY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    changed |= ui.add(egui::TextEdit::singleline(&mut slot.name).hint_text("Custom name").desired_width(150.0)).changed();
                                });
                            });
                            ui.add_space(12.0);
                            egui::Grid::new(("slot", i)).num_columns(2).spacing([14.0, 12.0]).show(ui, |ui| {
                                ui.label(RichText::new("RTMP URL").color(theme::TEXT_SECONDARY));
                                changed |= ui.add(egui::TextEdit::singleline(&mut slot.rtmp_url).hint_text("rtmp://live.platform.com/app").desired_width(f32::INFINITY)).changed();
                                ui.end_row();
                                ui.label(RichText::new("Stream Key").color(theme::TEXT_SECONDARY));
                                changed |= ui.add(egui::TextEdit::singleline(&mut slot.stream_key).password(!show_keys).desired_width(f32::INFINITY)).changed();
                                ui.end_row();
                            });
                        });
                    });
                    ui.add_space(12.0);
                }

                ui.add_space(4.0);

                // ---- Controls ----------------------------------------------
                ui.horizontal(|ui| {
                    if locked {
                        let stop = egui::Button::new(RichText::new("■  STOP").strong()).fill(Color32::from_rgb(190, 36, 20));
                        if ui.add_sized([170.0, 40.0], stop).clicked() {
                            self.engine.stop();
                        }
                    } else {
                        let has_target = self.cfg.slots.iter().any(|s| s.enabled && !s.rtmp_url.trim().is_empty() && !s.stream_key.trim().is_empty());
                        let mut go = egui::Button::new(RichText::new("▶  START MULTIDASH").strong()).fill(theme::ORANGE);
                        if !has_target { go = go.sense(egui::Sense::hover()); }
                        if has_target && ui.add_sized([210.0, 40.0], go).clicked() {
                            self.cfg.save();
                            self.engine.start(self.cfg.clone());
                        }
                        if !has_target {
                            ui.add_space(12.0);
                            ui.label(RichText::new("Enable at least one destination with RTMP URL and Stream Key").color(theme::TEXT_MUTED));
                        }
                    }
                    if !stats.is_empty() {
                        ui.add_space(16.0);
                        ui.label(RichText::new(&stats).monospace().color(theme::TEXT_MUTED));
                    }
                });

                ui.add_space(18.0);
                let log_frame = egui::Frame::none()
                    .fill(theme::BG_CARD)
                    .rounding(egui::Rounding::same(14))
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(egui::Margin::same(18));
                log_frame.show(ui, |ui| {
                    ui.label(RichText::new("ACTIVITY").size(11.0).color(theme::TEXT_MUTED).strong());
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical()
                        .max_height(190.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for l in &log {
                                ui.label(RichText::new(l).monospace().color(theme::TEXT_SECONDARY));
                            }
                        });
                });
                ui.add_space(12.0);
            });
        });

        if changed {
            self.cfg.save();
        }
    }
}
