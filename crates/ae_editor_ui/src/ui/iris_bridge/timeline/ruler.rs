// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Ruler and Interactive Scrubber Bridge
//!
//! Connects engine animation clip keyframe data and playback timestamp to pure declarative
//! [`UiScope`] ruler and scrubber primitives in Iris UI.
//!

use super::types::{
    TIMELINE_SIDEBAR_WIDTH, TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_SCRUBBER_TRACK,
    TimelineKeyframeMarker, TimelinePanelParams, TimelineRulerStyle,
};
use irisui::prelude::*;

/// Height of the time ruler section above the track in physical pixels.
pub const RULER_HEIGHT: f32 = 18.0;

/// Height of the interactive scrubber track in physical pixels.
pub const SCRUBBER_TRACK_HEIGHT: f32 = 36.0;

/// Total height occupied by the ruler and scrubber subsystem.
pub const RULER_TOTAL_HEIGHT: f32 = RULER_HEIGHT + SCRUBBER_TRACK_HEIGHT + 6.0;

/// Builds the time ruler, keyframe markers, and interactive playhead scrubber via [`UiScope`].
pub fn build_ruler_and_scrubber(
    scope: &mut UiScope<'_>,
    params: &TimelinePanelParams<'_>,
    duration: f32,
) {
    let player = params.animation_player;
    let current_time = player.map_or(0.0, |p| p.current_time).clamp(0.0, duration);
    let track_width = (params.panel_rect.width - 20.0).max(10.0);

    // Extract unique keyframe timestamps across all vector and rotation channels
    let mut keyframes: Vec<TimelineKeyframeMarker> = Vec::new();
    if let Some(clip) = player.and_then(|p| p.current_clip.as_ref()) {
        for channel in &clip.channels {
            if let Some(ref track) = channel.vector_track {
                for kf in &track.keyframes {
                    if !keyframes.iter().any(|k| (k.time - kf.time).abs() < 0.02) {
                        keyframes.push(TimelineKeyframeMarker::new(kf.time));
                    }
                }
            }
            if let Some(ref track) = channel.rotation_track {
                for kf in &track.keyframes {
                    if !keyframes.iter().any(|k| (k.time - kf.time).abs() < 0.02) {
                        keyframes.push(TimelineKeyframeMarker::new(kf.time));
                    }
                }
            }
        }
        keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let ruler_style = TimelineRulerStyle::dark_default();

    let section_style = Style::new()
        .flex_col()
        .gap(2.0)
        .padding_insets(Insets::new(4.0, 10.0, 4.0, 10.0));

    scope.container_named("TimelineRulerAndScrubberSection", section_style, |col| {
        let _ = render_ruler_bar(col, duration, RULER_HEIGHT, track_width, &ruler_style);
        let _ = render_scrubber_track(
            col,
            &ScrubberTrackParams {
                duration,
                current_time,
                is_dragging: params.is_dragging_scrubber,
                keyframes: &keyframes,
                height: SCRUBBER_TRACK_HEIGHT,
                track_width,
                style: &ruler_style,
            },
        );
    });
}

/// Emits a dynamic time ruler bar container displaying division tick marks and timestamp labels.
///
/// Uses declarative [`UiScope`] containers, leaf box primitives, and passive styled labels with
/// explicit absolute layout styles instead of raw node manipulation.
pub fn render_ruler_bar(
    scope: &mut UiScope<'_>,
    duration: f32,
    height: f32,
    track_width: f32,
    style: &TimelineRulerStyle,
) -> WidgetId {
    let ruler_style = Style::new()
        .height(height)
        .width(track_width)
        .clip_children(true)
        .background(Color::rgba(0.07, 0.08, 0.11, 0.60));

    scope.container_named("TimelineRulerRoot", ruler_style, |ruler| {
        let usable_width = (track_width - TIMELINE_SIDEBAR_WIDTH).max(10.0);
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

        // Left header indicator
        let header_style = Style::new()
            .position_absolute()
            .left(4.0)
            .top(2.0)
            .width(110.0)
            .height(height - 4.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0));
        ruler.container_named("TimelineRulerHeader", header_style, |hdr| {
            hdr.label(
                "⏱ TIME (s)",
                8.5,
                Color::rgba(0.55, 0.60, 0.72, 0.90),
                TextAlign::Left,
            );
        });

        let tick_count = ((duration / step).ceil() as usize).min(120);
        for i in 0..=tick_count {
            let t = (i as f32 * step).min(duration);
            let frac = (t / safe_duration).clamp(0.0, 1.0);
            let is_major = (i % 2 == 0) || (t == 0.0) || ((t - duration).abs() < 0.001);
            let tick_h = if is_major { 8.0 } else { 4.0 };
            let x = TIMELINE_SIDEBAR_WIDTH
                + (frac * usable_width).clamp(0.0, (usable_width - 1.0).max(0.0));

            let tick_style = Style::new()
                .position_absolute()
                .left(x)
                .top(height - tick_h)
                .width(1.0)
                .height(tick_h)
                .background(if is_major {
                    style.major_tick_color
                } else {
                    style.minor_tick_color
                });
            ruler.empty_box_passive_named("TimelineRulerTick", tick_style);

            if is_major {
                let label_style = Style::new()
                    .position_absolute()
                    .left(x + 2.0)
                    .top(0.0)
                    .width(32.0)
                    .height(12.0);
                ruler.label_styled_passive(
                    "TimelineRulerLabel",
                    format!("{:.1}s", t),
                    style.tick_label_font_size,
                    style.tick_label_color,
                    TextAlign::Left,
                    label_style,
                );
            }
        }
    })
}

