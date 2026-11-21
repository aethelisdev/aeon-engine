// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Animation Timeline panel unit tests.

use super::*;
use crate::ui::types::EngineUiAction;
use irisui::prelude::{UiScope, UiTree};
use irisui::widgets::panel::PanelBuilder;
use irisui::workbench::ActionBus;

#[test]
fn test_timeline_panel_build_empty_state() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut cache = TimelineClipCache::default();
    let mut bus = ActionBus::<EngineUiAction>::new();

    let params = TimelinePanelParams {
        entity: None,
        animation_player: None,
    };

    let duration = build_timeline_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut cache,
        &mut bus,
    );

    assert_eq!(duration, 0.0);
    assert!(bus.is_empty());
}

#[test]
fn test_timeline_panel_build_missing_player() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut cache = TimelineClipCache::default();
    let mut bus = ActionBus::<EngineUiAction>::new();

    let dummy_entity = hecs::World::new().spawn(());

    let params = TimelinePanelParams {
        entity: Some(dummy_entity),
        animation_player: None,
    };

    let duration = build_timeline_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut cache,
        &mut bus,
    );

    assert_eq!(duration, 0.0);
    assert!(bus.is_empty());
}

#[test]
fn test_timeline_panel_build_missing_player_declarative_click() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);
    let mut cache = TimelineClipCache::default();
    let mut bus = ActionBus::<EngineUiAction>::new();

    let dummy_entity = hecs::World::new().spawn(());
    let params = TimelinePanelParams {
        entity: Some(dummy_entity),
        animation_player: None,
    };

    build_timeline_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut cache,
        &mut bus,
    );
    assert!(bus.is_empty());

    let button_tag = tree
        .iter()
        .find(|(_, n)| n.text.as_deref() == Some("➕ Add AnimationPlayer"))
        .map(|(_, n)| n.tag)
        .expect("Add button must exist");

    let mut tree_click = UiTree::new();
    tree_click.event_state.frame_clicked_tag = Some(button_tag);
    let root_click = PanelBuilder::new(&mut tree_click).build();
    let _ = tree_click.set_root(root_click);

    build_timeline_panel(
        &mut UiScope::new(&mut tree_click, root_click),
        &params,
        &mut cache,
        &mut bus,
    );

    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions[0],
        EngineUiAction::AddComponent(ent, "AnimationPlayer") if ent == dummy_entity
    ));
}

#[test]
fn test_timeline_panel_build_with_player_and_transport_click() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);
    let mut cache = TimelineClipCache::default();
    let mut bus = ActionBus::<EngineUiAction>::new();

    let mut player = ae_animation::AnimationPlayer::new();
    player.current_clip = Some(ae_animation::AnimationClip {
        name: "TestClip".to_string(),
        duration: 3.5,
        channels: Vec::new(),
    });
    player.current_time = 1.2;

    let dummy_entity = hecs::World::new().spawn(());
    let params = TimelinePanelParams {
        entity: Some(dummy_entity),
        animation_player: Some(&player),
    };

    let duration = build_timeline_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut cache,
        &mut bus,
    );
    assert!((duration - 3.5).abs() < 1e-4);
    assert!(bus.is_empty());
    assert!(tree.find_node_by_tag(TIMELINE_TAG_PLAY_PAUSE).is_some());

    let mut tree_click = UiTree::new();
    tree_click.event_state.frame_clicked_tag = Some(TIMELINE_TAG_PLAY_PAUSE);
    let root_click = PanelBuilder::new(&mut tree_click).build();
    let _ = tree_click.set_root(root_click);

    build_timeline_panel(
        &mut UiScope::new(&mut tree_click, root_click),
        &params,
        &mut cache,
        &mut bus,
    );

    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        EngineUiAction::ModifyComponent(ent, "AnimationPlayer", _) if *ent == dummy_entity
    ));
}

#[test]
fn test_timeline_empty_and_missing_cards_are_horizontally_centered_within_panel() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);
    let mut cache = TimelineClipCache::default();
    let mut bus = ActionBus::<EngineUiAction>::new();

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .panel("TestViewport")
            .bounds(irisui::prelude::Rect::new(0.0, 0.0, 900.0, 220.0))
            .show(|panel_scope| {
                build_timeline_panel(
                    panel_scope,
                    &TimelinePanelParams {
                        entity: None,
                        animation_player: None,
                    },
                    &mut cache,
                    &mut bus,
                );
            });
        scope.finish_layout(irisui::prelude::Rect::new(0.0, 0.0, 900.0, 220.0));
    }

    let empty_card = tree
        .node_named("TimelineEmptyCard")
        .expect("TimelineEmptyCard must exist");
    assert!(
        empty_card.computed_rect.x > 120.0,
        "TimelineEmptyCard must be horizontally centered, got x={}",
        empty_card.computed_rect.x
    );
    assert!(empty_card.computed_rect.right() < 780.0);

    let msg_node = tree
        .node_named("TimelineEmptyMessage")
        .expect("TimelineEmptyMessage must exist");
    assert!(
        msg_node.computed_rect.x >= empty_card.computed_rect.x,
        "Empty message text must stay within card bounds (msg.x={}, card.x={})",
        msg_node.computed_rect.x,
        empty_card.computed_rect.x
    );

    let mut tree_missing = UiTree::new();
    let root_missing = PanelBuilder::new(&mut tree_missing).build();
    let _ = tree_missing.set_root(root_missing);
    let dummy_entity = hecs::World::new().spawn(());

    {
        let mut scope = UiScope::new(&mut tree_missing, root_missing);
        scope
            .panel("TestViewportMissing")
            .bounds(irisui::prelude::Rect::new(0.0, 0.0, 900.0, 220.0))
            .show(|panel_scope| {
                build_timeline_panel(
                    panel_scope,
                    &TimelinePanelParams {
                        entity: Some(dummy_entity),
                        animation_player: None,
                    },
                    &mut cache,
                    &mut bus,
                );
            });
        scope.finish_layout(irisui::prelude::Rect::new(0.0, 0.0, 900.0, 220.0));
    }

    let missing_card = tree_missing
        .node_named("TimelineMissingPlayerCard")
        .expect("TimelineMissingPlayerCard must exist");
    assert!(
        missing_card.computed_rect.x > 120.0,
        "TimelineMissingPlayerCard must be horizontally centered, got x={}",
        missing_card.computed_rect.x
    );
    assert!(missing_card.computed_rect.right() < 780.0);
}

