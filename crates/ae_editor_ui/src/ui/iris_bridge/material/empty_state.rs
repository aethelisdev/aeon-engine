// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Empty State Builders
//!
//! Provides informational placeholders when no entity is selected or when the
//! selected entity lacks renderable geometry (neither ModelId nor SpriteId).
//!

use super::types::MaterialPanelTargets;
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_PLUS, ICON_WORLD};
use irisui::prelude::*;

/// Builds an empty-state placeholder informing the user to select an entity.
pub fn build_no_entity_selected(tree: &mut UiTree, parent_id: WidgetId, panel_rect: Rect) {
    let card_w = (panel_rect.width - 32.0).clamp(180.0, 360.0);
    let card_h = 130.0;
    let card_x = panel_rect.x + (panel_rect.width - card_w) * 0.5;
    let card_y = panel_rect.y + 40.0 + (panel_rect.height - 40.0 - card_h) * 0.35;

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("MaterialEmptyCard");
        node.computed_rect = Rect::new(card_x, card_y, card_w, card_h);
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.12, 0.95))
            .border(1.0, Color::rgba(0.18, 0.20, 0.25, 0.80))
            .border_radius(8.0);
    }
    let _ = tree.add_child(parent_id, card_id);

    // Icon: ICON_WORLD
    let icon_size = 28.0;
    let icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("MaterialEmptyIcon");
        node.computed_rect = Rect::new(
            card_x + (card_w - icon_size) * 0.5,
            card_y + 16.0,
            icon_size,
            icon_size,
        );
        node.set_texture_uv(ICON_WORLD);
        node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.65));
    }
    let _ = tree.add_child(card_id, icon_id);

    // Primary Message
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("MaterialEmptyTitle");
        node.set_text("No Entity Selected");
        node.font_size = 12.0;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.90, 0.92, 0.95, 1.0);
        node.computed_rect = Rect::new(card_x + 12.0, card_y + 50.0, card_w - 24.0, 20.0);
    }
    let _ = tree.add_child(card_id, title_id);

    // Secondary Description
    let desc_id = tree.create_node();
    if let Some(node) = tree.get_mut(desc_id) {
        node.set_name("MaterialEmptyDesc");
        node.set_text(
            "Select a 3D model or 2D sprite in the viewport or hierarchy to edit materials.",
        );
        node.font_size = 10.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.55, 0.58, 0.64, 1.0);
        node.computed_rect = Rect::new(card_x + 14.0, card_y + 74.0, card_w - 28.0, 36.0);
    }
    let _ = tree.add_child(card_id, desc_id);
}

/// Builds an empty-state placeholder when the selected entity has no ModelId or SpriteId.
pub fn build_no_renderable_geometry(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    targets: &mut MaterialPanelTargets,
    cursor_pos: Point,
) {
    let card_w = (panel_rect.width - 32.0).clamp(200.0, 380.0);
    let card_h = 160.0;
    let card_x = panel_rect.x + (panel_rect.width - card_w) * 0.5;
    let card_y = panel_rect.y + 40.0 + (panel_rect.height - 40.0 - card_h) * 0.35;

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("MaterialNoGeomCard");
        node.computed_rect = Rect::new(card_x, card_y, card_w, card_h);
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.12, 0.95))
            .border(1.0, Color::rgba(0.18, 0.20, 0.25, 0.80))
            .border_radius(8.0);
    }
    let _ = tree.add_child(parent_id, card_id);

    // Icon: ICON_CUBE
    let icon_size = 28.0;
    let icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("MaterialNoGeomIcon");
        node.computed_rect = Rect::new(
            card_x + (card_w - icon_size) * 0.5,
            card_y + 16.0,
            icon_size,
            icon_size,
        );
        node.set_texture_uv(ICON_CUBE);
        node.set_texture_tint(Color::rgba(0.95, 0.65, 0.25, 0.75));
    }
    let _ = tree.add_child(card_id, icon_id);

    // Primary Message
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("MaterialNoGeomTitle");
        node.set_text("No Renderable Geometry");
        node.font_size = 12.0;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.90, 0.92, 0.95, 1.0);
        node.computed_rect = Rect::new(card_x + 12.0, card_y + 48.0, card_w - 24.0, 20.0);
    }
    let _ = tree.add_child(card_id, title_id);

    // Secondary Description
    let desc_id = tree.create_node();
    if let Some(node) = tree.get_mut(desc_id) {
        node.set_name("MaterialNoGeomDesc");
        node.set_text("Selected entity does not have a 3D Model or 2D Sprite component attached.");
        node.font_size = 10.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.55, 0.58, 0.64, 1.0);
        node.computed_rect = Rect::new(card_x + 14.0, card_y + 70.0, card_w - 28.0, 34.0);
    }
    let _ = tree.add_child(card_id, desc_id);

    // Quick Add Button: + Add Texture / Sprite
    let btn_w = 160.0;
    let btn_h = 24.0;
    let btn_rect = Rect::new(
        card_x + (card_w - btn_w) * 0.5,
        card_y + 116.0,
        btn_w,
        btn_h,
    );
    targets.btn_add_texture = Some(btn_rect);

    let is_hovered = btn_rect.contains_point(cursor_pos);
    let (bg_color, border_color) = if is_hovered {
        (
            Color::rgba(0.0, 0.40, 0.55, 0.95),
            Color::rgba(0.0, 0.85, 1.0, 0.95),
        )
    } else {
        (
            Color::rgba(0.12, 0.14, 0.18, 0.95),
            Color::rgba(0.20, 0.24, 0.30, 0.85),
        )
    };

    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("MaterialAddTextureBtn");
        node.computed_rect = btn_rect;
        node.style = Style::new()
            .background(bg_color)
            .border(1.0, border_color)
            .border_radius(4.0);
    }
    let _ = tree.add_child(card_id, btn_id);

    // ICON_PLUS on button
    let plus_id = tree.create_node();
    if let Some(node) = tree.get_mut(plus_id) {
        node.set_name("MaterialAddPlusIcon");
        node.computed_rect = Rect::new(btn_rect.x + 8.0, btn_rect.y + 6.0, 12.0, 12.0);
        node.set_texture_uv(ICON_PLUS);
        node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.95));
    }
    let _ = tree.add_child(btn_id, plus_id);

    let btn_text_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_text_id) {
        node.set_name("MaterialAddBtnText");
        node.set_text("Add Texture / Sprite");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_color = Color::rgba(0.92, 0.95, 0.98, 1.0);
        node.computed_rect = Rect::new(btn_rect.x + 24.0, btn_rect.y, btn_w - 28.0, btn_h);
    }
    let _ = tree.add_child(btn_id, btn_text_id);
}