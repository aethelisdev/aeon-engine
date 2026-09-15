// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Drag & Viewport Drop Overlay Subsystem
//!
//! Renders the visual feedback when dragging an asset item from the Content Browser
//! across the editor, including:
//! 1. 3D ground raycast landing indicator ring with inner dot on the viewport canvas.
//! 2. Floating cursor tooltip capsule displaying the category badge and asset name.
//!

use crate::ui::panels::assets::types::AssetDragPayload;
use ae_renderer::camera::Camera;
use irisui::prelude::{Color, Point, Rect, Style, UiTree, WidgetId};

/// Constructs topmost floating overlays for an active asset drag operation.
/// If the cursor is positioned over the 3D/2D viewport, computes the projected
/// ground intersection point and renders the cyan landing indicator ring.
/// In addition, constructs a floating tooltip capsule anchored to the cursor.
pub fn build_asset_drag_overlays(
    tree: &mut UiTree,
    parent_id: WidgetId,
    payload: &AssetDragPayload,
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) {
    // 1. 3D / 2D Viewport Landing Indicator
    if viewport_rect.contains_point(cursor_pos)
        && viewport_rect.width > 20.0
        && viewport_rect.height > 20.0
    {
        let center_opt = if !is_2d_mode {
            // 3D Mode: Raycast against horizontal ground plane Y = 0
            if let Some(world_pos) =
                crate::ui::panels::assets::drag_drop::compute_ground_intersection(
                    [cursor_pos.x, cursor_pos.y],
                    viewport_rect,
                    camera,
                )
            {
                let vp_matrix = camera.build_view_projection_matrix();
                let pos_v4 = cgmath::Vector4::new(world_pos[0], world_pos[1], world_pos[2], 1.0);
                let clip_v4 = vp_matrix * pos_v4;

                if clip_v4.w > 0.001 {
                    let ndc_x = clip_v4.x / clip_v4.w;
                    let ndc_y = clip_v4.y / clip_v4.w;

                    if (-1.2..=1.2).contains(&ndc_x) && (-1.2..=1.2).contains(&ndc_y) {
                        let screen_x = viewport_rect.x + (ndc_x + 1.0) * 0.5 * viewport_rect.width;
                        let screen_y = viewport_rect.y + (1.0 - ndc_y) * 0.5 * viewport_rect.height;
                        Some((screen_x, screen_y))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // 2D Mode: Directly position landing ring under cursor
            Some((cursor_pos.x, cursor_pos.y))
        };

        if let Some((center_x, center_y)) = center_opt {
            // Outer glowing landing ring (Radius = 18.0 px)
            let ring_radius = 18.0;
            let ring_node = tree.create_node();
            if let Some(node) = tree.get_mut(ring_node) {
                node.set_name("ViewportDropLandingRing");
                node.computed_rect = Rect::new(
                    center_x - ring_radius,
                    center_y - ring_radius,
                    ring_radius * 2.0,
                    ring_radius * 2.0,
                );
                node.set_style(
                    Style::new()
                        .border(2.0, Color::from_u8(0, 229, 255, 255))
                        .border_radius(ring_radius),
                );
            }
            let _ = tree.add_child(parent_id, ring_node);

            // Inner landing target dot (Radius = 4.0 px)
            let dot_radius = 4.0;
            let dot_node = tree.create_node();
            if let Some(node) = tree.get_mut(dot_node) {
                node.set_name("ViewportDropLandingDot");
                node.computed_rect = Rect::new(
                    center_x - dot_radius,
                    center_y - dot_radius,
                    dot_radius * 2.0,
                    dot_radius * 2.0,
                );
                node.set_style(
                    Style::new()
                        .background(Color::from_u8(0, 229, 255, 255))
                        .border_radius(dot_radius),
                );
            }
            let _ = tree.add_child(parent_id, dot_node);
        }
    }

    // 2. Floating Cursor Drag Tooltip Capsule
    let tip_x = cursor_pos.x + 14.0;
    let tip_y = cursor_pos.y + 14.0;
    let badge_str = payload.category.badge();
    let name_str = &payload.name;

    let badge_w = (badge_str.len() as f32) * 7.5 + 10.0;
    let name_w = (name_str.len() as f32) * 7.5;
    let badge_total_w = (badge_w + name_w + 24.0).max(80.0);
    let badge_height = 26.0;

    let capsule_node = tree.create_node();
    if let Some(node) = tree.get_mut(capsule_node) {
        node.set_name("AssetDragCapsule");
        node.computed_rect = Rect::new(tip_x, tip_y, badge_total_w, badge_height);
        node.set_style(
            Style::new()
                .background(Color::from_u8(18, 22, 32, 230))
                .border(1.0, Color::from_u8(0, 229, 255, 220))
                .border_radius(6.0),
        );
    }
    let _ = tree.add_child(parent_id, capsule_node);

    // Category badge text node
    let cat_node = tree.create_node();
    if let Some(node) = tree.get_mut(cat_node) {
        node.set_name("AssetDragCategoryBadge");
        node.computed_rect = Rect::new(tip_x + 8.0, tip_y + 4.0, badge_w, badge_height - 8.0);
        node.text = Some(badge_str.to_string());
        node.font_size = 11.0;
        node.text_color = payload.category.badge_color();
    }
    let _ = tree.add_child(capsule_node, cat_node);

    // Asset name text node
    let name_node = tree.create_node();
    if let Some(node) = tree.get_mut(name_node) {
        node.set_name("AssetDragNameLabel");
        node.computed_rect = Rect::new(
            tip_x + 12.0 + badge_w,
            tip_y + 4.0,
            name_w.max(20.0),
            badge_height - 8.0,
        );
        node.text = Some(name_str.to_string());
        node.font_size = 11.0;
        node.text_color = Color::WHITE;
    }
    let _ = tree.add_child(capsule_node, name_node);
}