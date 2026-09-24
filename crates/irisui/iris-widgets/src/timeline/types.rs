// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Timeline and Media Transport Core Types (`iris-widgets::timeline::types`)
//!
//! Provides parameter structures, keyframe markers, interaction actions, and evaluation
//! results for timeline rulers and media transport bars in Iris UI.
//!

use iris_core::color::Color;

/// Default height in physical pixels allocated for the top ruler ticks section.
pub const DEFAULT_RULER_HEIGHT: f32 = 18.0;

/// Default height in physical pixels allocated for the interactive scrubber track.
pub const DEFAULT_SCRUBBER_HEIGHT: f32 = 36.0;

/// A keyframe indicator marker positioned along a timeline scrubber track.
///
/// Contains the timestamp of the keyframe and an optional custom tint color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineKeyframeMarker {
    /// Timestamp of the keyframe in seconds.
    pub time: f32,
    /// Optional custom indicator color override. If `None`, the default style color is used.
    pub color: Option<Color>,
}

impl TimelineKeyframeMarker {
    /// Creates a new keyframe marker at the specified timestamp with default styling.
    #[must_use]
    pub const fn new(time: f32) -> Self {
        Self { time, color: None }
    }

    /// Creates a new keyframe marker with an explicit accent color.
    #[must_use]
    pub const fn with_color(time: f32, color: Color) -> Self {
        Self {
            time,
            color: Some(color),
        }
    }
}

/// User interaction actions dispatched by a media playback transport bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaTransportAction {
    /// Steps playback backward by one frame or discrete step.
    StepBack,
    /// Toggles playback between active playing and paused states.
    TogglePlayPause,
    /// Stops playback and resets the current playhead timestamp to zero.
    Stop,
    /// Steps playback forward by one frame or discrete step.
    StepForward,
    /// Toggles looping mode on or off.
    ToggleLoop,
    /// Selects a specific playback speed multiplier.
    SetSpeed(f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_marker_construction() {
        let default_kf = TimelineKeyframeMarker::new(2.5);
        assert_eq!(default_kf.time, 2.5);
        assert_eq!(default_kf.color, None);

        let custom_color = Color::rgba(1.0, 0.5, 0.2, 1.0);
        let colored_kf = TimelineKeyframeMarker::with_color(4.0, custom_color);
        assert_eq!(colored_kf.time, 4.0);
        assert_eq!(colored_kf.color, Some(custom_color));
    }

    #[test]
    fn test_transport_action_equality() {
        assert_eq!(
            MediaTransportAction::StepBack,
            MediaTransportAction::StepBack
        );
        assert_eq!(
            MediaTransportAction::SetSpeed(1.5),
            MediaTransportAction::SetSpeed(1.5)
        );
        assert_ne!(
            MediaTransportAction::SetSpeed(1.0),
            MediaTransportAction::SetSpeed(2.0)
        );
    }
}