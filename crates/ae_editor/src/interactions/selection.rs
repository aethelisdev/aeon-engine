// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::gizmo_drag::gizmo_screen_params;
use super::viewport::{map_window_to_viewport, should_pass_to_3d};
use crate::editor_state::EditorState;
use crate::input::{InputManager, KeyCode};
use crate::picking;
use ae_core::ecs::{Position, Rotation, Scale};
use cgmath::{EuclideanSpace, InnerSpace, Point3, Quaternion, SquareMatrix, Vector3};

/// Creates a picking ray from screen-space mouse coordinates.
pub fn create_ray(
    camera: &ae_core::camera::Camera,
    window_size: (u32, u32),
    mx: f32,
    my: f32,
) -> Option<picking::Ray> {
    let width = window_size.0 as f32;
    let height = window_size.1 as f32;
    let vp_matrix = camera.build_view_projection_matrix();
    picking::create_ray(mx, my, width, height, &vp_matrix)
}

/// Handles left-click press in Edit Mode — initiates gizmo drag or entity selection.
pub fn on_left_click_pressed(
    editor: &mut EditorState,
    camera: &ae_core::camera::Camera,
    gizmo_system: &mut crate::gizmo::GizmoSystem,
    ecs: &mut ae_core::ecs::EcsManager,
    spatial_grid: &ae_core::spatial::SpatialGrid,
    _ui_gizmo_mode: crate::gizmo::GizmoMode,
    ui_gizmo_space: crate::gizmo::space::GizmoSpace,
    window_size: (u32, u32),
    last_viewport_rect: ae_renderer::render::ViewportRect,
    scale_factor: f32,
    is_point_over_ui: &dyn Fn(egui::Pos2) -> bool,
    egui_context: &egui::Context,
    is_edit_mode: bool,
    input: &InputManager,
) {
    if !is_edit_mode {
        return;
    }
    if !should_pass_to_3d(
        editor,
        last_viewport_rect,
        scale_factor,
        is_point_over_ui,
        egui_context,
    ) {
        return;
    }

    let (mx, my) = editor.last_cursor_pos;
    let (local_mx, local_my, local_size) =
        map_window_to_viewport(mx, my, window_size, last_viewport_rect, scale_factor);
    let ray = match create_ray(camera, local_size, local_mx, local_my) {
        Some(r) => r,
        None => return,
    };

    let mut gizmo_drag_started = false;
    if let Some(&selected_ent) = editor.selected_entities.first() {
        let gizmo_world_pos = if let Ok(gt) = ecs
            .world
            .get::<&ae_core::ecs::GlobalTransform>(selected_ent)
        {
            let mat = gt.0;
            Some(Vector3::new(mat.w.x, mat.w.y, mat.w.z))
        } else if let Ok(pos0) = ecs.world.get::<&Position>(selected_ent) {
            Some(Vector3::new(pos0.x, pos0.y, pos0.z))
        } else {
            None
        };

        if let Some(pos) = gizmo_world_pos {
            let camera_pos = camera.position.to_vec();
            let cam_forward = camera.get_forward();
            let gizmo_screen = gizmo_screen_params(camera, local_size);

            gizmo_system.space = ui_gizmo_space;
            if let Ok(r) = ecs.world.get::<&Rotation>(selected_ent) {
                gizmo_system.entity_rotation = Quaternion::new(r.w, r.x, r.y, r.z);
            }

            gizmo_system.handle_input(
                ray.origin.to_vec(),
                ray.direction,
                pos,
                camera_pos,
                cam_forward,
                &gizmo_screen,
                true,
                false,
                false,
            );
        }

        if gizmo_system.dragging_active() {
            gizmo_drag_started = true;
            editor.gizmo_dragging = true;
            editor.current_edit_snapshots.clear();
            for &ent in &editor.selected_entities {
                editor.current_edit_snapshots.insert(
                    ent,
                    crate::undo_redo::EntitySnapshot::capture(&ecs.world, ent),
                );
            }

            editor.multi_start_positions.clear();
            editor.multi_start_scales.clear();
            editor.multi_start_rotations.clear();
            for &ent in &editor.selected_entities {
                if let Ok(p) = ecs.world.get::<&Position>(ent) {
                    editor
                        .multi_start_positions
                        .insert(ent, cgmath::Vector3::new(p.x, p.y, p.z));
                }
                if let Ok(r) = ecs.world.get::<&Rotation>(ent) {
                    editor
                        .multi_start_rotations
                        .insert(ent, cgmath::Quaternion::new(r.w, r.x, r.y, r.z));
                }
                if let Ok(s) = ecs.world.get::<&Scale>(ent) {
                    editor
                        .multi_start_scales
                        .insert(ent, cgmath::Vector3::new(s.x, s.y, s.z));
                }
            }
            log::info!("Gizmo dragging started for entity: {:?}", selected_ent);
        }
    }

    if !gizmo_drag_started {
        try_select_entity(editor, ecs, spatial_grid, input, &ray);
    }
}

