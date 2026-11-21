// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Dope Sheet Lanes
//!
//! Renders animation channel rows using [`UiScope::track_lane_fmt`] with zero-heap
//! `StackString` labels, zero temporary keyframe vectors, and zero `let` statements.

use super::types::{TIMELINE_SIDEBAR_WIDTH, TIMELINE_TAG_LANES_PLAYHEAD};
use ae_animation::clip::{Channel, TargetProperty};
use irisui::prelude::*;
use std::fmt::Write;

/// Height of an individual animation channel track lane in logical pixels.
pub const CHANNEL_LANE_HEIGHT: f32 = 24.0;

/// Maximum number of channel track lanes rendered simultaneously.
pub const MAX_VISIBLE_LANES: usize = 16;

#[inline]
const fn channel_prop_meta(prop: TargetProperty) -> (&'static str, Color) {
    match prop {
        TargetProperty::Rotation => ("ROT", Color::rgba(0.0, 0.85, 1.0, 0.95)),
        TargetProperty::Translation => ("POS", Color::rgba(0.20, 0.88, 0.45, 0.95)),
        TargetProperty::Scale => ("SCL", Color::rgba(1.0, 0.65, 0.20, 0.95)),
    }
}

#[inline]
fn channel_marker_times(channel: &Channel) -> impl Iterator<Item = f32> + '_ {
    channel
        .vector_track
        .iter()
        .flat_map(|t| t.keyframes.iter().map(|k| k.time))
        .chain(
            channel
                .rotation_track
                .iter()
                .flat_map(|t| t.keyframes.iter().map(|k| k.time)),
        )
}

/// Builds the animation dope sheet channel lanes section below the time ruler and scrubber.
pub fn build_dope_sheet_lanes(
    scope: &mut UiScope<'_>,
    player: &ae_animation::AnimationPlayer,
    duration: f32,
) {
    scope.container_named(
        "TimelineDopeSheetSection",
        Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .padding_insets(Insets::new(2.0, 10.0, 6.0, 10.0))
            .gap(2.0),
        |section| {
            render_lanes_header(
                section,
                player.current_clip.as_ref().map_or(0, |c| c.channels.len()),
            );

            match player
                .current_clip
                .as_ref()
                .filter(|c| !c.channels.is_empty())
            {
                None => render_empty_lanes_placeholder(section),
                Some(clip) => {
                    section.track_lane_group(
                        TrackLaneGroupSpec::new(
                            TIMELINE_TAG_LANES_PLAYHEAD,
                            duration,
                            player.current_time,
                        )
                        .sidebar_width(TIMELINE_SIDEBAR_WIDTH),
                        |group| {
                            clip.channels
                                .iter()
                                .take(MAX_VISIBLE_LANES)
                                .enumerate()
                                .for_each(|(idx, channel)| {
                                    group.track_lane_fmt::<24, _, _>(
                                        TrackLaneSpec::new(
                                            channel_prop_meta(channel.target_property).0,
                                            channel_prop_meta(channel.target_property).1,
                                            "",
                                            duration,
                                            player.current_time.clamp(0.0, duration),
                                        )
                                        .row_index(idx)
                                        .height(CHANNEL_LANE_HEIGHT)
                                        .sidebar_width(TIMELINE_SIDEBAR_WIDTH)
                                        .show_playhead(false),
                                        channel_marker_times(channel),
                                        |buf| {
                                            write!(buf, "Joint #{}", channel.joint_index).ok();
                                        },
                                    );
                                });
                        },
                    );
                }
            }
        },
    );
}

/// Updates the shared vertical dope-sheet playhead overlay line in place during `InPlaceRefresh`.
pub fn refresh_lanes_playhead_live(
    scope: &mut UiScope<'_>,
    duration: f32,
    current_time: f32,
) -> bool {
    scope.update_track_lane_playhead(
        TIMELINE_TAG_LANES_PLAYHEAD,
        duration,
        current_time,
        TimeRulerStyle::dark_default(),
    )
}

fn render_lanes_header(scope: &mut UiScope<'_>, channel_count: usize) {
    scope.container_named(
        "TimelineLanesHeader",
        Style::new()
            .height(18.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
            .background(Color::rgba(0.07, 0.08, 0.11, 0.70))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.50))
            .border_radius(3.0),
        |header| {
            header.colored_label(Color::rgba(0.65, 0.70, 0.82, 1.0), "✦ Dope Sheet Channels");
            header.text_colored_fmt::<32, _>(Color::rgba(0.45, 0.50, 0.62, 1.0), |buf| {
                write!(buf, "{channel_count} Active Channels").ok();
            });
        },
    );
}

fn render_empty_lanes_placeholder(scope: &mut UiScope<'_>) {
    scope.container_named(
        "TimelineEmptyLanesPlaceholder",
        Style::new()
            .height(CHANNEL_LANE_HEIGHT * 3.0 + 4.0)
            .flex_col()
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .gap(3.0)
            .background(Color::rgba(0.06, 0.07, 0.10, 0.40))
            .border(1.0, Color::rgba(0.16, 0.18, 0.25, 0.35))
            .border_radius(3.0),
        |holder| {
            holder.colored_label(
                Color::rgba(0.48, 0.52, 0.64, 1.0),
                "✦ Dope Sheet — No Recorded Channels in Active Clip",
            );
            holder.colored_label(
                Color::rgba(0.35, 0.38, 0.48, 1.0),
                "Keyframe markers and joint tracks will be displayed here during animation playback.",
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::widgets::panel::PanelBuilder;

    #[test]
    fn test_dope_sheet_lanes_empty_declarative_build() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let player = ae_animation::AnimationPlayer::default();
        {
            let mut scope = UiScope::new(&mut tree, root);
            build_dope_sheet_lanes(&mut scope, &player, 5.0);
        }

        assert!(tree.node_named("TimelineEmptyLanesPlaceholder").is_some());
    }

    #[test]
    fn test_dope_sheet_lanes_with_channels_and_sidebar_clearance() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let mut clip = ae_animation::AnimationClip::new("TestClip", 3.0);
        for i in 0..20 {
            clip.channels.push(ae_animation::clip::Channel {
                joint_index: i,
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
            });
        }

        let player = ae_animation::AnimationPlayer {
            current_clip: Some(clip),
            ..Default::default()
        };

        {
            let mut scope = UiScope::new(&mut tree, root);
            build_dope_sheet_lanes(&mut scope, &player, 3.0);
            scope.finish_layout(Rect::new(0.0, 0.0, 800.0, 500.0));
        }

        let lane_count = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("TrackLaneRow"))
            .count();
        assert_eq!(lane_count, MAX_VISIBLE_LANES);

        let canvas = tree
            .node_named("TrackLaneCanvas")
            .expect("TrackLaneCanvas must be emitted");
        assert!(canvas.computed_rect.x >= TIMELINE_SIDEBAR_WIDTH);
    }
}