// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Common UI Widget and Card Render Helpers
//!
//! Provides reusable helper routines for drawing headers, numeric input pills,
//! dropdown comboboxes, and checkboxes inside Iris UI Inspector component cards.

use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
use crate::ui::iris_bridge::inspector::types::{
    ComboboxRowParams, ComboboxWithButtonParams, CompactNumericRowParams, ComponentCheckboxId,
    InspectorNumberInputId,
};
use irisui::prelude::*;

/// Helper function rendering a standard component card header with an icon, title, and trash button.
pub fn render_component_header(
    tree: &mut UiTree,
    card_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    icon: &'static str,
    display_title: &'static str,
    header_color: Color,
    component_name: &'static str,
) {
    let padding = 8.0;
    let del_btn_size = 18.0; // Compact square pill matching Image 2
    let hdr_rect = Rect::new(
        ctx.base_x + padding,
        ctx.base_y + padding,
        ctx.card_w - padding * 2.0 - del_btn_size - 4.0,
        20.0,
    );

    // Title Node
    let hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(hdr_id) {
        node.set_name(format!("Header_{}", component_name));
        node.set_text(format!("{} {}", icon, display_title));
        node.font_size = 11.5;
        node.line_height = 20.0;
        node.text_color = header_color;
        node.computed_rect = hdr_rect;
    }
    let _ = tree.add_child(card_id, hdr_id);

    // Component Trash / Delete Button (Dark slate badge matching Image 2)
    let del_rect = Rect::new(
        ctx.base_x + ctx.card_w - padding - del_btn_size,
        ctx.base_y + padding,
        del_btn_size,
        del_btn_size,
    );
    let is_del_hovered = del_rect.contains_point(ctx.params.cursor_pos);

    let del_id = tree.create_node();
    if let Some(node) = tree.get_mut(del_id) {
        node.set_name(format!("DelBtn_{}", component_name));
        node.computed_rect = del_rect;
        node.set_text("🗑");
        node.font_size = 10.5;
        node.line_height = del_btn_size;
        node.text_align = TextAlign::Center;
        let (bg, border, txt_col) = if is_del_hovered {
            (
                Color::rgba(0.35, 0.10, 0.10, 0.95),
                Color::rgba(0.70, 0.18, 0.18, 0.85),
                Color::rgba(1.0, 0.40, 0.40, 1.0),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.54, 0.56, 0.60, 1.0),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
        node.text_color = txt_col;
    }
    let _ = tree.add_child(card_id, del_id);

    ctx.targets
        .component_delete_btns
        .push((component_name, del_rect));
}

/// Helper function rendering a compact numeric input row with optional unit suffix.
pub fn render_numeric_row_compact(
    tree: &mut UiTree,
    card_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    params: CompactNumericRowParams,
) {
    let padding = 8.0;
    let row_h = 22.0;

    // Label
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("NumLbl_{:?}", params.input_id));
        node.set_text(params.label);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding, params.row_y, params.label_w, row_h);
    }
    let _ = tree.add_child(card_id, lbl_id);

    // Pill Box
    let box_rect = Rect::new(
        ctx.base_x + padding + params.label_w + 4.0,
        params.row_y,
        params.box_w,
        row_h,
    );
    let is_editing = match ctx.params.active_number_input {
        Some((id, _)) => id == params.input_id,
        None => false,
    };
    let is_hovered = box_rect.contains_point(ctx.params.cursor_pos);

    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name(format!("NumPill_{:?}", params.input_id));
        node.computed_rect = box_rect;
        let (bg, border) = if is_editing {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95), // Clean neutral active ring
            )
        } else if is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, box_id);

    // Value Text
    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name(format!("NumVal_{:?}", params.input_id));
        let display_str = if is_editing {
            let buf = ctx.params.active_number_input.map(|(_, b)| b).unwrap_or("");
            if ctx.params.blink_caret {
                format!("{}|", buf)
            } else {
                buf.to_string()
            }
        } else if params.input_id == InspectorNumberInputId::CharacterMaxSlope {
            format!("{:.0}°", params.val)
        } else if params.input_id == InspectorNumberInputId::ActionSpeedRange {
            format!("{:.0}", params.val)
        } else {
            format!("{:.2}", params.val)
        };
        node.set_text(display_str);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_editing {
            Color::WHITE
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };
        node.computed_rect = box_rect;
    }
    let _ = tree.add_child(box_id, txt_id);

    let (min_val, max_val) = match params.input_id {
        InspectorNumberInputId::VelocityX
        | InspectorNumberInputId::VelocityY
        | InspectorNumberInputId::VelocityZ
        | InspectorNumberInputId::ColliderCenterY
        | InspectorNumberInputId::CharacterCenterY
        | InspectorNumberInputId::PosX
        | InspectorNumberInputId::PosY
        | InspectorNumberInputId::PosZ => (-10_000.0, 10_000.0),
        InspectorNumberInputId::RotX
        | InspectorNumberInputId::RotY
        | InspectorNumberInputId::RotZ => (-36000.0, 36000.0),
        InspectorNumberInputId::CharacterMaxSlope => (0.0, 90.0),
        InspectorNumberInputId::ColliderRestitution
        | InspectorNumberInputId::PhysMatRestitution => (0.0, 1.0),
        _ => (0.0, 10_000.0),
    };

    ctx.targets
        .number_inputs
        .push((params.input_id, box_rect, min_val, max_val, params.val));

    // Optional Suffix Unit (e.g. `m/s`, `s`, `°`)
    if let Some(unit_str) = params.unit {
        let unit_x = box_rect.right() + 4.0;
        let unit_rect = Rect::new(unit_x, params.row_y, 35.0, row_h);
        let unit_id = tree.create_node();
        if let Some(node) = tree.get_mut(unit_id) {
            node.set_name(format!("NumUnit_{:?}", params.input_id));
            node.set_text(unit_str);
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = unit_rect;
        }
        let _ = tree.add_child(card_id, unit_id);
    }
}

