// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Helper Builders
//!
//! Reusable widget builders for section headers, checkboxes, range sliders, and dropdown rows.

use super::super::super::types::PreferencesTargets;
use super::types::{CheckboxParams, DropdownRowParams, SectionHeaderParams, SliderRowParams};
use irisui::prelude::*;

/// Parameters for building a labeled collapsible section header.
pub fn build_section_header(
    tree: &mut UiTree,
    parent_id: WidgetId,
    p: SectionHeaderParams<'_>,
    targets: &mut PreferencesTargets,
) {
    let header_rect = Rect::new(p.base_x + 8.0, p.y + 6.0, p.width - 16.0, 24.0);
    let is_hovered = header_rect.contains_point(p.cursor_pos);
    targets.section_toggles.push((p.section_id, header_rect));

    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("SectionTitle");
        let arrow = if p.is_collapsed { "▸" } else { "▾" };
        node.set_text(format!("{}  {}", arrow, p.title));
        node.font_size = 13.0;
        node.line_height = 24.0;
        node.text_color = if is_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.91, 0.96, 1.0)
        };
        node.computed_rect = header_rect;
    }
    let _ = tree.add_child(parent_id, title_id);
}

/// Helper to render an interactive checkbox toggle.
pub fn build_checkbox(
    tree: &mut UiTree,
    parent_id: WidgetId,
    p: CheckboxParams<'_>,
    targets: &mut PreferencesTargets,
) {
    let is_hovered = p.rect.contains_point(p.cursor_pos);
    let box_rect = Rect::new(
        p.rect.x,
        p.rect.y + (p.rect.height - 16.0) * 0.5,
        16.0,
        16.0,
    );

    let box_node = tree.create_node();
    if let Some(node) = tree.get_mut(box_node) {
        node.set_name("CheckboxBox");
        node.computed_rect = box_rect;
        let bg = if p.is_checked {
            Color::rgba(0.0, 0.70, 0.85, 1.0)
        } else if is_hovered {
            Color::rgba(0.18, 0.20, 0.28, 1.0)
        } else {
            Color::rgba(0.11, 0.12, 0.16, 1.0)
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, Color::rgba(0.25, 0.30, 0.42, 1.0))
            .border_radius(3.0);
    }
    let _ = tree.add_child(parent_id, box_node);

    if p.is_checked {
        let chk_node = tree.create_node();
        if let Some(node) = tree.get_mut(chk_node) {
            node.set_name("CheckMark");
            node.set_text("✓");
            node.font_size = 11.0;
            node.line_height = 16.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::rgba(0.05, 0.06, 0.08, 1.0);
            node.computed_rect = box_rect;
        }
        let _ = tree.add_child(box_node, chk_node);
    }

    let lbl_node = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_node) {
        node.set_name("CheckboxLabel");
        node.set_text(p.label);
        node.font_size = 12.0;
        node.line_height = p.rect.height;
        node.text_color = if is_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.80, 0.83, 0.90, 1.0)
        };
        node.computed_rect = Rect::new(
            p.rect.x + 24.0,
            p.rect.y,
            p.rect.width - 24.0,
            p.rect.height,
        );
    }
    let _ = tree.add_child(parent_id, lbl_node);

    targets.toggles.push((p.toggle_id, p.rect));
}

