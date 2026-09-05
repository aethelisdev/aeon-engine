// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Event Hit-Testing Subsystem
//!
//! Evaluates mouse clicks, playback button activations, playhead dragging,
//! and timeline scrubbing timestamp projections.
//!

use super::types::{TimelineAction, TimelinePanelTargets};
use irisui::prelude::Point;

/// Evaluates a mouse click against timeline targets and returns the corresponding action.
/// Returns `Some((action, start_dragging))` where `start_dragging` is true when the
/// user clicked on the scrubber track or playhead needle to initiate dragging.
pub fn handle_timeline_click(
    targets: &TimelinePanelTargets,
    click_pos: Point,
    entity: Option<hecs::Entity>,
) -> Option<(TimelineAction, bool)> {
    if let Some(r) = targets.play_pause_btn
        && r.contains_point(click_pos)
    {
        return Some((TimelineAction::TogglePlayPause, false));
    }

    if let Some(r) = targets.stop_btn
        && r.contains_point(click_pos)
    {
        return Some((TimelineAction::Stop, false));
    }

    if let Some(r) = targets.step_back_btn
        && r.contains_point(click_pos)
    {
        return Some((TimelineAction::StepFrame(-1), false));
    }

    if let Some(r) = targets.step_fwd_btn
        && r.contains_point(click_pos)
    {
        return Some((TimelineAction::StepFrame(1), false));
    }

    if let Some(r) = targets.loop_toggle
        && r.contains_point(click_pos)
    {
        return Some((TimelineAction::ToggleLoop, false));
    }

    for &(speed, rect) in &targets.speed_buttons {
        if rect.contains_point(click_pos) {
            return Some((TimelineAction::SetSpeed(speed), false));
        }
    }

    if let Some(r) = targets.add_player_btn
        && r.contains_point(click_pos)
        && let Some(ent) = entity
    {
        return Some((TimelineAction::AddAnimationPlayer(ent), false));
    }

    // Playhead cap handle drag initiation
    if let Some(cap) = targets.playhead_needle_rect
        && cap.contains_point(click_pos)
        && let Some(track) = targets.scrubber_track_rect
    {
        let frac = ((click_pos.x - track.x) / track.width).clamp(0.0, 1.0);
        let scrub_t = frac * targets.clip_duration;
        return Some((TimelineAction::ScrubTo(scrub_t), true));
    }

    // Scrubber track click and drag initiation
    if let Some(track) = targets.scrubber_track_rect
        && track.contains_point(click_pos)
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