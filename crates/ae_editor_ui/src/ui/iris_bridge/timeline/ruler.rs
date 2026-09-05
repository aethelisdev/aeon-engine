// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Ruler and Interactive Scrubber Builder
//!
//! Renders the dynamic time ruler with adaptive second/millisecond intervals,
//! keyframe diamond indicators, translucent progress fill, and interactive
//! playhead needle scrubbing track.
//!

use super::types::{TimelinePanelParams, TimelinePanelTargets};
use irisui::prelude::*;

/// Height of the time ruler section above the track in physical pixels.
pub const RULER_HEIGHT: f32 = 18.0;

/// Height of the interactive scrubber track in physical pixels.
pub const SCRUBBER_TRACK_HEIGHT: f32 = 36.0;

/// Total height occupied by the ruler and scrubber subsystem.
pub const RULER_TOTAL_HEIGHT: f32 = RULER_HEIGHT + SCRUBBER_TRACK_HEIGHT + 6.0;

/// Builds the time ruler, keyframe markers, and interactive playhead scrubber into the UI tree.
pub fn build_ruler_and_scrubber(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    targets: &mut TimelinePanelTargets,
    start_y: f32,
    duration: f32,
) {
    let padding_x = 10.0;
    let available_w = (params.panel_rect.width - padding_x * 2.0).max(100.0);
    let ruler_y = start_y + 4.0;

    let track_rect = Rect::new(
        params.panel_rect.x + padding_x,
        ruler_y + RULER_HEIGHT + 2.0,
        available_w,
        SCRUBBER_TRACK_HEIGHT,
    );
    targets.scrubber_track_rect = Some(track_rect);
    targets.clip_duration = duration;

    let player = params.animation_player;
    let current_time = player.map_or(0.0, |p| p.current_time).clamp(0.0, duration);
    let progress_ratio = if duration > 0.001 {
        (current_time / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // ── 1. Dynamic Time Ruler Ticks and Labels ──
    let step = if duration <= 1.0 {
        0.1
    } else if duration <= 3.0 {
        0.25
    } else if duration <= 10.0 {
        0.5
    } else if duration <= 30.0 {
        1.0
    } else {
        5.0
    };

    let tick_count = ((duration / step).ceil() as usize).min(60);
    for i in 0..=tick_count {
        let t = (i as f32 * step).min(duration);
        let frac = if duration > 0.001 {
            (t / duration).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let tick_x = track_rect.x + frac * track_rect.width;

        let is_major = (i % 2 == 0) || (t == 0.0) || ((t - duration).abs() < 0.001);
        let tick_h = if is_major { 8.0 } else { 4.0 };
        let tick_y = ruler_y + RULER_HEIGHT - tick_h;

        // Tick mark line
        let tick_id = tree.create_node();
        if let Some(node) = tree.get_mut(tick_id) {
            node.set_name("TimelineRulerTick");
            node.computed_rect = Rect::new(tick_x, tick_y, 1.0, tick_h);
            node.style = Style::new().background(if is_major {
                Color::rgba(0.50, 0.55, 0.68, 0.90)
            } else {
                Color::rgba(0.30, 0.34, 0.44, 0.60)
            });
        }
        let _ = tree.add_child(parent_id, tick_id);

        // Major tick text label
        if is_major && tick_x + 28.0 <= track_rect.x + track_rect.width + 10.0 {
            let label_id = tree.create_node();
            if let Some(node) = tree.get_mut(label_id) {
                node.set_name("TimelineRulerLabel");
                node.set_text(format!("{:.1}s", t));
                node.font_size = 9.5;
                node.line_height = 12.0;
                node.text_align = TextAlign::Left;
                node.text_color = Color::rgba(0.55, 0.60, 0.72, 1.0);
                node.computed_rect = Rect::new(tick_x + 2.0, ruler_y, 30.0, 12.0);
            }
            let _ = tree.add_child(parent_id, label_id);
        }
    }

    // ── 2. Scrubber Track Background ──
    let is_track_hovered = track_rect.contains_point(params.cursor_pos);
    let track_id = tree.create_node();
    if let Some(node) = tree.get_mut(track_id) {
        node.set_name("TimelineScrubberTrack");
        node.computed_rect = track_rect;
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.95))
            .border_radius(4.0)
            .border(
                1.0,
                if is_track_hovered || params.is_dragging_scrubber {
                    Color::rgba(0.0, 0.85, 1.0, 0.60)
                } else {
                    Color::rgba(0.20, 0.23, 0.32, 0.70)
                },
            );
    }
    let _ = tree.add_child(parent_id, track_id);

    // ── 3. Translucent Progress Fill ──
    let fill_w = track_rect.width * progress_ratio;
    if fill_w > 1.0 {
        let fill_rect = Rect::new(track_rect.x, track_rect.y, fill_w, track_rect.height);
        let fill_id = tree.create_node();
        if let Some(node) = tree.get_mut(fill_id) {
            node.set_name("TimelineProgressFill");
            node.computed_rect = fill_rect;
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.75, 0.95, 0.16))
                .border_radius(4.0);
        }
        let _ = tree.add_child(track_id, fill_id);
    }

    // ── 4. Keyframe Diamond Markers ──
    if let Some(clip) = player.and_then(|p| p.current_clip.as_ref()) {
        let mut keyframe_times: Vec<f32> = Vec::new();
        for channel in &clip.channels {
            if let Some(ref track) = channel.vector_track {
                for kf in &track.keyframes {
                    if !keyframe_times.iter().any(|&kt| (kt - kf.time).abs() < 0.02) {
                        keyframe_times.push(kf.time);
                    }
                }
            }
            if let Some(ref track) = channel.rotation_track {
                for kf in &track.keyframes {
                    if !keyframe_times.iter().any(|&kt| (kt - kf.time).abs() < 0.02) {
                        keyframe_times.push(kf.time);
                    }
                }
            }
        }
        keyframe_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        for kf_t in keyframe_times {
            let kf_frac = if duration > 0.001 {
                (kf_t / duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let kf_x = track_rect.x + kf_frac * track_rect.width;
            let kf_y = track_rect.y + track_rect.height * 0.5 - 6.0;

            let kf_id = tree.create_node();
            if let Some(node) = tree.get_mut(kf_id) {
                node.set_name("TimelineKeyframeMarker");
                node.set_text("◆");
                node.font_size = 11.0;
                node.line_height = 12.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.96, 0.72, 0.18, 1.0);
                node.computed_rect = Rect::new(kf_x - 6.0, kf_y, 12.0, 12.0);
            }
            let _ = tree.add_child(track_id, kf_id);
        }
    }

    // ── 5. Playhead Needle and Draggable Handle ──
    let needle_x =
        (track_rect.x + fill_w).clamp(track_rect.x, track_rect.x + track_rect.width - 2.0);
    let needle_top = ruler_y + 4.0;
    let needle_bottom = track_rect.y + track_rect.height;

    // Draggable Playhead Cap Handle
    let cap_w = 12.0;
    let cap_h = 10.0;
    let cap_rect = Rect::new(needle_x - cap_w * 0.5 + 1.0, needle_top, cap_w, cap_h);
    targets.playhead_needle_rect = Some(cap_rect);

    let cap_id = tree.create_node();
    if let Some(node) = tree.get_mut(cap_id) {
        node.set_name("TimelinePlayheadCap");
        node.set_text("▼");
        node.font_size = 9.0;
        node.line_height = cap_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.0, 0.95, 1.0, 1.0);
        node.computed_rect = cap_rect;
    }
    let _ = tree.add_child(parent_id, cap_id);

    // Vertical Playhead Needle Line
    let needle_line_id = tree.create_node();
    if let Some(node) = tree.get_mut(needle_line_id) {
        node.set_name("TimelinePlayheadNeedle");
        node.computed_rect = Rect::new(
            needle_x,
            needle_top + cap_h,
            2.0,
            needle_bottom - (needle_top + cap_h),
        );
        node.style = Style::new().background(Color::rgba(0.0, 0.92, 1.0, 1.0));
    }
    let _ = tree.add_child(parent_id, needle_line_id);
}