// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Ruler and Interactive Scrubber
//!
//! Connects active animation clip duration, cached keyframes, and playback timestamp
//! directly to [`UiScope::time_ruler`] and [`UiScope::scrubber_track_value`] with
//! zero-`let` declarative `ActionBus` updates.

use super::types::{
    DEFAULT_RULER_HEIGHT, DEFAULT_SCRUBBER_HEIGHT, TIMELINE_SIDEBAR_WIDTH,
    TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_RULER_ROOT, TIMELINE_TAG_SCRUBBER_PLAYHEAD,
    TIMELINE_TAG_SCRUBBER_TRACK, TimelineClipCache, emit_player_mutation,
};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Emits the adaptive time ruler, keyframe markers, and interactive playhead scrubber via [`UiScope`].
pub fn build_ruler_and_scrubber(
    scope: &mut UiScope<'_>,
    entity: ae_ecs::Entity,
    player: &ae_animation::AnimationPlayer,
    duration: f32,
    cache: &mut TimelineClipCache,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    cache.sync(Some(player));

    scope.container_named(
        "TimelineRulerAndScrubberSection",
        Style::new()
            .flex_col()
            .gap(2.0)
            .padding_insets(Insets::new(4.0, 10.0, 4.0, 10.0)),
        |col| {
            col.time_ruler(
                TimeRulerSpec::new(TIMELINE_TAG_RULER_ROOT, duration)
                    .name("TimelineRulerRoot")
                    .height(DEFAULT_RULER_HEIGHT)
                    .sidebar_width(TIMELINE_SIDEBAR_WIDTH),
            );
            col.scrubber_track_value(
                ScrubberTrackSpec::new(TIMELINE_TAG_SCRUBBER_TRACK, duration)
                    .name("TimelineScrubberTrack")
                    .playhead_tag(TIMELINE_TAG_SCRUBBER_PLAYHEAD)
                    .cap_tag(TIMELINE_TAG_PLAYHEAD_CAP)
                    .height(DEFAULT_SCRUBBER_HEIGHT)
                    .sidebar_width(TIMELINE_SIDEBAR_WIDTH)
                    .markers(cache.markers()),
                player.current_time.clamp(0.0, duration),
            )
            .on_scrub(|new_time| {
                emit_player_mutation(entity, player, action_bus, |p| {
                    p.current_time = new_time.clamp(0.0, duration);
                });
            });
        },
    );
}

/// Updates only the volatile scrubber playhead overlay in place during `InPlaceRefresh`.
pub fn refresh_scrubber_live(scope: &mut UiScope<'_>, duration: f32, current_time: f32) -> bool {
    scope.update_scrubber_playhead(
        TIMELINE_TAG_SCRUBBER_PLAYHEAD,
        duration,
        current_time,
        DEFAULT_SCRUBBER_HEIGHT,
        TimeRulerStyle::dark_default(),
    )
}