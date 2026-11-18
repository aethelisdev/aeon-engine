// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Panel Autonomous Builder
//!
//! Autonomous builder coordinating animation clip playback, keyframe track rendering,
//! immediate canvas scrubbing needles, and transport controls.

use super::panel::build_timeline_panel;
use super::types::{TimelinePanelParams, TimelinePanelState};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;

impl TimelinePanelState {
    /// Builds the Animation Timeline Studio panel directly using typed timeline state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
    ) {
        if scope.is_collapsed() {
            self.is_dragging = false;
            self.active_scrubber_track = None;
            return;
        }

        self.selected_entity = params.scene.selected_entity;
        self.clip_duration = build_timeline_panel(
            scope,
            &TimelinePanelParams {
                panel_rect: scope.parent_rect(),
                entity: params.scene.selected_entity,
                animation_player: params
                    .scene
                    .selected_entity
                    .and_then(|ent| {
                        params
                            .scene
                            .world
                            .get::<&ae_animation::AnimationPlayer>(ent)
                            .ok()
                    })
                    .as_deref(),
                cursor_pos,
                is_dragging_scrubber: self.is_dragging,
                events: &std::mem::take(&mut self.pending_interaction_events),
            },
            &mut self.actions,
        );
    }
}