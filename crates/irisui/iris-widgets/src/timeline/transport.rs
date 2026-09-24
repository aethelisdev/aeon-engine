// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Media Playback Transport Tags and Evaluation (`iris-widgets::timeline::transport`)
//!
//! Exposes persistent semantic tags, speed multipliers, and evaluation routines
//! for timeline transport actions in Iris UI.
//!

use super::types::MediaTransportAction;

/// Standard playback speed multipliers.
pub const DEFAULT_SPEED_PRESETS: [f32; 4] = [0.25, 0.5, 1.0, 2.0];

/// Semantic tag for the timeline panel root container background.
pub const TIMELINE_TAG_PANEL_ROOT: u64 = 0xAA00;
/// Semantic tag for the timeline step back button.
pub const TIMELINE_TAG_STEP_BACK: u64 = 0xAA01;
/// Semantic tag for the timeline play/pause toggle button.
pub const TIMELINE_TAG_PLAY_PAUSE: u64 = 0xAA02;
/// Semantic tag for the timeline stop playback button.
pub const TIMELINE_TAG_STOP: u64 = 0xAA03;
/// Semantic tag for the timeline step forward button.
pub const TIMELINE_TAG_STEP_FWD: u64 = 0xAA04;
/// Semantic tag for the timeline loop toggle pill.
pub const TIMELINE_TAG_LOOP: u64 = 0xAA05;
/// Base semantic tag for playback speed multiplier pills (offset by preset index).
pub const TIMELINE_TAG_SPEED_BASE: u64 = 0xAA10;
/// Semantic tag for the timeline scrubber track interactive region.
pub const TIMELINE_TAG_SCRUBBER_TRACK: u64 = 0xAA20;
/// Semantic tag for the timeline playhead draggable needle cap handle.
pub const TIMELINE_TAG_PLAYHEAD_CAP: u64 = 0xAA21;

/// Returns true if the specified numeric tag belongs to the timeline subsystem.
#[inline]
#[must_use]
pub const fn is_timeline_tag(tag: u64) -> bool {
    tag >= 0xAA00 && tag <= 0xAAFF
}

/// Evaluates a semantic numeric tag against timeline transport controls.
///
/// Returns the matching [`MediaTransportAction`] if the tag corresponds to a transport button.
#[must_use]
pub fn evaluate_timeline_transport_tag(
    tag: u64,
    speed_presets: &[f32],
) -> Option<MediaTransportAction> {
    match tag {
        TIMELINE_TAG_STEP_BACK => Some(MediaTransportAction::StepBack),
        TIMELINE_TAG_PLAY_PAUSE => Some(MediaTransportAction::TogglePlayPause),
        TIMELINE_TAG_STOP => Some(MediaTransportAction::Stop),
        TIMELINE_TAG_STEP_FWD => Some(MediaTransportAction::StepForward),
        TIMELINE_TAG_LOOP => Some(MediaTransportAction::ToggleLoop),
        t if (TIMELINE_TAG_SPEED_BASE..TIMELINE_TAG_SPEED_BASE + speed_presets.len() as u64)
            .contains(&t) =>
        {
            let idx = (t - TIMELINE_TAG_SPEED_BASE) as usize;
            speed_presets
                .get(idx)
                .copied()
                .map(MediaTransportAction::SetSpeed)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_timeline_tag() {
        assert!(is_timeline_tag(TIMELINE_TAG_PANEL_ROOT));
        assert!(is_timeline_tag(TIMELINE_TAG_PLAY_PAUSE));
        assert!(is_timeline_tag(TIMELINE_TAG_PLAYHEAD_CAP));
        assert!(!is_timeline_tag(0x1000));
        assert!(!is_timeline_tag(0xFFFF));
    }

    #[test]
    fn test_evaluate_timeline_transport_tag() {
        let presets = [0.25, 0.5, 1.0, 2.0];
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_STEP_BACK, &presets),
            Some(MediaTransportAction::StepBack)
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_PLAY_PAUSE, &presets),
            Some(MediaTransportAction::TogglePlayPause)
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_STOP, &presets),
            Some(MediaTransportAction::Stop)
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_STEP_FWD, &presets),
            Some(MediaTransportAction::StepForward)
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_LOOP, &presets),
            Some(MediaTransportAction::ToggleLoop)
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_SPEED_BASE, &presets),
            Some(MediaTransportAction::SetSpeed(0.25))
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_SPEED_BASE + 3, &presets),
            Some(MediaTransportAction::SetSpeed(2.0))
        );
        assert_eq!(
            evaluate_timeline_transport_tag(TIMELINE_TAG_SPEED_BASE + 99, &presets),
            None
        );
        assert_eq!(evaluate_timeline_transport_tag(0x1234, &presets), None);
    }
}