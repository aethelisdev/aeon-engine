// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Animation Timeline panel unit tests.

use super::*;
use irisui::prelude::{Point, Rect, UiScope, UiTree};
use irisui::widgets::panel::PanelBuilder;

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
    };

    let duration = build_timeline_panel(&mut UiScope::new(&mut tree, root), &params, &mut actions);

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
    };

    let duration = build_timeline_panel(&mut UiScope::new(&mut tree, root), &params, &mut actions);

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
    };
    build_timeline_panel(
        &mut UiScope::new(&mut tree, root),
        &params_empty,
        &mut actions,
    );
    assert!(actions.is_empty());

    // 2. Find button tag dynamically from the tree
    let button_tag = tree
        .iter()
        .find(|(_, n)| n.text.as_deref() == Some("➕ Add AnimationPlayer"))
        .map(|(_, n)| n.tag)
        .expect("Add button must exist");

    let mut tree_click = UiTree::new();
    tree_click.event_state.frame_clicked_tag = Some(button_tag);
    let root_click = PanelBuilder::new(&mut tree_click).build();
    let _ = tree_click.set_root(root_click);
    let params_click = TimelinePanelParams {
        panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
        entity: Some(dummy_entity),
        animation_player: None,
        cursor_pos: Point::new(400.0, 480.0),
        is_dragging_scrubber: false,
        events: &[],
    };

    build_timeline_panel(
        &mut UiScope::new(&mut tree_click, root_click),
        &params_click,
        &mut actions,
    );
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
    };

    let duration = build_timeline_panel(&mut UiScope::new(&mut tree, root), &params, &mut actions);

    assert!((duration - 3.5).abs() < 1e-4);
    assert!(actions.is_empty());
}