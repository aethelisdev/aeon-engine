// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Inspector Cards Integration Tests
//!
//! Verifies the declarative `UiScope` widget creation, container nesting, and semantic tags
//! for physics, character, gameplay, and audio components without legacy targets buffers.

use ae_editor_ui::ui::iris_bridge::inspector::components::animation::draw_animation_player_card;
use ae_editor_ui::ui::iris_bridge::inspector::components::audio::{
    draw_audio_listener_card, draw_audio_source_card,
};
use ae_editor_ui::ui::iris_bridge::inspector::components::character::{
    draw_character_controller_card, draw_player_tag_card,
};
use ae_editor_ui::ui::iris_bridge::inspector::components::gameplay::*;
use ae_editor_ui::ui::iris_bridge::inspector::components::physics::material::draw_physics_material_card;
use ae_editor_ui::ui::iris_bridge::inspector::components::rendering::draw_shape_card;
use ae_editor_ui::ui::iris_bridge::inspector::components::ui_canvas::draw_ui_layout_group_card;
use irisui::prelude::*;

#[test]
fn test_player_tag_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::PlayerTag,));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    let mut scope = UiScope::new(&mut tree, root);
    draw_player_tag_card(&mut scope, &world, entity);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("PlayerTag"));
    assert!(has_delete, "PlayerTag delete button must be tagged");

    let has_desc = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("PlayerTagDesc")
            && n.text
                .as_deref()
                .is_some_and(|t| t.contains("active Player target"))
    });
    assert!(has_desc, "PlayerTag card must render descriptive label");
}

#[test]
fn test_velocity_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Velocity {
        x: 1.5,
        y: -2.0,
        z: 3.25,
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    let mut scope = UiScope::new(&mut tree, root);
    draw_velocity_card(&mut scope, &world, entity);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("Velocity"));
    assert!(has_delete, "Velocity delete button must be tagged");

    let numeric_pills = tree
        .iter()
        .filter(|(_, n)| {
            n.role == irisui::core::WidgetRole::NumericInput
                && n.name
                    .as_deref()
                    .unwrap_or("")
                    .starts_with("NumBox_Linear_")
        })
        .count();
    assert_eq!(
        numeric_pills, 3,
        "Velocity card must emit 3 numeric pills (X, Y, Z)"
    );
}

#[test]
fn test_shape_card_declarative_structure_and_chevron() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Shape::Cube,));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    let mut scope = UiScope::new(&mut tree, root);
    draw_shape_card(&mut scope, &world, entity);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("Shape"));
    assert!(has_delete, "Shape delete button must be tagged");

    let has_combo = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
    assert!(has_combo, "Shape dropdown combo pill must be rendered");

    let has_chevron_down = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▾"));
    assert!(
        has_chevron_down,
        "Closed combobox must render chevron down (▾)"
    );

    // Test open state renders chevron up
    let combo_tag = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
        .map(|(_, n)| n.tag)
        .expect("ComboPill node must exist in tree");

    let mut open_tree = UiTree::new();
    let open_root = open_tree.create_root().expect("Root node must exist");
    open_tree.active_combobox_tag = Some(combo_tag);
    let mut open_scope = UiScope::new(&mut open_tree, open_root);
    draw_shape_card(&mut open_scope, &world, entity);

    let has_chevron_up = open_tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▴"));
    assert!(has_chevron_up, "Open combobox must render chevron up (▴)");
}

#[test]
fn test_character_controller_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::CharacterController {
        height: 1.80,
        radius: 0.40,
        center_y: 0.0,
        max_slope_climb_angle: 45.0,
        step_height: 0.30,
        is_grounded: false,
    },));
    let mut tree = UiTree::new();

    let mut scope = UiScope::new(&mut tree, WidgetId::default());
    draw_character_controller_card(&mut scope, &world, entity);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("CharacterController"));
    assert!(
        has_delete,
        "CharacterController delete button must be tagged"
    );

    let numeric_pills = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
        .count();
    assert_eq!(
        numeric_pills, 5,
        "CharacterController must emit 5 numeric drag inputs (Height, Radius, Center Y, Max Slope, Step Height)"
    );

    let has_in_air_badge = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("GroundedStatusText")
            && n.text.as_deref().is_some_and(|t| t.contains("In Air"))
    });
    assert!(has_in_air_badge, "Must render In Air status text");
}

