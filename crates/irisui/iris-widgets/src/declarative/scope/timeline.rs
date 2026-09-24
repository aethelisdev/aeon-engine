// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Animation Timeline & Transport Primitives
//!
//! Provides pure declarative UI scope extensions for constructing media playback
//! toolbars, dynamic division time rulers, keyframe indicators, and interactive scrubber
//! tracks with automated flexbox and subtree flow layout.
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::timeline::{
    MediaTransportStyle, TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_SCRUBBER_TRACK,
    TimelineKeyframeMarker, TimelineRulerStyle,
};
use iris_core::{Color, Insets, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole};

impl<'a> UiScope<'a> {
    /// Emits a styled playback transport control button with semantic tagging and interaction evaluation.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the created UI node.
    /// * `glyph` - Text icon or character displayed centered inside the button (e.g. `▶`, `⏸`, `⏮`).
    /// * `tag` - Persistent 64-bit semantic tag for hit-testing and event dispatching.
    /// * `is_active` - Whether the button is in an active/toggled state (e.g. playback currently active).
    /// * `is_accent` - Whether to render with prominent brand accent colors.
    /// * `style` - Visual styling properties derived from [`MediaTransportStyle`].
    pub fn timeline_transport_button(
        &mut self,
        name: &'static str,
        glyph: &str,
        tag: u64,
        is_active: bool,
        is_accent: bool,
        style: &MediaTransportStyle,
    ) -> (WidgetId, WidgetResponse) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let bg = if is_active {
            style.play_bg_active
        } else if hovered {
            style.btn_bg_hover
        } else if is_accent {
            Color::rgba(0.12, 0.16, 0.22, 0.95)
        } else {
            style.btn_bg_idle
        };

        let border_color = if is_active {
            style.play_border_active
        } else if hovered {
            Color::rgba(0.0, 0.85, 1.0, 0.50)
        } else {
            style.btn_border
        };

        let text_color = if is_active {
            style.play_text_active
        } else if hovered {
            style.btn_text_hover
        } else if is_accent {
            Color::rgba(0.0, 0.90, 1.0, 1.0)
        } else {
            style.btn_text_idle
        };

