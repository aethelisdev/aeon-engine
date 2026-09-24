// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Type Definitions
//!
//! Exposes parameter bundles, panel runtime state, and user interaction
//! action variants for the Iris UI Animation Timeline Studio panel.
//!

use irisui::prelude::{InteractionEvent, Point, Rect};

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
    /// Currently hovered widget persistent tag, if any.
    pub hovered_tag: Option<u64>,
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