// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Animation Timeline panel unit tests.
//!

use super::*;
use irisui::prelude::{PanelBuilder, Point, Rect, UiTree, hash_label};

#[test]
fn test_timeline_panel_build_empty_state() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut actions = Vec::new();

    let params = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: None,
        animation_player: None,
        cursor_pos: Point::new(100.0, 420.0),
        is_dragging_scrubber: false,
        events: &[],
        hovered_tag: None,
    };

    let duration = build_timeline_panel(&mut tree, root, &params, &mut actions);

    assert_eq!(duration, 0.0);
    assert!(actions.is_empty());
}

#[test]
fn test_timeline_panel_build_missing_player() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut actions = Vec::new();

    let dummy_entity = hecs::World::new().spawn(());

    let params = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: Some(dummy_entity),
        animation_player: None,
        cursor_pos: Point::new(100.0, 420.0),
        is_dragging_scrubber: false,
        events: &[],
        hovered_tag: None,
    };

    let duration = build_timeline_panel(&mut tree, root, &params, &mut actions);

    assert_eq!(duration, 0.0);
    assert!(actions.is_empty());
}

#[test]
fn test_timeline_panel_build_missing_player_declarative_click() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);
    let mut actions = Vec::new();

    let dummy_entity = hecs::World::new().spawn(());

    // 1. Initial build to populate widget tree
    let params_empty = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: Some(dummy_entity),
        animation_player: None,
        cursor_pos: Point::new(100.0, 420.0),
        is_dragging_scrubber: false,
        events: &[],
        hovered_tag: None,
    };
    build_timeline_panel(&mut tree, root, &params_empty, &mut actions);
    assert!(actions.is_empty());

    // 2. Compute persistent tag for the declarative button inside card
    let button_tag = hash_label("➕ Add AnimationPlayer");

    // 3. Dispatch click event to tagged button
    let click_events = [(
        button_tag,
        irisui::prelude::InteractionEvent::Click {
            button: irisui::prelude::MouseButton::Left,
        },
    )];

    let mut tree_click = UiTree::new();
    let root_click = PanelBuilder::new(&mut tree_click).build();
    let _ = tree_click.set_root(root_click);
    let params_click = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: Some(dummy_entity),
        animation_player: None,
        cursor_pos: Point::new(400.0, 480.0),
        is_dragging_scrubber: false,
        events: &click_events,
        hovered_tag: Some(button_tag),
    };

    build_timeline_panel(&mut tree_click, root_click, &params_click, &mut actions);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], TimelineAction::AddAnimationPlayer(dummy_entity));
}

#[test]
fn test_timeline_panel_build_with_player() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut actions = Vec::new();

    let mut player = ae_animation::AnimationPlayer::new();
    player.current_clip = Some(ae_animation::AnimationClip {
        name: "TestClip".to_string(),
        duration: 3.5,
        channels: Vec::new(),
    });
    player.current_time = 1.2;

    let dummy_entity = hecs::World::new().spawn(());

    let params = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: Some(dummy_entity),
        animation_player: Some(&player),
        cursor_pos: Point::new(100.0, 420.0),
        is_dragging_scrubber: false,
        events: &[],
        hovered_tag: None,
    };

    let duration = build_timeline_panel(&mut tree, root, &params, &mut actions);

    assert!((duration - 3.5).abs() < 1e-4);
    assert!(actions.is_empty());
}

#[test]
fn test_timeline_scrubber_projection_math() {
    let track_x = 100.0;
    let track_width = 600.0;
    let clip_duration = 5.0;

    // Click at middle of track (x = 400.0) -> should be 50% = 2.5s
    let scrub_mid = compute_scrub_timestamp(400.0, track_x, track_width, clip_duration);
    assert!((scrub_mid - 2.5).abs() < 1e-3);

    // Click at 75% of track (x = 550.0) -> should be 3.75s
    let scrub_three_quarters = compute_scrub_timestamp(550.0, track_x, track_width, clip_duration);
    assert!((scrub_three_quarters - 3.75).abs() < 1e-3);

    // Click before track start -> clamp to 0.0s
    let scrub_before = compute_scrub_timestamp(50.0, track_x, track_width, clip_duration);
    assert_eq!(scrub_before, 0.0);

    // Click past track end -> clamp to duration
    let scrub_past = compute_scrub_timestamp(800.0, track_x, track_width, clip_duration);
    assert_eq!(scrub_past, 5.0);
}

#[test]
fn test_timeline_click_hit_testing() {
    // Click Play button
    let act_play = handle_timeline_click(irisui::prelude::TIMELINE_TAG_PLAY_PAUSE, None).unwrap();
    assert_eq!(act_play, TimelineAction::TogglePlayPause);

    // Click Stop button
    let act_stop = handle_timeline_click(irisui::prelude::TIMELINE_TAG_STOP, None).unwrap();
    assert_eq!(act_stop, TimelineAction::Stop);

    // Click Loop toggle
    let act_loop = handle_timeline_click(irisui::prelude::TIMELINE_TAG_LOOP, None).unwrap();
    assert_eq!(act_loop, TimelineAction::ToggleLoop);

    // Click Speed 2x button (index 3)
    let act_spd =
        handle_timeline_click(irisui::prelude::TIMELINE_TAG_SPEED_BASE + 3, None).unwrap();
    assert_eq!(act_spd, TimelineAction::SetSpeed(2.0));
}