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
                        ae_animation::AnimationState::Playing => egui::RichText::new("▶ PLAYING")
                            .color(egui::Color32::GREEN)
                            .strong(),
                        ae_animation::AnimationState::Paused => egui::RichText::new("⏸ PAUSED")
                            .color(egui::Color32::GOLD)
                            .strong(),
                        ae_animation::AnimationState::Stopped => {
                            egui::RichText::new("⏹ STOPPED").color(egui::Color32::LIGHT_GRAY)
                        }
                    };

                    // Status & Quick Link to Timeline Studio
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        ui.label(state_badge);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button("Open Timeline Studio ↗")
                                .on_hover_text("Open full Animation Timeline panel with transport controls and scrubbing")
                                .clicked()
                            {
                                ui_actions.push(EngineUiAction::OpenPanel(
                                    crate::ui::panel_layout::PanelId::AnimationTimeline,
                                ));
                            }
                        });
                    });

                    ui.separator();

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
                    let duration = updated.current_clip.as_ref().map_or(0.0, |c| c.duration);
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

                    if changed {
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveComponent(entity, "AnimationPlayer"));
            }
        }
    }
}

pub struct AnimationUiHandler;

impl super::registry::ComponentUiHandler for AnimationUiHandler {
    fn component_name(&self) -> &'static str {
        "AnimationPlayer"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Animation Player",
            "🎬",
            egui::Color32::from_rgb(255, 150, 200),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Animation", "Animation Player")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_animation::AnimationPlayer>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_animation_section(ui, ctx.world, ctx.entity, ctx.models, ctx.ui_actions);
    }
}