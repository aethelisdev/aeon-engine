// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Type Definitions
//!
//! Exposes parameter bundles, panel runtime state, and user interaction
//! action variants for the Iris UI Animation Timeline Studio panel.
//!

use irisui::prelude::{Color, InteractionEvent, Point, Rect};

/// Default height in physical pixels allocated for the top ruler ticks section.
pub const DEFAULT_RULER_HEIGHT: f32 = 18.0;

/// Default height in physical pixels allocated for the interactive scrubber track.
pub const DEFAULT_SCRUBBER_HEIGHT: f32 = 36.0;

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

/// Width of the left channel property badge column in physical pixels.
///
/// Leaves dedicated horizontal clearance for property pills (`POS`, `ROT`, `SCL`)
/// and joint labels, ensuring keyframe diamond markers never overlap badge text.
pub const TIMELINE_SIDEBAR_WIDTH: f32 = 118.0;

/// Returns true if the specified numeric tag belongs to the timeline subsystem.
#[inline]
#[must_use]
pub const fn is_timeline_tag(tag: u64) -> bool {
    tag >= 0xAA00 && tag <= 0xAAFF
}

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

/// Runtime parameters passed to the Animation Timeline Studio builder each frame.
pub struct TimelinePanelParams<'a> {
    /// Available docked panel bounding rectangle.
    pub panel_rect: Rect,
    /// Currently selected entity in the scene, if any.
    pub entity: Option<hecs::Entity>,
    /// Active animation player component borrowed from the ECS world, if present.
    pub animation_player: Option<&'a ae_animation::AnimationPlayer>,
    /// Current mouse cursor position for hover state calculation.
    pub cursor_pos: Point,
    /// Whether the user is actively dragging the scrubber playhead needle.
    pub is_dragging_scrubber: bool,
    /// Tagged interaction events emitted during this frame for declarative widgets.
    pub events: &'a [(u64, InteractionEvent)],
}

/// User interaction actions dispatched by the Animation Timeline Studio panel.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineAction {
    /// Toggles playback between Playing and Paused states.
    TogglePlayPause,
    /// Stops playback and resets the current timestamp to zero.
    Stop,
    /// Steps playback by a signed frame increment (`+1` or `-1`).
    StepFrame(i32),
    /// Toggles loop playback flag on the active player.
    ToggleLoop,
    /// Updates playback speed multiplier preset (`0.25x`, `0.5x`, `1.0x`, `2.0x`).
    SetSpeed(f32),
    /// Scrubs the current playhead position to an absolute timestamp in seconds.
    ScrubTo(f32),
    /// Dispatches an action requesting an `AnimationPlayer` component to be attached to the target entity.
    AddAnimationPlayer(hecs::Entity),
}

/// Runtime persistent state for the Animation Timeline Studio panel.
///
/// Maintains active panel bounds, hardware hit-test scrubber dragging track geometry,
/// and queued user interactions without retaining external node identifier tables.
#[derive(Debug, Default, Clone)]
pub struct TimelinePanelState {
    /// Common panel interaction state (actions queue).
    pub interactions: crate::ui::iris_bridge::types::PanelInteractionState<(), TimelineAction>,
    /// Whether user is actively dragging the timeline scrubber playhead needle.
    pub is_dragging: bool,
    /// Previously cached scrubber dragging state used for retained dirty-checking.
    pub last_is_dragging: bool,
    /// Cached hardware hit-test track bounds during active scrubber drag: `(track_x, track_width)`.
    pub active_scrubber_track: Option<(f32, f32)>,
    /// Selected entity handle cached for timeline interactions.
    pub selected_entity: Option<hecs::Entity>,
    /// Previously baked selected entity handle used for retained dirty-checking.
    pub last_selected_entity: Option<hecs::Entity>,
    /// Pending tagged interaction events collected during window event routing.
    pub pending_interaction_events: Vec<(u64, InteractionEvent)>,
    /// Active docked bounding rectangle of the Animation Timeline panel.
    pub panel_rect: Option<Rect>,
    /// Duration of the currently active animation clip in seconds.
    pub clip_duration: f32,
}

impl std::ops::Deref for TimelinePanelState {
    type Target = crate::ui::iris_bridge::types::PanelInteractionState<(), TimelineAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for TimelinePanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}

/// Semantic alias for [`TimelinePanelState`].
pub type TimelineState = TimelinePanelState;

impl TimelinePanelState {
    /// Evaluates whether the Animation Timeline panel requires an in-place repaint.
    ///
    /// Inspects whether the active ECS scene entity selection changed or the playhead
    /// scrubber needle is actively being dragged by the mouse cursor.
    pub fn is_dirty(
        &self,
        params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>,
    ) -> bool {
        self.last_selected_entity != params.scene.selected_entity
            || self.last_is_dragging != self.is_dragging
    }

    /// Synchronizes internal cached snapshot values against active frame parameters.
    pub fn sync_dirty(&mut self, params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>) {
        self.last_selected_entity = params.scene.selected_entity;
        self.selected_entity = params.scene.selected_entity;
        self.last_is_dragging = self.is_dragging;
    }

    /// Evaluates `is_dirty` and automatically updates snapshot caches if dirty.
    ///
    /// Returns `true` if the panel state changed and requires redraw tagging.
    pub fn check_and_sync_dirty(
        &mut self,
        params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>,
    ) -> bool {
        let dirty = self.is_dirty(params);
        if dirty {
            self.sync_dirty(params);
        }
        dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_tags_and_filter() {
        assert!(is_timeline_tag(TIMELINE_TAG_PANEL_ROOT));
        assert!(is_timeline_tag(TIMELINE_TAG_PLAY_PAUSE));
        assert!(is_timeline_tag(TIMELINE_TAG_SCRUBBER_TRACK));
        assert!(is_timeline_tag(TIMELINE_TAG_PLAYHEAD_CAP));
        assert!(!is_timeline_tag(0x1000));
        assert!(!is_timeline_tag(0xFFFF));
    }

    #[test]
    fn test_evaluate_transport_tags() {
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
            evaluate_timeline_transport_tag(TIMELINE_TAG_SPEED_BASE + 10, &presets),
            None
        );
    }

    #[test]
    fn test_keyframe_marker_construction() {
        let default_kf = TimelineKeyframeMarker::new(1.5);
        assert_eq!(default_kf.time, 1.5);
        assert_eq!(default_kf.color, None);

        let custom_color = Color::rgba(1.0, 0.2, 0.4, 1.0);
        let custom_kf = TimelineKeyframeMarker::with_color(3.0, custom_color);
        assert_eq!(custom_kf.time, 3.0);
        assert_eq!(custom_kf.color, Some(custom_color));
    }
}