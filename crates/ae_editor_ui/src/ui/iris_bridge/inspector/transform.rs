// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Transform Component Inspector Card Builder
//!
//! Renders the 3D Position, Rotation Euler, and Scale axes using high-level
//! declarative [`UiScope`] cards and property row primitives.

use super::registry::ComponentRenderContext;
use super::types::{InspectorAction, InspectorNumberInputId, TransformAxisType};
use irisui::prelude::*;

/// Renders a 3-channel vector row using autonomous [`NumericDrag`] widgets and a reset button.
fn render_vec3_numeric_row(
    card: &mut UiScope<'_>,
    title: &str,
    values: &mut [f32; 3],
    speed: f32,
    decimals: usize,
    reset_val: [f32; 3],
    edit_states: [Option<NumericInputEditState<'_>>; 3],
) -> (bool, bool) {
    let mut changed = false;
    let mut reset_clicked = false;

    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(24.0)
        .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
        .gap(4.0);

    let (box_names, reset_name): ([&'static str; 3], &'static str) = match title {
        "Position" => (
            [
                "NumBox_Position_X",
                "NumBox_Position_Y",
                "NumBox_Position_Z",
            ],
            "ResetBtn_Position",
        ),
        "Rotation" => (
            [
                "NumBox_Rotation_X",
                "NumBox_Rotation_Y",
                "NumBox_Rotation_Z",
            ],
            "ResetBtn_Rotation",
        ),
        "Scale" => (
            ["NumBox_Scale_X", "NumBox_Scale_Y", "NumBox_Scale_Z"],
            "ResetBtn_Scale",
        ),
        _ => (["NumericDrag", "NumericDrag", "NumericDrag"], "ResetBtn"),
    };

    card.container_named("Vec3Row", row_style, |row| {
        row.label_styled_passive(
            "RowLabel",
            title,
            11.0,
            row.theme().text_muted,
            TextAlign::Left,
            Style::new().width(52.0),
        );

        let slot_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(3.0);

        row.container_named("Slot", slot_style, |slot| {
            let prefixes = ["X", "Y", "Z"];
            for i in 0..3 {
                let tag = slot.tag_for(box_names[i]);
                let mut drag = NumericDrag::new(prefixes[i], &mut values[i])
                    .name(box_names[i])
                    .tag(tag)
                    .speed(speed)
                    .decimals(decimals)
                    .label_width(12.0)
                    .box_width(52.0);
                if let Some(es) = edit_states[i] {
                    drag = drag.edit_state(Some(es));
                }
                if slot.add(drag).changed() {
                    changed = true;
                }
            }

            let reset_resp = slot.add(
                Button::new("🔄")
                    .name(reset_name)
                    .size(18.0, 18.0)
                    .tag(slot.tag_for("Reset")),
            );
            if reset_resp.clicked() {
                *values = reset_val;
                changed = true;
                reset_clicked = true;
            }
        });
    });

    (changed, reset_clicked)
}

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

    scope.card_named("TransformCard", "📐 Transform", |card| {
        let (pos_changed, pos_reset) = render_vec3_numeric_row(
            card,
            "Position",
            &mut pos,
            0.1,
            3,
            [0.0, 0.0, 0.0],
            [
                ctx.edit_state(InspectorNumberInputId::PosX),
                ctx.edit_state(InspectorNumberInputId::PosY),
                ctx.edit_state(InspectorNumberInputId::PosZ),
            ],
        );
        if pos_changed {
            if let Ok(mut p) = ctx.world.get::<&mut ae_core::ecs::Position>(ctx.entity) {
                p.x = pos[0];
                p.y = pos[1];
                p.z = pos[2];
            }
            if pos_reset {
                ctx.push_action(InspectorAction::ResetTransform(
                    ctx.entity,
                    TransformAxisType::Position,
                ));
            } else {
                ctx.set_number(InspectorNumberInputId::PosX, pos[0]);
                ctx.set_number(InspectorNumberInputId::PosY, pos[1]);
                ctx.set_number(InspectorNumberInputId::PosZ, pos[2]);
            }
        }

        let (rot_changed, rot_reset) = render_vec3_numeric_row(
            card,
            "Rotation",
            &mut rot,
            0.5,
            1,
            [0.0, 0.0, 0.0],
            [
                ctx.edit_state(InspectorNumberInputId::RotX),
                ctx.edit_state(InspectorNumberInputId::RotY),
                ctx.edit_state(InspectorNumberInputId::RotZ),
            ],
        );
        if rot_changed {
            let quat =
                crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(rot[0], rot[1], rot[2]);
            if let Ok(mut r) = ctx.world.get::<&mut ae_core::ecs::Rotation>(ctx.entity) {
                r.x = quat.x;
                r.y = quat.y;
                r.z = quat.z;
                r.w = quat.w;
            }
            if rot_reset {
                ctx.push_action(InspectorAction::ResetTransform(
                    ctx.entity,
                    TransformAxisType::Rotation,
                ));
            } else {
                ctx.set_number(InspectorNumberInputId::RotX, rot[0]);
                ctx.set_number(InspectorNumberInputId::RotY, rot[1]);
                ctx.set_number(InspectorNumberInputId::RotZ, rot[2]);
            }
        }

        let (scale_changed, scale_reset) = render_vec3_numeric_row(
            card,
            "Scale",
            &mut scale,
            0.01,
            3,
            [1.0, 1.0, 1.0],
            [
                ctx.edit_state(InspectorNumberInputId::ScaleX),
                ctx.edit_state(InspectorNumberInputId::ScaleY),
                ctx.edit_state(InspectorNumberInputId::ScaleZ),
            ],
        );
        if scale_changed {
            if let Ok(mut s) = ctx.world.get::<&mut ae_core::ecs::Scale>(ctx.entity) {
                s.x = scale[0];
                s.y = scale[1];
                s.z = scale[2];
            }
            if scale_reset {
                ctx.push_action(InspectorAction::ResetTransform(
                    ctx.entity,
                    TransformAxisType::Scale,
                ));
            } else {
                ctx.set_number(InspectorNumberInputId::ScaleX, scale[0]);
                ctx.set_number(InspectorNumberInputId::ScaleY, scale[1]);
                ctx.set_number(InspectorNumberInputId::ScaleZ, scale[2]);
            }
        }
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

        let node = tree.assert_node("NumBox_Rotation_Y", WidgetRole::NumericInput);
        assert_eq!(node.text.as_deref(), Some("45.0"));
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

        tree.assert_node_text("NumBox_Rotation_X", "-34.0");
        tree.assert_node_text("NumBox_Rotation_Y", "120.0");
        tree.assert_node_text("NumBox_Rotation_Z", "0.0");
    }
}