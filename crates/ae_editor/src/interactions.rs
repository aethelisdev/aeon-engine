// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::editor_state::EditorState;
use crate::input::{InputManager, KeyCode};
use crate::picking;
use ae_core::ecs::{Position, Rotation, Scale};
use cgmath::{EuclideanSpace, Euler, InnerSpace, Point3, Quaternion, Rad, SquareMatrix, Vector3};

/// Handles mouse scroll wheel input for camera zoom in Edit Mode.
pub fn handle_mouse_scroll(
    camera: &mut ae_core::camera::Camera,
    editor: &EditorState,
    input: &InputManager,
    is_edit_mode: bool,
    delta: &winit::event::MouseScrollDelta,
) {
    if !is_edit_mode {
        return;
    }

    let scroll_amount = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.02,
    };

    if camera.mode == ae_core::camera::ProjectionMode::Orthographic {
        let zoom_speed = 2.0;
        camera.ortho_scale -= scroll_amount * zoom_speed;
        if camera.ortho_scale < 1.0 {
            camera.ortho_scale = 1.0;
        }
    } else {
        let forward = camera.get_forward();
        let scroll_base = editor.config.camera_scroll_speed;
        let speed = if input.is_key_pressed(KeyCode::ShiftLeft) {
            scroll_base * editor.config.camera_shift_multiplier
        } else {
            scroll_base
        };

        camera.position += forward * scroll_amount * speed;
    }
}

/// Handles cursor movement events — delegates to mouse-look and gizmo drag.
pub fn handle_cursor_moved(
    editor: &mut EditorState,
    camera: &mut ae_core::camera::Camera,
    gizmo_system: &mut crate::gizmo::GizmoSystem,
    ecs: &mut ae_core::ecs::EcsManager,
    ui_gizmo_mode: crate::gizmo::GizmoMode,
    ui_gizmo_space: crate::gizmo::space::GizmoSpace,
    window_size: (u32, u32),
    last_viewport_rect: ae_renderer::render::ViewportRect,
    scale_factor: f32,
    is_edit_mode: bool,
    x: f64,
    y: f64,
) {
    let dx = x - editor.last_cursor_pos.0;
    let dy = y - editor.last_cursor_pos.1;
    editor.last_cursor_pos = (x, y);

    // Accumulate mouse_delta for camera orbit look in Play mode and Edit mode
    editor.mouse_delta.0 += dx as f32;
    editor.mouse_delta.1 += dy as f32;

    if editor.right_mouse_pressed
        && is_edit_mode
        && camera.mode != ae_core::camera::ProjectionMode::Orthographic
    {
        crate::modes::handle_mouse_look(camera, editor, dx, dy);
    }

    if editor.gizmo_dragging && editor.left_mouse_pressed && is_edit_mode {
        let (local_x, local_y, local_size) =
            map_window_to_viewport(x, y, window_size, last_viewport_rect, scale_factor);
        handle_gizmo_drag(
            editor,
            camera,
            gizmo_system,
            ecs,
            ui_gizmo_mode,
            ui_gizmo_space,
            local_size,
            local_x as f64,
            local_y as f64,
        );
    }
}

