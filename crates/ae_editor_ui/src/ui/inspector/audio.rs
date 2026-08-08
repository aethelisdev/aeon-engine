// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the AudioSource inspector panel section.
    pub(super) fn draw_audio_source_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(source) = world.get::<&ae_audio::AudioSource>(entity) {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🔊 AudioSource")
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").on_hover_text("Remove AudioSource").clicked() {
                            ui_actions.push(EngineUiAction::RemoveAudioSource(entity));
                        }
                    });
                });
                ui.separator();

                let mut updated = (*source).clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Sound Path:");
                    if ui.text_edit_singleline(&mut updated.sound_path).changed() {
                        changed = true;
                    }
                    if ui
                        .button("📁")
                        .on_hover_text("Pick sound file (.wav, .ogg, .mp3)")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio File", &["wav", "ogg", "mp3", "flac"])
                            .pick_file()
                        {
                            updated.sound_path = path.to_string_lossy().to_string();
                            changed = true;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    if ui
                        .add(egui::Slider::new(&mut updated.volume, 0.0..=2.0).text("Gain"))
                        .changed()
                    {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Pitch:");
                    if ui
                        .add(egui::Slider::new(&mut updated.pitch, 0.1..=3.0).text("Speed"))
                        .changed()
                    {
                        changed = true;
                    }
                });

                if ui
                    .checkbox(&mut updated.is_spatial, "3D Spatial Audio")
                    .on_hover_text("Enable 3D distance falloff & stereo panning")
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .checkbox(&mut updated.looping, "Loop Sound")
                    .on_hover_text("Repeat sound when reaching EOF")
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .checkbox(&mut updated.play_on_start, "Play on Start")
                    .on_hover_text("Auto-start sound playback when spawned")
                    .changed()
                {
                    changed = true;
                }

                if updated.is_spatial {
                    ui.horizontal(|ui| {
                        ui.label("Min Dist:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut updated.min_distance)
                                    .speed(0.5)
                                    .range(0.1..=100.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label("Max Dist:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut updated.max_distance)
                                    .speed(1.0)
                                    .range(1.0..=1000.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                }

                if changed {
                    ui_actions.push(EngineUiAction::ModifyAudioSource(entity, updated));
                }
            });
        }
    }

    /// Renders the AudioListener inspector panel section.
    pub(super) fn draw_audio_listener_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if world.get::<&ae_audio::AudioListener>(entity).is_ok() {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("👂 AudioListener")
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button("🗑")
                            .on_hover_text("Remove AudioListener")
                            .clicked()
                        {
                            ui_actions.push(EngineUiAction::RemoveAudioListener(entity));
                        }
                    });
                });
                ui.label("Active 3D spatial ear position.");
            });
        }
    }
}