/// Parameters describing dimensions, time bounds, keyframes, and style for scrubber track rendering.
pub struct ScrubberTrackParams<'a> {
    /// Total duration of the active timeline sequence in seconds.
    pub duration: f32,
    /// Current playhead time offset in seconds.
    pub current_time: f32,
    /// Whether the user is actively dragging the scrubber playhead.
    pub is_dragging: bool,
    /// Read-only slice of keyframe markers along the sequence.
    pub keyframes: &'a [TimelineKeyframeMarker],
    /// Scrubber track height in logical pixels.
    pub height: f32,
    /// Total track width in logical pixels.
    pub track_width: f32,
    /// Visual styling configuration descriptor for the track and playhead.
    pub style: &'a TimelineRulerStyle,
}

/// Emits an interactive timeline scrubber track containing progress fill, keyframe diamonds,
/// and the draggable playhead needle and cap handle.
///
/// Returns `(track_widget_id, playhead_cap_widget_id)`.
pub fn render_scrubber_track(
    scope: &mut UiScope<'_>,
    params: &ScrubberTrackParams<'_>,
) -> (WidgetId, WidgetId) {
    let style = params.style;
    let height = params.height;
    let keyframes = params.keyframes;
    let safe_duration = params.duration.max(0.001);
    let progress_ratio = (params.current_time / safe_duration).clamp(0.0, 1.0);
    let usable_width = (params.track_width - TIMELINE_SIDEBAR_WIDTH).max(10.0);

    let track_border = if params.is_dragging {
        params.style.track_border_active
    } else {
        params.style.track_border_idle
    };

    let track_style = Style::new()
        .height(params.height)
        .width(params.track_width)
        .background(params.style.track_bg)
        .border_radius(params.style.track_border_radius)
        .border(params.style.track_border_width, track_border)
        .clip_children(false);

    let mut playhead_cap_id = WidgetId::default();

    let track_id = scope.container_tagged(
        "TimelineScrubberTrack",
        track_style,
        WidgetRole::Default,
        TIMELINE_TAG_SCRUBBER_TRACK,
        |track| {
            // 0. Left master track pill badge
            let master_badge_style = Style::new()
                .position_absolute()
                .left(4.0)
                .top(3.0)
                .width(110.0)
                .height(18.0)
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(4.0)
                .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
                .background(Color::rgba(0.05, 0.06, 0.08, 0.85))
                .border(1.0, Color::rgba(0.22, 0.25, 0.34, 0.60))
                .border_radius(3.0);
            track.container_named("TimelineMasterTrackBadge", master_badge_style, |badge| {
                badge.label(
                    "✦ MASTER",
                    8.5,
                    style.playhead_needle_color,
                    TextAlign::Left,
                );
            });

            // 1. Progress fill
            let fill_w = progress_ratio * usable_width;
            let fill_style = Style::new()
                .position_absolute()
                .left(TIMELINE_SIDEBAR_WIDTH)
                .top(0.0)
                .width(fill_w)
                .height(height)
                .background(style.progress_fill_color)
                .border_radius(style.track_border_radius);
            track.empty_box_passive_named("TimelineProgressFill", fill_style);

            // 2. Keyframe markers
            for kf in keyframes {
                let kf_frac = (kf.time / safe_duration).clamp(0.0, 1.0);
                let kf_x = TIMELINE_SIDEBAR_WIDTH
                    + (kf_frac * usable_width - 6.0).clamp(0.0, (usable_width - 12.0).max(0.0));
                let kf_y = (height - 12.0) * 0.5;
                let kf_style = Style::new()
                    .position_absolute()
                    .left(kf_x)
                    .top(kf_y)
                    .width(12.0)
                    .height(12.0);
                track.label_styled_passive(
                    "TimelineKeyframeMarker",
                    "◆",
                    11.0,
                    kf.color.unwrap_or(style.keyframe_color),
                    TextAlign::Center,
                    kf_style,
                );
            }

            // 3. Playhead needle line
            let needle_x = TIMELINE_SIDEBAR_WIDTH
                + (progress_ratio * usable_width - 1.0).clamp(0.0, (usable_width - 2.0).max(0.0));
            let needle_style = Style::new()
                .position_absolute()
                .left(needle_x)
                .top(0.0)
                .width(2.0)
                .height(height)
                .background(style.playhead_needle_color);
            track.empty_box_passive_named("TimelinePlayheadNeedle", needle_style);

            // 4. Playhead handle cap (top indicator ▼)
            let cap_w = 12.0;
            let cap_h = 10.0;
            let cap_x = TIMELINE_SIDEBAR_WIDTH
                + (progress_ratio * usable_width - cap_w * 0.5)
                    .clamp(0.0, (usable_width - cap_w).max(0.0));
            let cap_style = Style::new()
                .position_absolute()
                .left(cap_x)
                .top(0.0)
                .width(cap_w)
                .height(cap_h)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);
            playhead_cap_id = track.container_tagged(
                "TimelinePlayheadCap",
                cap_style,
                WidgetRole::Button,
                TIMELINE_TAG_PLAYHEAD_CAP,
                |cap| {
                    cap.label("▼", 9.0, style.playhead_cap_color, TextAlign::Center);
                },
            );
        },
    );

    (track_id, playhead_cap_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruler_bar_declarative_build() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let style = TimelineRulerStyle::dark_default();

        let ruler_id = {
            let mut scope = UiScope::new(&mut tree, root);
            render_ruler_bar(&mut scope, 5.0, 18.0, 500.0, &style)
        };
        assert!(tree.get(ruler_id).is_some());
        assert!(!tree.get(ruler_id).unwrap().children.is_empty());
    }

    #[test]
    fn test_scrubber_track_declarative_build() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let style = TimelineRulerStyle::dark_default();
        let keyframes = [
            TimelineKeyframeMarker::new(0.5),
            TimelineKeyframeMarker::new(2.0),
        ];

        let (track_id, cap_id) = {
            let mut scope = UiScope::new(&mut tree, root);
            render_scrubber_track(
                &mut scope,
                &ScrubberTrackParams {
                    duration: 4.0,
                    current_time: 1.5,
                    is_dragging: false,
                    keyframes: &keyframes,
                    height: 36.0,
                    track_width: 600.0,
                    style: &style,
                },
            )
        };

        assert!(tree.get(track_id).is_some());
        assert!(tree.get(cap_id).is_some());
        assert_eq!(tree.get(track_id).unwrap().tag, TIMELINE_TAG_SCRUBBER_TRACK);
        assert_eq!(tree.get(cap_id).unwrap().tag, TIMELINE_TAG_PLAYHEAD_CAP);
        // Track should contain children (progress, keyframes, needle, cap)
        assert!(tree.get(track_id).unwrap().children.len() >= 4);
    }
}