/// Processes gizmo drag movement for Translate, Rotate, and Scale modes.
pub fn handle_gizmo_drag(
    editor: &mut EditorState,
    camera: &ae_core::camera::Camera,
    gizmo_system: &mut crate::gizmo::GizmoSystem,
    ecs: &mut ae_core::ecs::EcsManager,
    ui_gizmo_mode: crate::gizmo::GizmoMode,
    ui_gizmo_space: crate::gizmo::space::GizmoSpace,
    window_size: (u32, u32),
    x: f64,
    y: f64,
) {
    let selected_ent = match editor.selected_entities.first() {
        Some(&ent) => ent,
        None => return,
    };

    let gizmo_pos = if let Ok(gt) = ecs
        .world
        .get::<&ae_core::ecs::GlobalTransform>(selected_ent)
    {
        let mat = gt.0;
        Vector3::new(mat.w.x, mat.w.y, mat.w.z)
    } else if let Ok(p) = ecs.world.get::<&Position>(selected_ent) {
        Vector3::new(p.x, p.y, p.z)
    } else {
        return;
    };

    if let Some(ray) = create_ray(camera, window_size, x as f32, y as f32) {
        let camera_pos = camera.position.to_vec();
        let cam_forward = camera.get_forward();
        let gizmo_screen = gizmo_screen_params(camera, window_size);

        gizmo_system.space = ui_gizmo_space;
        if let Ok(r) = ecs.world.get::<&Rotation>(selected_ent) {
            gizmo_system.entity_rotation = Quaternion::new(r.w, r.x, r.y, r.z);
        }

        let delta = gizmo_system.handle_input(
            ray.origin.to_vec(),
            ray.direction,
            gizmo_pos,
            camera_pos,
            cam_forward,
            &gizmo_screen,
            false,
            true,
            false,
        );

        let has_snapshots = !editor.current_edit_snapshots.is_empty();
        if delta.is_some() && has_snapshots {
            let d = delta.unwrap();
            let mut dirtied_entities = Vec::new();
            match ui_gizmo_mode {
                crate::gizmo::GizmoMode::Translate => {
                    for &ent in &editor.selected_entities {
                        if let Some(start) = editor.multi_start_positions.get(&ent) {
                            if let Ok(mut p) = ecs.world.get::<&mut Position>(ent) {
                                // Project the world drag delta d into parent's local space if the entity has a parent
                                let local_delta = if let Ok(parent_ref) =
                                    ecs.world.get::<&ae_core::ecs::Parent>(ent)
                                {
                                    if let Ok(parent_gt) = ecs
                                        .world
                                        .get::<&ae_core::ecs::GlobalTransform>(parent_ref.0)
                                    {
                                        use cgmath::SquareMatrix;
                                        if let Some(inv_parent) = parent_gt.0.invert() {
                                            let d_vec = cgmath::Vector4::new(d.x, d.y, d.z, 0.0);
                                            let local_d = inv_parent * d_vec;
                                            cgmath::Vector3::new(local_d.x, local_d.y, local_d.z)
                                        } else {
                                            d
                                        }
                                    } else {
                                        d
                                    }
                                } else {
                                    d
                                };

                                let mut target = cgmath::Vector3::new(
                                    start.x + local_delta.x,
                                    start.y + local_delta.y,
                                    start.z + local_delta.z,
                                );
                                if editor.snapping.current_enabled {
                                    target = crate::snapping::translate::snap_translation(
                                        target,
                                        editor.snapping.grid_size,
                                    );
                                }
                                p.x = target.x;
                                p.y = target.y;
                                p.z = target.z;
                                dirtied_entities.push(ent);
                            }
                        }
                    }
                }
                crate::gizmo::GizmoMode::Rotate => {
                    for &ent in &editor.selected_entities {
                        if let Some(start_rot) = editor.multi_start_rotations.get(&ent) {
                            if let Ok(mut r) = ecs.world.get::<&mut Rotation>(ent) {
                                let step = 15.0;
                                let rx = if editor.snapping.current_enabled {
                                    crate::snapping::rotate::snap_rotation(d.x, step)
                                } else {
                                    d.x
                                };
                                let ry = if editor.snapping.current_enabled {
                                    crate::snapping::rotate::snap_rotation(d.y, step)
                                } else {
                                    d.y
                                };
                                let rz = if editor.snapping.current_enabled {
                                    crate::snapping::rotate::snap_rotation(d.z, step)
                                } else {
                                    d.z
                                };

                                let q_delta = Quaternion::from(Euler {
                                    x: Rad(rx),
                                    y: Rad(ry),
                                    z: Rad(rz),
                                });
                                let q_new =
                                    if ui_gizmo_space == crate::gizmo::space::GizmoSpace::Local {
                                        start_rot * q_delta
                                    } else {
                                        q_delta * start_rot
                                    };
                                r.x = q_new.v.x;
                                r.y = q_new.v.y;
                                r.z = q_new.v.z;
                                r.w = q_new.s;
                                dirtied_entities.push(ent);
                            }
                        }
                    }
                }
                crate::gizmo::GizmoMode::Scale => {
                    for &ent in &editor.selected_entities {
                        if let Some(start) = editor.multi_start_scales.get(&ent) {
                            if let Ok(mut s) = ecs.world.get::<&mut Scale>(ent) {
                                let mut target = cgmath::Vector3::new(
                                    start.x * (1.0 + d.x),
                                    start.y * (1.0 + d.y),
                                    start.z * (1.0 + d.z),
                                );
                                if editor.snapping.current_enabled {
                                    target = crate::snapping::scale::snap_scale(target, 0.25);
                                }
                                let min = 0.001;
                                s.x = if target.x.abs() < min {
                                    f32::copysign(min, target.x)
                                } else {
                                    target.x
                                };
                                s.y = if target.y.abs() < min {
                                    f32::copysign(min, target.y)
                                } else {
                                    target.y
                                };
                                s.z = if target.z.abs() < min {
                                    f32::copysign(min, target.z)
                                } else {
                                    target.z
                                };
                                dirtied_entities.push(ent);
                            }
                        }
                    }
                }
            }

            for ent in dirtied_entities {
                let _ = ecs.world.insert_one(ent, ae_core::ecs::TransformDirty);
            }
        }
    }
}

