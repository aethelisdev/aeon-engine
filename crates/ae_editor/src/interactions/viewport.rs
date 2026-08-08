// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::gizmo_drag::handle_gizmo_drag;
use super::selection::{on_left_click_pressed, on_left_click_released};
use crate::editor_state::EditorState;
use crate::input::InputManager;

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