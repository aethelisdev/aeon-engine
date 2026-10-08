// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Inspector Cards Integration Tests
//!
//! Verifies the declarative `UiScope` widget creation, container nesting, and semantic tags
//! for physics, character, gameplay, and audio components without legacy targets buffers.
//!

use ae_editor_ui::ui::iris_bridge::icons::{ICON_CHEVRON_DOWN, ICON_CHEVRON_UP};
use ae_editor_ui::ui::iris_bridge::inspector::components::audio::{
    AudioListenerHandler, AudioSourceHandler,
};
use ae_editor_ui::ui::iris_bridge::inspector::components::character::PlayerTagHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::gameplay::VelocityHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::physics::material::PhysicsMaterialHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::rendering::ShapeHandler;
use ae_editor_ui::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use ae_editor_ui::ui::iris_bridge::inspector::tags::{
    resolve_audio_pick_tag, resolve_audio_play_tag, resolve_component_checkbox_tag,
    resolve_component_delete_tag, resolve_inspector_dropdown_tag,
    resolve_inspector_number_input_tag, resolve_preset_reset_tag,
};
use ae_editor_ui::ui::iris_bridge::inspector::types::{
    ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId, InspectorPanelParams,
};
use irisui::prelude::*;

#[test]
fn test_player_tag_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::PlayerTag,));
    let mut tree = UiTree::new();
    let root = WidgetId::default();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = PlayerTagHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("PlayerTag"));
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };
    let root = WidgetId::default();
    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = VelocityHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("Velocity"));
    assert!(has_delete, "Velocity delete button must be tagged");

    let has_vx = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::VelocityX)
    });
    let has_vy = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::VelocityY)
    });
    let has_vz = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::VelocityZ)
    });
    assert!(
        has_vx && has_vy && has_vz,
        "Velocity X/Y/Z inputs must be tagged"
    );
}