/// Routes mouse button press/release events to the appropriate handler.
pub fn handle_mouse_click(
    editor: &mut EditorState,
    camera: &ae_core::camera::Camera,
    gizmo_system: &mut crate::gizmo::GizmoSystem,
    ecs: &mut ae_core::ecs::EcsManager,
    spatial_grid: &ae_core::spatial::SpatialGrid,
    ui_gizmo_mode: crate::gizmo::GizmoMode,
    ui_gizmo_space: crate::gizmo::space::GizmoSpace,
    window_size: (u32, u32),
    last_viewport_rect: ae_renderer::render::ViewportRect,
    scale_factor: f32,
    is_point_over_ui: &dyn Fn(egui::Pos2) -> bool,
    egui_context: &egui::Context,
    is_edit_mode: bool,
    input: &InputManager,
    button: winit::event::MouseButton,
    state: winit::event::ElementState,
) {
    let is_pressed = state == winit::event::ElementState::Pressed;

    if button == winit::event::MouseButton::Right {
        editor.right_mouse_pressed = is_pressed;
        return;
    }

    if button == winit::event::MouseButton::Left {
        editor.left_mouse_pressed = is_pressed;
        if is_pressed {
            on_left_click_pressed(
                editor,
                camera,
                gizmo_system,
                ecs,
                spatial_grid,
                ui_gizmo_mode,
                ui_gizmo_space,
                window_size,
                last_viewport_rect,
                scale_factor,
                is_point_over_ui,
                egui_context,
                is_edit_mode,
                input,
            );
        } else {
            on_left_click_released(
                editor,
                camera,
                gizmo_system,
                ecs,
                ui_gizmo_space,
                window_size,
                last_viewport_rect,
                scale_factor,
                is_point_over_ui,
                egui_context,
                is_edit_mode,
            );
        }
    }
}

