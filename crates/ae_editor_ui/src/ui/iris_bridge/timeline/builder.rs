// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Panel Autonomous Builder
//!
//! Autonomous builder coordinating animation clip playback, keyframe track rendering,
//! immediate canvas scrubbing needles, and transport controls.
//!

use super::panel::build_timeline_panel;
use super::types::TimelinePanelParams;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Animation Timeline Studio panel if active in either docked or floating mode.
    ///
    /// Manages the full animation timeline lifecycle:
    /// 1. Resolves physical panel bounds directly from the unified [`UiScope`] docking hierarchy.
    /// 2. Resolves [`ae_animation::AnimationPlayer`] attached to the currently selected entity.
    /// 3. Drains pending pointer scrub and drag interaction events.
    /// 4. Assembles transport toolbar, track lane list, and dynamic hardware scrubber in unified coordinates.
    pub(crate) fn build_timeline_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            self.timeline.is_dragging = false;
            self.timeline.active_scrubber_track = None;
            return;
        }

        let anim_player = params.scene.selected_entity.and_then(|ent| {
            params
                .scene
                .world
                .get::<&ae_animation::AnimationPlayer>(ent)
                .ok()
        });

        let events = std::mem::take(&mut self.timeline.pending_interaction_events);

        let timeline_params = TimelinePanelParams {
            panel_rect,
            entity: params.scene.selected_entity,
            animation_player: anim_player.as_deref(),
            cursor_pos: self.cursor_pos(),
            is_dragging_scrubber: self.timeline.is_dragging,
            events: &events,
        };

        let duration = build_timeline_panel(scope, &timeline_params, &mut self.timeline.actions);
        self.timeline.clip_duration = duration;
    }
}