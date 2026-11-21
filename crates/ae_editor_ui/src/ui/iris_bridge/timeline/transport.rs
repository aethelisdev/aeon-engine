// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Transport Toolbar
//!
//! Connects engine animation playback controls (Play/Pause, Stop, Step, Loop, Speed presets,
//! Clip badge, and Time readout) directly to `ActionBus<EngineUiAction>` with zero intermediate
//! `let` statements and zero heap `format!` allocations.

use super::types::{
    TIMELINE_TAG_CLIP_BADGE, TIMELINE_TAG_LOOP, TIMELINE_TAG_PLAY_PAUSE, TIMELINE_TAG_STEP_BACK,
    TIMELINE_TAG_STEP_FWD, TIMELINE_TAG_STOP, TIMELINE_TAG_TIME_READOUT,
    TIMELINE_TAG_TIME_READOUT_TEXT, emit_player_mutation,
};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;
use std::fmt::Write;

/// Height of the transport controls bar in logical pixels.
pub const TRANSPORT_TOOLBAR_HEIGHT: f32 = 36.0;

/// Playback speed multiplier presets paired with display labels and deterministic tags.
pub const SPEED_PRESETS: &[(f32, &str, u64)] = &[
    (0.25, "0.25x", UiTag::from_str("timeline.speed.025").raw()),
    (0.5, "0.5x", UiTag::from_str("timeline.speed.050").raw()),
    (1.0, "1x", UiTag::from_str("timeline.speed.100").raw()),
    (2.0, "2x", UiTag::from_str("timeline.speed.200").raw()),
];

const TRANSPORT_BG: Color = Color::rgba(0.08, 0.09, 0.12, 0.98);
const TRANSPORT_BORDER: Color = Color::rgba(0.18, 0.21, 0.28, 0.70);
const BTN_IDLE_BG: Color = Color::rgba(0.12, 0.14, 0.18, 0.95);
const BTN_BORDER: Color = Color::rgba(0.25, 0.28, 0.38, 0.60);
const PLAY_ACTIVE_BG: Color = Color::rgba(0.08, 0.25, 0.15, 0.95);
const PLAY_ACTIVE_BORDER: Color = Color::rgba(0.20, 0.85, 0.40, 0.80);
const LOOP_ACTIVE_BG: Color = Color::rgba(0.0, 0.40, 0.55, 0.35);
const LOOP_ACTIVE_BORDER: Color = Color::rgba(0.0, 0.92, 1.0, 1.0);
const CLIP_BADGE_BG: Color = Color::rgba(0.22, 0.12, 0.18, 0.70);
const CLIP_BADGE_BORDER: Color = Color::rgba(0.85, 0.40, 0.65, 0.40);
const CLIP_BADGE_TEXT: Color = Color::rgba(0.95, 0.55, 0.75, 1.0);
const READOUT_BG: Color = Color::rgba(0.06, 0.08, 0.11, 0.85);
const READOUT_BORDER: Color = Color::rgba(0.18, 0.22, 0.28, 0.60);
const READOUT_TEXT: Color = Color::rgba(0.0, 0.88, 1.0, 1.0);

