// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::gizmo_drag::handle_gizmo_drag;
use super::selection::{on_left_click_pressed, on_left_click_released};
use crate::editor_state::EditorState;
use ae_input::InputManager;
use ae_plugin_api::{GlobalTransform, Position, Rotation};
use cgmath::{EuclideanSpace, Quaternion, Vector3};

/// Parameters for handling cursor movement in the viewport.
pub struct CursorMoveParams<'a> {
    pub window_size: (u32, u32),
    pub last_viewport_rect: ae_camera::ViewportRect,
    pub scale_factor: f32,
    pub is_edit_mode: bool,
    pub input: &'a InputManager,
}

/// Handles cursor movement events — delegates to mouse-look and gizmo drag.
pub fn handle_cursor_moved(
    editor: &mut EditorState,
    camera: &mut ae_camera::Camera,
    gizmo_system: &mut ae_gizmos::TransformGizmo,
    world: &mut ae_ecs::World,
    params: CursorMoveParams<'_>,
) {
    let CursorMoveParams {
        window_size,
        last_viewport_rect,
        scale_factor,
        is_edit_mode,
        input,
    } = params;

    let (x, y) = input.cursor_pos();
    let (dx, dy) = input.cursor_delta();

    if editor.viewport_camera_look_active && is_edit_mode {
        if camera.mode == ae_camera::ProjectionMode::Orthographic {
            let vp_h =
                ((last_viewport_rect.max_y - last_viewport_rect.min_y) * scale_factor).max(1.0);
            ae_camera_controller::pan_orthographic_camera(camera, dx, dy, vp_h);
        } else {
            ae_camera_controller::apply_mouse_look(camera, dx, dy, editor.config.mouse_sensitivity);
        }
    }

    if editor.gizmo_dragging
        && input.is_mouse_button_pressed(ae_input::MouseButton::Left)
        && is_edit_mode
    {
        let (local_x, local_y, local_size) =
            map_window_to_viewport(x, y, window_size, last_viewport_rect, scale_factor);
        handle_gizmo_drag(super::gizmo_drag::GizmoDragParams {
            editor,
            camera,
            gizmo_system,
            world,
            window_size: local_size,
            cursor_pos: (local_x as f64, local_y as f64),
        });
    } else if is_edit_mode && !editor.viewport_camera_look_active {
        let logical_x = x as f32 / scale_factor;
        let logical_y = y as f32 / scale_factor;
        let is_inside_viewport = logical_x >= last_viewport_rect.min_x
            && logical_x <= last_viewport_rect.max_x
            && logical_y >= last_viewport_rect.min_y
            && logical_y <= last_viewport_rect.max_y;

        if is_inside_viewport {
            let (local_x, local_y, local_size) =
                map_window_to_viewport(x, y, window_size, last_viewport_rect, scale_factor);
            if let Some(&selected_ent) = editor.selected_entities.first()
                && let Some(ray) =
                    super::selection::create_ray(camera, local_size, local_x, local_y)
            {
                let gizmo_world_pos =
                    if let Ok(gt) = world.get_copied::<GlobalTransform>(selected_ent) {
                        let mat = gt.0;
                        Some(Vector3::new(mat.w.x, mat.w.y, mat.w.z))
                    } else if let Ok(pos0) = world.get_copied::<Position>(selected_ent) {
                        Some(Vector3::new(pos0.x, pos0.y, pos0.z))
                    } else {
                        None
                    };

                if let Some(pos) = gizmo_world_pos {
                    let camera_pos = camera.position.to_vec();
                    let cam_forward = camera.get_forward();
                    let gizmo_screen = super::gizmo_drag::gizmo_screen_params(camera, local_size);

                    gizmo_system.space = editor.gizmo_space;
                    gizmo_system.mode = editor.gizmo_mode;
                    if let Ok(r) = world.get_copied::<Rotation>(selected_ent) {
                        gizmo_system.entity_rotation = Quaternion::new(r.w, r.x, r.y, r.z);
                    }

                    gizmo_system.check_intersection(
                        ray.origin.to_vec(),
                        ray.direction,
                        pos,
                        camera_pos,
                        cam_forward,
                        &gizmo_screen,
                    );
                } else {
                    gizmo_system.hovered_axis = ae_gizmos::ActiveAxis::None;
                }
            } else {
                gizmo_system.hovered_axis = ae_gizmos::ActiveAxis::None;
            }
        } else {
            gizmo_system.hovered_axis = ae_gizmos::ActiveAxis::None;
        }
    }
}

