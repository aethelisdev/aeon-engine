// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Drag & Viewport Drop Overlay Subsystem
//!
//! Renders the visual feedback when dragging an asset item from the Content Browser
//! across the editor, including:
//! 1. 3D ground raycast landing indicator ring with inner dot on the viewport canvas.
//! 2. Floating cursor tooltip capsule displaying the category badge and asset name.

use crate::assets::types::AssetDragPayload;
use ae_renderer::camera::Camera;
use irisui::prelude::{Color, Point, Rect, UiScope, UiTree, WidgetId, WorkbenchUiScopeExt};

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
    build_asset_drag_overlays_scope(
        &mut UiScope::new(tree, parent_id),
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
    if let Some((center_x, center_y)) =
        compute_landing_center(cursor_pos, viewport_rect, camera, is_2d_mode)
    {
        scope.landing_indicator(
            Point::new(center_x, center_y),
            18.0,
            4.0,
            Color::from_u8(0, 229, 255, 255),
        );
    }

    // 2. Floating Cursor Drag Tooltip Capsule
    let mut controller = irisui::workbench::DragDropController::new();
    let drag_payload =
        irisui::workbench::TypedDragDropPayload::new((), payload.name.clone(), "Assets")
            .with_badge(payload.category.badge(), payload.category.badge_color())
            .into_erased();
    controller.start_drag(drag_payload, cursor_pos);

    scope.drag_drop_preview(&controller, cursor_pos, None);
}

/// Computes the 2D screen coordinate center for the viewport landing drop target.
#[inline]
fn compute_landing_center(
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) -> Option<(f32, f32)> {
    let in_viewport = cursor_pos.x >= viewport_rect.x
        && cursor_pos.x <= viewport_rect.x + viewport_rect.width
        && cursor_pos.y >= viewport_rect.y
        && cursor_pos.y <= viewport_rect.y + viewport_rect.height;

    if !in_viewport || viewport_rect.width <= 20.0 || viewport_rect.height <= 20.0 {
        return None;
    }

    if is_2d_mode {
        Some((cursor_pos.x, cursor_pos.y))
    } else {
        crate::assets::drag_drop::compute_ground_intersection(
            [cursor_pos.x, cursor_pos.y],
            viewport_rect,
            camera,
        )
        .and_then(|world_pos| {
            let vp_matrix = camera.build_view_projection_matrix();
            let pos_v4 = cgmath::Vector4::new(world_pos[0], world_pos[1], world_pos[2], 1.0);
            let clip_v4 = vp_matrix * pos_v4;

            if clip_v4.w > 0.001 {
                let ndc_x = clip_v4.x / clip_v4.w;
                let ndc_y = clip_v4.y / clip_v4.w;

                if (-1.2..=1.2).contains(&ndc_x) && (-1.2..=1.2).contains(&ndc_y) {
                    Some((
                        viewport_rect.x + (ndc_x + 1.0) * 0.5 * viewport_rect.width,
                        viewport_rect.y + (1.0 - ndc_y) * 0.5 * viewport_rect.height,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}