/// Performs raycast entity selection via AABB intersection testing.
pub fn try_select_entity(
    editor: &mut EditorState,
    ecs: &mut ae_core::ecs::EcsManager,
    spatial_grid: &ae_core::spatial::SpatialGrid,
    input: &InputManager,
    ray: &picking::Ray,
) {
    let mut closest_dist = f32::MAX;
    let mut selected = None;

    let total_ents = ecs.world.len() as usize;
    let use_spatial = total_ents >= 150_000 && !spatial_grid.cells.is_empty();

    if use_spatial {
        let cell_size = spatial_grid.cell_size;
        let mut candidates = Vec::new();

        let pad = 25.0;
        for (&(cx, cy, cz), entities) in &spatial_grid.cells {
            let min = [
                cx as f32 * cell_size - pad,
                cy as f32 * cell_size - pad,
                cz as f32 * cell_size - pad,
            ];
            let max = [
                (cx + 1) as f32 * cell_size + pad,
                (cy + 1) as f32 * cell_size + pad,
                (cz + 1) as f32 * cell_size + pad,
            ];

            if picking::intersect_aabb(ray, min, max).is_some() {
                candidates.extend_from_slice(entities);
            }
        }

        for &ent in &candidates {
            let mut q = ecs.world.query_one::<(
                Option<&ae_core::ecs::GlobalTransform>,
                Option<&Position>,
                Option<&Rotation>,
                Option<&Scale>,
                Option<&ae_core::ecs::BoundingBox>,
            )>(ent);

            if let Ok((gt, pos, rot, scale, bbox)) = q.get() {
                let model = picking::compute_model_matrix(gt, pos, rot, scale);

                let inv_model = model.invert().unwrap_or_else(|| {
                    cgmath::Matrix4::from_translation(Vector3::new(model.w.x, model.w.y, model.w.z))
                        .invert()
                        .unwrap_or(cgmath::Matrix4::identity())
                });
                {
                    let l_org_v4 = inv_model * ray.origin.to_homogeneous();
                    let l_dir_v4 = inv_model
                        * cgmath::Vector4::new(
                            ray.direction.x,
                            ray.direction.y,
                            ray.direction.z,
                            0.0,
                        );
                    let l_org = Point3::from_vec(l_org_v4.truncate() / l_org_v4.w);
                    let l_dir = l_dir_v4.truncate();

                    let l_max_dist = ray.max_dist / ray.direction.magnitude() * l_dir.magnitude();
                    let local_ray = picking::Ray {
                        origin: l_org,
                        direction: l_dir.normalize(),
                        max_dist: l_max_dist,
                    };

                    let (min, max) = if let Some(b) = bbox {
                        (b.min, b.max)
                    } else {
                        ([-0.5; 3], [0.5; 3])
                    };

                    if let Some(t_local) = picking::intersect_aabb(&local_ray, min, max) {
                        let t_world = t_local * (ray.direction.magnitude() / l_dir.magnitude());
                        if t_world < closest_dist {
                            closest_dist = t_world;
                            selected = Some(ent);
                        }
                    }
                }
            }
        }
    } else {
        let mut query = ecs.world.query::<(
            hecs::Entity,
            Option<&ae_core::ecs::GlobalTransform>,
            Option<&Position>,
            Option<&Rotation>,
            Option<&Scale>,
            Option<&ae_core::ecs::Shape>,
            Option<&ae_core::ecs::Collider>,
            Option<&ae_core::ecs::BoundingBox>,
        )>();

        for (ent, gt, pos, rot, scale, shape_opt, col_opt, bbox) in query.iter() {
            let model = picking::compute_model_matrix(gt, pos, rot, scale);

            let inv_model = model.invert().unwrap_or_else(|| {
                cgmath::Matrix4::from_translation(Vector3::new(model.w.x, model.w.y, model.w.z))
                    .invert()
                    .unwrap_or(cgmath::Matrix4::identity())
            });
            {
                let l_org_v4 = inv_model * ray.origin.to_homogeneous();
                let l_dir_v4 = inv_model
                    * cgmath::Vector4::new(ray.direction.x, ray.direction.y, ray.direction.z, 0.0);
                let l_org = Point3::from_vec(l_org_v4.truncate() / l_org_v4.w);
                let l_dir = l_dir_v4.truncate();

                let l_max_dist = ray.max_dist / ray.direction.magnitude() * l_dir.magnitude();
                let local_ray = picking::Ray {
                    origin: l_org,
                    direction: l_dir.normalize(),
                    max_dist: l_max_dist,
                };

                let (min, max) = if let Some(b) = bbox {
                    (b.min, b.max)
                } else if let Some(shape) = shape_opt {
                    match shape {
                        ae_core::ecs::Shape::Capsule => ([-0.35, -0.5, -0.35], [0.35, 0.5, 0.35]),
                        ae_core::ecs::Shape::Torus => ([-0.5, -0.15, -0.5], [0.5, 0.15, 0.5]),
                        _ => ([-0.5; 3], [0.5; 3]),
                    }
                } else if let Some(col) = col_opt {
                    match col.shape {
                        ae_core::ecs::ColliderShape::Box { half_extents } => (
                            [-half_extents[0], -half_extents[1], -half_extents[2]],
                            [half_extents[0], half_extents[1], half_extents[2]],
                        ),
                        ae_core::ecs::ColliderShape::Sphere { radius } => {
                            ([-radius, -radius, -radius], [radius, radius, radius])
                        }
                        ae_core::ecs::ColliderShape::Capsule {
                            half_height,
                            radius,
                        } => (
                            [-radius, -half_height - radius, -radius],
                            [radius, half_height + radius, radius],
                        ),
                        _ => ([-0.5; 3], [0.5; 3]),
                    }
                } else {
                    ([-0.5; 3], [0.5; 3])
                };

                if let Some(t_local) = picking::intersect_aabb(&local_ray, min, max) {
                    let t_world = t_local * (ray.direction.magnitude() / l_dir.magnitude());
                    if t_world < closest_dist {
                        closest_dist = t_world;
                        selected = Some(ent);
                    }
                }
            }
        }
    }

    // --- 3D VIEWPORT BILLBOARD ICON PICKING ---
    // Allows selecting Light, Audio, Camera, or empty entities by clicking their 3D billboard icons
    for (ent, pos) in ecs.world.query::<(hecs::Entity, &Position)>().iter() {
        let sphere_center = Point3::new(pos.x, pos.y, pos.z);
        if let Some(t_world) = picking::intersect_sphere(ray, sphere_center, 0.75) {
            if t_world < closest_dist {
                closest_dist = t_world;
                selected = Some(ent);
            }
        }
    }

    if let Some(ent) = selected {
        let shift =
            input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight);
        if shift {
            if editor.selected_entities.contains(&ent) {
                editor.selected_entities.retain(|e| *e != ent);
                editor.selected_entities_set.remove(&ent);
            } else {
                editor.selected_entities.push(ent);
                editor.selected_entities_set.insert(ent);
            }
        } else {
            editor.selected_entities.clear();
            editor.selected_entities.push(ent);
            editor.selected_entities_set.clear();
            editor.selected_entities_set.insert(ent);
        }
    } else {
        editor.selected_entities.clear();
        editor.selected_entities_set.clear();
    }
}

