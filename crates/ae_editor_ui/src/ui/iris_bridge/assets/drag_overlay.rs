// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Drag & Viewport Drop Overlay Subsystem
//!
//! Renders the visual feedback when dragging an asset item from the Content Browser
//! across the editor via [`UiScope::viewport_drop_overlay`].

use crate::assets::types::AssetDragPayload;
use ae_renderer::camera::Camera;
use irisui::prelude::{Point, Rect, UiScope, ViewportDropOverlay};

/// Constructs topmost floating overlays for an active asset drag operation via [`UiScope::viewport_drop_overlay`].
pub fn build_asset_drag_overlays_scope(
    scope: &mut UiScope<'_>,
    payload: &AssetDragPayload,
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) {
    scope.viewport_drop_overlay(
        ViewportDropOverlay::new(
            viewport_rect,
            cursor_pos,
            &payload.name,
            payload.category.badge(),
            payload.category.badge_color(),
        ),
        |cursor| {
            crate::assets::drag_drop::compute_viewport_landing_center(
                cursor,
                viewport_rect,
                camera,
                is_2d_mode,
            )
        },
    );
}