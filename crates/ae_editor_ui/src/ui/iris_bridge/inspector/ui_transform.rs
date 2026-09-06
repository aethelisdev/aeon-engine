// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen Transform (RectTransform) Component Inspector Card
//!
//! Renders the 2D layout properties of `UiElement`:
//! - Anchor Presets (interactive ComboBox dropdown)
//! - Screen Offset X and Y (precision drag/number input boxes)
//! - Size Width and Height (precision drag/number input boxes)
//! - Pivot X and Y (center point alignment)
//! - Z-Index (layer ordering) and Alpha (opacity multiplier)
//! - Visibility toggle checkbox
//!

use super::components::physics::helpers::render_component_header;
use super::registry::ComponentRenderContext;
use super::types::{
    ComboboxRowParams, ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId,
};
use ae_core::ecs::{UiAnchor, UiElement};
use irisui::prelude::*;

/// Descriptor parameters for rendering a dual-axis numeric row without clippy warnings.
struct DualNumberRowParams<'a> {
    label: &'a str,
    prefixes: [&'a str; 2],
    values: [f32; 2],
    ids: [InspectorNumberInputId; 2],
    row_y: f32,
    decimals: usize,
    unit: &'a str,
}

/// Builds the `📐 2D Screen Transform` card in the `UiTree` and returns the computed height.
pub fn build_ui_transform_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
) -> f32 {
    let padding = 8.0;
    let row_h = 22.0;
    let row_gap = 4.0;
    // Header (24) + 6 rows (Anchor, Offset, Size, Pivot, Z-Index/Alpha, Visible) + padding
    let num_rows = 6.0;
    let card_h = 24.0 + num_rows * (row_h + row_gap) + padding * 2.0;
    let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

    // 1. Outer Card Container
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("UiTransformCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.70))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card_id);

    // 2. Card Header
    render_component_header(
        tree,
        card_id,
        ctx,
        "📐",
        "2D Screen Transform",
        Color::rgba(0.0, 0.85, 1.0, 1.0),
        "UiElement",
    );

    let mut cur_y = ctx.base_y + padding + 22.0;

    // Fetch UiElement component data
    let (anchor, offset, size, pivot, z_index, alpha, visible) = ctx
        .world
        .get::<&UiElement>(ctx.entity)
        .map(|u| {
            (
                u.anchor, u.offset, u.size, u.pivot, u.z_index, u.alpha, u.visible,
            )
        })
        .unwrap_or((
            UiAnchor::Center,
            [0.0, 0.0],
            [100.0, 30.0],
            [0.5, 0.5],
            0,
            1.0,
            true,
        ));

    // 3. Row 1: Anchor Preset Selector Dropdown
    let anchor_str = match anchor {
        UiAnchor::TopLeft => "Top-Left",
        UiAnchor::TopCenter => "Top-Center",
        UiAnchor::TopRight => "Top-Right",
        UiAnchor::CenterLeft => "Center-Left",
        UiAnchor::Center => "Center",
        UiAnchor::CenterRight => "Center-Right",
        UiAnchor::BottomLeft => "Bottom-Left",
        UiAnchor::BottomCenter => "Bottom-Center",
        UiAnchor::BottomRight => "Bottom-Right",
    };

    super::components::physics::helpers::render_combobox_row(
        tree,
        card_id,
        ctx,
        ComboboxRowParams {
            label: "Anchor",
            selected_text: anchor_str,
            dropdown_id: InspectorDropdownId::UiAnchor,
            label_w: 52.0,
            row_y: cur_y,
        },
    );
    cur_y += row_h + row_gap;

    // 4. Row 2: Offset X & Y (px)
    build_dual_number_row(
        tree,
        card_id,
        ctx,
        DualNumberRowParams {
            label: "Offset",
            prefixes: ["X: ", "Y: "],
            values: [offset[0], offset[1]],
            ids: [
                InspectorNumberInputId::UiOffsetX,
                InspectorNumberInputId::UiOffsetY,
            ],
            row_y: cur_y,
            decimals: 0,
            unit: "px",
        },
    );
    cur_y += row_h + row_gap;

    // 5. Row 3: Size W & H (px)
    build_dual_number_row(
        tree,
        card_id,
        ctx,
        DualNumberRowParams {
            label: "Size",
            prefixes: ["W: ", "H: "],
            values: [size[0], size[1]],
            ids: [
                InspectorNumberInputId::UiSizeW,
                InspectorNumberInputId::UiSizeH,
            ],
            row_y: cur_y,
            decimals: 0,
            unit: "px",
        },
    );
    cur_y += row_h + row_gap;

    // 6. Row 4: Pivot X & Y (0.0 .. 1.0)
    build_dual_number_row(
        tree,
        card_id,
        ctx,
        DualNumberRowParams {
            label: "Pivot",
            prefixes: ["X: ", "Y: "],
            values: [pivot[0], pivot[1]],
            ids: [
                InspectorNumberInputId::UiPivotX,
                InspectorNumberInputId::UiPivotY,
            ],
            row_y: cur_y,
            decimals: 2,
            unit: "",
        },
    );
    cur_y += row_h + row_gap;

    // 7. Row 5: Z-Index and Alpha
    build_dual_number_row(
        tree,
        card_id,
        ctx,
        DualNumberRowParams {
            label: "Layer",
            prefixes: ["Z: ", "α: "],
            values: [z_index as f32, alpha],
            ids: [
                InspectorNumberInputId::UiZIndex,
                InspectorNumberInputId::UiAlpha,
            ],
            row_y: cur_y,
            decimals: 2,
            unit: "",
        },
    );
    cur_y += row_h + row_gap;

    // 8. Row 6: Visibility Checkbox
    super::components::physics::helpers::render_checkbox_row(
        tree,
        card_id,
        ctx,
        "Visible",
        ComponentCheckboxId::UiVisible,
        visible,
        cur_y,
    );

    card_h
}

/// Helper function to build a 2-axis numeric input row (e.g. Offset X/Y, Size W/H).
fn build_dual_number_row(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    params: DualNumberRowParams<'_>,
) {
    let padding_x = 8.0;
    let label_w = if ctx.card_w < 220.0 { 40.0 } else { 50.0 };
    let gap = 4.0;
    let total_content_w = ctx.card_w - padding_x * 2.0;
    let available_w = (total_content_w - label_w - 4.0).max(60.0);
    let box_w = ((available_w - gap) * 0.5).max(28.0);
    let box_h = 20.0;

    // Label
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("Ui_{}Label", params.label));
        node.set_text(params.label);
        node.font_size = 11.0;
        node.line_height = box_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding_x, params.row_y, label_w, box_h);
    }
    let _ = tree.add_child(parent_id, lbl_id);

    let mut cur_box_x = ctx.base_x + padding_x + label_w + 4.0;

    for (i, prefix) in params.prefixes.iter().enumerate() {
        let input_id = params.ids[i];
        let val = params.values[i];
        let box_rect = Rect::new(cur_box_x, params.row_y, box_w, box_h);

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
                    Color::rgba(0.0, 0.85, 1.0, 0.95), // Active cyan ring
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
                .border_radius(4.0);
        }
        let _ = tree.add_child(parent_id, box_node_id);

        let txt_node_id = tree.create_node();
        if let Some(node) = tree.get_mut(txt_node_id) {
            node.set_name(format!("NumVal_{:?}", input_id));
            let display_str = if is_editing {
                let buf = ctx.params.active_number_input.map(|(_, b)| b).unwrap_or("");
                if ctx.params.blink_caret {
                    format!("{}{}|", prefix, buf)
                } else {
                    format!("{}{}", prefix, buf)
                }
            } else if params.unit.is_empty() {
                format!("{}{:.prec$}", prefix, val, prec = params.decimals)
            } else {
                format!(
                    "{}{:.prec$} {}",
                    prefix,
                    val,
                    params.unit,
                    prec = params.decimals
                )
            };
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
        let _ = tree.add_child(box_node_id, txt_node_id);

        let (min_val, max_val) = match input_id {
            InspectorNumberInputId::UiAlpha => (0.0, 1.0),
            InspectorNumberInputId::UiPivotX | InspectorNumberInputId::UiPivotY => (0.0, 1.0),
            InspectorNumberInputId::UiSizeW | InspectorNumberInputId::UiSizeH => (1.0, 10_000.0),
            _ => (-10_000.0, 10_000.0),
        };

        ctx.targets
            .number_inputs
            .push((input_id, box_rect, min_val, max_val, val));

        cur_box_x += box_w + 4.0;
    }
}