// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Media Playback Transport Bar Widget (`iris-widgets::timeline::transport`)
//!
//! Provides the fluent builder, layout generator, and interactive styling for playback
//! transport controls (Step Back, Play/Pause, Stop, Step Forward, Loop, Speed, Badges)
//! in Iris UI.
//!

use super::style::MediaTransportStyle;
use super::types::MediaTransportAction;
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;
use iris_core::{WidgetCursor, WidgetRole};

/// Standard playback speed multipliers.
pub const DEFAULT_SPEED_PRESETS: [f32; 4] = [0.25, 0.5, 1.0, 2.0];

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

/// Output layout frame returned after constructing a media transport bar.
///
/// Contains hit-testing bounding boxes for transport buttons, loop toggles, and speed pills,
/// along with an evaluator helper for click events.
#[derive(Clone, Debug, PartialEq)]
pub struct MediaTransportBarFrame {
    /// Root container node ID for the transport toolbar.
    pub root_id: WidgetId,
    /// Bounding rectangle of the entire transport toolbar.
    pub rect: Rect,
    /// Step back one frame button bounding box.
    pub step_back_rect: Rect,
    /// Play/Pause toggle button bounding box.
    pub play_pause_rect: Rect,
    /// Stop button bounding box.
    pub stop_rect: Rect,
    /// Step forward one frame button bounding box.
    pub step_forward_rect: Rect,
    /// Loop toggle pill bounding box, if enabled.
    pub loop_toggle_rect: Option<Rect>,
    /// Playback speed selector button bounding boxes: `(multiplier, rect)`.
    pub speed_button_rects: Vec<(f32, Rect)>,
    /// Active media or animation clip title badge bounding box, if displayed.
    pub clip_badge_rect: Option<Rect>,
    /// Timestamp and frame index readout bounding box, if displayed.
    pub readout_rect: Option<Rect>,
}

impl MediaTransportBarFrame {
    /// Evaluates a mouse click against all transport buttons and returns the corresponding action.
    #[must_use]
    pub fn evaluate_click(&self, click_pos: Point) -> Option<MediaTransportAction> {
        if self.play_pause_rect.contains_point(click_pos) {
            return Some(MediaTransportAction::TogglePlayPause);
        }
        if self.stop_rect.contains_point(click_pos) {
            return Some(MediaTransportAction::Stop);
        }
        if self.step_back_rect.contains_point(click_pos) {
            return Some(MediaTransportAction::StepBack);
        }
        if self.step_forward_rect.contains_point(click_pos) {
            return Some(MediaTransportAction::StepForward);
        }
        if let Some(r) = self.loop_toggle_rect
            && r.contains_point(click_pos)
        {
            return Some(MediaTransportAction::ToggleLoop);
        }
        for &(speed, rect) in &self.speed_button_rects {
            if rect.contains_point(click_pos) {
                return Some(MediaTransportAction::SetSpeed(speed));
            }
        }
        None
    }
}

/// Fluent builder for constructing a generic media playback transport toolbar widget.
pub struct MediaTransportBarBuilder<'a> {
    rect: Rect,
    duration: f32,
    current_time: f32,
    is_playing: bool,
    is_looping: bool,
    current_speed: f32,
    speed_presets: &'a [f32],
    clip_name: Option<&'a str>,
    fps: f32,
    cursor_pos: Option<Point>,
    style: MediaTransportStyle,
    show_loop: bool,
    show_speed: bool,
    show_readout: bool,
}

impl<'a> MediaTransportBarBuilder<'a> {
    /// Creates a new media transport bar builder with bounding rect, duration, and current time.
    #[must_use]
    pub fn new(rect: Rect, duration: f32, current_time: f32) -> Self {
        Self {
            rect,
            duration: duration.max(0.0),
            current_time: current_time.max(0.0),
            is_playing: false,
            is_looping: false,
            current_speed: 1.0,
            speed_presets: &DEFAULT_SPEED_PRESETS,
            clip_name: None,
            fps: 30.0,
            cursor_pos: None,
            style: MediaTransportStyle::default(),
            show_loop: true,
            show_speed: true,
            show_readout: true,
        }
    }

