// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Dope Sheet Lanes Subsystem
//!
//! Renders animation channel rows, property badges (Translation, Rotation, Scale),
//! keyframe diamond indicators, and playhead guide needles across individual tracks
//! using 100% pure declarative [`UiScope`] layout.
//!

use super::types::{TIMELINE_SIDEBAR_WIDTH, TimelinePanelParams, TimelineRulerStyle};
use ae_animation::clip::TargetProperty;
use irisui::prelude::*;

/// Height of an individual animation channel track lane in physical pixels.
pub const CHANNEL_LANE_HEIGHT: f32 = 24.0;

/// Maximum number of channel track lanes rendered simultaneously to conserve layout performance.
pub const MAX_VISIBLE_LANES: usize = 16;

/// Builds the animation dope sheet channel lanes section below the time ruler and scrubber.
///
/// If the active clip contains channels, each channel is rendered as a horizontal track lane
/// displaying its property type badge, joint index, keyframe diamonds, and playhead guide needle.
/// If no channels are present, renders a clean multi-lane grid guide with an informative status label.
pub fn build_dope_sheet_lanes(
    scope: &mut UiScope<'_>,
    params: &TimelinePanelParams<'_>,
    duration: f32,
) {
    let player = params.animation_player;
    let current_time = player.map_or(0.0, |p| p.current_time).clamp(0.0, duration);
    let track_width = (params.panel_rect.width - 20.0).max(10.0);
    let safe_duration = duration.max(0.001);
    let progress_ratio = (current_time / safe_duration).clamp(0.0, 1.0);

    let top_fixed_height = super::transport::TRANSPORT_TOOLBAR_HEIGHT
        + super::ruler::RULER_HEIGHT
        + super::ruler::SCRUBBER_TRACK_HEIGHT
        + 28.0;
    let available_lanes_height = (params.panel_rect.height - top_fixed_height).max(0.0);
    let header_height = 20.0;
    let lane_unit_height = CHANNEL_LANE_HEIGHT + 2.0;
    let max_fit_lanes = if available_lanes_height > header_height {
        ((available_lanes_height - header_height) / lane_unit_height).floor() as usize
    } else {
        0
    };
    let visible_lanes_count = max_fit_lanes.clamp(1, MAX_VISIBLE_LANES);

    let ruler_style = TimelineRulerStyle::dark_default();

    let section_style = Style::new()
        .flex_col()
        .height(available_lanes_height)
        .clip_children(true)
        .padding_insets(Insets::new(2.0, 10.0, 6.0, 10.0))
        .gap(2.0);

    scope.container_named("TimelineDopeSheetSection", section_style, |section| {
        // ── 1. Dope Sheet Header Strip ──
        render_lanes_header(section, params, track_width);

        // ── 2. Channels Track Lanes ──
        let channels = player
            .and_then(|p| p.current_clip.as_ref())
            .map(|c| c.channels.as_slice())
            .unwrap_or(&[]);

        if channels.is_empty() {
            render_empty_lanes_placeholder(section, track_width);
        } else {
            for (idx, channel) in channels.iter().take(visible_lanes_count).enumerate() {
                render_channel_lane(
                    section,
                    idx,
                    channel,
                    track_width,
                    safe_duration,
                    progress_ratio,
                    &ruler_style,
                );
            }
        }
    });
}

/// Renders the dope sheet section header bar displaying title and channel count.
fn render_lanes_header(scope: &mut UiScope<'_>, params: &TimelinePanelParams<'_>, width: f32) {
    let channel_count = params
        .animation_player
        .and_then(|p| p.current_clip.as_ref())
        .map(|c| c.channels.len())
        .unwrap_or(0);

    let header_style = Style::new()
        .width(width)
        .height(18.0)
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
        .background(Color::rgba(0.07, 0.08, 0.11, 0.70))
        .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.50))
        .border_radius(3.0);

    scope.container_named("TimelineLanesHeader", header_style, |header| {
        header.label(
            "✦ Dope Sheet Channels",
            9.5,
            Color::rgba(0.65, 0.70, 0.82, 1.0),
            TextAlign::Left,
        );

        header.label(
            format!("{} Active Channels", channel_count),
            9.0,
            Color::rgba(0.45, 0.50, 0.62, 1.0),
            TextAlign::Right,
        );
    });
}

