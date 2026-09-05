// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Transport Toolbar Builder
//!
//! Renders playback transport controls (Step Back, Play/Pause, Stop, Step Forward),
//! loop toggles, speed selection buttons, clip title badges, and high-precision
//! time and frame indicators.
//!

use super::types::{TimelinePanelParams, TimelinePanelTargets};
use irisui::prelude::*;

/// Height of the transport controls bar in physical pixels.
pub const TRANSPORT_TOOLBAR_HEIGHT: f32 = 36.0;

/// Available speed preset multipliers.
pub const SPEED_PRESETS: [f32; 4] = [0.25, 0.5, 1.0, 2.0];

/// Builds the transport controls toolbar at the top of the animation timeline panel.
pub fn build_transport_toolbar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    targets: &mut TimelinePanelTargets,
    duration: f32,
) {
    let tb_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.y,
        params.panel_rect.width,
        TRANSPORT_TOOLBAR_HEIGHT,
    );

    let tb_id = tree.create_node();
    if let Some(node) = tree.get_mut(tb_id) {
        node.set_name("TimelineTransportToolbar");
        node.computed_rect = tb_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .border(1.0, Color::rgba(0.18, 0.21, 0.28, 0.70));
    }
    let _ = tree.add_child(parent_id, tb_id);

    let player = params.animation_player;
    let is_playing = player.is_some_and(|p| p.state == ae_animation::AnimationState::Playing);
    let is_looping = player.is_some_and(|p| p.looping);
    let current_speed = player.map_or(1.0, |p| p.speed);
    let current_time = player.map_or(0.0, |p| p.current_time);

    let mut cur_x = tb_rect.x + 8.0;
    let btn_y = tb_rect.y + 5.0;
    let btn_h = 26.0;

    // ── 1. Step Back Button ──
    let step_back_w = 28.0;
    let step_back_rect = Rect::new(cur_x, btn_y, step_back_w, btn_h);
    let is_step_back_hovered = step_back_rect.contains_point(params.cursor_pos);
    targets.step_back_btn = Some(step_back_rect);

    let step_back_id = tree.create_node();
    if let Some(node) = tree.get_mut(step_back_id) {
        node.set_name("TimelineStepBackBtn");
        node.set_text("⏮");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_step_back_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.75, 0.78, 0.85, 1.0)
        };
        node.computed_rect = step_back_rect;
        node.style = Style::new()
            .background(if is_step_back_hovered {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.95)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.25, 0.28, 0.38, 0.60));
    }
    let _ = tree.add_child(tb_id, step_back_id);
    cur_x += step_back_w + 4.0;

    // ── 2. Play / Pause Button ──
    let play_pause_w = 34.0;
    let play_pause_rect = Rect::new(cur_x, btn_y, play_pause_w, btn_h);
    let is_play_hovered = play_pause_rect.contains_point(params.cursor_pos);
    targets.play_pause_btn = Some(play_pause_rect);

    let play_pause_id = tree.create_node();
    if let Some(node) = tree.get_mut(play_pause_id) {
        node.set_name("TimelinePlayPauseBtn");
        node.set_text(if is_playing { "⏸" } else { "▶" });
        node.font_size = 12.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_playing {
            Color::rgba(0.20, 0.95, 0.45, 1.0)
        } else if is_play_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.0, 0.90, 1.0, 1.0)
        };
        node.computed_rect = play_pause_rect;
        node.style = Style::new()
            .background(if is_playing {
                Color::rgba(0.08, 0.25, 0.15, 0.95)
            } else if is_play_hovered {
                Color::rgba(0.18, 0.25, 0.35, 1.0)
            } else {
                Color::rgba(0.12, 0.16, 0.22, 0.95)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_playing {
                    Color::rgba(0.20, 0.85, 0.40, 0.80)
                } else if is_play_hovered {
                    Color::rgba(0.0, 0.85, 1.0, 0.80)
                } else {
                    Color::rgba(0.0, 0.70, 0.85, 0.50)
                },
            );
    }
    let _ = tree.add_child(tb_id, play_pause_id);
    cur_x += play_pause_w + 4.0;

    // ── 3. Stop Button ──
    let stop_w = 28.0;
    let stop_rect = Rect::new(cur_x, btn_y, stop_w, btn_h);
    let is_stop_hovered = stop_rect.contains_point(params.cursor_pos);
    targets.stop_btn = Some(stop_rect);

    let stop_id = tree.create_node();
    if let Some(node) = tree.get_mut(stop_id) {
        node.set_name("TimelineStopBtn");
        node.set_text("⏹");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_stop_hovered {
            Color::rgba(1.0, 0.40, 0.40, 1.0)
        } else {
            Color::rgba(0.75, 0.78, 0.85, 1.0)
        };
        node.computed_rect = stop_rect;
        node.style = Style::new()
            .background(if is_stop_hovered {
                Color::rgba(0.25, 0.15, 0.18, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.95)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.25, 0.28, 0.38, 0.60));
    }
    let _ = tree.add_child(tb_id, stop_id);
    cur_x += stop_w + 4.0;

    // ── 4. Step Forward Button ──
    let step_fwd_w = 28.0;
    let step_fwd_rect = Rect::new(cur_x, btn_y, step_fwd_w, btn_h);
    let is_step_fwd_hovered = step_fwd_rect.contains_point(params.cursor_pos);
    targets.step_fwd_btn = Some(step_fwd_rect);

    let step_fwd_id = tree.create_node();
    if let Some(node) = tree.get_mut(step_fwd_id) {
        node.set_name("TimelineStepFwdBtn");
        node.set_text("⏭");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_step_fwd_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.75, 0.78, 0.85, 1.0)
        };
        node.computed_rect = step_fwd_rect;
        node.style = Style::new()
            .background(if is_step_fwd_hovered {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.95)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.25, 0.28, 0.38, 0.60));
    }
    let _ = tree.add_child(tb_id, step_fwd_id);
    cur_x += step_fwd_w + 8.0;

    // ── Divider ──
    let sep_rect = Rect::new(cur_x, btn_y + 3.0, 1.0, btn_h - 6.0);
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("TimelineToolbarDivider");
        node.computed_rect = sep_rect;
        node.style = Style::new().background(Color::rgba(0.25, 0.28, 0.38, 0.70));
    }
    let _ = tree.add_child(tb_id, sep_id);
    cur_x += 9.0;

    // ── 5. Loop Toggle Pill ──
    let loop_w = 64.0;
    let loop_rect = Rect::new(cur_x, btn_y, loop_w, btn_h);
    let is_loop_hovered = loop_rect.contains_point(params.cursor_pos);
    targets.loop_toggle = Some(loop_rect);

    let loop_id = tree.create_node();
    if let Some(node) = tree.get_mut(loop_id) {
        node.set_name("TimelineLoopToggle");
        node.set_text("🔁 Loop");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_looping {
            Color::rgba(0.0, 0.92, 1.0, 1.0)
        } else {
            Color::rgba(0.60, 0.64, 0.72, 1.0)
        };
        node.computed_rect = loop_rect;
        node.style = Style::new()
            .background(if is_looping {
                Color::rgba(0.0, 0.40, 0.55, 0.35)
            } else if is_loop_hovered {
                Color::rgba(0.18, 0.22, 0.28, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.95)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_looping {
                    Color::rgba(0.0, 0.85, 1.0, 0.70)
                } else {
                    Color::rgba(0.25, 0.28, 0.38, 0.60)
                },
            );
    }
    let _ = tree.add_child(tb_id, loop_id);
    cur_x += loop_w + 8.0;

    // ── 6. Speed Buttons (0.25x, 0.5x, 1.0x, 2.0x) ──
    targets.speed_buttons.clear();
    for &speed in &SPEED_PRESETS {
        let spd_w = 38.0;
        let spd_rect = Rect::new(cur_x, btn_y, spd_w, btn_h);
        let is_spd_active = (current_speed - speed).abs() < 0.05;
        let is_spd_hovered = spd_rect.contains_point(params.cursor_pos);
        targets.speed_buttons.push((speed, spd_rect));

        let spd_id = tree.create_node();
        if let Some(node) = tree.get_mut(spd_id) {
            node.set_name("TimelineSpeedBtn");
            node.set_text(match speed {
                0.25 => ".25x",
                0.5 => ".5x",
                1.0 => "1x",
                2.0 => "2x",
                _ => "1x",
            });
            node.font_size = 10.5;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_spd_active {
                Color::rgba(0.0, 0.92, 1.0, 1.0)
            } else if is_spd_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.65, 0.68, 0.76, 1.0)
            };
            node.computed_rect = spd_rect;
            node.style = Style::new()
                .background(if is_spd_active {
                    Color::rgba(0.0, 0.35, 0.50, 0.40)
                } else if is_spd_hovered {
                    Color::rgba(0.18, 0.22, 0.28, 1.0)
                } else {
                    Color::rgba(0.12, 0.14, 0.18, 0.95)
                })
                .border_radius(4.0)
                .border(
                    1.0,
                    if is_spd_active {
                        Color::rgba(0.0, 0.85, 1.0, 0.80)
                    } else {
                        Color::rgba(0.24, 0.27, 0.35, 0.60)
                    },
                );
        }
        let _ = tree.add_child(tb_id, spd_id);
        cur_x += spd_w + 3.0;
    }
    cur_x += 5.0;

    // ── 7. Active Clip Badge ──
    let clip_name = player
        .and_then(|p| p.current_clip.as_ref())
        .map_or("No Clip", |c| c.name.as_str());

    let clip_w = (clip_name.len() as f32 * 7.5 + 32.0).clamp(90.0, 200.0);
    let clip_rect = Rect::new(cur_x, btn_y, clip_w, btn_h);

    let clip_id = tree.create_node();
    if let Some(node) = tree.get_mut(clip_id) {
        node.set_name("TimelineClipBadge");
        node.set_text(format!("🎬 {}", clip_name));
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.95, 0.55, 0.75, 1.0);
        node.computed_rect = clip_rect;
        node.style = Style::new()
            .background(Color::rgba(0.22, 0.12, 0.18, 0.70))
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.85, 0.40, 0.65, 0.40));
    }
    let _ = tree.add_child(tb_id, clip_id);

    // ── 8. Time & Frame Readout (Right-Aligned) ──
    let readout_w = 170.0;
    let readout_x = (tb_rect.x + tb_rect.width - readout_w - 10.0).max(cur_x + clip_w + 10.0);
    let readout_rect = Rect::new(readout_x, btn_y, readout_w, btn_h);

    let current_frame = (current_time * 30.0).round() as i32;
    let total_frames = (duration * 30.0).round() as i32;

    let readout_id = tree.create_node();
    if let Some(node) = tree.get_mut(readout_id) {
        node.set_name("TimelineTimeReadout");
        node.set_text(format!(
            "{:.2}s / {:.2}s • F: {}/{}",
            current_time, duration, current_frame, total_frames
        ));
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.0, 0.88, 1.0, 1.0);
        node.computed_rect = readout_rect;
    }
    let _ = tree.add_child(tb_id, readout_id);
}