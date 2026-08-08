// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::selection::create_ray;
use crate::editor_state::EditorState;
use ae_core::ecs::{Position, Rotation, Scale};
use cgmath::{EuclideanSpace, Euler, Quaternion, Rad, Vector3};

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