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

/// Builds the `📐 2D Screen Transform` card inside the declarative [`UiScope`].
pub fn build_ui_transform_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    let (anchor, [mut ox, mut oy], [mut sw, mut sh], [mut px, mut py], z_index, mut alpha, visible) =
        ctx.world
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
    let mut z_index_f = z_index as f32;

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

    let label_w = if ctx.card_w < 220.0 { 42.0 } else { 52.0 };

    scope.card_panel("UiTransformCard", |card| {
        let accent = card.theme().accent;
        build_declarative_card_header(
            card,
            ComponentHeaderProps {
                atlas_icon: None,
                icon: "📐",
                display_title: "2D Screen Transform",
                header_color: accent,
                component_name: "UiElement",
            },
            false,
        );

        // Row 1: Anchor Preset Selector Dropdown
        render_declarative_combobox_row(
            card,
            ctx,
            DeclarativeComboboxRowParams {
                dropdown_id: InspectorDropdownId::UiAnchor,
                label: Some("Anchor"),
                label_w,
                selected_text: anchor_str,
                combo_w: 90.0,
                options: &[
                    "Top-Left",
                    "Top-Center",
                    "Top-Right",
                    "Center-Left",
                    "Center",
                    "Center-Right",
                    "Bottom-Left",
                    "Bottom-Center",
                    "Bottom-Right",
                ],
            },
        );

        // Row 2: Offset X & Y (px)
        render_declarative_dual_number_row(
            card,
            ctx,
            label_w,
            DualNumberRowDescriptor {
                label: "Offset",
                prefixes: ["X: ", "Y: "],
                values: (&mut ox, &mut oy),
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
            DualNumberRowDescriptor {
                label: "Size",
                prefixes: ["W: ", "H: "],
                values: (&mut sw, &mut sh),
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
            DualNumberRowDescriptor {
                label: "Pivot",
                prefixes: ["X: ", "Y: "],
                values: (&mut px, &mut py),
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
            DualNumberRowDescriptor {
                label: "Layer",
                prefixes: ["Z: ", "α: "],
                values: (&mut z_index_f, &mut alpha),
                ids: [
                    InspectorNumberInputId::UiZIndex,
                    InspectorNumberInputId::UiAlpha,
                ],
                decimals: 2,
                unit: "",
            },
        );

        // Row 6: Visibility Checkbox
        let resp_visible = render_declarative_checkbox_row(
            card,
            ComponentCheckboxId::UiVisible,
            "Visible",
            visible,
        );
        if resp_visible.clicked() {
            ctx.toggle_checkbox(ComponentCheckboxId::UiVisible);
        }
    });
}

/// Descriptor parameters for rendering a dual-axis numeric property row.
struct DualNumberRowDescriptor<'a> {
    /// Left column title label.
    label: &'static str,
    /// Axis prefix tags (e.g. `["X: ", "Y: "]`).
    prefixes: [&'static str; 2],
    /// Mutable references to scalar values.
    values: (&'a mut f32, &'a mut f32),
    /// Numeric input property identifiers.
    ids: [InspectorNumberInputId; 2],
    /// Decimal precision when formatting idle display values.
    decimals: usize,
    /// Physical unit suffix string (e.g. `"px"`, `""`).
    unit: &'static str,
}

/// Helper function to build a declarative 2-axis numeric input row (e.g. Offset X/Y, Size W/H).
fn render_declarative_dual_number_row(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    label_w: f32,
    desc: DualNumberRowDescriptor<'_>,
) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(20.0)
        .gap(4.0);

    scope.container_named("DualNumRow", row_style, |row| {
        row.label_styled_passive(
            "UiRowLabel",
            desc.label,
            11.0,
            row.theme().text_muted,
            TextAlign::Left,
            Style::new().width(label_w).height(20.0),
        );

        let mut render_box = |prefix: &str, input_id: InspectorNumberInputId, val: &mut f32| {
            let tag = super::tags::encode_inspector_number_input_tag(input_id);
            let edit = ctx.edit_state(input_id);
            let current = *val;

            let formatted;
            let mut drag = NumericDrag::new("", val)
                .tag(tag)
                .decimals(desc.decimals)
                .edit_state(edit);

            if edit.is_none() {
                if desc.unit.is_empty() {
                    formatted = format!("{}{:.prec$}", prefix, current, prec = desc.decimals);
                } else {
                    formatted = format!(
                        "{}{:.prec$} {}",
                        prefix,
                        current,
                        desc.unit,
                        prec = desc.decimals
                    );
                }
                drag = drag.custom_text(&formatted);
            }

            let resp = row.add(drag);
            if resp.changed() {
                ctx.set_number(input_id, *val);
            }
            if resp.committed() {
                ctx.commit_number(input_id);
            }
        };

        render_box(desc.prefixes[0], desc.ids[0], desc.values.0);
        render_box(desc.prefixes[1], desc.ids[1], desc.values.1);
    });
}