#[test]
fn test_timeline_live_playback_updates_in_place_without_tree_rebuild() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);
    let mut state = TimelinePanelState::default();
    let mut bus = ActionBus::<EngineUiAction>::new();

    let mut world = hecs::World::new();
    let mut clip = ae_animation::AnimationClip::new("Idle", 4.0);
    clip.channels.push(ae_animation::clip::Channel {
        joint_index: 1,
        target_property: ae_animation::clip::TargetProperty::Rotation,
        vector_track: None,
        rotation_track: Some(ae_animation::clip::RotationTrack {
            keyframes: vec![ae_animation::clip::Keyframe {
                time: 1.0,
                value: Default::default(),
            }],
            interpolation: ae_animation::clip::Interpolation::Linear,
        }),
    });
    let entity = world.spawn((ae_animation::AnimationPlayer {
        current_clip: Some(clip),
        current_time: 0.5,
        state: ae_animation::AnimationState::Playing,
        ..Default::default()
    },));

    // Initial full build
    {
        let mut scope = UiScope::new(&mut tree, root);
        let player_ref = world.get::<&ae_animation::AnimationPlayer>(entity).unwrap();
        let dur = build_timeline_panel(
            &mut scope,
            &TimelinePanelParams {
                entity: Some(entity),
                animation_player: Some(&player_ref),
            },
            &mut state.cache,
            &mut bus,
        );
        state.clip_duration = dur;
        scope.record_retained_revision(
            TIMELINE_TAG_PANEL_ROOT,
            types::timeline_structural_hash(Some(entity), Some(&player_ref)),
            types::timeline_live_hash(Some(&player_ref)),
        );
        scope.finish_layout(irisui::prelude::Rect::new(0.0, 0.0, 900.0, 300.0));
    }

    assert!(tree.is_live_tag(TIMELINE_TAG_TIME_READOUT_TEXT));
    assert!(tree.is_live_tag(TIMELINE_TAG_SCRUBBER_PLAYHEAD));
    assert!(tree.is_live_tag(TIMELINE_TAG_LANES_PLAYHEAD));

    // Unchanged time -> Unchanged (zero work)
    {
        let player_ref = world.get::<&ae_animation::AnimationPlayer>(entity).unwrap();
        let mut scope = UiScope::new(&mut tree, root);
        assert_eq!(
            scope.sync_retained_revision(
                TIMELINE_TAG_PANEL_ROOT,
                types::timeline_structural_hash(Some(entity), Some(&player_ref)),
                types::timeline_live_hash(Some(&player_ref)),
            ),
            irisui::prelude::RetainedRevisionStatus::Unchanged
        );
    }

    // Advance playback time -> updates readout + scrubber + lanes in place
    world
        .get::<&mut ae_animation::AnimationPlayer>(entity)
        .unwrap()
        .current_time = 2.0;
    {
        let player_ref = world.get::<&ae_animation::AnimationPlayer>(entity).unwrap();
        let mut scope = UiScope::new(&mut tree, root);
        let status = scope.sync_retained_revision(
            TIMELINE_TAG_PANEL_ROOT,
            types::timeline_structural_hash(Some(entity), Some(&player_ref)),
            types::timeline_live_hash(Some(&player_ref)),
        );
        assert_eq!(status, irisui::prelude::RetainedRevisionStatus::RefreshLive);
        assert!(transport::update_time_readout_live(
            &mut scope, 2.0, 4.0, 30.0
        ));
        assert!(ruler::refresh_scrubber_live(&mut scope, 4.0, 2.0));
        assert!(lanes::refresh_lanes_playhead_live(&mut scope, 4.0, 2.0));
    }

    assert_eq!(
        tree.node_tagged(TIMELINE_TAG_TIME_READOUT_TEXT)
            .and_then(|n| n.text.as_deref()),
        Some("2.00s / 4.00s  (F 60)")
    );

    // State transition (Playing -> Paused) triggers Rebuild
    world
        .get::<&mut ae_animation::AnimationPlayer>(entity)
        .unwrap()
        .state = ae_animation::AnimationState::Paused;
    let player_ref = world.get::<&ae_animation::AnimationPlayer>(entity).unwrap();
    let mut scope = UiScope::new(&mut tree, root);
    assert_eq!(
        scope.sync_retained_revision(
            TIMELINE_TAG_PANEL_ROOT,
            types::timeline_structural_hash(Some(entity), Some(&player_ref)),
            types::timeline_live_hash(Some(&player_ref)),
        ),
        irisui::prelude::RetainedRevisionStatus::Rebuild
    );
}