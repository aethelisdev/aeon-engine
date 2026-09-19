// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Ruler and Interactive Scrubber Bridge
//!
//! Connects engine animation clip keyframe data and playback timestamp to the generic
//! [`TimelineRulerBuilder`] widget in Iris UI.
//!

use super::types::{TimelinePanelParams, TimelinePanelTargets};
use irisui::prelude::*;

/// Height of the time ruler section above the track in physical pixels.
pub const RULER_HEIGHT: f32 = 18.0;

/// Height of the interactive scrubber track in physical pixels.
pub const SCRUBBER_TRACK_HEIGHT: f32 = 36.0;

/// Total height occupied by the ruler and scrubber subsystem.
pub const RULER_TOTAL_HEIGHT: f32 = RULER_HEIGHT + SCRUBBER_TRACK_HEIGHT + 6.0;

/// Builds the time ruler, keyframe markers, and interactive playhead scrubber into the UI tree.
///
/// Delegates all node hierarchy generation, dynamic tick calculations, and playhead rendering
/// directly to Iris UI's [`TimelineRulerBuilder`].
pub fn build_ruler_and_scrubber(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    targets: &mut TimelinePanelTargets,
    start_y: f32,
    duration: f32,
) {
    let padding_x = 10.0;
    let available_w = (params.panel_rect.width - padding_x * 2.0).max(100.0);
    let ruler_y = start_y + 4.0;

    let player = params.animation_player;
    let current_time = player.map_or(0.0, |p| p.current_time).clamp(0.0, duration);

    // Extract unique keyframe timestamps across all vector and rotation channels
    let mut keyframes: Vec<TimelineKeyframeMarker> = Vec::new();
    if let Some(clip) = player.and_then(|p| p.current_clip.as_ref()) {
        for channel in &clip.channels {
            if let Some(ref track) = channel.vector_track {
                for kf in &track.keyframes {
                    if !keyframes.iter().any(|k| (k.time - kf.time).abs() < 0.02) {
                        keyframes.push(TimelineKeyframeMarker::new(kf.time));
                    }
                }
            }
            if let Some(ref track) = channel.rotation_track {
                for kf in &track.keyframes {
                    if !keyframes.iter().any(|k| (k.time - kf.time).abs() < 0.02) {
                        keyframes.push(TimelineKeyframeMarker::new(kf.time));
                    }
                }
            }
        }
        keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let ruler_rect = Rect::new(
        params.panel_rect.x + padding_x,
        ruler_y,
        available_w,
        RULER_HEIGHT + SCRUBBER_TRACK_HEIGHT + 4.0,
    );

    let frame = TimelineRulerBuilder::new(ruler_rect, duration, current_time)
        .keyframes(&keyframes)
        .is_dragging(params.is_dragging_scrubber)
        .cursor_pos(Some(params.cursor_pos))
        .heights(RULER_HEIGHT, SCRUBBER_TRACK_HEIGHT)
        .build(tree, parent_id);

    targets.scrubber_track_rect = Some(frame.track_rect);
    targets.playhead_needle_rect = Some(frame.playhead_handle_rect);
    targets.clip_duration = duration;
}