// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Event Hit-Testing Subsystem
//!
//! Evaluates playback button activations, playhead dragging,
//! and timeline scrubbing timestamp projections via semantic tags.
//!

use super::types::{MediaTransportAction, TimelineAction, evaluate_timeline_transport_tag};

/// Evaluates a mouse click against timeline semantic tags and returns the corresponding action.
pub fn handle_timeline_click(
    hit_tag: u64,
    _entity: Option<hecs::Entity>,
) -> Option<TimelineAction> {
    if let Some(transport_action) =
        evaluate_timeline_transport_tag(hit_tag, &super::transport::SPEED_PRESETS)
    {
        let action = match transport_action {
            MediaTransportAction::TogglePlayPause => TimelineAction::TogglePlayPause,
            MediaTransportAction::Stop => TimelineAction::Stop,
            MediaTransportAction::StepBack => TimelineAction::StepFrame(-1),
            MediaTransportAction::StepForward => TimelineAction::StepFrame(1),
            MediaTransportAction::ToggleLoop => TimelineAction::ToggleLoop,
            MediaTransportAction::SetSpeed(speed) => TimelineAction::SetSpeed(speed),
        };
        return Some(action);
    }

    None
}

/// Computes an absolute scrub timestamp from a physical cursor position and track bounding metrics.
#[inline]
pub fn compute_scrub_timestamp(
    cursor_x: f32,
    track_x: f32,
    track_width: f32,
    clip_duration: f32,
) -> f32 {
    let safe_width = track_width.max(1.0);
    let frac = ((cursor_x - track_x) / safe_width).clamp(0.0, 1.0);
    frac * clip_duration.max(0.0)
}