/// Helper function rendering a single-line numeric input row.
pub fn render_numeric_row(
    tree: &mut UiTree,
    card_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    label: &'static str,
    input_id: InspectorNumberInputId,
    val: f32,
    row_y: f32,
) {
    let padding = 8.0;
    let row_h = 24.0;
    let label_w = (ctx.card_w * 0.45).clamp(80.0, 140.0);
    let box_w = (ctx.card_w - padding * 2.0 - label_w - 6.0).max(40.0);

    // Label
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("NumLbl_{:?}", input_id));
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding, row_y, label_w, row_h);
    }
    let _ = tree.add_child(card_id, lbl_id);

    // Box
    let box_rect = Rect::new(ctx.base_x + padding + label_w + 6.0, row_y, box_w, row_h);
    let is_editing = match ctx.params.active_number_input {
        Some((id, _)) => id == input_id,
        None => false,
    };
    let is_hovered = box_rect.contains_point(ctx.params.cursor_pos);

    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name(format!("NumBox_{:?}", input_id));
        node.computed_rect = box_rect;
        let (bg, border) = if is_editing {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95),
            )
        } else if is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, box_id);

    // Value Text
    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name(format!("NumVal_{:?}", input_id));
        let display_str = if is_editing {
            if let Some((_, ref buf)) = ctx.params.active_number_input {
                format!("{}|", buf)
            } else {
                format!("{:.2}", val)
            }
        } else {
            format!("{:.2}", val)
        };
        node.set_text(display_str);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_editing {
            Color::WHITE
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };
        node.computed_rect = box_rect;
    }
    let _ = tree.add_child(box_id, txt_id);

    let (min_val, max_val) = match input_id {
        InspectorNumberInputId::VelocityX
        | InspectorNumberInputId::VelocityY
        | InspectorNumberInputId::VelocityZ
        | InspectorNumberInputId::ColliderCenterY
        | InspectorNumberInputId::CharacterCenterY
        | InspectorNumberInputId::PosX
        | InspectorNumberInputId::PosY
        | InspectorNumberInputId::PosZ => (-10_000.0, 10_000.0),
        InspectorNumberInputId::RotX
        | InspectorNumberInputId::RotY
        | InspectorNumberInputId::RotZ => (-36000.0, 36000.0),
        InspectorNumberInputId::CharacterMaxSlope => (0.0, 90.0),
        InspectorNumberInputId::ColliderRestitution
        | InspectorNumberInputId::PhysMatRestitution => (0.0, 1.0),
        _ => (0.0, 10_000.0),
    };

    ctx.targets
        .number_inputs
        .push((input_id, box_rect, min_val, max_val, val));
}

/// Helper function rendering a standard compact dropdown combobox (e.g. `Shape: [ Capsule ▼ ]`).
pub fn render_combobox_row(
    tree: &mut UiTree,
    card_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    params: ComboboxRowParams,
) {
    let padding = 8.0;
    let row_h = 22.0;
    let combo_w = 88.0; // Compact combobox width matching Image 2!

    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("ComboLbl_{:?}", params.dropdown_id));
        node.set_text(params.label);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding, params.row_y, params.label_w, row_h);
    }
    let _ = tree.add_child(card_id, lbl_id);

    let combo_rect = Rect::new(
        ctx.base_x + padding + params.label_w + 4.0,
        params.row_y,
        combo_w,
        row_h,
    );
    let is_open = ctx.params.active_dropdown == Some(params.dropdown_id);
    let is_hovered = combo_rect.contains_point(ctx.params.cursor_pos);

    let combo_node_id = tree.create_node();
    if let Some(node) = tree.get_mut(combo_node_id) {
        node.set_name(format!("ComboboxPill_{:?}", params.dropdown_id));
        node.computed_rect = combo_rect;
        let (bg, border) = if is_open {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95), // Clean neutral active ring
            )
        } else if is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, combo_node_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name(format!("ComboboxPillTxt_{:?}", params.dropdown_id));
        let arrow = if is_open { "▲" } else { "▼" };
        node.set_text(format!("{}  {}", params.selected_text, arrow));
        node.font_size = 10.5;
        node.line_height = row_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_open {
            Color::WHITE
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };
        node.computed_rect = combo_rect;
    }
    let _ = tree.add_child(combo_node_id, txt_id);

    ctx.targets
        .dropdowns
        .push((params.dropdown_id, combo_rect, 0));
}

