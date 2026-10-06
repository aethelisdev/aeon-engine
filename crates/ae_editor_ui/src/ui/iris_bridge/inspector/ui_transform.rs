// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen Transform (RectTransform) Component Inspector Card
//!
//! Renders the 2D layout properties of `UiElement` using declarative `UiScope`:
//! - Anchor Presets (interactive ComboBox dropdown)
//! - Screen Offset X and Y (precision drag/number input boxes)
//! - Size Width and Height (precision drag/number input boxes)
//! - Pivot X and Y (center point alignment)
//! - Z-Index (layer ordering) and Alpha (opacity multiplier)
//! - Visibility toggle checkbox
//!

use super::components::physics::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, build_declarative_card_header,
    render_declarative_checkbox_row, render_declarative_combobox_row,
};
use super::registry::ComponentRenderContext;
use super::types::{ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId};
use ae_core::ecs::{UiAnchor, UiElement};
use irisui::prelude::*;

/// Descriptor parameters for rendering a declarative dual-axis numeric row.
struct DeclarativeDualNumberRowParams<'a> {
    /// Human-readable label displayed on the left column.
    label: &'static str,
    /// Prefix tags displayed inside the two boxes (e.g. `["X: ", "Y: "]`).
    prefixes: [&'static str; 2],
    /// Current scalar values for axis 0 and 1.
    values: [f32; 2],
    /// Distinct input identifier IDs for axis 0 and 1.
    ids: [InspectorNumberInputId; 2],
    /// Decimal precision when formatting idle display values.
    decimals: usize,
    /// Optional physical unit suffix string (e.g. `"px"`).
    unit: &'a str,
}

/// Builds the `📐 2D Screen Transform` card inside the declarative [`UiScope`].
pub fn build_ui_transform_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
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

    let is_anchor_open = ctx.params.active_dropdown == Some(InspectorDropdownId::UiAnchor);

    let card_style = Style::new()
        .flex_col()
        .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
        .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
        .border_radius(6.0)
        .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
        .gap(4.0);

    let label_w = if ctx.card_w < 220.0 { 42.0 } else { 52.0 };

    scope.container_named("UiTransformCard", card_style, |card| {
        build_declarative_card_header(
            card,
            ComponentHeaderProps {
                atlas_icon: None,
                icon: "📐",
                display_title: "2D Screen Transform",
                header_color: Color::rgba(0.0, 0.85, 1.0, 1.0),
                component_name: "UiElement",
            },
            false,
        );

        // Row 1: Anchor Preset Selector Dropdown
        render_declarative_combobox_row(
            card,
            DeclarativeComboboxRowParams {
                dropdown_id: InspectorDropdownId::UiAnchor,
                label: Some("Anchor"),
                label_w,
                selected_text: anchor_str,
                is_open: is_anchor_open,
                is_hovered: false,
                combo_w: 90.0,
            },
        );

        // Row 2: Offset X & Y (px)
        render_declarative_dual_number_row(
            card,
            ctx,
            label_w,
            DeclarativeDualNumberRowParams {
                label: "Offset",
                prefixes: ["X: ", "Y: "],
                values: [offset[0], offset[1]],
                ids: [
                    InspectorNumberInputId::UiOffsetX,
                    InspectorNumberInputId::UiOffsetY,
                ],
                decimals: 0,
                unit: "px",
            },
        );

        // Row 3: Size W & H (px)
        render_declarative_dual_number_row(
            card,
            ctx,
            label_w,
            DeclarativeDualNumberRowParams {
                label: "Size",
                prefixes: ["W: ", "H: "],
                values: [size[0], size[1]],
                ids: [
                    InspectorNumberInputId::UiSizeW,
                    InspectorNumberInputId::UiSizeH,
                ],
                decimals: 0,
                unit: "px",
            },
        );

        // Row 4: Pivot X & Y (0.0 .. 1.0)
        render_declarative_dual_number_row(
            card,
            ctx,
            label_w,
            DeclarativeDualNumberRowParams {
                label: "Pivot",
                prefixes: ["X: ", "Y: "],
                values: [pivot[0], pivot[1]],
                ids: [
                    InspectorNumberInputId::UiPivotX,
                    InspectorNumberInputId::UiPivotY,
                ],
                decimals: 2,
                unit: "",
            },
        );

        // Row 5: Z-Index (Layer) and Alpha
        render_declarative_dual_number_row(
            card,
            ctx,
            label_w,
            DeclarativeDualNumberRowParams {
                label: "Layer",
                prefixes: ["Z: ", "α: "],
                values: [z_index as f32, alpha],
                ids: [
                    InspectorNumberInputId::UiZIndex,
                    InspectorNumberInputId::UiAlpha,
                ],
                decimals: 2,
                unit: "",
            },
        );

        // Row 6: Visibility Checkbox
        render_declarative_checkbox_row(
            card,
            ComponentCheckboxId::UiVisible,
            "Visible",
            visible,
            false,
        );
    });
}

