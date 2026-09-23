// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Transport Toolbar Bridge
//!
//! Connects engine animation playback state (Play/Pause, Stop, Step, Loop, Speed, Clip Name)
//! to the generic [`MediaTransportBarBuilder`] widget in Iris UI.
//!

use super::types::{TimelinePanelParams, TimelinePanelTargets};
use irisui::prelude::*;

/// Height of the transport controls bar in physical pixels.
pub const TRANSPORT_TOOLBAR_HEIGHT: f32 = 36.0;

/// Available speed preset multipliers.
pub const SPEED_PRESETS: [f32; 4] = [0.25, 0.5, 1.0, 2.0];

/// Builds the transport controls toolbar at the top of the animation timeline panel.
///
/// Delegates all node hierarchy generation, button styling, badges, and readouts
/// directly to Iris UI's [`MediaTransportBarBuilder`].
pub fn build_transport_toolbar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    _targets: &mut TimelinePanelTargets,
    duration: f32,
) {
    let tb_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.y,
        params.panel_rect.width,
        TRANSPORT_TOOLBAR_HEIGHT,
    );

    let player = params.animation_player;
    let is_playing = player.is_some_and(|p| p.state == ae_animation::AnimationState::Playing);
    let is_looping = player.is_some_and(|p| p.looping);
    let current_speed = player.map_or(1.0, |p| p.speed);
    let current_time = player.map_or(0.0, |p| p.current_time);
    let clip_name = player
        .and_then(|p| p.current_clip.as_ref())
        .map(|c| c.name.as_str());

    let _frame = MediaTransportBarBuilder::new(tb_rect, duration, current_time)
        .is_playing(is_playing)
        .is_looping(is_looping)
        .current_speed(current_speed)
        .speed_presets(&SPEED_PRESETS)
        .clip_name(clip_name)
        .cursor_pos(Some(params.cursor_pos))
        .build(tree, parent_id);
}