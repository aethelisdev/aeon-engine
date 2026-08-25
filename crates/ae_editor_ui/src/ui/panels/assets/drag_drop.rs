// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Drag-and-Drop & Viewport Projection Subsystem.
//!
//! Manages dragging assets from the Content Browser into the 3D Viewport,
//! computing screen-to-world raycasts against horizontal ground planes,
//! drawing landing indicators, and spawning entities at the target coordinates.
//!

use super::types::{AssetBrowserState, AssetCategory, AssetDragPayload, AssetItem};
use crate::ui::types::EngineUiAction;
use ae_renderer::camera::Camera;
use egui::{Color32, Context, Rect, Stroke, Vec2};

/// Initiates dragging an asset item when mouse drag is detected.
pub fn handle_asset_drag_source(
    response: &egui::Response,
    item: &AssetItem,
    state: &mut AssetBrowserState,
) {
    if response.drag_started() {
        state.drag_payload = Some(AssetDragPayload {
            path: item.path.clone(),
            name: item.name.clone(),
            category: item.category,
            model_handle: item.model_handle,
            texture_handle: item.texture_handle,
        });
    }
}

/// Renders a floating preview tooltip near the cursor while an asset is actively dragged.
pub fn draw_drag_cursor_tooltip(ctx: &Context, state: &AssetBrowserState) {
    if let Some(payload) = &state.drag_payload
        && let Some(pos) = ctx.pointer_latest_pos()
    {
        egui::Area::new(egui::Id::new("asset_drag_tooltip"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos + Vec2::new(14.0, 14.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(18, 22, 32, 230))
                    .corner_radius(egui::CornerRadius::same(6))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(0, 229, 255)))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(payload.category.badge())
                                    .color(payload.category.badge_color())
                                    .size(10.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new(&payload.name)
                                    .color(Color32::WHITE)
                                    .size(11.0)
                                    .strong(),
                            );
                        });
                    });
            });
    }
}

/// Computes the 3D world intersection point on the ground plane (Y = 0) from screen coordinates.
pub fn compute_ground_intersection(
    mouse_screen_pos: egui::Pos2,
    viewport_rect: Rect,
    camera: &Camera,
) -> Option<[f32; 3]> {
    let vp_matrix = camera.build_view_projection_matrix();
    let rel_x = mouse_screen_pos.x - viewport_rect.left();
    let rel_y = mouse_screen_pos.y - viewport_rect.top();

    let ray = ae_editor::picking::create_ray(
        rel_x,
        rel_y,
        viewport_rect.width(),
        viewport_rect.height(),
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

/// Renders a ghost landing indicator in the 3D viewport at the projected world coordinate.
pub fn draw_viewport_drop_indicator(
    ctx: &Context,
    viewport_rect: Rect,
    world_pos: [f32; 3],
    camera: &Camera,
) {
    let vp_matrix = camera.build_view_projection_matrix();
    let pos_v4 = cgmath::Vector4::new(world_pos[0], world_pos[1], world_pos[2], 1.0);
    let clip_v4 = vp_matrix * pos_v4;

    if clip_v4.w <= 0.001 {
        return;
    }

    let ndc_x = clip_v4.x / clip_v4.w;
    let ndc_y = clip_v4.y / clip_v4.w;

    if !(-1.2..=1.2).contains(&ndc_x) || !(-1.2..=1.2).contains(&ndc_y) {
        return;
    }

    let screen_x = viewport_rect.left() + (ndc_x + 1.0) * 0.5 * viewport_rect.width();
    let screen_y = viewport_rect.top() + (1.0 - ndc_y) * 0.5 * viewport_rect.height();

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("viewport_drop_landing_ring"),
    ));

    let center = egui::pos2(screen_x, screen_y);

    // Glowing spawn landing ring
    painter.circle_stroke(
        center,
        18.0,
        Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
    );
    painter.circle_filled(center, 4.0, Color32::from_rgb(0, 229, 255));
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