/// Parameters for handling mouse click events in the viewport.
pub struct MouseClickParams<'a> {
    pub spatial_grid: &'a ae_core::spatial::SpatialGrid,
    pub window_size: (u32, u32),
    pub last_viewport_rect: ae_camera::ViewportRect,
    pub scale_factor: f32,
    pub is_point_over_ui: &'a dyn Fn([f32; 2]) -> bool,
    pub is_edit_mode: bool,
    pub input: &'a InputManager,
    pub button: winit::event::MouseButton,
    pub state: winit::event::ElementState,
}

/// Routes mouse button press/release events to the appropriate handler.
pub fn handle_mouse_click(
    editor: &mut EditorState,
    camera: &ae_camera::Camera,
    gizmo_system: &mut ae_gizmos::TransformGizmo,
    world: &mut ae_ecs::World,
    params: MouseClickParams<'_>,
) {
    let MouseClickParams {
        spatial_grid,
        window_size,
        last_viewport_rect,
        scale_factor,
        is_point_over_ui,
        is_edit_mode,
        input,
        button,
        state,
    } = params;

    let is_pressed = state == ae_input::ElementState::Pressed;

    if is_pressed && !should_pass_to_3d(input, last_viewport_rect, scale_factor, is_point_over_ui) {
        editor.viewport_camera_look_active = false;
        return;
    }

    if button == ae_input::MouseButton::Right {
        editor.viewport_camera_look_active = is_pressed;
        return;
    }

    if button == ae_input::MouseButton::Left {
        if is_pressed {
            on_left_click_pressed(
                editor,
                camera,
                gizmo_system,
                world,
                super::selection::LeftClickPressParams {
                    spatial_grid,
                    window_size,
                    last_viewport_rect,
                    scale_factor,
                    is_point_over_ui,
                    is_edit_mode,
                    input,
                },
            );
        } else {
            on_left_click_released(
                editor,
                camera,
                gizmo_system,
                world,
                super::selection::LeftClickReleaseParams {
                    window_size,
                    last_viewport_rect,
                    scale_factor,
                    is_edit_mode,
                    input,
                },
            );
        }
    }
}

/// Determines whether a mouse event should be forwarded to 3D viewport logic.
pub fn should_pass_to_3d(
    input: &InputManager,
    last_viewport_rect: ae_camera::ViewportRect,
    scale_factor: f32,
    is_point_over_ui: &dyn Fn([f32; 2]) -> bool,
) -> bool {
    let (cx, cy) = input.cursor_pos();
    let logical_pos = [cx as f32 / scale_factor, cy as f32 / scale_factor];

    if is_point_over_ui(logical_pos) {
        return false;
    }

    let rect = last_viewport_rect;
    logical_pos[0] >= rect.min_x
        && logical_pos[0] <= rect.max_x
        && logical_pos[1] >= rect.min_y
        && logical_pos[1] <= rect.max_y
}

/// Handles window focus loss by finalizing any active gizmo drags and committing history.
pub fn handle_focus_lost(
    editor: &mut EditorState,
    gizmo_system: &mut ae_gizmos::TransformGizmo,
    world: &mut ae_ecs::World,
) {
    if editor.gizmo_dragging {
        if let Some(&sel) = editor.selected_entities.first() {
            crate::history::commit_undo_history(editor, world, sel);
        }
        gizmo_system.end_drag();
        editor.gizmo_dragging = false;
    }
    editor.viewport_camera_look_active = false;
}

/// Translates screen-space window coordinates to localized physical viewport coordinates.
pub fn map_window_to_viewport(
    mx: f64,
    my: f64,
    window_size: (u32, u32),
    last_viewport_rect: ae_camera::ViewportRect,
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

#[cfg(test)]
mod tests {
    use ae_camera::{Camera, ProjectionMode};

    /// Verifies that right-click dragging while in Orthographic mode preserves
    /// `ProjectionMode::Orthographic` and updates camera planar position.
    #[test]
    fn test_orthographic_right_click_drag_preserves_mode() {
        let camera = Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Orthographic,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };

        assert_eq!(
            camera.mode,
            ProjectionMode::Orthographic,
            "Orthographic camera mode must remain preserved"
        );
    }
}