/// Helper to render a continuous interactive numerical slider.
pub fn build_slider_row(
    tree: &mut UiTree,
    parent_id: WidgetId,
    p: SliderRowParams<'_>,
    targets: &mut PreferencesTargets,
) {
    let lbl_w = 170.0;
    let val_box_w = 64.0;
    let val_box_h = 22.0;
    let track_w = p.width - lbl_w - val_box_w - 16.0;

    // Label
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("SliderLabel");
        node.set_text(p.label);
        node.font_size = 11.5;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.75, 0.78, 0.85, 1.0);
        node.computed_rect = Rect::new(p.base_x, p.y, lbl_w, 20.0);
    }
    let _ = tree.add_child(parent_id, lbl_id);

    // Track
    let track_rect = Rect::new(p.base_x + lbl_w, p.y + 8.0, track_w, 4.0);
    let is_track_hovered = Rect::new(track_rect.x, p.y, track_w, 20.0).contains_point(p.cursor_pos);

    let track_id = tree.create_node();
    if let Some(node) = tree.get_mut(track_id) {
        node.set_name("SliderTrack");
        node.computed_rect = track_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.90))
            .border(1.0, Color::rgba(0.18, 0.21, 0.30, 0.50))
            .border_radius(2.0);
    }
    let _ = tree.add_child(parent_id, track_id);

    // Fill & Thumb
    let norm = ((p.current_val - p.min_val) / (p.max_val - p.min_val)).clamp(0.0, 1.0);
    let fill_w = (track_w * norm).max(2.0);
    let fill_rect = Rect::new(track_rect.x, track_rect.y, fill_w, 4.0);

    let fill_id = tree.create_node();
    if let Some(node) = tree.get_mut(fill_id) {
        node.set_name("SliderFill");
        node.computed_rect = fill_rect;
        node.style = Style::new()
            .background(Color::rgba(0.0, 0.72, 0.88, 0.95))
            .border_radius(2.0);
    }
    let _ = tree.add_child(track_id, fill_id);

    let thumb_x = (track_rect.x + track_w * norm - 4.0)
        .clamp(track_rect.x - 2.0, track_rect.x + track_w - 6.0);
    let thumb_rect = Rect::new(thumb_x, p.y + 3.0, 8.0, 14.0);
    let thumb_id = tree.create_node();
    if let Some(node) = tree.get_mut(thumb_id) {
        node.set_name("SliderThumb");
        node.computed_rect = thumb_rect;
        let thumb_bg = if is_track_hovered {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.92, 0.98, 1.0)
        };
        node.style = Style::new().background(thumb_bg).border_radius(2.0);
    }
    let _ = tree.add_child(parent_id, thumb_id);

    // Number Input Box / Pill
    let val_box_rect = Rect::new(
        p.base_x + lbl_w + track_w + 8.0,
        p.y - 1.0,
        val_box_w,
        val_box_h,
    );
    let is_box_hovered = val_box_rect.contains_point(p.cursor_pos);

    let val_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(val_box_id) {
        node.set_name("SliderValBox");
        node.computed_rect = val_box_rect;
        let (bg, border_color) = if p.is_editing {
            (
                Color::rgba(0.06, 0.08, 0.12, 1.0),
                Color::rgba(0.0, 0.85, 1.0, 1.0),
            )
        } else if is_box_hovered {
            (
                Color::rgba(0.24, 0.27, 0.37, 1.0),
                Color::rgba(0.38, 0.46, 0.62, 1.0),
            )
        } else {
            (
                Color::rgba(0.18, 0.20, 0.27, 0.95),
                Color::rgba(0.26, 0.30, 0.42, 0.80),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border_color)
            .border_radius(5.0);
    }
    let _ = tree.add_child(parent_id, val_box_id);

    // Value Text inside Box
    let val_id = tree.create_node();
    if let Some(node) = tree.get_mut(val_id) {
        node.set_name("SliderVal");
        if p.is_editing {
            let cursor_str = if p.blink_caret { "|" } else { "" };
            node.set_text(format!("{}{}", p.editing_buffer, cursor_str));
            node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        } else {
            node.set_text(p.val_text);
            node.text_color = if is_box_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.90, 0.93, 0.98, 1.0)
            };
        }
        node.font_size = 11.5;
        node.line_height = val_box_h;
        node.text_align = TextAlign::Center;
        node.computed_rect = val_box_rect;
    }
    let _ = tree.add_child(val_box_id, val_id);

    targets.sliders.push((
        p.slider_id,
        Rect::new(track_rect.x, p.y, track_w, 20.0),
        p.min_val,
        p.max_val,
        p.current_val,
    ));

    targets.number_inputs.push((
        p.slider_id,
        val_box_rect,
        p.min_val,
        p.max_val,
        p.current_val,
    ));
}

/// Helper to render an interactive dropdown row.
pub fn build_dropdown_row(
    tree: &mut UiTree,
    parent_id: WidgetId,
    p: DropdownRowParams<'_>,
    targets: &mut PreferencesTargets,
) {
    let lbl_w = 170.0;
    let combo_w = 200.0;

    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("DropdownLabel");
        node.set_text(p.label);
        node.font_size = 11.5;
        node.line_height = 24.0;
        node.text_color = Color::rgba(0.75, 0.78, 0.85, 1.0);
        node.computed_rect = Rect::new(p.base_x, p.y, lbl_w, 24.0);
    }
    let _ = tree.add_child(parent_id, lbl_id);

    let combo_rect = Rect::new(p.base_x + p.width - combo_w, p.y, combo_w, 24.0);
    let is_hovered = combo_rect.contains_point(p.cursor_pos);

    let combo_id = tree.create_node();
    if let Some(node) = tree.get_mut(combo_id) {
        node.set_name("DropdownCombo");
        node.computed_rect = combo_rect;
        let (bg, border_color) = if p.is_open {
            (
                Color::rgba(0.06, 0.08, 0.12, 1.0),
                Color::rgba(0.0, 0.85, 1.0, 1.0),
            )
        } else if is_hovered {
            (
                Color::rgba(0.24, 0.27, 0.37, 1.0),
                Color::rgba(0.38, 0.46, 0.62, 1.0),
            )
        } else {
            (
                Color::rgba(0.18, 0.20, 0.27, 0.95),
                Color::rgba(0.26, 0.30, 0.42, 0.80),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border_color)
            .border_radius(5.0);
    }
    let _ = tree.add_child(parent_id, combo_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name("DropdownComboText");
        let arrow = if p.is_open { "▲" } else { "▼" };
        node.set_text(format!("{}  {}", p.selected_text, arrow));
        node.font_size = 11.5;
        node.line_height = 24.0;
        node.text_color = if p.is_open || is_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.90, 0.93, 0.98, 1.0)
        };
        node.computed_rect = Rect::new(
            combo_rect.x + 10.0,
            combo_rect.y,
            combo_rect.width - 20.0,
            24.0,
        );
    }
    let _ = tree.add_child(combo_id, txt_id);

    targets.dropdowns.push((p.dropdown_id, combo_rect));
}