    /// Sets whether playback is currently playing.
    #[must_use]
    pub fn is_playing(mut self, is_playing: bool) -> Self {
        self.is_playing = is_playing;
        self
    }

    /// Sets whether looping mode is active.
    #[must_use]
    pub fn is_looping(mut self, is_looping: bool) -> Self {
        self.is_looping = is_looping;
        self
    }

    /// Sets the current playback speed multiplier.
    #[must_use]
    pub fn current_speed(mut self, current_speed: f32) -> Self {
        self.current_speed = current_speed;
        self
    }

    /// Overrides the speed multiplier options shown in the transport bar.
    #[must_use]
    pub fn speed_presets(mut self, speed_presets: &'a [f32]) -> Self {
        self.speed_presets = speed_presets;
        self
    }

    /// Sets an optional clip or media title string to display in a badge.
    #[must_use]
    pub fn clip_name(mut self, clip_name: Option<&'a str>) -> Self {
        self.clip_name = clip_name;
        self
    }

    /// Sets the playback frames per second used for frame count formatting.
    #[must_use]
    pub fn fps(mut self, fps: f32) -> Self {
        self.fps = fps.max(1.0);
        self
    }

    /// Sets the current mouse cursor position for button hover evaluation.
    #[must_use]
    pub fn cursor_pos(mut self, cursor_pos: Option<Point>) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Overrides the visual styling configuration for the transport bar.
    #[must_use]
    pub fn style(mut self, style: MediaTransportStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets whether to render the loop toggle button.
    #[must_use]
    pub fn show_loop(mut self, show_loop: bool) -> Self {
        self.show_loop = show_loop;
        self
    }

    /// Sets whether to render playback speed selector buttons.
    #[must_use]
    pub fn show_speed(mut self, show_speed: bool) -> Self {
        self.show_speed = show_speed;
        self
    }

    /// Sets whether to render the timestamp and frame readout display.
    #[must_use]
    pub fn show_readout(mut self, show_readout: bool) -> Self {
        self.show_readout = show_readout;
        self
    }

    /// Builds the transport controls toolbar hierarchy into the given UI tree.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> MediaTransportBarFrame {
        let root_id = tree.create_node();
        if let Some(node) = tree.get_mut(root_id) {
            node.set_name("TimelineTransportToolbar");
            node.computed_rect = self.rect;
            node.role = WidgetRole::Default;
            node.style = Style::new()
                .background(self.style.bg)
                .border(self.style.border_width, self.style.border_color);
        }
        let _ = tree.add_child(parent_id, root_id);

        let mut cur_x = self.rect.x + 8.0;
        let btn_y = self.rect.y + 5.0;
        let btn_h = (self.rect.height - 10.0).max(18.0);

        // ── 1. Step Back Button ──
        let step_back_w = 28.0;
        let step_back_rect = Rect::new(cur_x, btn_y, step_back_w, btn_h);
        let is_step_back_hovered = self
            .cursor_pos
            .is_some_and(|pos| step_back_rect.contains_point(pos));

        let step_back_id = tree.create_node();
        if let Some(node) = tree.get_mut(step_back_id) {
            node.set_name("TimelineStepBackBtn");
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.tag = TIMELINE_TAG_STEP_BACK;
            node.set_text("⏮");
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_step_back_hovered {
                self.style.btn_text_hover
            } else {
                self.style.btn_text_idle
            };
            node.computed_rect = step_back_rect;
            node.style = Style::new()
                .background(if is_step_back_hovered {
                    self.style.btn_bg_hover
                } else {
                    self.style.btn_bg_idle
                })
                .border_radius(4.0)
                .border(1.0, self.style.btn_border);
        }
        let _ = tree.add_child(root_id, step_back_id);
        cur_x += step_back_w + 4.0;

        // ── 2. Play / Pause Button ──
        let play_pause_w = 34.0;
        let play_pause_rect = Rect::new(cur_x, btn_y, play_pause_w, btn_h);
        let is_play_hovered = self
            .cursor_pos
            .is_some_and(|pos| play_pause_rect.contains_point(pos));

        let play_pause_id = tree.create_node();
        if let Some(node) = tree.get_mut(play_pause_id) {
            node.set_name("TimelinePlayPauseBtn");
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.tag = TIMELINE_TAG_PLAY_PAUSE;
            node.set_text(if self.is_playing { "⏸" } else { "▶" });
            node.font_size = 12.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if self.is_playing {
                self.style.play_text_active
            } else if is_play_hovered {
                self.style.btn_text_hover
            } else {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            };
            node.computed_rect = play_pause_rect;
            node.style = Style::new()
                .background(if self.is_playing {
                    self.style.play_bg_active
                } else if is_play_hovered {
                    Color::rgba(0.18, 0.25, 0.35, 1.0)
                } else {
                    Color::rgba(0.12, 0.16, 0.22, 0.95)
                })
                .border_radius(4.0)
                .border(
                    1.0,
                    if self.is_playing {
                        self.style.play_border_active
                    } else if is_play_hovered {
                        Color::rgba(0.0, 0.85, 1.0, 0.80)
                    } else {
                        Color::rgba(0.0, 0.70, 0.85, 0.50)
                    },
                );
        }
        let _ = tree.add_child(root_id, play_pause_id);
        cur_x += play_pause_w + 4.0;

        // ── 3. Stop Button ──
        let stop_w = 28.0;
        let stop_rect = Rect::new(cur_x, btn_y, stop_w, btn_h);
        let is_stop_hovered = self
            .cursor_pos
            .is_some_and(|pos| stop_rect.contains_point(pos));

        let stop_id = tree.create_node();
        if let Some(node) = tree.get_mut(stop_id) {
            node.set_name("TimelineStopBtn");
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.tag = TIMELINE_TAG_STOP;
            node.set_text("⏹");
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_stop_hovered {
                self.style.stop_text_hover
            } else {
                self.style.btn_text_idle
            };
            node.computed_rect = stop_rect;
            node.style = Style::new()
                .background(if is_stop_hovered {
                    Color::rgba(0.25, 0.15, 0.18, 1.0)
                } else {
                    self.style.btn_bg_idle
                })
                .border_radius(4.0)
                .border(1.0, self.style.btn_border);
        }
        let _ = tree.add_child(root_id, stop_id);
        cur_x += stop_w + 4.0;

        // ── 4. Step Forward Button ──
        let step_fwd_w = 28.0;
        let step_fwd_rect = Rect::new(cur_x, btn_y, step_fwd_w, btn_h);
        let is_step_fwd_hovered = self
            .cursor_pos
            .is_some_and(|pos| step_fwd_rect.contains_point(pos));

        let step_fwd_id = tree.create_node();
        if let Some(node) = tree.get_mut(step_fwd_id) {
            node.set_name("TimelineStepFwdBtn");
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.interactive = true;
            node.tag = TIMELINE_TAG_STEP_FWD;
            node.set_text("⏭");
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_step_fwd_hovered {
                self.style.btn_text_hover
            } else {
                self.style.btn_text_idle
            };
            node.computed_rect = step_fwd_rect;
            node.style = Style::new()
                .background(if is_step_fwd_hovered {
                    self.style.btn_bg_hover
                } else {
                    self.style.btn_bg_idle
                })
                .border_radius(4.0)
                .border(1.0, self.style.btn_border);
        }
        let _ = tree.add_child(root_id, step_fwd_id);
        cur_x += step_fwd_w + 8.0;

        // ── Divider ──
        let sep_rect = Rect::new(cur_x, btn_y + 3.0, 1.0, (btn_h - 6.0).max(4.0));
        let sep_id = tree.create_node();
        if let Some(node) = tree.get_mut(sep_id) {
            node.set_name("TimelineToolbarDivider");
            node.computed_rect = sep_rect;
            node.style = Style::new().background(self.style.divider_color);
        }
        let _ = tree.add_child(root_id, sep_id);
        cur_x += 9.0;

        // ── 5. Loop Toggle Pill ──
        let loop_toggle_rect = if self.show_loop {
            let loop_w = 64.0;
            let loop_rect = Rect::new(cur_x, btn_y, loop_w, btn_h);
            let is_loop_hovered = self
                .cursor_pos
                .is_some_and(|pos| loop_rect.contains_point(pos));

            let loop_id = tree.create_node();
            if let Some(node) = tree.get_mut(loop_id) {
                node.set_name("TimelineLoopToggle");
                node.role = WidgetRole::Button;
                node.cursor = Some(WidgetCursor::Pointer);
                node.interactive = true;
                node.tag = TIMELINE_TAG_LOOP;
                node.set_text("🔁 Loop");
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_align = TextAlign::Center;
                node.text_color = if self.is_looping {
                    self.style.loop_active_color
                } else {
                    Color::rgba(0.60, 0.64, 0.72, 1.0)
                };
                node.computed_rect = loop_rect;
                node.style = Style::new()
                    .background(if self.is_looping {
                        self.style.loop_active_bg
                    } else if is_loop_hovered {
                        Color::rgba(0.18, 0.22, 0.28, 1.0)
                    } else {
                        self.style.btn_bg_idle
                    })
                    .border_radius(4.0)
                    .border(
                        1.0,
                        if self.is_looping {
                            Color::rgba(0.0, 0.85, 1.0, 0.70)
                        } else {
                            self.style.btn_border
                        },
                    );
            }
            let _ = tree.add_child(root_id, loop_id);
            cur_x += loop_w + 8.0;
            Some(loop_rect)
        } else {
            None
        };

        // ── 6. Speed Buttons ──
        let mut speed_button_rects = Vec::new();
        if self.show_speed {
            for (idx, &speed) in self.speed_presets.iter().enumerate() {
                let spd_w = 38.0;
                let spd_rect = Rect::new(cur_x, btn_y, spd_w, btn_h);
                let is_spd_active = (self.current_speed - speed).abs() < 0.05;
                let is_spd_hovered = self
                    .cursor_pos
                    .is_some_and(|pos| spd_rect.contains_point(pos));
                speed_button_rects.push((speed, spd_rect));

                let spd_id = tree.create_node();
                if let Some(node) = tree.get_mut(spd_id) {
                    node.set_name("TimelineSpeedBtn");
                    node.role = WidgetRole::Button;
                    node.cursor = Some(WidgetCursor::Pointer);
                    node.interactive = true;
                    node.tag = TIMELINE_TAG_SPEED_BASE + idx as u64;
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
                        self.style.speed_active_color
                    } else if is_spd_hovered {
                        self.style.btn_text_hover
                    } else {
                        Color::rgba(0.65, 0.68, 0.76, 1.0)
                    };
                    node.computed_rect = spd_rect;
                    node.style = Style::new()
                        .background(if is_spd_active {
                            self.style.speed_active_bg
                        } else if is_spd_hovered {
                            Color::rgba(0.18, 0.22, 0.28, 1.0)
                        } else {
                            self.style.btn_bg_idle
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
                let _ = tree.add_child(root_id, spd_id);
                cur_x += spd_w + 3.0;
            }
            cur_x += 5.0;
        }

        // ── 7. Active Clip / Media Badge ──
        let clip_badge_rect = if let Some(clip_name) = self.clip_name {
            let clip_w = (clip_name.len() as f32 * 7.5 + 32.0).clamp(90.0, 200.0);
            let clip_rect = Rect::new(cur_x, btn_y, clip_w, btn_h);

            let clip_id = tree.create_node();
            if let Some(node) = tree.get_mut(clip_id) {
                node.set_name("TimelineClipBadge");
                node.set_text(format!("🎬 {}", clip_name));
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_align = TextAlign::Center;
                node.text_color = self.style.clip_badge_text;
                node.computed_rect = clip_rect;
                node.style = Style::new()
                    .background(self.style.clip_badge_bg)
                    .border_radius(4.0)
                    .border(1.0, self.style.clip_badge_border);
            }
            let _ = tree.add_child(root_id, clip_id);
            cur_x += clip_w;
            Some(clip_rect)
        } else {
            None
        };

        // ── 8. Time & Frame Readout (Right-Aligned) ──
        let readout_rect = if self.show_readout {
            let readout_w = 170.0;
            let readout_x = (self.rect.x + self.rect.width - readout_w - 10.0).max(cur_x + 10.0);
            let readout_rect = Rect::new(readout_x, btn_y, readout_w, btn_h);

            let current_frame = (self.current_time * self.fps).round() as i32;
            let total_frames = (self.duration * self.fps).round() as i32;

            let readout_id = tree.create_node();
            if let Some(node) = tree.get_mut(readout_id) {
                node.set_name("TimelineTimeReadout");
                node.set_text(format!(
                    "{:.2}s / {:.2}s • F: {}/{}",
                    self.current_time, self.duration, current_frame, total_frames
                ));
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_align = TextAlign::Right;
                node.text_color = self.style.time_readout_color;
                node.computed_rect = readout_rect;
            }
            let _ = tree.add_child(root_id, readout_id);
            Some(readout_rect)
        } else {
            None
        };

        MediaTransportBarFrame {
            root_id,
            rect: self.rect,
            step_back_rect,
            play_pause_rect,
            stop_rect,
            step_forward_rect: step_fwd_rect,
            loop_toggle_rect,
            speed_button_rects,
            clip_badge_rect,
            readout_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_transport_build_and_click_eval() {
        let mut tree = UiTree::new();
        let parent = tree.create_node();
        let bounds = Rect::new(0.0, 0.0, 800.0, 36.0);

        let frame = MediaTransportBarBuilder::new(bounds, 10.0, 3.2)
            .is_playing(true)
            .is_looping(false)
            .current_speed(1.0)
            .clip_name(Some("WalkCycle"))
            .build(&mut tree, parent);

        assert_eq!(frame.rect, bounds);
        assert!(frame.loop_toggle_rect.is_some());
        assert_eq!(frame.speed_button_rects.len(), 4);
        assert!(frame.clip_badge_rect.is_some());
        assert!(frame.readout_rect.is_some());

        // Test click on play/pause
        let play_center = Point::new(frame.play_pause_rect.x + 5.0, frame.play_pause_rect.y + 5.0);
        assert_eq!(
            frame.evaluate_click(play_center),
            Some(MediaTransportAction::TogglePlayPause)
        );

        // Test click on stop
        let stop_center = Point::new(frame.stop_rect.x + 5.0, frame.stop_rect.y + 5.0);
        assert_eq!(
            frame.evaluate_click(stop_center),
            Some(MediaTransportAction::Stop)
        );

        // Test click on step back
        let back_center = Point::new(frame.step_back_rect.x + 5.0, frame.step_back_rect.y + 5.0);
        assert_eq!(
            frame.evaluate_click(back_center),
            Some(MediaTransportAction::StepBack)
        );

        // Test click on step forward
        let fwd_center = Point::new(
            frame.step_forward_rect.x + 5.0,
            frame.step_forward_rect.y + 5.0,
        );
        assert_eq!(
            frame.evaluate_click(fwd_center),
            Some(MediaTransportAction::StepForward)
        );

        // Test click on loop toggle
        let loop_center = Point::new(
            frame.loop_toggle_rect.unwrap().x + 5.0,
            frame.loop_toggle_rect.unwrap().y + 5.0,
        );
        assert_eq!(
            frame.evaluate_click(loop_center),
            Some(MediaTransportAction::ToggleLoop)
        );

        // Test click on speed button 0.5x
        let spd_half = frame.speed_button_rects[1].1;
        let spd_center = Point::new(spd_half.x + 5.0, spd_half.y + 5.0);
        assert_eq!(
            frame.evaluate_click(spd_center),
            Some(MediaTransportAction::SetSpeed(0.5))
        );
    }

    #[test]
    fn test_evaluate_timeline_transport_tag() {
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
            evaluate_timeline_transport_tag(TIMELINE_TAG_SPEED_BASE + 99, &presets),
            None
        );
        assert_eq!(evaluate_timeline_transport_tag(0x1234, &presets), None);
    }
}