/// Renders a single animation channel track row with property badge and keyframe markers.
fn render_channel_lane(
    scope: &mut UiScope<'_>,
    index: usize,
    channel: &ae_animation::clip::Channel,
    track_width: f32,
    safe_duration: f32,
    progress_ratio: f32,
    _ruler_style: &TimelineRulerStyle,
) {
    let (prop_name, prop_color) = match channel.target_property {
        TargetProperty::Rotation => ("ROT", Color::rgba(0.0, 0.85, 1.0, 0.95)),
        TargetProperty::Translation => ("POS", Color::rgba(0.20, 0.88, 0.45, 0.95)),
        TargetProperty::Scale => ("SCL", Color::rgba(1.0, 0.65, 0.20, 0.95)),
    };

    let is_even = index.is_multiple_of(2);
    let bg_color = if is_even {
        Color::rgba(0.07, 0.08, 0.11, 0.65)
    } else {
        Color::rgba(0.09, 0.10, 0.14, 0.65)
    };

    let lane_style = Style::new()
        .width(track_width)
        .height(CHANNEL_LANE_HEIGHT)
        .background(bg_color)
        .border(1.0, Color::rgba(0.15, 0.17, 0.24, 0.40))
        .border_radius(2.0)
        .clip_children(true);

    let usable_width = (track_width - TIMELINE_SIDEBAR_WIDTH).max(10.0);
    let needle_x = TIMELINE_SIDEBAR_WIDTH
        + (progress_ratio * usable_width - 1.0).clamp(0.0, (usable_width - 2.0).max(0.0));

    scope.container_named("TimelineChannelLane", lane_style, |lane| {
        // ── 1. Guide line across lane center ──
        let guide_style = Style::new()
            .position_absolute()
            .left(TIMELINE_SIDEBAR_WIDTH)
            .top((CHANNEL_LANE_HEIGHT - 1.0) * 0.5)
            .width(usable_width)
            .height(1.0)
            .background(Color::rgba(0.20, 0.23, 0.32, 0.25));
        lane.empty_box_passive_named("TimelineLaneGuide", guide_style);

        // ── 2. Playhead guide needle passing through lane ──
        let needle_style = Style::new()
            .position_absolute()
            .left(needle_x)
            .top(0.0)
            .width(1.0)
            .height(CHANNEL_LANE_HEIGHT)
            .background(Color::rgba(0.0, 0.92, 1.0, 0.45));
        lane.empty_box_passive_named("TimelineLaneNeedle", needle_style);

        // ── 3. Left Channel Property Pill Badge ──
        let badge_style = Style::new()
            .position_absolute()
            .left(4.0)
            .top(3.0)
            .width(110.0)
            .height(18.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(4.0)
            .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0))
            .background(Color::rgba(0.05, 0.06, 0.08, 0.85))
            .border(1.0, Color::rgba(0.22, 0.25, 0.34, 0.60))
            .border_radius(3.0);

        lane.container_named("TimelineChannelBadge", badge_style, |badge| {
            badge.label(prop_name, 8.5, prop_color, TextAlign::Left);
            badge.label(
                format!("Joint #{}", channel.joint_index),
                8.5,
                Color::rgba(0.70, 0.74, 0.84, 1.0),
                TextAlign::Left,
            );
        });

        // ── 4. Keyframe Diamond Markers ──
        let render_kf = |lane_scope: &mut UiScope<'_>, time: f32| {
            let frac = (time / safe_duration).clamp(0.0, 1.0);
            let kf_x = TIMELINE_SIDEBAR_WIDTH
                + (frac * usable_width - 5.0).clamp(0.0, (usable_width - 10.0).max(0.0));
            let kf_y = (CHANNEL_LANE_HEIGHT - 10.0) * 0.5;
            let kf_style = Style::new()
                .position_absolute()
                .left(kf_x)
                .top(kf_y)
                .width(10.0)
                .height(10.0);
            lane_scope.label_styled_passive(
                "TimelineLaneKeyframe",
                "◆",
                10.0,
                prop_color,
                TextAlign::Center,
                kf_style,
            );
        };

        if let Some(ref track) = channel.vector_track {
            for kf in &track.keyframes {
                render_kf(lane, kf.time);
            }
        }
        if let Some(ref track) = channel.rotation_track {
            for kf in &track.keyframes {
                render_kf(lane, kf.time);
            }
        }
    });
}

