// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Drag-and-Drop & Viewport Projection Subsystem.
//!
//! Manages dragging assets from the Content Browser into the 3D Viewport,
//! computing screen-to-world raycasts against horizontal ground planes,
//! drawing landing indicators, and spawning entities at the target coordinates.

use super::types::{AssetBrowserState, AssetCategory};
use crate::ui::types::EngineUiAction;
use ae_renderer::camera::Camera;
use irisui::prelude::Rect;

/// Computes the 3D world intersection point on the ground plane (Y = 0) from screen coordinates.
pub fn compute_ground_intersection(
    mouse_screen_pos: [f32; 2],
    viewport_rect: Rect,
    camera: &Camera,
) -> Option<[f32; 3]> {
    let vp_matrix = camera.build_view_projection_matrix();
    let rel_x = mouse_screen_pos[0] - viewport_rect.x;
    let rel_y = mouse_screen_pos[1] - viewport_rect.y;

    let ray = ae_editor::picking::create_ray(
        rel_x,
        rel_y,
        viewport_rect.width,
        viewport_rect.height,
        &vp_matrix,
    )?;

    // Ray vs horizontal plane at Y = 0: origin.y + t * dir.y = 0 => t = -origin.y / dir.y
    if ray.direction.y.abs() < 1e-5 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t <= 0.0 || t > 1000.0 {
        return None;
    }

    let hit_x = ray.origin.x + t * ray.direction.x;
    let hit_z = ray.origin.z + t * ray.direction.z;

    Some([hit_x, 0.0, hit_z])
}

/// Handles dropping an asset onto the 3D viewport.
///
/// Spawns the corresponding 3D model or sprite entity at the calculated world location.
pub fn handle_viewport_drop(
    state: &mut AssetBrowserState,
    world_pos: [f32; 3],
    ui_actions: &mut Vec<EngineUiAction>,
) {
    if let Some(payload) = state.drag_payload.take() {
        match payload.category {
            AssetCategory::Models3D => {
                if let Some(handle) = payload.model_handle {
                    ui_actions.push(EngineUiAction::SpawnModelAt(handle, world_pos));
                } else {
                    ui_actions.push(EngineUiAction::SpawnModelPathAt(payload.path, world_pos));
                }
            }
            AssetCategory::Textures2D => {
                if let Some(handle) = payload.texture_handle {
                    ui_actions.push(EngineUiAction::SpawnSpriteAt(handle, world_pos));
                } else {
                    ui_actions.push(EngineUiAction::SpawnSpritePathAt(payload.path, world_pos));
                }
            }
            AssetCategory::Scenes => {
                ui_actions.push(EngineUiAction::LoadSceneFromPath(payload.path));
            }
            _ => {}
        }
    }
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
    fn test_compute_ground_intersection_center_of_screen() {
        // Camera positioned at (0, 10, 10) looking down toward origin (pitch -45 deg, yaw -90 deg looking along -Z)
        let camera = make_test_camera([0.0, 10.0, 10.0], -45.0, -90.0);
        let viewport_rect = Rect::new(0.0, 0.0, 800.0, 600.0);

        // Center of the screen should point towards (0, 0, 0)
        let hit = compute_ground_intersection([400.0, 300.0], viewport_rect, &camera);
        assert!(hit.is_some());
        let pos = hit.unwrap();
        assert!((pos[0] - 0.0).abs() < 0.2);
        assert_eq!(pos[1], 0.0);
        assert!((pos[2] - 0.0).abs() < 0.2);
    }

    #[test]
    fn test_compute_ground_intersection_parallel_or_upward_ray() {
        // Looking horizontally along -Z (pitch 0 deg)
        let camera = make_test_camera([0.0, 10.0, 10.0], 0.0, -90.0);
        let viewport_rect = Rect::new(0.0, 0.0, 800.0, 600.0);

        // Screen center ray is horizontal (parallel to ground)
        let hit_center = compute_ground_intersection([400.0, 300.0], viewport_rect, &camera);
        assert!(hit_center.is_none());

        // Top of screen points upwards away from ground plane
        let hit_top = compute_ground_intersection([400.0, 50.0], viewport_rect, &camera);
        assert!(hit_top.is_none());
    }

    #[test]
    fn test_handle_viewport_drop_actions() {
        let mut state = AssetBrowserState::default();
        let mut actions = Vec::new();

        state.drag_payload = Some(super::super::types::AssetDragPayload {
            category: AssetCategory::Models3D,
            path: std::path::PathBuf::from("models/test.glb"),
            model_handle: None,
            texture_handle: None,
            name: "test.glb".to_string(),
        });

        handle_viewport_drop(&mut state, [1.0, 0.0, 3.0], &mut actions);

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            EngineUiAction::SpawnModelPathAt(path, pos) => {
                assert_eq!(path, &std::path::PathBuf::from("models/test.glb"));
                assert_eq!(*pos, [1.0, 0.0, 3.0]);
            }
            _ => panic!("Expected SpawnModelPathAt"),
        }
    }
}