/// Builds the playback transport controls toolbar at the top of the animation timeline panel.
pub fn build_transport_toolbar(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    player: &ae_animation::AnimationPlayer,
    duration: f32,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.container_named(
        "TimelineTransportToolbar",
        Style::new()
            .height(TRANSPORT_TOOLBAR_HEIGHT)
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(6.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .background(TRANSPORT_BG)
            .border(1.0, TRANSPORT_BORDER)
            .clip_children(true),
        |row| {
            render_transport_button(
                row,
                "TimelineStepBackBtn",
                "⏮",
                TIMELINE_TAG_STEP_BACK,
                false,
                BTN_IDLE_BG,
                BTN_BORDER,
            )
            .on_click(|| {
                emit_player_mutation(entity, player, action_bus, |p| {
                    p.current_time = (p.current_time - (1.0 / 30.0)).clamp(0.0, duration);
                });
            });

            render_transport_button(
                row,
                "TimelinePlayPauseBtn",
                if player.state == ae_animation::AnimationState::Playing {
                    "⏸"
                } else {
                    "▶"
                },
                TIMELINE_TAG_PLAY_PAUSE,
                player.state == ae_animation::AnimationState::Playing,
                PLAY_ACTIVE_BG,
                PLAY_ACTIVE_BORDER,
            )
            .on_click(|| {
                emit_player_mutation(entity, player, action_bus, |p| {
                    p.state = if p.state == ae_animation::AnimationState::Playing {
                        ae_animation::AnimationState::Paused
                    } else {
                        ae_animation::AnimationState::Playing
                    };
                });
            });

            render_transport_button(
                row,
                "TimelineStopBtn",
                "⏹",
                TIMELINE_TAG_STOP,
                false,
                BTN_IDLE_BG,
                BTN_BORDER,
            )
            .on_click(|| {
                emit_player_mutation(entity, player, action_bus, |p| {
                    p.state = ae_animation::AnimationState::Stopped;
                    p.current_time = 0.0;
                });
            });

            render_transport_button(
                row,
                "TimelineStepFwdBtn",
                "⏭",
                TIMELINE_TAG_STEP_FWD,
                false,
                BTN_IDLE_BG,
                BTN_BORDER,
            )
            .on_click(|| {
                emit_player_mutation(entity, player, action_bus, |p| {
                    p.current_time = (p.current_time + (1.0 / 30.0)).clamp(0.0, duration);
                });
            });

            row.vertical_divider(18.0, Color::rgba(0.25, 0.28, 0.38, 0.70));

            render_loop_pill(row, player.looping, TIMELINE_TAG_LOOP).on_click(|| {
                emit_player_mutation(entity, player, action_bus, |p| {
                    p.looping = !p.looping;
                });
            });

            row.vertical_divider(18.0, Color::rgba(0.25, 0.28, 0.38, 0.70));

            SPEED_PRESETS.iter().for_each(|&(preset, label, tag)| {
                row.add(
                    Button::new(label)
                        .small()
                        .active((preset - player.speed).abs() < 0.05)
                        .tag(tag),
                )
                .on_click(|| {
                    emit_player_mutation(entity, player, action_bus, |p| {
                        p.speed = preset;
                    });
                });
            });

            row.spacer();

            render_clip_badge(row, player.current_clip.as_ref().map(|c| c.name.as_str()));
            render_time_readout(row, player.current_time, duration, 30.0);
        },
    );
}

/// Renders a fixed-size transport icon button (`26x24px`) with explicit active colors and tag.
pub fn render_transport_button(
    scope: &mut UiScope<'_>,
    name: &'static str,
    glyph: &str,
    tag: u64,
    is_active: bool,
    active_bg: Color,
    active_border: Color,
) -> WidgetResponse {
    scope
        .container_tagged(
            name,
            Style::new()
                .width(26.0)
                .height(24.0)
                .background(if is_active { active_bg } else { BTN_IDLE_BG })
                .border(1.0, if is_active { active_border } else { BTN_BORDER })
                .border_radius(4.0)
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .flex_shrink(0.0),
            WidgetRole::Button,
            tag,
            |btn| {
                btn.label_custom(
                    glyph,
                    12.0,
                    if is_active {
                        Color::rgba(0.95, 0.98, 1.0, 1.0)
                    } else {
                        Color::rgba(0.80, 0.84, 0.90, 1.0)
                    },
                    TextAlign::Center,
                );
            },
        )
        .response
}

/// Renders the interactive playback looping toggle pill button (`48x24px`).
pub fn render_loop_pill(scope: &mut UiScope<'_>, is_looping: bool, tag: u64) -> WidgetResponse {
    scope
        .container_tagged(
            "TimelineLoopBtn",
            Style::new()
                .width(48.0)
                .height(24.0)
                .background(if is_looping {
                    LOOP_ACTIVE_BG
                } else {
                    BTN_IDLE_BG
                })
                .border(
                    1.0,
                    if is_looping {
                        LOOP_ACTIVE_BORDER
                    } else {
                        BTN_BORDER
                    },
                )
                .border_radius(4.0)
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .flex_shrink(0.0),
            WidgetRole::Button,
            tag,
            |btn| {
                btn.label_custom(
                    "Loop",
                    10.5,
                    if is_looping {
                        Color::rgba(0.0, 0.92, 1.0, 1.0)
                    } else {
                        Color::rgba(0.75, 0.80, 0.88, 1.0)
                    },
                    TextAlign::Center,
                );
            },
        )
        .response
}

/// Renders the active animation clip name badge using zero-heap [`StackString`] formatting.
pub fn render_clip_badge(scope: &mut UiScope<'_>, clip_name: Option<&str>) {
    scope.badge_fmt::<64, _>(
        BadgePillSpec::new(
            "TimelineClipBadge",
            TIMELINE_TAG_CLIP_BADGE,
            CLIP_BADGE_BG,
            CLIP_BADGE_BORDER,
            CLIP_BADGE_TEXT,
        )
        .typography(10.0, 6.2),
        |buf| {
            match clip_name {
                Some(name) if !name.is_empty() => write!(buf, "Clip: {name}").ok(),
                _ => write!(buf, "No Clip").ok(),
            };
        },
    );
}

/// Renders the current playback timestamp, duration, and frame counter pill as a live volatile text badge.
pub fn render_time_readout(scope: &mut UiScope<'_>, current_time: f32, duration: f32, fps: f32) {
    scope.badge_live_fmt::<48, _>(
        BadgePillSpec::new(
            "TimelineTimeReadout",
            TIMELINE_TAG_TIME_READOUT,
            READOUT_BG,
            READOUT_BORDER,
            READOUT_TEXT,
        )
        .min_width(146.0)
        .typography(10.5, 6.5),
        TIMELINE_TAG_TIME_READOUT_TEXT,
        |buf| {
            write!(
                buf,
                "{current_time:.2}s / {duration:.2}s  (F {})",
                (current_time * fps.max(1.0)).round() as i32
            )
            .ok();
        },
    );
}

/// Updates the transport time readout pill text in place at 60 FPS during `InPlaceRefresh`.
pub fn update_time_readout_live(
    scope: &mut UiScope<'_>,
    current_time: f32,
    duration: f32,
    fps: f32,
) -> bool {
    scope.update_live_text_fmt::<48, _>(TIMELINE_TAG_TIME_READOUT_TEXT, |buf| {
        write!(
            buf,
            "{current_time:.2}s / {duration:.2}s  (F {})",
            (current_time * fps.max(1.0)).round() as i32
        )
        .ok();
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::widgets::panel::PanelBuilder;

    #[test]
    fn test_transport_toolbar_emits_badges_and_play_action() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);
        let entity = hecs::World::new().spawn(());
        let player = ae_animation::AnimationPlayer {
            current_clip: Some(ae_animation::AnimationClip::new("RunCycle", 2.0)),
            ..Default::default()
        };
        let mut bus = ActionBus::<EngineUiAction>::new();

        build_transport_toolbar(
            &mut UiScope::new(&mut tree, root),
            entity,
            &player,
            2.0,
            &mut bus,
        );

        assert!(tree.find_node_by_tag(TIMELINE_TAG_PLAY_PAUSE).is_some());
        assert!(tree.find_node_by_tag(TIMELINE_TAG_CLIP_BADGE).is_some());
        assert!(tree.find_node_by_tag(TIMELINE_TAG_TIME_READOUT).is_some());

        let mut tree_click = UiTree::new();
        tree_click.event_state.frame_clicked_tag = Some(TIMELINE_TAG_PLAY_PAUSE);
        let root_click = PanelBuilder::new(&mut tree_click).build();
        let _ = tree_click.set_root(root_click);
        build_transport_toolbar(
            &mut UiScope::new(&mut tree_click, root_click),
            entity,
            &player,
            2.0,
            &mut bus,
        );
        assert_eq!(bus.len(), 1);
    }
}