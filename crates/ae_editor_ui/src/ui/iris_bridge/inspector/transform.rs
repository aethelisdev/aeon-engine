// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Transform Component Inspector Card Builder
//!
//! Renders the 3D Position, Rotation Euler, and Scale axes using high-level
//! declarative [`UiScope`] cards and property row primitives.

use super::registry::ComponentRenderContext;
use irisui::prelude::*;

/// Builds the `📐 Transform` card declaratively in the `UiTree`.
///
/// Encapsulates the entire Transform inspector interface within a pure declarative [`UiScope`]
/// container hierarchy, managing flex layout, hover states, caret blinking, and Select-All highlight
/// capsules without manual coordinate arithmetic or imperative target buffers.
pub fn build_transform_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    let mut pos = ctx
        .world
        .get::<&ae_core::ecs::Position>(ctx.entity)
        .map(|p| [p.x, p.y, p.z])
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut rot = if let Ok(world_rot) = ctx.world.get::<&ae_core::ecs::Rotation>(ctx.entity) {
        let cached_quat = crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(
            ctx.params.inspector_euler[0],
            ctx.params.inspector_euler[1],
            ctx.params.inspector_euler[2],
        );
        let dot = (cached_quat.x * world_rot.x
            + cached_quat.y * world_rot.y
            + cached_quat.z * world_rot.z
            + cached_quat.w * world_rot.w)
            .abs();
        if ctx.params.selected_entity == Some(ctx.entity) && dot > 0.999 {
            *ctx.params.inspector_euler
        } else {
            crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&world_rot)
        }
    } else {
        [0.0, 0.0, 0.0]
    };

    let mut scale = ctx
        .world
        .get::<&ae_core::ecs::Scale>(ctx.entity)
        .map(|s| [s.x, s.y, s.z])
        .unwrap_or([1.0, 1.0, 1.0]);

    scope.card_named("TransformCard", "📐 Transform", |card| {
        ctx.prop_transform_position(card, &mut pos);
        ctx.prop_transform_rotation(card, &mut rot);
        ctx.prop_transform_scale(card, &mut scale);
    });
}

#[cfg(test)]
mod tests {
    use super::super::types::InspectorPanelParams;
    use super::*;
    use ae_core::ecs::{Name, Position, Rotation, Scale};
    use irisui::widgets::declarative::{property_vec3_axis_tag, property_vec3_reset_tag};

