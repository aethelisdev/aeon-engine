// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Entity Name and Identity Header Builder
//!
//! Renders the top entity rename input box, entity icon, and UUID badge.

use super::types::{InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;

/// Output node handles created during Inspector header construction.
pub struct HeaderNodes {
    /// Name input container node ID.
    pub name_box_id: WidgetId,
    /// Name text node ID.
    pub name_text_id: WidgetId,
}

/// Builds the top Entity Name header and returns the computed height.
pub fn build_entity_header(
    tree: &mut UiTree,
    parent_id: WidgetId,
    entity: hecs::Entity,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
    start_y: f32,
) -> (f32, HeaderNodes) {
    let padding_x = 6.0;
    let header_h = 24.0;
    let header_w = params.panel_rect.width - padding_x * 2.0;
    let base_x = params.panel_rect.x + padding_x;

    let header_rect = Rect::new(base_x, start_y, header_w, header_h);

    // Header Container
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("EntityHeaderCard");
        node.computed_rect = header_rect;
    }
    let _ = tree.add_child(parent_id, card_id);

    // Label "🏷 Name:"
    let lbl_w = 60.0;
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("NamePrefixLabel");
        node.set_text("🏷 Name:");
        node.font_size = 11.0;
        node.line_height = header_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(base_x + 2.0, start_y, lbl_w, header_h);
    }
    let _ = tree.add_child(card_id, lbl_id);

    // Name Input Box
    let input_x = base_x + lbl_w + 4.0;
    let input_w = (header_w - lbl_w - 6.0).max(80.0);
    let input_rect = Rect::new(input_x, start_y, input_w, header_h);
    targets.name_input_rect = input_rect;

    let is_editing = params.active_rename_buffer.is_some();
    let is_hovered = input_rect.contains_point(params.cursor_pos);

    let name_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_box_id) {
        node.set_name("NameInputBox");
        node.computed_rect = input_rect;
        let (bg, border_col) = if is_editing {
            (
                Color::rgba(0.086, 0.090, 0.106, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95), // Clean neutral active ring
            )
        } else if is_hovered {
            (
                Color::rgba(0.125, 0.133, 0.149, 0.98),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.106, 0.110, 0.125, 0.98),
                Color::rgba(0.173, 0.180, 0.208, 0.85),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border_col)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, name_box_id);

    // Fetch entity name from ECS or editing buffer
    let entity_name = if let Some(buf) = params.active_rename_buffer {
        buf.to_string()
    } else {
        params
            .world
            .get::<&ae_core::ecs::Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| format!("Entity {:?}", entity))
    };

    let display_text = if is_editing {
        if params.blink_caret {
            format!("{}|", entity_name)
        } else {
            entity_name
        }
    } else {
        entity_name
    };

    let name_text_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_text_id) {
        node.set_name("NameInputText");
        node.set_text(display_text);
        node.font_size = 11.5;
        node.line_height = header_h;
        node.text_color = if is_editing {
            Color::WHITE
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };
        node.computed_rect = Rect::new(input_x + 8.0, start_y, input_w - 16.0, header_h);
    }
    let _ = tree.add_child(name_box_id, name_text_id);

    (
        header_h + 6.0,
        HeaderNodes {
            name_box_id,
            name_text_id,
        },
    )
}