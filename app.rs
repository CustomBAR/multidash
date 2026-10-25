use crate::{
    config::{random_key, Config},
    engine::{Engine, Status},
};
use eframe::egui::{self, Color32, RichText};
use std::time::Duration;

pub struct MultiDash {
    cfg: Config,
    engine: Engine,
    local_ip: String,
    show_keys: bool,
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
        }
    }
}

impl eframe::App for MultiDash {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(250));

        let (status, stats, log) = {
            let s = self.engine.shared.lock().unwrap();
            (s.status, s.stats.clone(), s.log.iter().cloned().collect::<Vec<_>>())
        };
        let locked = status != Status::Stopped;
        let show_keys = self.show_keys;
        let mut changed = false;

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("MultiDash");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (txt, col) = match status {
                        Status::Stopped => ("Stopped", Color32::GRAY),
                        Status::Listening => ("Waiting for OBS", Color32::from_rgb(230, 170, 40)),
                        Status::Live => ("LIVE", Color32::from_rgb(230, 60, 60)),
                    };
                    ui.label(RichText::new(format!("● {txt}")).color(col).strong());
                });
            });
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ---- Ingest -------------------------------------------------
                ui.group(|ui| {
                    ui.strong("Ingest - put this in OBS (Settings > Stream > Custom)");
                    ui.add_space(4.0);
                    let server = format!("rtmp://{}:{}/live", self.local_ip, self.cfg.ingest_port);
                    ui.horizontal(|ui| {
                        ui.label("Server:    ");
                        ui.monospace(&server);
                        if ui.small_button("Copy").clicked() {
                            ui.ctx().copy_text(server.clone());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Stream Key:");
                        ui.monospace(&self.cfg.ingest_key);
                        if ui.small_button("Copy").clicked() {
                            ui.ctx().copy_text(self.cfg.ingest_key.clone());
                        }
                        if ui.add_enabled(!locked, egui::Button::new("New key").small()).clicked() {
                            self.cfg.ingest_key = random_key();
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Port:      ");
                        changed |= ui
                            .add_enabled(
                                !locked,
                                egui::DragValue::new(&mut self.cfg.ingest_port).range(1024..=65535),
                            )
                            .changed();
                    });
                });

                ui.add_space(8.0);

                // ---- Platform slots ----------------------------------------
                for (i, slot) in self.cfg.slots.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.add_enabled_ui(!locked, |ui| {
                            ui.horizontal(|ui| {
                                changed |= ui.checkbox(&mut slot.enabled, "").changed();
                                ui.label(format!("Slot {}", i + 1));
                                changed |= ui
                                    .add(egui::TextEdit::singleline(&mut slot.name).desired_width(160.0))
                                    .changed();
                            });
                            egui::Grid::new(("slot", i)).num_columns(2).show(ui, |ui| {
                                ui.label("RTMP URL");
                                changed |= ui
                                    .add(
                                        egui::TextEdit::singleline(&mut slot.rtmp_url)
                                            .hint_text("rtmp://live.twitch.tv/app")
                                            .desired_width(f32::INFINITY),
                                    )
                                    .changed();
                                ui.end_row();
                                ui.label("Stream Key");
                                changed |= ui
                                    .add(
                                        egui::TextEdit::singleline(&mut slot.stream_key)
                                            .password(!show_keys)
                                            .desired_width(f32::INFINITY),
                                    )
                                    .changed();
                                ui.end_row();
                            });
                        });
                    });
                    ui.add_space(4.0);
                }

                ui.checkbox(&mut self.show_keys, "Show stream keys");
                ui.add_space(8.0);

                // ---- Controls ----------------------------------------------
                ui.horizontal(|ui| {
                    if locked {
                        let stop = egui::Button::new(RichText::new("■  Stop MultiDash").strong())
                            .fill(Color32::from_rgb(150, 40, 40));
                        if ui.add_sized([180.0, 32.0], stop).clicked() {
                            self.engine.stop();
                        }
                    } else {
                        let go = egui::Button::new(RichText::new("▶  Start MultiDash").strong());
                        if ui.add_sized([180.0, 32.0], go).clicked() {
                            self.cfg.save();
                            self.engine.start(self.cfg.clone());
                        }
                    }
                    if !stats.is_empty() {
                        ui.monospace(&stats);
                    }
                });

                ui.add_space(8.0);
                ui.collapsing("Log", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for l in &log {
                                ui.monospace(l);
                            }
                        });
                });
            });
        });

        if changed {
            self.cfg.save();
        }
    }
}
