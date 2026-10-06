// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Drag & Viewport Drop Overlay Subsystem
//!
//! Renders the visual feedback when dragging an asset item from the Content Browser
//! across the editor, including:
//! 1. 3D ground raycast landing indicator ring with inner dot on the viewport canvas.
//! 2. Floating cursor tooltip capsule displaying the category badge and asset name.
//!

use crate::assets::types::AssetDragPayload;
use ae_renderer::camera::Camera;
use irisui::prelude::{
    AlignItems, Color, Insets, Point, Rect, Style, TextAlign, UiScope, UiTree, WidgetId,
    layout_subtree, measure_height, measure_width,
};

/// Constructs topmost floating overlays for an active asset drag operation into the `UiTree`.
pub fn build_asset_drag_overlays(
    tree: &mut UiTree,
    parent_id: WidgetId,
    payload: &AssetDragPayload,
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) {
    let mut scope = UiScope::new(tree, parent_id);
    build_asset_drag_overlays_scope(
        &mut scope,
        payload,
        cursor_pos,
        viewport_rect,
        camera,
        is_2d_mode,
    );
}

/// Constructs topmost floating overlays for an active asset drag operation via a declarative [`UiScope`].
///
/// Renders 3D/2D landing indicators and cursor tooltip capsule using 100% declarative UI scope
/// primitives with zero imperative node allocations.
pub fn build_asset_drag_overlays_scope(
    scope: &mut UiScope<'_>,
    payload: &AssetDragPayload,
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) {
    // 1. 3D / 2D Viewport Landing Indicator
    let in_viewport = cursor_pos.x >= viewport_rect.x
        && cursor_pos.x <= viewport_rect.x + viewport_rect.width
        && cursor_pos.y >= viewport_rect.y
        && cursor_pos.y <= viewport_rect.y + viewport_rect.height;
    if in_viewport && viewport_rect.width > 20.0 && viewport_rect.height > 20.0 {
        let center_opt = if !is_2d_mode {
            // 3D Mode: Raycast against horizontal ground plane Y = 0
            if let Some(world_pos) = crate::assets::drag_drop::compute_ground_intersection(
                [cursor_pos.x, cursor_pos.y],
                viewport_rect,
                camera,
            ) {
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
            let ring_rect = Rect::new(
                center_x - ring_radius,
                center_y - ring_radius,
                ring_radius * 2.0,
                ring_radius * 2.0,
            );
            let ring_id = scope.empty_box_passive_named(
                "ViewportDropLandingRing",
                Style::new()
                    .position_absolute()
                    .left(ring_rect.x)
                    .top(ring_rect.y)
                    .width(ring_rect.width)
                    .height(ring_rect.height)
                    .border(2.0, Color::from_u8(0, 229, 255, 255))
                    .border_radius(ring_radius),
            );
            layout_subtree(scope.tree_mut(), ring_id, ring_rect);

            // Inner landing target dot (Radius = 4.0 px)
            let dot_radius = 4.0;
            let dot_rect = Rect::new(
                center_x - dot_radius,
                center_y - dot_radius,
                dot_radius * 2.0,
                dot_radius * 2.0,
            );
            let dot_id = scope.empty_box_passive_named(
                "ViewportDropLandingDot",
                Style::new()
                    .position_absolute()
                    .left(dot_rect.x)
                    .top(dot_rect.y)
                    .width(dot_rect.width)
                    .height(dot_rect.height)
                    .background(Color::from_u8(0, 229, 255, 255))
                    .border_radius(dot_radius),
            );
            layout_subtree(scope.tree_mut(), dot_id, dot_rect);
        }
    }

    // 2. Floating Cursor Drag Tooltip Capsule
    let tip_x = cursor_pos.x + 14.0;
    let tip_y = cursor_pos.y + 14.0;
    let badge_str = payload.category.badge();
    let name_str = &payload.name;

    let capsule_id = scope.container_named(
        "AssetDragCapsule",
        Style::new()
            .position_absolute()
            .left(tip_x)
            .top(tip_y)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(4.0, 8.0, 4.0, 8.0))
            .gap(6.0)
            .background(Color::from_u8(18, 22, 32, 230))
            .border(1.0, Color::from_u8(0, 229, 255, 220))
            .border_radius(6.0),
        |capsule| {
            capsule.label(
                badge_str,
                11.0,
                payload.category.badge_color(),
                TextAlign::Left,
            );
            capsule.label(name_str, 11.0, Color::WHITE, TextAlign::Left);
        },
    );

    let measured_w = measure_width(scope.tree(), capsule_id);
    let measured_h = measure_height(scope.tree(), capsule_id).max(24.0);
    layout_subtree(
        scope.tree_mut(),
        capsule_id,
        Rect::new(tip_x, tip_y, measured_w, measured_h),
    );
}