/// Helper function to build a declarative 2-axis numeric input row (e.g. Offset X/Y, Size W/H).
fn render_declarative_dual_number_row(
    scope: &mut UiScope<'_>,
    ctx: &ComponentRenderContext<'_>,
    label_w: f32,
    params: DeclarativeDualNumberRowParams<'_>,
) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(20.0)
        .gap(4.0);

    scope.container_named("DualNumRow", row_style, |row| {
        row.label_styled_passive(
            "UiRowLabel",
            params.label,
            11.0,
            Color::rgba(0.620, 0.635, 0.678, 1.0),
            TextAlign::Left,
            Style::new().width(label_w).height(20.0),
        );

        for (i, prefix) in params.prefixes.iter().enumerate() {
            let input_id = params.ids[i];
            let val = params.values[i];

            let editing_state = ctx.params.active_number_input.filter(|s| s.id == input_id);
            let is_editing = editing_state.is_some();
            let is_all_sel = editing_state.map(|s| s.is_all_selected).unwrap_or(false);

            let (bg, border_col) = if is_editing {
                if is_all_sel {
                    (
                        Color::rgba(0.14, 0.46, 0.88, 0.95),
                        Color::rgba(0.0, 0.80, 1.00, 0.95),
                    )
                } else {
                    (
                        Color::rgba(0.118, 0.125, 0.145, 1.0),
                        Color::rgba(0.0, 0.80, 1.00, 0.95),
                    )
                }
            } else {
                (
                    Color::rgba(0.125, 0.133, 0.153, 0.98),
                    Color::rgba(0.180, 0.192, 0.227, 0.85),
                )
            };

            let display_str = if let Some(s) = editing_state {
                let buf = s.buffer;
                let cursor = s.cursor_idx.min(buf.len());
                let (left, right) = buf.split_at(cursor);
                if s.is_all_selected {
                    format!("{}{}", prefix, buf)
                } else if ctx.params.blink_caret {
                    format!("{}{}|{}", prefix, left, right)
                } else {
                    format!("{}{}{}", prefix, left, right)
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

            let box_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .flex_grow(1.0)
                .height(20.0)
                .background(bg)
                .border(1.0, border_col)
                .border_radius(4.0);

            let input_tag = super::tags::encode_inspector_number_input_tag(input_id);
            row.container_tagged(
                "NumBox",
                box_style,
                WidgetRole::TextInput,
                input_tag,
                |tb| {
                    tb.label_styled_passive(
                        "NumVal",
                        &display_str,
                        10.5,
                        if is_editing {
                            Color::WHITE
                        } else {
                            Color::rgba(0.886, 0.894, 0.918, 1.0)
                        },
                        TextAlign::Center,
                        Style::new().flex_grow(1.0).height(20.0),
                    );
                },
            );
        }
    });
}