/// Determines whether a mouse event should be forwarded to 3D viewport logic.
pub fn should_pass_to_3d(
    editor: &EditorState,
    last_viewport_rect: ae_renderer::render::ViewportRect,
    scale_factor: f32,
    is_point_over_ui: &dyn Fn(egui::Pos2) -> bool,
    egui_context: &egui::Context,
) -> bool {
    let (cx, cy) = editor.last_cursor_pos;
    let logical_pos = egui::pos2(cx as f32 / scale_factor, cy as f32 / scale_factor);

    if is_point_over_ui(logical_pos) {
        return false;
    }

    if egui::Popup::is_any_open(egui_context) {
        return false;
    }

    let rect = last_viewport_rect;
    let in_viewport = logical_pos.x >= rect.min_x
        && logical_pos.x <= rect.max_x
        && logical_pos.y >= rect.min_y
        && logical_pos.y <= rect.max_y;
    if in_viewport {
        return true;
    }

    if editor.right_mouse_pressed {
        return true;
    }

    false
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

                if let Some(inv_model) = model.invert() {
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

            if let Some(inv_model) = model.invert() {
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
        if let Some(t_world) = picking::intersect_sphere(&ray, sphere_center, 0.75) {
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

/// Builds a `GizmoScreenParams` struct from the current state.
pub fn gizmo_screen_params(
    camera: &ae_core::camera::Camera,
    window_size: (u32, u32),
) -> crate::gizmo::GizmoScreenParams {
    crate::gizmo::GizmoScreenParams {
        viewport_height_px: window_size.1 as f32,
        camera_fovy_deg: camera.fovy,
        axis_length_px: 80.0,
        pick_radius_px: 8.0,
        camera_mode: camera.mode,
        ortho_scale: camera.ortho_scale,
    }
}

/// Focuses the camera on the first selected entity.
pub fn focus_selected(
    camera: &mut ae_core::camera::Camera,
    editor: &EditorState,
    world: &hecs::World,
) {
    let entity = match editor.selected_entities.first() {
        Some(&e) => e,
        None => return,
    };

    let pos = match world.get::<&Position>(entity) {
        Ok(p) => Vector3::new(p.x, p.y, p.z),
        Err(_) => return,
    };

    let forward = camera.get_forward();
    let distance = 5.0;
    let new_pos = pos - forward * distance;
    camera.position = Point3::from_vec(new_pos);
    camera.target = Point3::from_vec(pos);

    let dir = (pos - new_pos).normalize();
    camera.pitch = Rad(dir.y.asin());
    camera.yaw = Rad(dir.x.atan2(dir.z));
}

/// Handles window focus loss by finalizing any active gizmo drags and committing history.
pub fn handle_focus_lost(
    editor: &mut EditorState,
    gizmo_system: &mut crate::gizmo::GizmoSystem,
    ecs: &mut ae_core::ecs::EcsManager,
) {
    if editor.gizmo_dragging {
        if let Some(&sel) = editor.selected_entities.first() {
            crate::history::commit_undo_history(editor, &ecs.world, sel);
        }
        gizmo_system.end_drag();
        editor.gizmo_dragging = false;
    }
    editor.left_mouse_pressed = false;
    editor.right_mouse_pressed = false;
}

/// Translates screen-space window coordinates to localized physical viewport coordinates.
pub fn map_window_to_viewport(
    mx: f64,
    my: f64,
    window_size: (u32, u32),
    last_viewport_rect: ae_renderer::render::ViewportRect,
    scale_factor: f32,
) -> (f32, f32, (u32, u32)) {
    let rect = last_viewport_rect;
    let vp_w_logical = rect.max_x - rect.min_x;
    let vp_h_logical = rect.max_y - rect.min_y;

    if vp_w_logical > 0.0 && vp_h_logical > 0.0 {
        let logical_x = mx as f32 / scale_factor;
        let logical_y = my as f32 / scale_factor;
        let relative_x = (logical_x - rect.min_x) * scale_factor;
        let relative_y = (logical_y - rect.min_y) * scale_factor;
        let relative_w = vp_w_logical * scale_factor;
        let relative_h = vp_h_logical * scale_factor;
        (
            relative_x,
            relative_y,
            (relative_w as u32, relative_h as u32),
        )
    } else {
        (mx as f32, my as f32, window_size)
    }
}