// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Ruler and Interactive Scrubber Bridge
//!
//! Connects engine animation clip keyframe data and playback timestamp to pure declarative
//! [`UiScope`] ruler and scrubber primitives in Iris UI.
//!

use super::types::TimelinePanelParams;
use irisui::prelude::*;

/// Height of the time ruler section above the track in physical pixels.
pub const RULER_HEIGHT: f32 = 18.0;

/// Height of the interactive scrubber track in physical pixels.
pub const SCRUBBER_TRACK_HEIGHT: f32 = 36.0;

/// Total height occupied by the ruler and scrubber subsystem.
pub const RULER_TOTAL_HEIGHT: f32 = RULER_HEIGHT + SCRUBBER_TRACK_HEIGHT + 6.0;

/// Builds the time ruler, keyframe markers, and interactive playhead scrubber via [`UiScope`].
pub fn build_ruler_and_scrubber(
    scope: &mut UiScope<'_>,
    params: &TimelinePanelParams<'_>,
    duration: f32,
) {
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

    let ruler_style = TimelineRulerStyle::dark_default();

    let section_style = Style::new()
        .flex_col()
        .gap(2.0)
        .padding_insets(Insets::new(4.0, 10.0, 4.0, 10.0));

    scope.container_named("TimelineRulerAndScrubberSection", section_style, |col| {
        let _ = col.timeline_ruler_bar(duration, RULER_HEIGHT, &ruler_style);
        let _ = col.timeline_scrubber_track(
            duration,
            current_time,
            params.is_dragging_scrubber,
            &keyframes,
            SCRUBBER_TRACK_HEIGHT,
            &ruler_style,
        );
    });
}