        let btn_w = if is_accent { 34.0 } else { 28.0 };
        let btn_h = 24.0;

        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.set_text(glyph);
            node.font_size = if is_accent { 12.0 } else { 11.0 };
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.set_style(
                Style::new()
                    .width(btn_w)
                    .height(btn_h)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(4.0),
            );
        }
        (
            node_id,
            WidgetResponse::new(node_id, clicked, hovered, false),
        )
    }

    /// Emits a playback speed multiplier pill button.
    ///
    /// # Arguments
    /// * `speed` - Speed multiplier factor (e.g. `0.5`, `1.0`, `2.0`).
    /// * `is_selected` - Whether this multiplier matches the active playback speed.
    /// * `tag` - Persistent semantic tag assigned to this speed preset.
    /// * `style` - Visual styling properties derived from [`MediaTransportStyle`].
    pub fn timeline_speed_pill(
        &mut self,
        speed: f32,
        is_selected: bool,
        tag: u64,
        style: &MediaTransportStyle,
    ) -> (WidgetId, WidgetResponse) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TimelineSpeedPill");
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let bg = if is_selected {
            style.speed_active_bg
        } else if hovered {
            style.btn_bg_hover
        } else {
            style.btn_bg_idle
        };

        let border_color = if is_selected {
            style.speed_active_color
        } else {
            style.btn_border
        };

        let text_color = if is_selected {
            style.speed_active_color
        } else if hovered {
            style.btn_text_hover
        } else {
            style.btn_text_idle
        };

        let label = format!("{:.2}x", speed)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
            + "x";

        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.set_text(label);
            node.font_size = 9.5;
            node.line_height = 20.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.set_style(
                Style::new()
                    .width(32.0)
                    .height(20.0)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(3.0),
            );
        }
        (
            node_id,
            WidgetResponse::new(node_id, clicked, hovered, false),
        )
    }

    /// Emits a playback looping toggle pill button.
    ///
    /// # Arguments
    /// * `is_looping` - Whether playback looping is active.
    /// * `tag` - Semantic tag assigned to the loop toggle control.
    /// * `style` - Visual styling properties derived from [`MediaTransportStyle`].
    pub fn timeline_loop_pill(
        &mut self,
        is_looping: bool,
        tag: u64,
        style: &MediaTransportStyle,
    ) -> (WidgetId, WidgetResponse) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TimelineLoopPill");
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let bg = if is_looping {
            style.loop_active_bg
        } else if hovered {
            style.btn_bg_hover
        } else {
            style.btn_bg_idle
        };

        let border_color = if is_looping {
            style.loop_active_color
        } else {
            style.btn_border
        };

        let text_color = if is_looping {
            style.loop_active_color
        } else if hovered {
            style.btn_text_hover
        } else {
            style.btn_text_idle
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.set_text("🔁");
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.set_style(
                Style::new()
                    .width(28.0)
                    .height(24.0)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(4.0),
            );
        }
        (
            node_id,
            WidgetResponse::new(node_id, clicked, hovered, false),
        )
    }

    /// Emits a badge pill displaying the title of the active animation clip.
    ///
    /// # Arguments
    /// * `clip_name` - Optional clip title string. If `None`, renders `No Clip`.
    /// * `style` - Visual styling properties derived from [`MediaTransportStyle`].
    pub fn timeline_clip_badge(
        &mut self,
        clip_name: Option<&str>,
        style: &MediaTransportStyle,
    ) -> WidgetId {
        let text = clip_name
            .map(|name| format!("🎬  {}", name))
            .unwrap_or_else(|| "🎬  No Clip".to_string());

        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TimelineClipBadge");
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.font_size = 10.0;
            node.line_height = 22.0;
            node.text_align = TextAlign::Center;
            node.text_color = style.clip_badge_text;
            node.set_style(
                Style::new()
                    .height(22.0)
                    .background(style.clip_badge_bg)
                    .border(1.0, style.clip_badge_border)
                    .border_radius(4.0)
                    .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a timestamp and frame index readout display badge.
    ///
    /// Formats playback position into elapsed timestamp and discrete frame indices:
    /// `00:01.50 / 00:03.00  (F45 / F90)`.
    ///
    /// # Arguments
    /// * `current_time` - Current playhead timestamp in seconds.
    /// * `duration` - Total clip length in seconds.
    /// * `fps` - Playback frames per second.
    /// * `style` - Visual styling properties derived from [`MediaTransportStyle`].
    pub fn timeline_time_readout(
        &mut self,
        current_time: f32,
        duration: f32,
        fps: f32,
        style: &MediaTransportStyle,
    ) -> WidgetId {
        let cur_m = (current_time / 60.0).floor() as u32;
        let cur_s = current_time % 60.0;
        let dur_m = (duration / 60.0).floor() as u32;
        let dur_s = duration % 60.0;

        let cur_f = (current_time * fps).round() as u32;
        let total_f = (duration * fps).round() as u32;

        let text = format!(
            "{:02}:{:05.2} / {:02}:{:05.2}  (F{} / F{})",
            cur_m, cur_s, dur_m, dur_s, cur_f, total_f
        );

        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TimelineTimeReadout");
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.font_size = 9.5;
            node.line_height = 22.0;
            node.text_align = TextAlign::Center;
            node.text_color = style.time_readout_color;
            node.set_style(
                Style::new()
                    .height(22.0)
                    .background(Color::rgba(0.06, 0.08, 0.11, 0.80))
                    .border(1.0, Color::rgba(0.18, 0.22, 0.28, 0.50))
                    .border_radius(4.0)
                    .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a dynamic time ruler bar container displaying division tick marks and timestamp labels.
    ///
    /// # Arguments
    /// * `duration` - Total duration of the animation clip in seconds.
    /// * `height` - Fixed vertical height of the ruler section in physical pixels.
    /// * `style` - Visual styling properties derived from [`TimelineRulerStyle`].
    pub fn timeline_ruler_bar(
        &mut self,
        duration: f32,
        height: f32,
        style: &TimelineRulerStyle,
    ) -> WidgetId {
        let ruler_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(ruler_id) {
            node.set_name("TimelineRulerRoot");
            node.role = WidgetRole::TimelineRuler;
            node.set_style(
                Style::new()
                    .height(height)
                    .clip_children(true)
                    .background(Color::rgba(0.07, 0.08, 0.11, 0.60)),
            );
        }
        let _ = self.tree.add_child(self.parent, ruler_id);

        let safe_duration = duration.max(0.001);
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

        let tick_count = ((duration / step).ceil() as usize).min(120);
        for i in 0..=tick_count {
            let t = (i as f32 * step).min(duration);
            let frac = (t / safe_duration).clamp(0.0, 1.0);
            let is_major = (i % 2 == 0) || (t == 0.0) || ((t - duration).abs() < 0.001);
            let tick_h = if is_major { 8.0 } else { 4.0 };

            let tick_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(tick_id) {
                node.set_name("TimelineRulerTick");
                node.set_style(
                    Style::new()
                        .height(tick_h)
                        .left(frac)
                        .background(if is_major {
                            style.major_tick_color
                        } else {
                            style.minor_tick_color
                        }),
                );
            }
            let _ = self.tree.add_child(ruler_id, tick_id);

            if is_major {
                let label_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(label_id) {
                    node.set_name("TimelineRulerLabel");
                    node.set_text(format!("{:.1}s", t));
                    node.font_size = style.tick_label_font_size;
                    node.line_height = 12.0;
                    node.text_align = TextAlign::Left;
                    node.text_color = style.tick_label_color;
                    node.set_style(Style::new().left(frac));
                }
                let _ = self.tree.add_child(ruler_id, label_id);
            }
        }

        ruler_id
    }

    /// Emits an interactive timeline scrubber track containing progress fill, keyframe diamonds,
    /// and the draggable playhead needle and cap handle.
    ///
    /// Returns `(track_widget_id, playhead_cap_widget_id)`.
    ///
    /// # Arguments
    /// * `duration` - Total duration of the animation clip in seconds.
    /// * `current_time` - Current playhead timestamp in seconds.
    /// * `is_dragging` - Whether the user is actively dragging the scrubber playhead.
    /// * `keyframes` - Slice of keyframe markers along the timeline.
    /// * `height` - Vertical height allocated for the track in physical pixels.
    /// * `style` - Visual styling properties derived from [`TimelineRulerStyle`].
    pub fn timeline_scrubber_track(
        &mut self,
        duration: f32,
        current_time: f32,
        is_dragging: bool,
        keyframes: &[TimelineKeyframeMarker],
        height: f32,
        style: &TimelineRulerStyle,
    ) -> (WidgetId, WidgetId) {
        let safe_duration = duration.max(0.001);
        let progress_ratio = (current_time / safe_duration).clamp(0.0, 1.0);

        let track_id = self.tree.create_node();
        let (_, is_hovered, _) = self.check_interaction(track_id);

        let track_border = if is_hovered || is_dragging {
            style.track_border_active
        } else {
            style.track_border_idle
        };

        if let Some(node) = self.tree.get_mut(track_id) {
            node.set_name("TimelineScrubberTrack");
            node.role = WidgetRole::TimelineTrack;
            node.cursor = Some(WidgetCursor::ColResize);
            node.interactive = true;
            node.tag = TIMELINE_TAG_SCRUBBER_TRACK;
            node.set_style(
                Style::new()
                    .height(height)
                    .background(style.track_bg)
                    .border_radius(style.track_border_radius)
                    .border(style.track_border_width, track_border)
                    .clip_children(false),
            );
        }
        let _ = self.tree.add_child(self.parent, track_id);

        // 1. Progress fill
        let fill_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(fill_id) {
            node.set_name("TimelineProgressFill");
            node.role = WidgetRole::Default;
            node.interactive = false;
            node.set_style(
                Style::new()
                    .left(progress_ratio)
                    .background(style.progress_fill_color)
                    .border_radius(style.track_border_radius),
            );
        }
        let _ = self.tree.add_child(track_id, fill_id);

        // 2. Keyframe markers
        for kf in keyframes {
            let kf_frac = (kf.time / safe_duration).clamp(0.0, 1.0);
            let kf_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(kf_id) {
                node.set_name("TimelineKeyframeMarker");
                node.role = WidgetRole::TimelineKeyframe;
                node.interactive = false;
                node.set_text("◆");
                node.font_size = 11.0;
                node.line_height = 12.0;
                node.text_align = TextAlign::Center;
                node.text_color = kf.color.unwrap_or(style.keyframe_color);
                node.set_style(Style::new().left(kf_frac));
            }
            let _ = self.tree.add_child(track_id, kf_id);
        }

        // 3. Playhead needle line
        let needle_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(needle_id) {
            node.set_name("TimelinePlayheadNeedle");
            node.role = WidgetRole::TimelinePlayhead;
            node.interactive = false;
            node.set_style(
                Style::new()
                    .left(progress_ratio)
                    .background(style.playhead_needle_color),
            );
        }
        let _ = self.tree.add_child(track_id, needle_id);

        // 4. Playhead handle cap (top indicator ▼)
        let cap_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(cap_id) {
            node.set_name("TimelinePlayheadCap");
            node.role = WidgetRole::TimelinePlayhead;
            node.cursor = Some(WidgetCursor::ColResize);
            node.interactive = true;
            node.tag = TIMELINE_TAG_PLAYHEAD_CAP;
            node.set_text("▼");
            node.font_size = 9.0;
            node.line_height = 10.0;
            node.text_align = TextAlign::Center;
            node.text_color = style.playhead_cap_color;
            node.set_style(Style::new().left(progress_ratio));
        }
        let _ = self.tree.add_child(track_id, cap_id);

        (track_id, cap_id)
    }
}