// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::selection::create_ray;
use crate::editor_state::EditorState;
use ae_plugin_api::{Position, Rotation, Scale};
use cgmath::{EuclideanSpace, InnerSpace, Quaternion, Rad, Rotation3, Vector3};

/// Parameters for processing gizmo drag interactions.
pub struct GizmoDragParams<'a> {
    pub editor: &'a mut EditorState,
    pub camera: &'a ae_core::camera::Camera,
    pub gizmo_system: &'a mut ae_gizmos::TransformGizmo,
    pub world: &'a mut ae_ecs::World,
    pub window_size: (u32, u32),
    pub cursor_pos: (f64, f64),
}

/// Processes gizmo drag movement for Translate, Rotate, and Scale modes.
pub fn handle_gizmo_drag(params: GizmoDragParams<'_>) {
    let editor = params.editor;
    let camera = params.camera;
    let gizmo_system = params.gizmo_system;
    let world = params.world;
    let gizmo_mode = editor.gizmo_mode;
    let gizmo_space = editor.gizmo_space;
    let window_size = params.window_size;
    let (x, y) = params.cursor_pos;
    let selected_ent = match editor.selected_entities.first() {
        Some(&ent) => ent,
        None => return,
    };

    let gizmo_pos = if let Ok(gt) = world.get_copied::<ae_plugin_api::GlobalTransform>(selected_ent)
    {
        let mat = gt.0;
        Vector3::new(mat.w.x, mat.w.y, mat.w.z)
    } else if let Ok(p) = world.get_copied::<Position>(selected_ent) {
        Vector3::new(p.x, p.y, p.z)
    } else {
        return;
    };

    if let Some(ray) = create_ray(camera, window_size, x as f32, y as f32) {
        let camera_pos = camera.position.to_vec();
        let cam_forward = camera.get_forward();
        let gizmo_screen = gizmo_screen_params(camera, window_size);

        gizmo_system.space = gizmo_space;
        if let Ok(r) = world.get_copied::<Rotation>(selected_ent) {
            gizmo_system.entity_rotation = Quaternion::new(r.w, r.x, r.y, r.z);
        }

        let delta = gizmo_system.handle_input(ae_gizmos::GizmoInputParams {
            ray_origin: ray.origin.to_vec(),
            ray_dir: ray.direction,
            gizmo_pos,
            camera_pos,
            cam_forward,
            screen: &gizmo_screen,
            left_just_pressed: false,
            left_pressed: true,
            left_released: false,
        });

        let has_snapshots = !editor.current_edit_snapshots.is_empty();
        if let Some(d) = delta
            && has_snapshots
        {
            let mut dirtied_entities = Vec::new();
            match gizmo_mode {
                ae_gizmos::GizmoMode::Select => {}
                ae_gizmos::GizmoMode::Translate => {
                    for &ent in &editor.selected_entities {
                        let Some(start) = editor.multi_start_positions.get(&ent).copied() else {
                            continue;
                        };
                        // Project the world drag delta d into parent's local space if the entity has a parent
                        let local_delta = if let Ok(parent_ref) =
                            world.get_copied::<ae_plugin_api::Parent>(ent)
                            && let Ok(parent_gt) =
                                world.get_copied::<ae_plugin_api::GlobalTransform>(parent_ref.0)
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
                        };
                        if let Ok(p) = world.get_mut::<Position>(ent) {
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
                ae_gizmos::GizmoMode::Rotate => {
                    for &ent in &editor.selected_entities {
                        if let Some(start_rot) = editor.multi_start_rotations.get(&ent)
                            && let Ok(r) = world.get_mut::<Rotation>(ent)
                        {
                            let step = 15.0;
                            let (axis_vec, raw_angle) = match gizmo_system.active_axis {
                                ae_gizmos::ActiveAxis::X => (Vector3::unit_x(), d.x),
                                ae_gizmos::ActiveAxis::Y => (Vector3::unit_y(), d.y),
                                ae_gizmos::ActiveAxis::Z => (Vector3::unit_z(), d.z),
                                ae_gizmos::ActiveAxis::Screen => {
                                    (camera.get_forward().normalize(), d.x)
                                }
                                _ => (Vector3::unit_y(), 0.0),
                            };
                            let angle = if editor.snapping.current_enabled {
                                crate::snapping::rotate::snap_rotation(raw_angle, step)
                            } else {
                                raw_angle
                            };
                            let q_delta = Quaternion::from_axis_angle(axis_vec, Rad(angle));

                            let q_new = if gizmo_system.active_axis == ae_gizmos::ActiveAxis::Screen
                            {
                                q_delta * start_rot
                            } else if gizmo_space == ae_gizmos::GizmoSpace::Local {
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
                ae_gizmos::GizmoMode::Scale => {
                    for &ent in &editor.selected_entities {
                        if let Some(start) = editor.multi_start_scales.get(&ent)
                            && let Ok(s) = world.get_mut::<Scale>(ent)
                        {
                            let mut target = cgmath::Vector3::new(
                                start.x * (1.0 + d.x),
                                start.y * (1.0 + d.y),
                                start.z * (1.0 + d.z),
                            );
                            if editor.snapping.current_enabled {
                                target = crate::snapping::scale::snap_scale(target, 0.25);
                            }
                            let min = 1e-4;
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

            for ent in dirtied_entities {
                let _ = world.insert_one(ent, ae_plugin_api::TransformDirty);
            }
        }
    }
}

/// Builds a `GizmoScreenParams` struct from the current state.
pub fn gizmo_screen_params(
    camera: &ae_core::camera::Camera,
    window_size: (u32, u32),
) -> ae_gizmos::GizmoScreenParams {
    ae_gizmos::GizmoScreenParams {
        viewport_height_px: window_size.1 as f32,
        camera_fovy_deg: camera.fovy,
        axis_length_px: 96.0,
        pick_radius_px: 14.0,
        camera_mode: camera.mode,
        ortho_scale: camera.ortho_scale,
    }
}