#[test]
fn test_gameplay_cards_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        ae_core::ecs::CharacterAction {
            speed: 50.0,
            cooldown: 0.20,
            ..ae_core::ecs::CharacterAction::new()
        },
        ae_core::ecs::Rotator {
            speed: 1.5,
            axis: [0.0, 1.0, 0.0],
        },
        ae_core::ecs::MovingPlatform {
            speed: 2.5,
            target_position: [0.0, 5.0, 0.0],
            ..Default::default()
        },
        ae_core::ecs::TriggerZone {
            speed: 3.0,
            target_position: [0.0, 4.0, 0.0],
            is_triggered: false,
            ..Default::default()
        },
        ae_core::ecs::DestructibleTarget::new(100.0),
    ));

    // CharacterAction
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_character_action_card(&mut scope, &world, entity);
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "CharacterAction must emit 2 numeric drag pills (Speed, Cooldown)"
        );
    }

    // Rotator
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_rotator_card(&mut scope, &world, entity);
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.role == irisui::core::WidgetRole::NumericInput)
            .count();
        assert_eq!(
            numeric_pills, 4,
            "Rotator must emit 4 numeric inputs (1 Speed + 3 Axis axes)"
        );
        let has_axis_x = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NumBox_Axis:_X"));
        assert!(has_axis_x, "Rotator must render NumBox_Axis:_X");
    }

    // MovingPlatform
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_moving_platform_card(&mut scope, &world, entity);
        let has_delete = tree
            .iter()
            .any(|(_, n)| n.tag == UiScope::action_tag("MovingPlatform"));
        assert!(has_delete, "MovingPlatform delete button must be tagged");
        let has_target = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NumBox_Target Pos:_X"));
        assert!(
            has_target,
            "MovingPlatform must render NumBox_Target Pos:_X"
        );
    }

    // TriggerZone
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_trigger_zone_card(&mut scope, &world, entity);
        let has_idle = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("TriggerStatusLbl")
                && n.text.as_deref().is_some_and(|t| t.contains("IDLE"))
        });
        assert!(has_idle);
    }

    // DestructibleTarget
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_destructible_target_card(&mut scope, &world, entity);
        let has_track = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("HealthBarTrack"));
        let has_fill = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("HealthBarFill"));
        assert!(has_track && has_fill);
    }
}

#[test]
fn test_animation_and_ui_layout_cards_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        ae_animation::AnimationPlayer {
            state: ae_animation::AnimationState::Playing,
            speed: 1.0,
            looping: true,
            ..Default::default()
        },
        ae_core::ecs::UiLayoutGroup {
            layout_type: ae_core::ecs::UiLayoutType::Vertical,
            spacing: 8.0,
            ..Default::default()
        },
    ));

    // AnimationPlayer
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_animation_player_card(&mut scope, &world, entity);
        let has_playing = tree.iter().any(|(_, n)| {
            (n.name.as_deref() == Some("AnimStatusVal")
                || n.name.as_deref() == Some("LabeledStatusValue"))
                && n.text.as_deref().is_some_and(|t| t.contains("PLAYING"))
        });
        assert!(has_playing);
    }

    // UiLayoutGroup
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_ui_layout_group_card(&mut scope, &world, entity);
        let has_vertical = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("UiLayoutProps")
                && n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Vertical") && t.contains("8.0 px"))
        });
        assert!(has_vertical);
    }
}

#[test]
fn test_physics_material_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::PhysicsMaterial {
        friction: 0.75,
        restitution: 0.25,
        surface_type: ae_core::ecs::SurfaceType::Wood,
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    let mut scope = UiScope::new(&mut tree, root);
    draw_physics_material_card(&mut scope, &world, entity);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("PhysicsMaterial"));
    assert!(has_delete, "PhysicsMaterial delete button must be tagged");

    let has_combo = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
    assert!(has_combo, "SurfaceType dropdown ComboPill must be present");

    let has_preset = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("PresetBtn") && n.role == WidgetRole::Button);
    assert!(has_preset, "Preset reset button must exist");

    let numeric_pills = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
        .count();
    assert_eq!(
        numeric_pills, 2,
        "PhysMatFriction and PhysMatRestitution must be emitted as numeric drag pills"
    );
}

#[test]
fn test_audio_cards_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        ae_audio::AudioSource {
            sound_path: "assets/audio/explosion.wav".to_string(),
            volume: 0.85,
            pitch: 1.10,
            is_spatial: true,
            looping: false,
            play_on_start: true,
            is_playing: false,
            ..Default::default()
        },
        ae_audio::AudioListener,
    ));

    // AudioSource test
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_audio_source_card(&mut scope, &world, entity);

        let has_delete = tree
            .iter()
            .any(|(_, n)| n.tag == UiScope::action_tag("AudioSource"));
        assert!(has_delete, "AudioSource delete button must be tagged");

        let has_pick = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AudioPickBtn") && n.role == WidgetRole::Button
        });
        let has_play = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AudioPlayToggleBtn") && n.role == WidgetRole::Button
        });
        assert!(
            has_pick && has_play,
            "Audio pick and play buttons must be present"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "AudioSource must emit 2 numeric drag pills (Volume, Pitch)"
        );

        let checkboxes = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("Checkbox"))
            .count();
        assert_eq!(
            checkboxes, 3,
            "AudioSource must emit 3 checkboxes (Spatial, Loop, Play on Start)"
        );
    }

    // AudioListener test
    {
        let mut tree = UiTree::new();
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_audio_listener_card(&mut scope, &world, entity);

        let has_delete = tree
            .iter()
            .any(|(_, n)| n.tag == UiScope::action_tag("AudioListener"));
        assert!(has_delete, "AudioListener delete button must be tagged");

        let has_desc = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AudioListenerDesc")
                && n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Active 3D spatial microphone"))
        });
        assert!(has_desc);
    }
}