/// Handles left-click release — finalizes gizmo drag and commits undo history.
pub fn on_left_click_released(
    editor: &mut EditorState,
    camera: &ae_core::camera::Camera,
    gizmo_system: &mut crate::gizmo::GizmoSystem,
    ecs: &mut ae_core::ecs::EcsManager,
    ui_gizmo_space: crate::gizmo::space::GizmoSpace,
    window_size: (u32, u32),
    last_viewport_rect: ae_renderer::render::ViewportRect,
    scale_factor: f32,
    _is_point_over_ui: &dyn Fn(egui::Pos2) -> bool,
    _egui_context: &egui::Context,
    is_edit_mode: bool,
) {
    if editor.gizmo_dragging && is_edit_mode {
        let (mx, my) = editor.last_cursor_pos;
        let (local_mx, local_my, local_size) =
            map_window_to_viewport(mx, my, window_size, last_viewport_rect, scale_factor);
        if let (Some(&sel), Some(ray)) = (
            editor.selected_entities.first(),
            create_ray(camera, local_size, local_mx, local_my),
        ) {
            if let Ok(pos0) = ecs.world.get::<&Position>(sel) {
                let camera_pos = camera.position.to_vec();
                let cam_forward = camera.get_forward();
                let gizmo_screen = gizmo_screen_params(camera, local_size);

                gizmo_system.space = ui_gizmo_space;
                if let Ok(r) = ecs.world.get::<&Rotation>(sel) {
                    gizmo_system.entity_rotation = Quaternion::new(r.w, r.x, r.y, r.z);
                }

                gizmo_system.handle_input(
                    ray.origin.to_vec(),
                    ray.direction,
                    Vector3::new(pos0.x, pos0.y, pos0.z),
                    camera_pos,
                    cam_forward,
                    &gizmo_screen,
                    false,
                    false,
                    true,
                );
            }
            crate::history::commit_undo_history(editor, &ecs.world, sel);
        }
        editor.gizmo_dragging = false;
    }
}