/// Helper function rendering a dropdown combobox with a side action button (e.g. `[ ↺ Preset ]`).
pub fn render_combobox_row_with_btn(
    tree: &mut UiTree,
    card_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    params: ComboboxWithButtonParams,
) {
    let padding = 8.0;
    let row_h = 22.0;
    let label_w = 85.0;
    let combo_w = 80.0;
    let btn_w = 58.0;

    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("ComboWithBtnLbl_{:?}", params.dropdown_id));
        node.set_text(params.label);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding, params.row_y, label_w, row_h);
    }
    let _ = tree.add_child(card_id, lbl_id);

    // Combobox Pill
    let combo_rect = Rect::new(
        ctx.base_x + padding + label_w + 4.0,
        params.row_y,
        combo_w,
        row_h,
    );
    let is_open = ctx.params.active_dropdown == Some(params.dropdown_id);
    let is_hovered = combo_rect.contains_point(ctx.params.cursor_pos);

    let combo_node_id = tree.create_node();
    if let Some(node) = tree.get_mut(combo_node_id) {
        node.set_name(format!("ComboboxPill_{:?}", params.dropdown_id));
        node.computed_rect = combo_rect;
        let (bg, border) = if is_open {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95),
            )
        } else if is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, combo_node_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name(format!("ComboboxPillTxt_{:?}", params.dropdown_id));
        let arrow = if is_open { "▲" } else { "▼" };
        node.set_text(format!("{}  {}", params.selected_text, arrow));
        node.font_size = 10.5;
        node.line_height = row_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_open {
            Color::WHITE
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };
        node.computed_rect = combo_rect;
    }
    let _ = tree.add_child(combo_node_id, txt_id);

    ctx.targets
        .dropdowns
        .push((params.dropdown_id, combo_rect, 0));

    // Preset Action Button `[ ↺ Preset ]`
    let btn_rect = Rect::new(combo_rect.right() + 6.0, params.row_y, btn_w, row_h);
    let is_btn_hovered = btn_rect.contains_point(ctx.params.cursor_pos);

    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("PresetResetBtn");
        node.computed_rect = btn_rect;
        let (bg, border, txt_col) = if is_btn_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.82, 0.84, 0.88, 1.0),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
        node.set_text(params.btn_label);
        node.font_size = 10.5;
        node.line_height = row_h;
        node.text_align = TextAlign::Center;
        node.text_color = txt_col;
    }
    let _ = tree.add_child(card_id, btn_id);

    ctx.targets.preset_btn_rect = Some(btn_rect);
}

/// Helper function rendering a boolean checkbox row.
pub fn render_checkbox_row(
    tree: &mut UiTree,
    card_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
    label: &'static str,
    cb_id: ComponentCheckboxId,
    is_checked: bool,
    row_y: f32,
) {
    let padding = 8.0;
    let row_h = 20.0;
    let box_size = 14.0;
    let cb_rect = Rect::new(ctx.base_x + padding, row_y + 3.0, box_size, box_size);
    let is_hovered = cb_rect.contains_point(ctx.params.cursor_pos);

    let cb_node_id = tree.create_node();
    if let Some(node) = tree.get_mut(cb_node_id) {
        node.set_name(format!("CheckboxPill_{:?}", cb_id));
        node.computed_rect = cb_rect;
        let (bg, border) = if is_checked {
            (
                Color::rgba(0.20, 0.28, 0.38, 1.0),
                Color::rgba(0.40, 0.55, 0.75, 0.95),
            )
        } else if is_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(3.0);
        if is_checked {
            node.set_text("✓");
            node.font_size = 10.0;
            node.line_height = box_size;
            node.text_align = TextAlign::Center;
            node.text_color = Color::WHITE;
        }
    }
    let _ = tree.add_child(card_id, cb_node_id);

    // Label
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name(format!("CheckboxLbl_{:?}", cb_id));
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = row_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(
            cb_rect.right() + 8.0,
            row_y,
            ctx.card_w - padding * 2.0 - box_size - 10.0,
            row_h,
        );
    }
    let _ = tree.add_child(card_id, lbl_id);

    ctx.targets.checkboxes.push((cb_id, cb_rect, is_checked));
}