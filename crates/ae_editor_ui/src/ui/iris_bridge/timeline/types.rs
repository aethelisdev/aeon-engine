// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Type Definitions
//!
//! Exposes parameter bundles, hit-testing target descriptors, and user interaction
//! action variants for the Iris UI Animation Timeline Studio panel.
//!

use irisui::prelude::{Point, Rect};

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
}

/// Hit-testing targets and interactive bounding boxes for timeline controls.
#[derive(Debug, Clone, Default)]
pub struct TimelinePanelTargets {
    /// Total panel bounding rectangle for clipping and overlay hit-testing.
    pub panel_rect: Rect,
    /// Play/Pause toggle button bounding box.
    pub play_pause_btn: Option<Rect>,
    /// Stop button bounding box.
    pub stop_btn: Option<Rect>,
    /// Step back one frame button bounding box.
    pub step_back_btn: Option<Rect>,
    /// Step forward one frame button bounding box.
    pub step_fwd_btn: Option<Rect>,
    /// Loop playback toggle pill bounding box.
    pub loop_toggle: Option<Rect>,
    /// Playback speed selector button bounding boxes: `(speed_value, rect)`.
    pub speed_buttons: Vec<(f32, Rect)>,
    /// Interactive scrubber track bounding box.
    pub scrubber_track_rect: Option<Rect>,
    /// Current playhead needle draggable position and handle.
    pub playhead_needle_rect: Option<Rect>,
    /// "Add AnimationPlayer" button bounding box for entities missing the component.
    pub add_player_btn: Option<Rect>,
    /// Duration of the currently active animation clip in seconds.
    pub clip_duration: f32,
}

/// User interaction actions dispatched by the Animation Timeline Studio panel.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineAction {
    /// Toggles playback between Playing and Paused states.
    TogglePlayPause,
    /// Stops playback and resets the current timestamp to zero.
    Stop,
    /// Steps playback forward or backward by the specified frame count.
    StepFrame(i32),
    /// Toggles the looping flag of the active animation player.
    ToggleLoop,
    /// Sets the playback speed multiplier.
    SetSpeed(f32),
    /// Scrubs the animation player timestamp to the target time in seconds.
    ScrubTo(f32),
    /// Adds an AnimationPlayer component to the currently selected entity.
    AddAnimationPlayer(hecs::Entity),
}