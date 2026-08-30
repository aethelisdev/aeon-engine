// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Transform Component Inspector Card Builder
//!
//! Renders the 3D Position, Rotation Euler, and Scale axes with precision
//! drag/number input pill boxes and individual axis reset buttons.

use super::registry::ComponentRenderContext;
use super::types::{InspectorNumberInputId, TransformAxisType};
use irisui::prelude::*;

/// Builds the `📐 Transform` card in the `UiTree` and returns the computed height.
pub fn build_transform_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
) -> f32 {
    let row_h = 22.0;
    let card_padding = 8.0;
    let card_h = 24.0 + 3.0 * (row_h + 4.0) + card_padding * 2.0;
    let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

    // 1. Outer Card Container
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("TransformCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card_id);

    // 2. Card Header
    let hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(hdr_id) {
        node.set_name("TransformHeader");
        node.set_text("📐 Transform");
        node.font_size = 11.5;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
        node.computed_rect = Rect::new(
            ctx.base_x + card_padding,
            ctx.base_y + card_padding,
            ctx.card_w - card_padding * 2.0,
            20.0,
        );
    }
    let _ = tree.add_child(card_id, hdr_id);

    let mut cur_y = ctx.base_y + card_padding + 22.0;

    // Fetch ECS Transform components
    let pos = ctx
        .world
        .get::<&ae_core::ecs::Position>(ctx.entity)
        .map(|p| [p.x, p.y, p.z])
        .unwrap_or([0.0, 0.0, 0.0]);

    let rot = *ctx.params.inspector_euler;

    let scale = ctx
        .world
        .get::<&ae_core::ecs::Scale>(ctx.entity)
        .map(|s| [s.x, s.y, s.z])
        .unwrap_or([1.0, 1.0, 1.0]);

    // 3. Position Row
    build_axis_row(
        tree,
        card_id,
        ctx,
        AxisRowDescriptor {
            label: "Position",
            axis_type: TransformAxisType::Position,
            values: [pos[0], pos[1], pos[2]],
            ids: [
                InspectorNumberInputId::PosX,
                InspectorNumberInputId::PosY,
                InspectorNumberInputId::PosZ,
            ],
            row_y: cur_y,
            decimals: 3,
        },
    );
    cur_y += row_h + 4.0;

    // 4. Rotation Row
    build_axis_row(
        tree,
        card_id,
        ctx,
        AxisRowDescriptor {
            label: "Rotation",
            axis_type: TransformAxisType::Rotation,
            values: [rot[0], rot[1], rot[2]],
            ids: [
                InspectorNumberInputId::RotX,
                InspectorNumberInputId::RotY,
                InspectorNumberInputId::RotZ,
            ],
            row_y: cur_y,
            decimals: 1,
        },
    );
    cur_y += row_h + 4.0;

    // 5. Scale Row
    build_axis_row(
        tree,
        card_id,
        ctx,
        AxisRowDescriptor {
            label: "Scale",
            axis_type: TransformAxisType::Scale,
            values: [scale[0], scale[1], scale[2]],
            ids: [
                InspectorNumberInputId::ScaleX,
                InspectorNumberInputId::ScaleY,
                InspectorNumberInputId::ScaleZ,
            ],
            row_y: cur_y,
            decimals: 3,
        },
    );

    card_h
}

/// Parameters descriptor for rendering a 3-axis transform row.
struct AxisRowDescriptor {
    label: &'static str,
    axis_type: TransformAxisType,
    values: [f32; 3],
    ids: [InspectorNumberInputId; 3],
    row_y: f32,
    decimals: usize,
}

/// Helper function to build a 3-axis (X, Y, Z) row with reset button.
fn build_axis_row(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    desc: AxisRowDescriptor,
) {
    let padding_x = 8.0;
    let label_w = 52.0;
    let box_w = 54.0; // Compact fixed width matching Image 2!
    let box_h = 20.0;
    let reset_btn_size = 18.0;

    // Label
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("{}Label", desc.label));
        node.set_text(desc.label);
        node.font_size = 11.0;
        node.line_height = box_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding_x, desc.row_y, label_w, box_h);
    }
    let _ = tree.add_child(parent_id, lbl_id);

    let prefixes = ["X: ", "Y: ", "Z: "];
    let mut cur_box_x = ctx.base_x + padding_x + label_w + 2.0;

    for (i, prefix) in prefixes.iter().enumerate() {
        let input_id = desc.ids[i];
        let val = desc.values[i];
        let box_rect = Rect::new(cur_box_x, desc.row_y, box_w, box_h);

        let is_editing = match ctx.params.active_number_input {
            Some((id, _)) => id == input_id,
            None => false,
        };
        let is_hovered = box_rect.contains_point(ctx.params.cursor_pos);

        let box_node_id = tree.create_node();
        if let Some(node) = tree.get_mut(box_node_id) {
            node.set_name(format!("NumBox_{:?}", input_id));
            node.computed_rect = box_rect;
            let (bg, border_col) = if is_editing {
                (
                    Color::rgba(0.118, 0.125, 0.145, 1.0),
                    Color::rgba(0.353, 0.376, 0.439, 0.95), // Clean neutral active ring
                )
            } else if is_hovered {
                (
                    Color::rgba(0.157, 0.169, 0.200, 1.0),
                    Color::rgba(0.235, 0.247, 0.286, 0.95),
                )
            } else {
                (
                    Color::rgba(0.125, 0.133, 0.153, 0.98),
                    Color::rgba(0.180, 0.192, 0.227, 0.85),
                )
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, border_col)
                .border_radius(5.0);
        }
        let _ = tree.add_child(parent_id, box_node_id);

        let display_str = if is_editing {
            let buf = ctx.params.active_number_input.map(|(_, b)| b).unwrap_or("");
            if ctx.params.blink_caret {
                format!("{}{}|", prefix, buf)
            } else {
                format!("{}{}", prefix, buf)
            }
        } else if desc.decimals == 1 {
            format!("{}{:.1}", prefix, val)
        } else {
            format!("{}{:.3}", prefix, val)
        };

        let txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(txt_id) {
            node.set_name(format!("NumText_{:?}", input_id));
            node.set_text(display_str);
            node.font_size = 10.5;
            node.line_height = box_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_editing {
                Color::WHITE
            } else {
                Color::rgba(0.886, 0.894, 0.918, 1.0)
            };
            node.computed_rect = box_rect;
        }
        let _ = tree.add_child(box_node_id, txt_id);

        ctx.targets
            .number_inputs
            .push((input_id, box_rect, -10_000.0, 10_000.0, val));

        cur_box_x += box_w + 3.0;
    }

    // Reset Button "🔄" placed right next to the Z box!
    let reset_rect = Rect::new(
        cur_box_x + 2.0,
        desc.row_y + 1.0,
        reset_btn_size,
        reset_btn_size,
    );
    let is_reset_hovered = reset_rect.contains_point(ctx.params.cursor_pos);

    let reset_id = tree.create_node();
    if let Some(node) = tree.get_mut(reset_id) {
        node.set_name(format!("ResetBtn_{:?}", desc.axis_type));
        node.computed_rect = reset_rect;
        let (bg, border_col, txt_col) = if is_reset_hovered {
            (
                Color::rgba(0.157, 0.169, 0.200, 1.0),
                Color::rgba(0.235, 0.247, 0.286, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.125, 0.133, 0.153, 0.98),
                Color::rgba(0.180, 0.192, 0.227, 0.85),
                Color::rgba(0.70, 0.73, 0.80, 0.90),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border_col)
            .border_radius(5.0);
        node.set_text("🔄");
        node.font_size = 9.5;
        node.line_height = reset_btn_size;
        node.text_align = TextAlign::Center;
        node.text_color = txt_col;
    }
    let _ = tree.add_child(parent_id, reset_id);

    ctx.targets
        .transform_reset_btns
        .push((desc.axis_type, reset_rect));
}