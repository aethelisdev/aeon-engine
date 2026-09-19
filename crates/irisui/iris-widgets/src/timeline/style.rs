// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Timeline and Media Transport Styling (`iris-widgets::timeline::style`)
//!
//! Provides color schemes, typography sizes, and border geometries for timeline rulers
//! and media transport bars.
//!

use iris_core::color::Color;

/// Visual styling configuration for a timeline ruler and interactive scrubber track.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineRulerStyle {
    /// Background color of the interactive scrubber track.
    pub track_bg: Color,
    /// Border color of the scrubber track when idle.
    pub track_border_idle: Color,
    /// Border color of the scrubber track when hovered or dragged.
    pub track_border_active: Color,
    /// Border width of the scrubber track in physical pixels.
    pub track_border_width: f32,
    /// Corner border radius of the scrubber track.
    pub track_border_radius: f32,
    /// Fill color representing elapsed playback duration.
    pub progress_fill_color: Color,
    /// Color of major division tick marks on the ruler.
    pub major_tick_color: Color,
    /// Color of minor subdivision tick marks on the ruler.
    pub minor_tick_color: Color,
    /// Text color for timestamp labels alongside major tick marks.
    pub tick_label_color: Color,
    /// Text font size for timestamp labels in physical points.
    pub tick_label_font_size: f32,
    /// Color of keyframe diamond markers along the track.
    pub keyframe_color: Color,
    /// Accent color of the vertical playhead needle line.
    pub playhead_needle_color: Color,
    /// Text and glyph color of the draggable playhead handle cap (`▼`).
    pub playhead_cap_color: Color,
    /// Width of the vertical playhead needle line in physical pixels.
    pub playhead_needle_width: f32,
}

impl Default for TimelineRulerStyle {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl TimelineRulerStyle {
    /// Standard dark slate studio theme for the timeline ruler and scrubber.
    #[must_use]
    pub const fn dark_default() -> Self {
        Self {
            track_bg: Color::rgba(0.09, 0.10, 0.14, 0.95),
            track_border_idle: Color::rgba(0.20, 0.23, 0.32, 0.70),
            track_border_active: Color::rgba(0.0, 0.85, 1.0, 0.60),
            track_border_width: 1.0,
            track_border_radius: 4.0,
            progress_fill_color: Color::rgba(0.0, 0.75, 0.95, 0.16),
            major_tick_color: Color::rgba(0.50, 0.55, 0.68, 0.90),
            minor_tick_color: Color::rgba(0.30, 0.34, 0.44, 0.60),
            tick_label_color: Color::rgba(0.55, 0.60, 0.72, 1.0),
            tick_label_font_size: 9.5,
            keyframe_color: Color::rgba(0.96, 0.72, 0.18, 1.0),
            playhead_needle_color: Color::rgba(0.0, 0.92, 1.0, 1.0),
            playhead_cap_color: Color::rgba(0.0, 0.95, 1.0, 1.0),
            playhead_needle_width: 2.0,
        }
    }
}

/// Visual styling configuration for a media transport playback toolbar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaTransportStyle {
    /// Background color of the toolbar strip container.
    pub bg: Color,
    /// Border outline color of the toolbar container.
    pub border_color: Color,
    /// Border outline thickness in physical pixels.
    pub border_width: f32,
    /// Background color of standard buttons in idle state.
    pub btn_bg_idle: Color,
    /// Background color of standard buttons when hovered.
    pub btn_bg_hover: Color,
    /// Border outline color of standard buttons.
    pub btn_border: Color,
    /// Text/glyph color of standard buttons when idle.
    pub btn_text_idle: Color,
    /// Text/glyph color of standard buttons when hovered.
    pub btn_text_hover: Color,
    /// Background color of the Play button when playback is active.
    pub play_bg_active: Color,
    /// Border color of the Play button when playback is active.
    pub play_border_active: Color,
    /// Glyph color of the Play button when playback is active.
    pub play_text_active: Color,
    /// Glyph color of the Stop button when hovered.
    pub stop_text_hover: Color,
    /// Divider line color separating control groups.
    pub divider_color: Color,
    /// Text and outline color of the Loop toggle button when active.
    pub loop_active_color: Color,
    /// Background tint of the Loop toggle button when active.
    pub loop_active_bg: Color,
    /// Text and outline color of the currently selected speed preset pill.
    pub speed_active_color: Color,
    /// Background tint of the currently selected speed preset pill.
    pub speed_active_bg: Color,
    /// Background color of the media/clip name badge.
    pub clip_badge_bg: Color,
    /// Border color of the media/clip name badge.
    pub clip_badge_border: Color,
    /// Text color of the media/clip name badge.
    pub clip_badge_text: Color,
    /// Text color of the elapsed time and frame readout display.
    pub time_readout_color: Color,
}

impl Default for MediaTransportStyle {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl MediaTransportStyle {
    /// Standard dark slate studio theme for the media transport bar.
    #[must_use]
    pub const fn dark_default() -> Self {
        Self {
            bg: Color::rgba(0.08, 0.09, 0.12, 0.98),
            border_color: Color::rgba(0.18, 0.21, 0.28, 0.70),
            border_width: 1.0,
            btn_bg_idle: Color::rgba(0.12, 0.14, 0.18, 0.95),
            btn_bg_hover: Color::rgba(0.20, 0.24, 0.32, 1.0),
            btn_border: Color::rgba(0.25, 0.28, 0.38, 0.60),
            btn_text_idle: Color::rgba(0.75, 0.78, 0.85, 1.0),
            btn_text_hover: Color::WHITE,
            play_bg_active: Color::rgba(0.08, 0.25, 0.15, 0.95),
            play_border_active: Color::rgba(0.20, 0.85, 0.40, 0.80),
            play_text_active: Color::rgba(0.20, 0.95, 0.45, 1.0),
            stop_text_hover: Color::rgba(1.0, 0.40, 0.40, 1.0),
            divider_color: Color::rgba(0.25, 0.28, 0.38, 0.70),
            loop_active_color: Color::rgba(0.0, 0.92, 1.0, 1.0),
            loop_active_bg: Color::rgba(0.0, 0.40, 0.55, 0.35),
            speed_active_color: Color::rgba(0.0, 0.92, 1.0, 1.0),
            speed_active_bg: Color::rgba(0.0, 0.35, 0.50, 0.40),
            clip_badge_bg: Color::rgba(0.22, 0.12, 0.18, 0.70),
            clip_badge_border: Color::rgba(0.85, 0.40, 0.65, 0.40),
            clip_badge_text: Color::rgba(0.95, 0.55, 0.75, 1.0),
            time_readout_color: Color::rgba(0.0, 0.88, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruler_style_default() {
        let style = TimelineRulerStyle::default();
        assert_eq!(style, TimelineRulerStyle::dark_default());
        assert_eq!(style.track_border_width, 1.0);
        assert_eq!(style.playhead_needle_width, 2.0);
    }

    #[test]
    fn test_media_transport_style_default() {
        let style = MediaTransportStyle::default();
        assert_eq!(style, MediaTransportStyle::dark_default());
        assert_eq!(style.border_width, 1.0);
    }
}