// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Transport Toolbar Bridge
//!
//! Connects engine animation playback state (Play/Pause, Stop, Step, Loop, Speed, Clip Name)
//! to pure declarative [`UiScope`] timeline primitives in Iris UI.
//!

use super::types::TimelinePanelParams;
use irisui::prelude::*;

/// Height of the transport controls bar in physical pixels.
pub const TRANSPORT_TOOLBAR_HEIGHT: f32 = 36.0;

/// Available speed preset multipliers.
pub const SPEED_PRESETS: [f32; 4] = [0.25, 0.5, 1.0, 2.0];

/// Builds the transport controls toolbar at the top of the animation timeline panel.
///
/// Emits pure declarative buttons, pills, dividers, and badges directly via [`UiScope`].
pub fn build_transport_toolbar(
    scope: &mut UiScope<'_>,
    params: &TimelinePanelParams<'_>,
    duration: f32,
) {
    let player = params.animation_player;
    let is_playing = player.is_some_and(|p| p.state == ae_animation::AnimationState::Playing);
    let is_looping = player.is_some_and(|p| p.looping);
    let current_speed = player.map_or(1.0, |p| p.speed);
    let current_time = player.map_or(0.0, |p| p.current_time);
    let clip_name = player
        .and_then(|p| p.current_clip.as_ref())
        .map(|c| c.name.as_str());

    let style = MediaTransportStyle::dark_default();

    let toolbar_style = Style::new()
        .height(TRANSPORT_TOOLBAR_HEIGHT)
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(6.0)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .background(style.bg)
        .border(style.border_width, style.border_color);

    scope.container_named("TimelineTransportToolbar", toolbar_style, |row| {
        // 1. Step Back Button
        let _ = row.timeline_transport_button(
            "TimelineStepBackBtn",
            "⏮",
            TIMELINE_TAG_STEP_BACK,
            false,
            false,
            &style,
        );

        // 2. Play / Pause Button
        let _ = row.timeline_transport_button(
            "TimelinePlayPauseBtn",
            if is_playing { "⏸" } else { "▶" },
            TIMELINE_TAG_PLAY_PAUSE,
            is_playing,
            true,
            &style,
        );

        // 3. Stop Button
        let _ = row.timeline_transport_button(
            "TimelineStopBtn",
            "⏹",
            TIMELINE_TAG_STOP,
            false,
            false,
            &style,
        );

        // 4. Step Forward Button
        let _ = row.timeline_transport_button(
            "TimelineStepFwdBtn",
            "⏭",
            TIMELINE_TAG_STEP_FWD,
            false,
            false,
            &style,
        );

        // 5. Divider
        row.vertical_divider(18.0, style.divider_color);

        // 6. Loop Toggle Pill
        let _ = row.timeline_loop_pill(is_looping, TIMELINE_TAG_LOOP, &style);

        // 7. Divider
        row.vertical_divider(18.0, style.divider_color);

        // 8. Speed Presets Pills
        for (idx, &preset) in SPEED_PRESETS.iter().enumerate() {
            let tag = TIMELINE_TAG_SPEED_BASE + idx as u64;
            let is_selected = (preset - current_speed).abs() < 0.05;
            let _ = row.timeline_speed_pill(preset, is_selected, tag, &style);
        }

        // 9. Spacer
        row.spacer();

        // 10. Clip Title Badge
        row.timeline_clip_badge(clip_name, &style);

        // 11. Time & Frame Readout Display
        row.timeline_time_readout(current_time, duration, 30.0, &style);
    });
}