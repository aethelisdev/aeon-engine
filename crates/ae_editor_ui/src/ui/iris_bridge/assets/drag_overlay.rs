// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Drag & Viewport Drop Overlay Subsystem
//!
//! Renders visual drop indicators and translates content-browser drag gestures into
//! [`EngineUiAction`]s using 3D viewport picking math from [`ae_editor::picking`].

use super::components::resolve_category_color;
use crate::ui::types::EngineUiAction;
use ae_assets::types::{AssetBrowserState, AssetCategory, AssetDragPayload};
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
            resolve_category_color(payload.category),
        ),
        |cursor| compute_viewport_landing_center(cursor, viewport_rect, camera, is_2d_mode),
    );
}

/// Computes the projected 2D screen coordinate center for the viewport landing drop indicator.
#[must_use]
pub fn compute_viewport_landing_center(
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) -> Option<Point> {
    if !viewport_rect.contains_point(cursor_pos)
        || viewport_rect.width <= 20.0
        || viewport_rect.height <= 20.0
    {
        return None;
    }

    if is_2d_mode {
        return Some(cursor_pos);
    }

    ae_editor::picking::compute_ground_intersection(
        [cursor_pos.x, cursor_pos.y],
        [
            viewport_rect.x,
            viewport_rect.y,
            viewport_rect.width,
            viewport_rect.height,
        ],
        &camera.build_view_projection_matrix(),
    )
    .and_then(|world_pos| {
        ae_editor::picking::project_world_point_to_viewport(
            world_pos,
            [
                viewport_rect.x,
                viewport_rect.y,
                viewport_rect.width,
                viewport_rect.height,
            ],
            &camera.build_view_projection_matrix(),
        )
    })
    .map(|[x, y]| Point::new(x, y))
}

#[inline]
fn resolve_drop_world_pos(
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) -> [f32; 3] {
    if is_2d_mode {
        return [
            cursor_pos.x - viewport_rect.x - viewport_rect.width * 0.5,
            -(cursor_pos.y - viewport_rect.y - viewport_rect.height * 0.5),
            0.0,
        ];
    }

    ae_editor::picking::compute_ground_intersection(
        [cursor_pos.x, cursor_pos.y],
        [
            viewport_rect.x,
            viewport_rect.y,
            viewport_rect.width,
            viewport_rect.height,
        ],
        &camera.build_view_projection_matrix(),
    )
    .unwrap_or_else(|| {
        ae_editor::picking::compute_camera_ground_fallback(
            [camera.position.x, camera.position.y, camera.position.z],
            [camera.target.x, camera.target.y, camera.target.z],
            3.0,
        )
    })
}

#[inline]
fn payload_to_viewport_drop_action(
    payload: AssetDragPayload,
    world_pos: [f32; 3],
    is_2d_mode: bool,
) -> Option<EngineUiAction> {
    match payload.category {
        AssetCategory::Models3D => Some(payload.model_handle.map_or_else(
            || EngineUiAction::SpawnModelPathAt(payload.path, world_pos),
            |handle| EngineUiAction::SpawnModelAt(handle, world_pos),
        )),
        AssetCategory::Textures2D => Some(payload.texture_handle.map_or_else(
            || EngineUiAction::SpawnSpritePathAt(payload.path, world_pos),
            |handle| EngineUiAction::SpawnSpriteAt(handle, world_pos),
        )),
        AssetCategory::Scenes => {
            if is_2d_mode && ae_assets::scanner::is_world_file_3d(&payload.path) {
                None
            } else {
                Some(EngineUiAction::LoadSceneFromPath(payload.path))
            }
        }
        _ => None,
    }
}

/// Resolves an active asset drag release over `viewport_rect` into an [`EngineUiAction`],
/// consuming `state.drag_payload`.
#[must_use]
pub fn resolve_viewport_drop_action(
    state: &mut AssetBrowserState,
    cursor_pos: Point,
    viewport_rect: Rect,
    camera: &Camera,
    is_2d_mode: bool,
) -> Option<EngineUiAction> {
    if !viewport_rect.contains_point(cursor_pos)
        || viewport_rect.width <= 20.0
        || viewport_rect.height <= 20.0
    {
        state.drag_payload = None;
        return None;
    }

    state.drag_payload.take().and_then(|payload| {
        payload_to_viewport_drop_action(
            payload,
            resolve_drop_world_pos(cursor_pos, viewport_rect, camera, is_2d_mode),
            is_2d_mode,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_renderer::camera::ProjectionMode;
    use cgmath::{Deg, Point3, Rad};

    fn make_test_camera(pos: [f32; 3], pitch_deg: f32, yaw_deg: f32) -> Camera {
        Camera {
            position: Point3::new(pos[0], pos[1], pos[2]),
            yaw: Rad::from(Deg(yaw_deg)),
            pitch: Rad::from(Deg(pitch_deg)),
            aspect: 800.0 / 600.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_compute_viewport_landing_center_projects_to_viewport() {
        let camera = make_test_camera([0.0, 10.0, 10.0], -45.0, -90.0);
        let viewport_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        let center = compute_viewport_landing_center(
            Point::new(400.0, 300.0),
            viewport_rect,
            &camera,
            false,
        );
        let pt = center.expect("center screen ray hits ground");
        assert!((pt.x - 400.0).abs() < 1.0);
        assert!((pt.y - 300.0).abs() < 1.0);
    }

    #[test]
    fn test_resolve_viewport_drop_action_emits_spawn_model() {
        let mut state = AssetBrowserState::default();
        let camera = make_test_camera([0.0, 10.0, 10.0], -45.0, -90.0);
        let viewport_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        state.drag_payload = Some(AssetDragPayload {
            category: AssetCategory::Models3D,
            path: std::path::PathBuf::from("models/test.glb"),
            model_handle: None,
            texture_handle: None,
            name: "test.glb".to_string(),
        });

        let action = resolve_viewport_drop_action(
            &mut state,
            Point::new(400.0, 300.0),
            viewport_rect,
            &camera,
            false,
        );
        assert!(matches!(
            action,
            Some(EngineUiAction::SpawnModelPathAt(ref path, _))
                if path == std::path::Path::new("models/test.glb")
        ));
    }
}