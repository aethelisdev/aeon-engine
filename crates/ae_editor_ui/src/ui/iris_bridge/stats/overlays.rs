// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Overlays Card Builder
//!
//! Renders the wireframe mode and grid visibility checkboxes in retained mode.

use super::types::{StatsPanelParams, StatsPanelTargets};
use irisui::prelude::*;

/// Node handles for the Viewport Overlays card in retained mode.
pub struct OverlaysNodes {
    /// Wireframe checkbox box node ID.
    pub wireframe_box_id: WidgetId,
    /// Wireframe checkmark text node ID.
    pub wireframe_check_id: WidgetId,
    /// Grid checkbox box node ID.
    pub grid_box_id: WidgetId,
    /// Grid checkmark text node ID.
    pub grid_check_id: WidgetId,
}

/// Builds the static Viewport Overlays card layout.
pub fn build_viewport_overlays_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
    targets: &mut StatsPanelTargets,
) -> OverlaysNodes {
    let mut cur_y = content_rect.y;

    // 1. Wireframe Mode Checkbox
    let wire_rect = Rect::new(content_rect.x, cur_y, content_rect.width, 20.0);
    let (wire_box_id, wire_check_id) =
        build_checkbox(tree, parent_id, "🕸 Wireframe Mode (Edges)", wire_rect);
    targets.wireframe_checkbox_rect = Some(wire_rect);
    cur_y += 24.0;

    // 2. Show Grid Checkbox
    let grid_rect = Rect::new(content_rect.x, cur_y, content_rect.width, 20.0);
    let (grid_box_id, grid_check_id) = build_checkbox(tree, parent_id, "🔲 Show Grid", grid_rect);
    targets.grid_checkbox_rect = Some(grid_rect);

    OverlaysNodes {
        wireframe_box_id: wire_box_id,
        wireframe_check_id: wire_check_id,
        grid_box_id,
        grid_check_id,
    }
}

/// Updates the dynamic state (checked / hover) of the checkboxes in place (0 allocations).
pub fn update_viewport_overlays_values(
    tree: &mut UiTree,
    nodes: &OverlaysNodes,
    params: &StatsPanelParams<'_>,
    targets: &StatsPanelTargets,
) {
    // 1. Wireframe Checkbox State
    if let Some(rect) = targets.wireframe_checkbox_rect {
        let is_hover = rect.contains_point(params.cursor_pos);
        update_checkbox_state(
            tree,
            nodes.wireframe_box_id,
            nodes.wireframe_check_id,
            params.wireframe_enabled,
            is_hover,
        );
    }

    // 2. Grid Checkbox State
    if let Some(rect) = targets.grid_checkbox_rect {
        let is_hover = rect.contains_point(params.cursor_pos);
        update_checkbox_state(
            tree,
            nodes.grid_box_id,
            nodes.grid_check_id,
            params.grid_enabled,
            is_hover,
        );
    }
}

/// Helper to render an initial checkbox structure.
fn build_checkbox(
    tree: &mut UiTree,
    parent_id: WidgetId,
    label: &str,
    rect: Rect,
) -> (WidgetId, WidgetId) {
    let box_size = 14.0;
    let box_rect = Rect::new(
        rect.x + 2.0,
        rect.y + (rect.height - box_size) * 0.5,
        box_size,
        box_size,
    );

    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("CheckboxBox");
        node.computed_rect = box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.10, 0.11, 0.15, 0.90))
            .border(1.0, Color::rgba(0.24, 0.28, 0.38, 0.70))
            .border_radius(3.0);
    }
    let _ = tree.add_child(parent_id, box_id);

    let check_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(check_txt_id) {
        node.set_name("CheckmarkText");
        node.set_text("✓");
        node.font_size = 10.0;
        node.line_height = box_size;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        node.computed_rect = box_rect;
        node.visible = false;
    }
    let _ = tree.add_child(box_id, check_txt_id);

    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("CheckboxLabel");
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_color = Color::rgba(0.85, 0.88, 0.95, 1.0);
        node.computed_rect = Rect::new(
            rect.x + box_size + 8.0,
            rect.y,
            rect.width - box_size - 8.0,
            rect.height,
        );
    }
    let _ = tree.add_child(parent_id, lbl_id);

    (box_id, check_txt_id)
}

/// Helper to update checkbox style and checkmark visibility.
fn update_checkbox_state(
    tree: &mut UiTree,
    box_id: WidgetId,
    check_id: WidgetId,
    is_checked: bool,
    is_hover: bool,
) {
    if let Some(node) = tree.get_mut(box_id) {
        let (bg, border) = if is_checked {
            (
                Color::rgba(0.0, 0.55, 0.75, 1.0),
                Color::rgba(0.0, 0.85, 1.0, 1.0),
            )
        } else if is_hover {
            (
                Color::rgba(0.18, 0.20, 0.27, 1.0),
                Color::rgba(0.38, 0.44, 0.58, 1.0),
            )
        } else {
            (
                Color::rgba(0.10, 0.11, 0.15, 0.90),
                Color::rgba(0.24, 0.28, 0.38, 0.70),
            )
        };
        node.style.background_color = bg;
        node.style.border.color = border;
    }
    if let Some(node) = tree.get_mut(check_id) {
        node.visible = is_checked;
    }
}