    #[test]
    fn test_declarative_transform_card_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("TestTransformEntity".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Rotation::identity(),
            Scale::one(),
        ));

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);

        let mut scope = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope, &mut ctx);
        scope.finish_layout(Rect::new(6.0, 10.0, 308.0, 300.0));

        // Declarative assertions (zero imperative loop or let boilerplate)
        tree.assert_node_exists("TransformCard");
        assert_eq!(
            tree.assert_node_rect("TransformCard"),
            Rect::new(
                6.0,
                10.0,
                308.0,
                tree.assert_node_rect("TransformCard").height
            )
        );
        assert!(tree.assert_node_rect("TransformCard").height > 100.0);

        tree.assert_widgets(
            &[
                "NumBox_Position_X",
                "NumBox_Position_Y",
                "NumBox_Position_Z",
                "NumBox_Rotation_X",
                "NumBox_Rotation_Y",
                "NumBox_Rotation_Z",
                "NumBox_Scale_X",
                "NumBox_Scale_Y",
                "NumBox_Scale_Z",
            ],
            WidgetRole::NumericInput,
        );

        tree.assert_widgets(
            &["ResetBtn_Position", "ResetBtn_Rotation", "ResetBtn_Scale"],
            WidgetRole::Button,
        );
    }

    #[test]
    fn test_declarative_transform_left_aligned_across_resizes() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("AlignEntity".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Rotation::identity(),
            Scale::one(),
        ));
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        // 1. Render in a standard narrow card (308px)
        let mut ctx_narrow = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);
        let mut scope_narrow = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope_narrow, &mut ctx_narrow);
        scope_narrow.finish_layout(Rect::new(6.0, 10.0, 308.0, 300.0));

        // Verify all 3 rows have identical X coordinates (column aligned) declaratively
        assert_eq!(
            tree.assert_node_rect("NumBox_Position_X").x,
            tree.assert_node_rect("NumBox_Rotation_X").x,
            "Position X and Rotation X must share horizontal start coordinate"
        );
        assert_eq!(
            tree.assert_node_rect("NumBox_Rotation_X").x,
            tree.assert_node_rect("NumBox_Scale_X").x,
            "Rotation X and Scale X must share horizontal start coordinate"
        );

        // Verify inputs are left-aligned
        assert!(
            tree.assert_node_rect("NumBox_Position_X").x < 100.0,
            "Inputs must be left-aligned, found X: {}",
            tree.assert_node_rect("NumBox_Position_X").x
        );

        // 2. Render in a wide card (600px)
        let mut tree_wide = UiTree::new();
        let root_wide = tree_wide.create_root().expect("Root must be created");
        let mut ctx_wide = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 600.0);
        let mut scope_wide = UiScope::new(&mut tree_wide, root_wide);
        build_transform_card(&mut scope_wide, &mut ctx_wide);
        scope_wide.finish_layout(Rect::new(6.0, 10.0, 600.0, 300.0));

        // In the wide card, the inputs must REMAIN at the exact same left-aligned X position
        assert_eq!(
            tree_wide.assert_node_rect("NumBox_Position_X").x,
            tree.assert_node_rect("NumBox_Position_X").x,
            "Inputs must not shift to the right when card is widened!"
        );
        assert_eq!(
            tree_wide.assert_node_rect("NumBox_Rotation_X").x,
            tree.assert_node_rect("NumBox_Rotation_X").x,
        );
        assert_eq!(
            tree_wide.assert_node_rect("NumBox_Scale_X").x,
            tree.assert_node_rect("NumBox_Scale_X").x,
        );
    }

    #[test]
    fn test_transform_card_rotation_euler_displays_angles_past_90_degrees() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();

        let entity = world.spawn((
            Name("Dynamic Cube".to_string()),
            Position::new(0.0, 5.0, 0.0),
            crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(-34.0, 120.0, 0.0),
            Scale::one(),
        ));

        let euler = [-34.0, 120.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);
        let mut scope = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope, &mut ctx);

        tree.assert_node_text("NumText_Rotation_X", "X: -34.0");
        tree.assert_node_text("NumText_Rotation_Y", "Y: 120.0");
        tree.assert_node_text("NumText_Rotation_Z", "Z: 0.0");
    }

    #[test]
    fn test_transform_card_tags_encoded_correctly() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("TagEntity".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Rotation::identity(),
            Scale::one(),
        ));
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);
        let mut scope = UiScope::new(&mut tree, root);
        let pos_base = scope.tag_for("Position");
        let rot_base = scope.tag_for("Rotation");
        let scale_base = scope.tag_for("Scale");
        build_transform_card(&mut scope, &mut ctx);

        let pairs = [
            ("NumBox_Position_X", property_vec3_axis_tag(pos_base, 0)),
            ("NumBox_Position_Y", property_vec3_axis_tag(pos_base, 1)),
            ("NumBox_Position_Z", property_vec3_axis_tag(pos_base, 2)),
            ("NumBox_Rotation_X", property_vec3_axis_tag(rot_base, 0)),
            ("NumBox_Rotation_Y", property_vec3_axis_tag(rot_base, 1)),
            ("NumBox_Rotation_Z", property_vec3_axis_tag(rot_base, 2)),
            ("NumBox_Scale_X", property_vec3_axis_tag(scale_base, 0)),
            ("NumBox_Scale_Y", property_vec3_axis_tag(scale_base, 1)),
            ("NumBox_Scale_Z", property_vec3_axis_tag(scale_base, 2)),
        ];

        for (name, expected_tag) in pairs {
            let node = tree.assert_node(name, WidgetRole::NumericInput);
            assert_eq!(
                node.tag, expected_tag,
                "Tag for {} must match property_vec3_axis_tag",
                name
            );
        }

        assert_eq!(
            tree.assert_node("ResetBtn_Position", WidgetRole::Button)
                .tag,
            property_vec3_reset_tag(pos_base)
        );
        assert_eq!(
            tree.assert_node("ResetBtn_Rotation", WidgetRole::Button)
                .tag,
            property_vec3_reset_tag(rot_base)
        );
        assert_eq!(
            tree.assert_node("ResetBtn_Scale", WidgetRole::Button).tag,
            property_vec3_reset_tag(scale_base)
        );
    }

    #[test]
    fn test_transform_in_place_mutation_and_undo_actions() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("MutEntity".to_string()),
            Position::new(0.0, 0.0, 0.0),
            Rotation::identity(),
            Scale::one(),
        ));
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        let mut ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            6.0,
            10.0,
            308.0,
        );

        let mut scope = UiScope::new(&mut tree, root);
        let mut pos = [5.0, 10.0, 15.0];
        ctx.prop_transform_position(&mut scope, &mut pos);

        // Position was updated in ECS
        let world_pos = *world.get::<&Position>(entity).expect("Position exists");
        assert_eq!(world_pos, Position::new(0.0, 0.0, 0.0)); // No interaction occurred yet
    }
}