#[test]
fn test_shape_card_declarative_structure_and_chevron() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Shape::Cube,));
    let mut tree = UiTree::new();
    let root = WidgetId::default();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = ShapeHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("Shape"));
    assert!(has_delete, "Shape delete button must be tagged");

    let has_dropdown = tree.iter().any(|(_, n)| {
        resolve_inspector_dropdown_tag(n.tag) == Some(InspectorDropdownId::ShapeType)
    });
    assert!(has_dropdown, "Shape dropdown must be tagged");

    let has_chevron_down = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("ComboChevron") && n.texture_uv == Some(ICON_CHEVRON_DOWN)
    });
    assert!(
        has_chevron_down,
        "Closed combobox must render ICON_CHEVRON_DOWN"
    );

    // Test open state renders chevron up
    let mut open_params = params;
    open_params.active_dropdown = Some(InspectorDropdownId::ShapeType);
    let mut open_ctx = ComponentRenderContext::new(entity, &world, &open_params, 10.0, 20.0, 260.0);
    let mut open_tree = UiTree::new();
    let mut open_scope = UiScope::new(&mut open_tree, root);
    handler.render_card(&mut open_scope, &mut open_ctx);

    let has_chevron_up = open_tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("ComboChevron") && n.texture_uv == Some(ICON_CHEVRON_UP)
    });
    assert!(has_chevron_up, "Open combobox must render ICON_CHEVRON_UP");
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler =
        ae_editor_ui::ui::iris_bridge::inspector::components::character::CharacterControllerHandler;
    let mut scope = UiScope::new(&mut tree, WidgetId::default());
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("CharacterController"));
    assert!(
        has_delete,
        "CharacterController delete button must be tagged"
    );

    let has_height = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::CharacterHeight)
    });
    assert!(has_height, "CharacterHeight must be tagged");

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

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    use ae_editor_ui::ui::iris_bridge::inspector::components::gameplay::*;

    // CharacterAction
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        CharacterActionHandler.render_card(&mut scope, &mut ctx);
        let has_speed = tree.iter().any(|(_, n)| {
            resolve_inspector_number_input_tag(n.tag)
                .is_some_and(|(id, ..)| id == InspectorNumberInputId::ActionSpeedRange)
        });
        assert!(has_speed, "ActionSpeedRange must be tagged");
    }

    // Rotator
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        RotatorHandler.render_card(&mut scope, &mut ctx);
        let has_speed = tree.iter().any(|(_, n)| {
            resolve_inspector_number_input_tag(n.tag)
                .is_some_and(|(id, ..)| id == InspectorNumberInputId::RotatorSpeed)
        });
        assert!(has_speed, "RotatorSpeed must be tagged");
    }

    // MovingPlatform
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        MovingPlatformHandler.render_card(&mut scope, &mut ctx);
        let has_delete = tree
            .iter()
            .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("MovingPlatform"));
        assert!(has_delete, "MovingPlatform delete button must be tagged");
        let has_speed = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("PlatformSpeedLbl")
                && n.text.as_deref().is_some_and(|t| t.contains("2.5 m/s"))
        });
        assert!(has_speed);
    }

    // TriggerZone
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        TriggerZoneHandler.render_card(&mut scope, &mut ctx);
        let has_idle = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("TriggerStatusLbl")
                && n.text.as_deref().is_some_and(|t| t.contains("IDLE"))
        });
        assert!(has_idle);
    }

    // DestructibleTarget
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        DestructibleTargetHandler.render_card(&mut scope, &mut ctx);
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

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    // AnimationPlayerHandler
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let handler =
            ae_editor_ui::ui::iris_bridge::inspector::components::animation::AnimationPlayerHandler;
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        handler.render_card(&mut scope, &mut ctx);
        let has_playing = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AnimStatusVal")
                && n.text.as_deref().is_some_and(|t| t.contains("PLAYING"))
        });
        assert!(has_playing);
    }

    // UiLayoutGroupHandler
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let handler =
            ae_editor_ui::ui::iris_bridge::inspector::components::ui_canvas::UiLayoutGroupHandler;
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        handler.render_card(&mut scope, &mut ctx);
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = PhysicsMaterialHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("PhysicsMaterial"));
    assert!(has_delete, "PhysicsMaterial delete button must be tagged");

    let has_dropdown = tree.iter().any(|(_, n)| {
        resolve_inspector_dropdown_tag(n.tag) == Some(InspectorDropdownId::SurfaceType)
    });
    assert!(has_dropdown, "SurfaceType dropdown must be tagged");

    let has_preset = tree.iter().any(|(_, n)| resolve_preset_reset_tag(n.tag));
    assert!(has_preset, "Preset reset button must be tagged");

    let has_fric = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::PhysMatFriction)
    });
    let has_rest = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::PhysMatRestitution)
    });
    assert!(
        has_fric && has_rest,
        "PhysMatFriction and PhysMatRestitution must be tagged"
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

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    };

    // AudioSourceHandler test
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        let handler = AudioSourceHandler;
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        handler.render_card(&mut scope, &mut ctx);

        let has_delete = tree
            .iter()
            .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("AudioSource"));
        assert!(has_delete, "AudioSource delete button must be tagged");

        let has_pick = tree.iter().any(|(_, n)| resolve_audio_pick_tag(n.tag));
        let has_play = tree.iter().any(|(_, n)| resolve_audio_play_tag(n.tag));
        assert!(
            has_pick && has_play,
            "Audio pick and play buttons must be tagged"
        );

        let has_vol = tree.iter().any(|(_, n)| {
            resolve_inspector_number_input_tag(n.tag)
                .is_some_and(|(id, ..)| id == InspectorNumberInputId::AudioVolume)
        });
        let has_pitch = tree.iter().any(|(_, n)| {
            resolve_inspector_number_input_tag(n.tag)
                .is_some_and(|(id, ..)| id == InspectorNumberInputId::AudioPitch)
        });
        assert!(
            has_vol && has_pitch,
            "AudioVolume and AudioPitch must be tagged"
        );

        let has_spatial = tree.iter().any(|(_, n)| {
            resolve_component_checkbox_tag(n.tag) == Some(ComponentCheckboxId::AudioSpatial)
        });
        let has_loop = tree.iter().any(|(_, n)| {
            resolve_component_checkbox_tag(n.tag) == Some(ComponentCheckboxId::AudioLoop)
        });
        let has_play_on_start = tree.iter().any(|(_, n)| {
            resolve_component_checkbox_tag(n.tag) == Some(ComponentCheckboxId::AudioPlayOnStart)
        });
        assert!(
            has_spatial && has_loop && has_play_on_start,
            "Audio checkboxes must be tagged"
        );
    }

    // AudioListenerHandler test
    {
        let mut tree = UiTree::new();
        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        let handler = AudioListenerHandler;
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        handler.render_card(&mut scope, &mut ctx);

        let has_delete = tree
            .iter()
            .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("AudioListener"));
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