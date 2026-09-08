// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Drag-and-Drop & Viewport Projection Subsystem.
//!
//! Manages dragging assets from the Content Browser into the 3D Viewport,
//! computing screen-to-world raycasts against horizontal ground planes,
//! drawing landing indicators, and spawning entities at the target coordinates.
//!

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