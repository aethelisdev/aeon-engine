// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Panel Autonomous Builder
//!
//! Autonomous builder coordinating animation clip playback, keyframe track rendering,
//! immediate canvas scrubbing needles, and transport controls.

use super::panel::build_timeline_panel;
use super::types::{
    TIMELINE_TAG_PANEL_ROOT, TimelinePanelParams, TimelinePanelState, timeline_live_hash,
    timeline_structural_hash,
};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use irisui::dock::DeferredNotifier;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl TimelinePanelState {
    /// Builds the Animation Timeline Studio panel directly using typed timeline state,
    /// cached keyframe markers, and `ActionBus<EngineUiAction>`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        action_bus: &mut ActionBus<EngineUiAction>,
    ) {
        if scope.is_collapsed() {
            return;
        }

        self.build_and_record(
            scope,
            params.scene.selected_entity,
            params
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
            action_bus,
        );
    }

    fn build_and_record(
        &mut self,
        scope: &mut UiScope<'_>,
        entity: Option<hecs::Entity>,
        player: Option<&ae_animation::AnimationPlayer>,
        action_bus: &mut ActionBus<EngineUiAction>,
    ) {
        self.selected_entity = entity;
        self.clip_duration = build_timeline_panel(
            scope,
            &TimelinePanelParams {
                entity,
                animation_player: player,
            },
            &mut self.cache,
            action_bus,
        );
        scope.record_retained_revision(
            TIMELINE_TAG_PANEL_ROOT,
            timeline_structural_hash(entity, player),
            timeline_live_hash(player),
        );
    }

    /// Streams 60 FPS playhead, scrubber, and time readout updates in place during `InPlaceRefresh`
    /// via `UiScope::sync_retained_revision`, triggering granular rebuilds only on structural edits.
    pub(crate) fn refresh_live(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        notifier: &mut DeferredNotifier<PanelId>,
    ) {
        Self::refresh_player_live(
            scope,
            params.scene.selected_entity,
            params
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
            notifier,
        );
    }

    fn refresh_player_live(
        scope: &mut UiScope<'_>,
        entity: Option<hecs::Entity>,
        player: Option<&ae_animation::AnimationPlayer>,
        notifier: &mut DeferredNotifier<PanelId>,
    ) {
        match scope.sync_retained_revision(
            TIMELINE_TAG_PANEL_ROOT,
            timeline_structural_hash(entity, player),
            timeline_live_hash(player),
        ) {
            RetainedRevisionStatus::Unchanged => {}
            RetainedRevisionStatus::Rebuild => {
                notifier.tag_redraw(PanelId::AnimationTimeline);
            }
            RetainedRevisionStatus::RefreshLive => {
                if let Some(p) = player {
                    super::transport::update_time_readout_live(
                        scope,
                        p.current_time,
                        p.current_clip.as_ref().map_or(1.0, |c| c.duration.max(0.1)),
                        30.0,
                    );
                    super::ruler::refresh_scrubber_live(
                        scope,
                        p.current_clip.as_ref().map_or(1.0, |c| c.duration.max(0.1)),
                        p.current_time,
                    );
                    super::lanes::refresh_lanes_playhead_live(
                        scope,
                        p.current_clip.as_ref().map_or(1.0, |c| c.duration.max(0.1)),
                        p.current_time,
                    );
                }
            }
        }
    }
}