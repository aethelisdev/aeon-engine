// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Event Hit-Testing Subsystem
//!
//! Evaluates mouse clicks, playback button activations, playhead dragging,
//! and timeline scrubbing timestamp projections.
//!

use super::types::{TimelineAction, TimelinePanelTargets};
use irisui::prelude::Point;

/// Evaluates a mouse click against timeline semantic tags and returns the corresponding action.
///
/// Returns `Some((action, start_dragging))` where `start_dragging` is true when the
/// user clicked on the scrubber track or playhead needle to initiate dragging.
pub fn handle_timeline_click(
    targets: &TimelinePanelTargets,
    hit_tag: u64,
    click_pos: Point,
    _entity: Option<hecs::Entity>,
) -> Option<(TimelineAction, bool)> {
    if let Some(transport_action) =
        irisui::prelude::evaluate_timeline_transport_tag(hit_tag, &super::transport::SPEED_PRESETS)
    {
        let action = match transport_action {
            irisui::prelude::MediaTransportAction::TogglePlayPause => {
                TimelineAction::TogglePlayPause
            }
            irisui::prelude::MediaTransportAction::Stop => TimelineAction::Stop,
            irisui::prelude::MediaTransportAction::StepBack => TimelineAction::StepFrame(-1),
            irisui::prelude::MediaTransportAction::StepForward => TimelineAction::StepFrame(1),
            irisui::prelude::MediaTransportAction::ToggleLoop => TimelineAction::ToggleLoop,
            irisui::prelude::MediaTransportAction::SetSpeed(speed) => {
                TimelineAction::SetSpeed(speed)
            }
        };
        return Some((action, false));
    }

    // Playhead cap or scrubber track click and drag initiation
    if (hit_tag == irisui::prelude::TIMELINE_TAG_PLAYHEAD_CAP
        || hit_tag == irisui::prelude::TIMELINE_TAG_SCRUBBER_TRACK)
        && let Some(track) = targets.scrubber_track_rect
    {
        let frac = ((click_pos.x - track.x) / track.width).clamp(0.0, 1.0);
        let scrub_t = frac * targets.clip_duration;
        return Some((TimelineAction::ScrubTo(scrub_t), true));
    }

    None
}

/// Evaluates mouse dragging movement across the scrubber track and returns the scrub timestamp.
pub fn handle_timeline_drag(
    targets: &TimelinePanelTargets,
    cursor_pos: Point,
) -> Option<TimelineAction> {
    let track = targets.scrubber_track_rect?;
    let frac = ((cursor_pos.x - track.x) / track.width).clamp(0.0, 1.0);
    let scrub_t = frac * targets.clip_duration;
    Some(TimelineAction::ScrubTo(scrub_t))
}