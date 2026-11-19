// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport & Auxiliary Tool Overlay Action Dispatchers
//!
//! Handles action events for Viewport HUD, Performance Stats, Scene Hierarchy,
//! Animation Timeline, Material Studio, and UI Designer overlays.

use crate::ui::iris_bridge;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::EngineUi;

impl EngineUi {
    /// Drains and processes all queued actions from the Animation Timeline Studio panel.
    pub fn process_timeline_actions(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let Some(entity) = self.selected_entity else {
            self.iris_overlay.timeline.actions.clear();
            return;
        };

        for action in self.iris_overlay.timeline.take_actions() {
            match action {
                iris_bridge::TimelineAction::TogglePlayPause => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.state = if updated.state == ae_animation::AnimationState::Playing {
                            ae_animation::AnimationState::Paused
                        } else {
                            ae_animation::AnimationState::Playing
                        };
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::Stop => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.state = ae_animation::AnimationState::Stopped;
                        updated.current_time = 0.0;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::StepFrame(delta_frames) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        let duration = updated
                            .current_clip
                            .as_ref()
                            .map_or(1.0, |c| c.duration.max(0.1));
                        let frame_step = 1.0 / 30.0;
                        updated.current_time = (updated.current_time
                            + delta_frames as f32 * frame_step)
                            .clamp(0.0, duration);
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::ToggleLoop => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.looping = !updated.looping;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::SetSpeed(speed) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.speed = speed;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::ScrubTo(time) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        let duration = updated
                            .current_clip
                            .as_ref()
                            .map_or(1.0, |c| c.duration.max(0.1));
                        updated.current_time = time.clamp(0.0, duration);
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::AddAnimationPlayer(ent) => {
                    ui_actions.push(EngineUiAction::AddComponent(ent, "AnimationPlayer"));
                }
            }
        }
    }
}