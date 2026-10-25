use crate::{
    config::{random_key, Config},
    engine::{Engine, Status},
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
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if let Some(tex) = &self.logo_tex {
                    ui.add(egui::Image::new((tex.id(), egui::vec2(38.0, 38.0))).fit_to_exact_size(egui::vec2(38.0, 38.0)));
                    ui.add_space(10.0);
                }
                ui.heading(RichText::new("MultiDash").size(22.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (txt, col) = match status {
                        Status::Stopped => ("STOPPED", Color32::from_rgb(120, 130, 150)),
                        Status::Listening => ("WAITING", Color32::from_rgb(245, 190, 60)),
                        Status::Live => ("LIVE", Color32::from_rgb(255, 70, 70)),
                    };
                    egui::Frame::none()
                        .fill(col.gamma_multiply(0.18))
                        .rounding(egui::Rounding::same(20))
                        .inner_margin(egui::Margin::symmetric(12, 4))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("● {txt}")).color(col).strong());
                        });
                });
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ---- Ingest -------------------------------------------------
                let ingest_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(22, 24, 30))
                    .rounding(egui::Rounding::same(12))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(48, 52, 66)))
                    .inner_margin(egui::Margin::same(16));
                ingest_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("INGEST").size(12.0).color(Color32::from_rgb(150, 160, 190)).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Put this in OBS → Settings → Stream → Custom").color(Color32::from_rgb(130, 140, 170)));
                        });
                    });
                    ui.add_space(10.0);
                    let server = format!("rtmp://{}:{}/live", self.local_ip, self.cfg.ingest_port);
                    egui::Grid::new("ingest_grid").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
                        ui.label(RichText::new("Server").color(Color32::from_rgb(190, 200, 220)));
                        ui.horizontal(|ui| {
                            let mut server_txt = server.clone();
                            ui.add_sized([380.0, 0.0], egui::TextEdit::singleline(&mut server_txt).font(egui::TextStyle::Monospace));
                            if ui.add(egui::Button::new("Copy")).clicked() {
                                ui.ctx().copy_text(server.clone());
                            }
                        });
                        ui.end_row();
                        ui.label(RichText::new("Stream Key").color(Color32::from_rgb(190, 200, 220)));
                        ui.horizontal(|ui| {
                            ui.add_sized([320.0, 0.0], egui::TextEdit::singleline(&mut self.cfg.ingest_key).font(egui::TextStyle::Monospace));
                            if ui.add(egui::Button::new("Copy")).clicked() {
                                ui.ctx().copy_text(self.cfg.ingest_key.clone());
                            }
                            if ui.add_enabled(!locked, egui::Button::new("New Key")).clicked() {
                                self.cfg.ingest_key = random_key();
                                changed = true;
                            }
                        });
                        ui.end_row();
                        ui.label(RichText::new("Port").color(Color32::from_rgb(190, 200, 220)));
                        changed |= ui.add_enabled(!locked, egui::DragValue::new(&mut self.cfg.ingest_port).range(1024..=65535)).changed();
                        ui.end_row();
                    });
                });

                ui.add_space(12.0);

                // ---- Platform slots ----------------------------------------
                ui.horizontal(|ui| {
                    ui.label(RichText::new("DESTINATIONS").size(12.0).color(Color32::from_rgb(150, 160, 190)).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.show_keys, "Show keys");
                    });
                });
                ui.add_space(8.0);
                for (i, slot) in self.cfg.slots.iter_mut().enumerate() {
                    let slot_frame = egui::Frame::none()
                        .fill(Color32::from_rgb(22, 24, 30))
                        .rounding(egui::Rounding::same(12))
                        .stroke(egui::Stroke::new(1.0, if slot.enabled { Color32::from_rgb(90, 110, 255) } else { Color32::from_rgb(48, 52, 66) }))
                        .inner_margin(egui::Margin::same(16));
                    slot_frame.show(ui, |ui| {
                        ui.add_enabled_ui(!locked, |ui| {
                            ui.horizontal(|ui| {
                                changed |= ui.checkbox(&mut slot.enabled, "").changed();
                                let name_label = if slot.name.trim().is_empty() { format!("Destination {}", i + 1) } else { slot.name.clone() };
                                ui.label(RichText::new(name_label).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    changed |= ui.add(egui::TextEdit::singleline(&mut slot.name).hint_text("Name").desired_width(140.0)).changed();
                                });
                            });
                            ui.add_space(10.0);
                            egui::Grid::new(("slot", i)).num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
                                ui.label(RichText::new("RTMP URL").color(Color32::from_rgb(190, 200, 220)));
                                changed |= ui.add(egui::TextEdit::singleline(&mut slot.rtmp_url).hint_text("rtmp://.../app").desired_width(f32::INFINITY)).changed();
                                ui.end_row();
                                ui.label(RichText::new("Stream Key").color(Color32::from_rgb(190, 200, 220)));
                                changed |= ui.add(egui::TextEdit::singleline(&mut slot.stream_key).password(!show_keys).desired_width(f32::INFINITY)).changed();
                                ui.end_row();
                            });
                        });
                    });
                    ui.add_space(10.0);
                }

                ui.add_space(6.0);

                // ---- Controls ----------------------------------------------
                ui.horizontal(|ui| {
                    if locked {
                        let stop = egui::Button::new(RichText::new("■  STOP MULTIDASH").strong()).fill(Color32::from_rgb(190, 40, 40));
                        if ui.add_sized([200.0, 38.0], stop).clicked() {
                            self.engine.stop();
                        }
                    } else {
                        let has_target = self.cfg.slots.iter().any(|s| s.enabled && !s.rtmp_url.trim().is_empty() && !s.stream_key.trim().is_empty());
                        let mut go = egui::Button::new(RichText::new("▶  START MULTIDASH").strong()).fill(Color32::from_rgb(90, 110, 255));
                        if !has_target { go = go.sense(egui::Sense::hover()); }
                        if has_target && ui.add_sized([200.0, 38.0], go).clicked() {
                            self.cfg.save();
                            self.engine.start(self.cfg.clone());
                        }
                        if !has_target {
                            ui.add_space(10.0);
                            ui.label(RichText::new("Enable at least one destination with RTMP URL + stream key").color(Color32::from_rgb(150, 160, 190)));
                        }
                    }
                    if !stats.is_empty() {
                        ui.add_space(14.0);
                        ui.label(RichText::new(&stats).monospace().color(Color32::from_rgb(150, 160, 190)));
                    }
                });

                ui.add_space(16.0);
                let log_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(18, 20, 26))
                    .rounding(egui::Rounding::same(12))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(44, 48, 62)))
                    .inner_margin(egui::Margin::same(14));
                log_frame.show(ui, |ui| {
                    ui.label(RichText::new("ACTIVITY LOG").size(12.0).color(Color32::from_rgb(150, 160, 190)).strong());
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for l in &log {
                                ui.label(RichText::new(l).monospace().color(Color32::from_rgb(200, 206, 226)));
                            }
                        });
                });
                ui.add_space(10.0);
            });
        });

        if changed {
            self.cfg.save();
        }
    }
}
