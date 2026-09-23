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
    let mut targets = TimelinePanelTargets::default();
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

    build_timeline_panel(&mut tree, root, &params, &mut targets, &mut actions);

    assert_eq!(targets.panel_rect, params.panel_rect);
    assert!(targets.scrubber_track_rect.is_none());
    assert!(actions.is_empty());
}

#[test]
fn test_timeline_panel_build_missing_player() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut targets = TimelinePanelTargets::default();
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

    build_timeline_panel(&mut tree, root, &params, &mut targets, &mut actions);

    assert_eq!(targets.panel_rect, params.panel_rect);
    assert!(targets.scrubber_track_rect.is_none());
    assert!(actions.is_empty());
}

#[test]
fn test_timeline_panel_build_missing_player_declarative_click() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);
    let mut targets = TimelinePanelTargets::default();
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
    build_timeline_panel(&mut tree, root, &params_empty, &mut targets, &mut actions);
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
    let mut targets_click = TimelinePanelTargets::default();
    let params_click = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: Some(dummy_entity),
        animation_player: None,
        cursor_pos: Point::new(400.0, 480.0),
        is_dragging_scrubber: false,
        events: &click_events,
        hovered_tag: Some(button_tag),
    };

    build_timeline_panel(
        &mut tree_click,
        root_click,
        &params_click,
        &mut targets_click,
        &mut actions,
    );
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], TimelineAction::AddAnimationPlayer(dummy_entity));
}

#[test]
fn test_timeline_panel_build_with_player() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut targets = TimelinePanelTargets::default();
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

    build_timeline_panel(&mut tree, root, &params, &mut targets, &mut actions);

    assert_eq!(targets.panel_rect, params.panel_rect);
    assert!(targets.scrubber_track_rect.is_some());
    assert!(targets.playhead_needle_rect.is_some());
    assert!((targets.clip_duration - 3.5).abs() < 1e-4);
}

#[test]
fn test_timeline_scrubber_projection_math() {
    let mut targets = TimelinePanelTargets::default();
    let track_rect = Rect::new(100.0, 450.0, 600.0, 36.0);
    targets.scrubber_track_rect = Some(track_rect);
    targets.clip_duration = 5.0;

    // Click at middle of track (x = 400.0) -> should be 50% = 2.5s
    let click_pos = Point::new(400.0, 460.0);
    let res = handle_timeline_click(
        &targets,
        irisui::prelude::TIMELINE_TAG_SCRUBBER_TRACK,
        click_pos,
        None,
    );
    assert!(res.is_some());
    let (action, dragging) = res.unwrap();
    assert!(dragging);
    match action {
        TimelineAction::ScrubTo(time) => {
            assert!((time - 2.5).abs() < 1e-3);
        }
        _ => panic!("Expected ScrubTo action"),
    }

    // Drag to 75% of track (x = 550.0) -> should be 3.75s
    let drag_pos = Point::new(550.0, 460.0);
    let drag_res = handle_timeline_drag(&targets, drag_pos);
    assert!(drag_res.is_some());
    match drag_res.unwrap() {
        TimelineAction::ScrubTo(time) => {
            assert!((time - 3.75).abs() < 1e-3);
        }
        _ => panic!("Expected ScrubTo action"),
    }
}

#[test]
fn test_timeline_click_hit_testing() {
    let targets = TimelinePanelTargets::default();

    // Click Play button
    let (act_play, drag_play) = handle_timeline_click(
        &targets,
        irisui::prelude::TIMELINE_TAG_PLAY_PAUSE,
        Point::ZERO,
        None,
    )
    .unwrap();
    assert_eq!(act_play, TimelineAction::TogglePlayPause);
    assert!(!drag_play);

    // Click Stop button
    let (act_stop, drag_stop) = handle_timeline_click(
        &targets,
        irisui::prelude::TIMELINE_TAG_STOP,
        Point::ZERO,
        None,
    )
    .unwrap();
    assert_eq!(act_stop, TimelineAction::Stop);
    assert!(!drag_stop);

    // Click Loop toggle
    let (act_loop, drag_loop) = handle_timeline_click(
        &targets,
        irisui::prelude::TIMELINE_TAG_LOOP,
        Point::ZERO,
        None,
    )
    .unwrap();
    assert_eq!(act_loop, TimelineAction::ToggleLoop);
    assert!(!drag_loop);

    // Click Speed 2x button (index 3)
    let (act_spd, drag_spd) = handle_timeline_click(
        &targets,
        irisui::prelude::TIMELINE_TAG_SPEED_BASE + 3,
        Point::ZERO,
        None,
    )
    .unwrap();
    assert_eq!(act_spd, TimelineAction::SetSpeed(2.0));
    assert!(!drag_spd);
}