// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Header Bar
//!
//! Renders the top title bar with active geometry type badge and hardware texture icons.
//!

use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_WORLD};
use irisui::prelude::*;

/// Height of the Material Studio header bar in logical pixels.
pub const MATERIAL_HEADER_HEIGHT: f32 = 32.0;

/// Builds the top title and badge header for the Material & Surface Studio panel.
pub fn build_material_header(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    entity: Option<hecs::Entity>,
    world: &hecs::World,
) -> f32 {
    let padding = 8.0;
    let header_rect = Rect::new(
        panel_rect.x,
        panel_rect.y,
        panel_rect.width,
        MATERIAL_HEADER_HEIGHT,
    );

    // Header container with dark background
    let hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(hdr_id) {
        node.set_name("MaterialHeader");
        node.computed_rect = header_rect;
        node.style = Style::new()
            .background(Color::rgba(0.082, 0.086, 0.102, 0.98))
            .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.90));
    }
    let _ = tree.add_child(parent_id, hdr_id);

    // Left Icon: ICON_WORLD
    let icon_size = 16.0;
    let icon_rect = Rect::new(
        panel_rect.x + padding,
        panel_rect.y + (MATERIAL_HEADER_HEIGHT - icon_size) * 0.5,
        icon_size,
        icon_size,
    );
    let icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("MaterialHeaderIcon");
        node.computed_rect = icon_rect;
        node.set_texture_uv(ICON_WORLD);
        node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.95));
    }
    let _ = tree.add_child(hdr_id, icon_id);

    // Title label
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("MaterialHeaderTitle");
        node.set_text("Material & Surface Studio");
        node.font_size = 11.5;
        node.line_height = MATERIAL_HEADER_HEIGHT;
        node.text_color = Color::rgba(0.92, 0.93, 0.95, 1.0);
        node.computed_rect = Rect::new(
            panel_rect.x + padding + icon_size + 6.0,
            panel_rect.y,
            160.0,
            MATERIAL_HEADER_HEIGHT,
        );
    }
    let _ = tree.add_child(hdr_id, title_id);

    // Right-aligned Entity Geometry Badge
    if let Some(ent) = entity {
        let has_model = world.get::<&ae_core::ecs::ModelId>(ent).is_ok();
        let has_sprite = world.get::<&ae_core::ecs::SpriteId>(ent).is_ok();

        if has_model || has_sprite {
            let (badge_text, badge_icon, badge_color) = if has_model {
                ("3D Model", ICON_CUBE, Color::rgba(0.0, 0.80, 0.95, 1.0))
            } else {
                ("2D Sprite", ICON_WORLD, Color::rgba(0.30, 0.85, 0.45, 1.0))
            };

            let badge_w = 80.0;
            let badge_h = 20.0;
            let badge_x = panel_rect.x + panel_rect.width - padding - badge_w;
            let badge_y = panel_rect.y + (MATERIAL_HEADER_HEIGHT - badge_h) * 0.5;

            let badge_id = tree.create_node();
            if let Some(node) = tree.get_mut(badge_id) {
                node.set_name("MaterialHeaderBadge");
                node.computed_rect = Rect::new(badge_x, badge_y, badge_w, badge_h);
                node.style = Style::new()
                    .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                    .border(1.0, Color::rgba(0.20, 0.22, 0.27, 0.90))
                    .border_radius(4.0);
            }
            let _ = tree.add_child(hdr_id, badge_id);

            // Badge Icon
            let b_icon_id = tree.create_node();
            if let Some(node) = tree.get_mut(b_icon_id) {
                node.set_name("MaterialBadgeIcon");
                node.computed_rect = Rect::new(badge_x + 5.0, badge_y + 4.0, 12.0, 12.0);
                node.set_texture_uv(badge_icon);
                node.set_texture_tint(badge_color);
            }
            let _ = tree.add_child(badge_id, b_icon_id);

            // Badge Text
            let b_txt_id = tree.create_node();
            if let Some(node) = tree.get_mut(b_txt_id) {
                node.set_name("MaterialBadgeText");
                node.set_text(badge_text);
                node.font_size = 10.0;
                node.line_height = badge_h;
                node.text_color = badge_color;
                node.computed_rect = Rect::new(badge_x + 21.0, badge_y, badge_w - 23.0, badge_h);
            }
            let _ = tree.add_child(badge_id, b_txt_id);
        }
    }

    MATERIAL_HEADER_HEIGHT
}