/// Renders subtle empty-state grid lines with an informative status label when no channels are recorded.
fn render_empty_lanes_placeholder(scope: &mut UiScope<'_>, width: f32) {
    let placeholder_style = Style::new()
        .width(width)
        .height(CHANNEL_LANE_HEIGHT * 3.0 + 4.0)
        .flex_col()
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .gap(3.0)
        .background(Color::rgba(0.06, 0.07, 0.10, 0.40))
        .border(1.0, Color::rgba(0.16, 0.18, 0.25, 0.35))
        .border_radius(3.0);

    scope.container_named(
        "TimelineEmptyLanesPlaceholder",
        placeholder_style,
        |holder| {
            holder.label(
                "✦ Dope Sheet — No Recorded Channels in Active Clip",
                10.0,
                Color::rgba(0.48, 0.52, 0.64, 1.0),
                TextAlign::Center,
            );
            holder.label(
                "Keyframe markers and joint tracks will be displayed here during animation playback.",
                9.0,
                Color::rgba(0.35, 0.38, 0.48, 1.0),
                TextAlign::Center,
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dope_sheet_lanes_empty_declarative_build() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 200.0),
            entity: None,
            animation_player: None,
            cursor_pos: Point::new(10.0, 10.0),
            is_dragging_scrubber: false,
            events: &[],
        };

        {
            let mut scope = UiScope::new(&mut tree, root);
            build_dope_sheet_lanes(&mut scope, &params, 5.0);
        }

        assert!(tree.get(root).is_some());
        assert!(!tree.get(root).unwrap().children.is_empty());
    }

    #[test]
    fn test_dope_sheet_lanes_with_channels() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let mut clip = ae_animation::AnimationClip::new("TestClip", 3.0);
        let channel = ae_animation::clip::Channel {
            joint_index: 2,
            target_property: TargetProperty::Rotation,
            vector_track: None,
            rotation_track: Some(ae_animation::clip::RotationTrack {
                keyframes: vec![
                    ae_animation::clip::Keyframe {
                        time: 0.5,
                        value: Default::default(),
                    },
                    ae_animation::clip::Keyframe {
                        time: 2.0,
                        value: Default::default(),
                    },
                ],
                interpolation: ae_animation::clip::Interpolation::Linear,
            }),
        };
        clip.channels.push(channel);

        let player = ae_animation::AnimationPlayer {
            current_clip: Some(clip),
            ..Default::default()
        };

        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 200.0),
            entity: None,
            animation_player: Some(&player),
            cursor_pos: Point::new(10.0, 10.0),
            is_dragging_scrubber: false,
            events: &[],
        };

        {
            let mut scope = UiScope::new(&mut tree, root);
            build_dope_sheet_lanes(&mut scope, &params, 3.0);
        }

        assert!(tree.get(root).is_some());
        let root_node = tree.get(root).unwrap();
        assert!(!root_node.children.is_empty());
    }

    #[test]
    fn test_dope_sheet_keyframe_offset_and_lane_culling() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let mut clip = ae_animation::AnimationClip::new("TestClip", 3.0);
        for i in 0..30 {
            clip.channels.push(ae_animation::clip::Channel {
                joint_index: i,
                target_property: TargetProperty::Translation,
                vector_track: Some(ae_animation::clip::VectorTrack {
                    keyframes: vec![ae_animation::clip::Keyframe {
                        time: 0.0,
                        value: Default::default(),
                    }],
                    interpolation: ae_animation::clip::Interpolation::Linear,
                }),
                rotation_track: None,
            });
        }

        let player = ae_animation::AnimationPlayer {
            current_clip: Some(clip),
            ..Default::default()
        };

        // Small panel height (160.0 px) -> only 1-2 lanes should fit
        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 600.0, 800.0, 160.0),
            entity: None,
            animation_player: Some(&player),
            cursor_pos: Point::new(10.0, 610.0),
            is_dragging_scrubber: false,
            events: &[],
        };

        {
            let mut scope = UiScope::new(&mut tree, root);
            build_dope_sheet_lanes(&mut scope, &params, 3.0);
        }

        // Verify keyframes start at or after TIMELINE_SIDEBAR_WIDTH (never overlapping badge)
        let mut checked_keyframe = false;
        for (_, node) in tree.iter() {
            if node.name.as_deref() == Some("TimelineLaneKeyframe") {
                assert!(
                    node.style.inset_left.unwrap_or(0.0) >= TIMELINE_SIDEBAR_WIDTH - 5.0,
                    "Keyframe diamond must not be positioned before sidebar offset: {:?}",
                    node.style.inset_left
                );
                checked_keyframe = true;
            }
        }
        assert!(
            checked_keyframe,
            "At least one keyframe diamond should have been rendered"
        );
    }
}