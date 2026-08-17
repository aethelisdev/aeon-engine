// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the Animation Player inspector panel section.
    /// Shows active animation state controls (Play, Pause, Stop), playback speed slider,
    /// clip selector dropdown from model animations, loop toggle, and crossfade transition controls.
    pub fn draw_animation_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
            let mut updated = (*player).clone();
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "Animation Player",
                "🎬",
                egui::Color32::from_rgb(255, 150, 200),
                true,
                |ui| {
                    let state_badge = match updated.state {
                        ae_animation::AnimationState::Playing => {
                            egui::RichText::new(" [▶ PLAYING]")
                                .color(egui::Color32::GREEN)
                                .strong()
                        }
                        ae_animation::AnimationState::Paused => egui::RichText::new(" [⏸ PAUSED]")
                            .color(egui::Color32::GOLD)
                            .strong(),
                        ae_animation::AnimationState::Stopped => {
                            egui::RichText::new(" [⏹ STOPPED]").color(egui::Color32::LIGHT_GRAY)
                        }
                    };

                    // State controls
                    ui.horizontal(|ui| {
                        ui.label("State:");
                        if ui.button("▶ Play").clicked() {
                            updated.state = ae_animation::AnimationState::Playing;
                            changed = true;
                        }
                        if ui.button("⏸ Pause").clicked() {
                            updated.state = ae_animation::AnimationState::Paused;
                            changed = true;
                        }
                        if ui.button("⏹ Stop").clicked() {
                            updated.state = ae_animation::AnimationState::Stopped;
                            updated.current_time = 0.0;
                            changed = true;
                        }
                        ui.label(state_badge);
                    });

                    // Timeline Seek / Scrubbing Slider
                    let duration = updated
                        .current_clip
                        .as_ref()
                        .map_or(1.0, |c| c.duration.max(0.1));
                    ui.horizontal(|ui| {
                        ui.label("Timeline:");
                        let progress_pct =
                            (updated.current_time / duration * 100.0).clamp(0.0, 100.0);
                        if ui
                            .add(
                                egui::Slider::new(&mut updated.current_time, 0.0..=duration)
                                    .text(format!("s ({:.0}%)", progress_pct))
                                    .fixed_decimals(2),
                            )
                            .on_hover_text("Drag slider to seek / scrub timeline position")
                            .changed()
                        {
                            changed = true;
                        }
                    });

                    // Active clip display & dropdown selector
                    let model_anims =
                        if let Ok(model_id) = world.get::<&ae_core::ecs::ModelId>(entity) {
                            models.get(model_id.0).map(|m| &m.animations)
                        } else {
                            None
                        };

                    let current_name = updated
                        .current_clip
                        .as_ref()
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "No Clip Selected".to_string());

                    ui.horizontal(|ui| {
                        ui.label("Active Clip:");
                        #[allow(deprecated)]
                        egui::ComboBox::from_id_salt(egui::Id::new(("anim_clip_combo", entity)))
                            .selected_text(current_name)
                            .show_ui(ui, |ui| {
                                if let Some(anims) = model_anims {
                                    for clip in anims {
                                        let is_selected = updated
                                            .current_clip
                                            .as_ref()
                                            .is_some_and(|c| c.name == clip.name);
                                        if ui.selectable_label(is_selected, &clip.name).clicked() {
                                            updated.play(clip.clone());
                                            changed = true;
                                        }
                                    }
                                }
                            });
                    });

                    // Clip & Skeleton Info
                    if let Ok(skel) = world.get::<&ae_animation::Skeleton>(entity) {
                        let num_channels = updated
                            .current_clip
                            .as_ref()
                            .map_or(0, |c| c.channels.len());
                        ui.label(
                            egui::RichText::new(format!(
                                "🦴 Joints: {} | 🎞 Channels: {} | ⏱ Duration: {:.2}s",
                                skel.joints.len(),
                                num_channels,
                                duration
                            ))
                            .small()
                            .color(egui::Color32::LIGHT_BLUE),
                        );
                    } else {
                        ui.colored_label(
                        egui::Color32::GOLD,
                        "ℹ Static 3D Model (No Skeletal Armature/Bones found).\nExport with 3D Skeletal Armature/Bones to use skeletal animation.",
                    );
                    }

                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        if ui
                            .add(egui::Slider::new(&mut updated.speed, 0.1..=5.0).text("x"))
                            .changed()
                        {
                            changed = true;
                        }
                    });

                    if ui
                        .checkbox(&mut updated.looping, "Loop Animation")
                        .on_hover_text("Repeat animation when reaching clip duration")
                        .changed()
                    {
                        changed = true;
                    }

                    if changed {
                        ui_actions.push(EngineUiAction::ModifyAnimationPlayer(entity, updated));
                    }
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveAnimationPlayer(entity));
            }
        }
    }
}