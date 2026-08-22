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
            let mut updated = (*source).clone();
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "Audio Source",
                "🔊",
                egui::Color32::from_rgb(150, 220, 255),
                true,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Sound Path:");
                        if ui.text_edit_singleline(&mut updated.sound_path).changed() {
                            changed = true;
                        }
                        if ui
                            .button("📁")
                            .on_hover_text("Pick sound file (.wav, .ogg, .mp3)")
                            .clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .add_filter("Audio File", &["wav", "ogg", "mp3", "flac"])
                                .pick_file()
                        {
                            updated.sound_path = path.to_string_lossy().to_string();
                            changed = true;
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

                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut updated.looping, "Looping").changed() {
                            changed = true;
                        }
                        if ui
                            .checkbox(&mut updated.is_spatial, "Spatial (3D)")
                            .on_hover_text("Calculates 3D attenuation and panning based on entity and listener transforms")
                            .changed()
                        {
                            changed = true;
                        }
                    });

                    if updated.is_spatial {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Min Dist:");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut updated.min_distance)
                                        .speed(0.1)
                                        .range(0.1..=100.0),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
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
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveComponent(entity, "AudioSource"));
            }

            if changed {
                ui_actions.push(EngineUiAction::modify_component(
                    entity,
                    "AudioSource",
                    &updated,
                ));
            }
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
            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "Audio Listener",
                "👂",
                egui::Color32::from_rgb(150, 220, 255),
                true,
                |ui| {
                    ui.label("Active 3D spatial ear position.");
                },
            );
            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveComponent(entity, "AudioListener"));
            }
        }
    }
}

pub struct AudioSourceUiHandler;

impl super::registry::ComponentUiHandler for AudioSourceUiHandler {
    fn component_name(&self) -> &'static str {
        "AudioSource"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Audio Source", "🔊", egui::Color32::from_rgb(150, 220, 255))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Audio", "Audio Source")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_audio::AudioSource>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_audio_source_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }
}

pub struct AudioListenerUiHandler;

impl super::registry::ComponentUiHandler for AudioListenerUiHandler {
    fn component_name(&self) -> &'static str {
        "AudioListener"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Audio Listener",
            "👂",
            egui::Color32::from_rgb(150, 220, 255),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Audio", "Audio Listener")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_audio::AudioListener>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_audio_listener_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }
}