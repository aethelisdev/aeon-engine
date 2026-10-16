// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Transform Component Inspector Card Builder
//!
//! Renders the 3D Position, Rotation Euler, and Scale axes using high-level
//! declarative [`UiScope`] cards and property row primitives.

use super::registry::ComponentRenderContext;
use super::types::InspectorNumberInputId;
use irisui::prelude::*;

/// Builds the `📐 Transform` card declaratively in the `UiTree` and returns the measured height.
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

    let get_edit_state = |id: InspectorNumberInputId| {
        ctx.params
            .active_number_input
            .filter(|s| s.id == id)
            .map(|s| s.to_edit_state(ctx.params.blink_caret))
    };

    scope.card_named("TransformCard", "📐 Transform", |card| {
        card.add(
            DragVec3::new("Position", &mut pos)
                .speed(0.1)
                .decimals(3)
                .reset([0.0, 0.0, 0.0])
                .edit_states([
                    get_edit_state(InspectorNumberInputId::PosX),
                    get_edit_state(InspectorNumberInputId::PosY),
                    get_edit_state(InspectorNumberInputId::PosZ),
                ]),
        );
        card.add(
            DragVec3::new("Rotation", &mut rot)
                .speed(0.5)
                .decimals(1)
                .reset([0.0, 0.0, 0.0])
                .edit_states([
                    get_edit_state(InspectorNumberInputId::RotX),
                    get_edit_state(InspectorNumberInputId::RotY),
                    get_edit_state(InspectorNumberInputId::RotZ),
                ]),
        );
        card.add(
            DragVec3::new("Scale", &mut scale)
                .speed(0.01)
                .decimals(3)
                .reset([1.0, 1.0, 1.0])
                .edit_states([
                    get_edit_state(InspectorNumberInputId::ScaleX),
                    get_edit_state(InspectorNumberInputId::ScaleY),
                    get_edit_state(InspectorNumberInputId::ScaleZ),
                ]),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::{ActiveNumberInputState, InspectorPanelParams};
    use ae_core::ecs::{Name, Position, Rotation, Scale};

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
    fn test_declarative_transform_active_edit_and_select_all() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("ActiveEditEntity".to_string()),
            Position::new(5.0, 10.0, 15.0),
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
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: Some(ActiveNumberInputState {
                id: InspectorNumberInputId::RotY,
                buffer: "45.0",
                cursor_idx: 4,
                is_all_selected: true,
            }),
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

        // Verify active cyan border and selection highlight pill declaratively
        assert_eq!(
            tree.assert_node("NumBox_Rotation_Y", WidgetRole::NumericInput)
                .style
                .border
                .color,
            Color::rgba(0.0, 0.80, 1.00, 0.95)
        );
        tree.assert_node_exists("NumSel_Rotation_Y");
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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);
        let mut scope = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope, &mut ctx);

        // Verify NumText_Rotation_X, Y, Z declaratively without loops or node let bindings
        tree.assert_node_text("NumText_Rotation_X", "X: -34.0");
        tree.assert_node_text("NumText_Rotation_Y", "Y: 120.0");
        tree.assert_node_text("NumText_Rotation_Z", "Z: 0.0");
    }
}