// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::engine::AeEngine;
use ae_core::modules::EngineMode;
use winit::event::WindowEvent;

impl AeEngine {
    /// Forwards mouse scroll to the interaction subsystem.
    pub fn handle_mouse_scroll(
        &mut self,
        delta: &winit::event::MouseScrollDelta,
        _cursor_x: f64,
        _cursor_y: f64,
    ) {
        ae_editor::interactions::handle_mouse_scroll(
            &mut self.camera,
            &self.editor,
            &self.input,
            self.mode == EngineMode::Edit,
            delta,
        );
    }

    pub fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        let window_size = (self.render_state.size.width, self.render_state.size.height);
        let scale_factor = self.render_state.window.scale_factor() as f32;
        let last_viewport_rect = self.render_state.last_viewport_rect;
        ae_editor::interactions::handle_cursor_moved(
            &mut self.editor,
            &mut self.camera,
            &mut self.gizmo_system,
            &mut self.ecs,
            self.ui.gizmo_mode,
            self.ui.gizmo_space,
            window_size,
            last_viewport_rect,
            scale_factor,
            self.mode == EngineMode::Edit,
            x,
            y,
        );
    }

    /// Forwards mouse click to the interaction subsystem.
    pub fn handle_mouse_click(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        if self.mode == EngineMode::Play && state == winit::event::ElementState::Pressed {
            self.set_cursor_grab(true);
        }

        let window_size = (self.render_state.size.width, self.render_state.size.height);
        let scale_factor = self.render_state.window.scale_factor() as f32;
        let ui_ref = &self.ui;
        let is_point_over_ui = move |pos| ui_ref.is_point_over_ui_rects(pos);

        ae_editor::interactions::handle_mouse_click(
            &mut self.editor,
            &self.camera,
            &mut self.gizmo_system,
            &mut self.ecs,
            &self.spatial_grid,
            self.ui.gizmo_mode,
            self.ui.gizmo_space,
            window_size,
            self.render_state.last_viewport_rect,
            scale_factor,
            &is_point_over_ui,
            &self.ui.context,
            self.mode == EngineMode::Edit,
            &self.input,
            button,
            state,
        );
    }

    /// Configures mouse cursor locking and visibility for Play Mode vs Edit Mode transitions.
    /// Prevents redundant OS `ShowCursor` calls by tracking `is_cursor_grabbed`.
    /// Uses `CursorGrabMode::Locked` for zero-lag pointer lock in FPS/TPS viewports,
    /// falling back to `CursorGrabMode::Confined` if the host OS window manager restricts locking.
    /// Hides the mouse cursor when grabbed, and restores normal cursor visibility when ungrabbed.
    pub fn set_cursor_grab(&mut self, grabbed: bool) {
        let window = &self.render_state.window;
        if grabbed {
            if !self.is_cursor_grabbed {
                self.is_cursor_grabbed = true;
                window.set_cursor_visible(false);
                let grab_result = window
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
                if let Err(e) = grab_result {
                    log::warn!("Failed to lock mouse cursor: {:?}", e);
                    window.set_cursor_visible(true);
                    self.is_cursor_grabbed = false;
                }
            }
        } else {
            if self.is_cursor_grabbed || self.mode == EngineMode::Edit {
                self.is_cursor_grabbed = false;
                window.set_cursor_visible(true);
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                self.editor.mouse_delta = (0.0, 0.0);
            }
        }
    }

    /// Handles window resize by updating the render surface and camera aspect ratio.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render_state.resize(new_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    /// Creates a ray from 2D viewport mouse coordinates for entity picking.
    pub fn create_ray(&self, mx: f32, my: f32) -> Option<ae_editor::picking::Ray> {
        let window_size = (self.render_state.size.width, self.render_state.size.height);
        ae_editor::interactions::create_ray(&self.camera, window_size, mx, my)
    }

    pub fn gizmo_screen_params(&self) -> ae_editor::gizmo::GizmoScreenParams {
        let window_size = (self.render_state.size.width, self.render_state.size.height);
        ae_editor::interactions::gizmo_screen_params(&self.camera, window_size)
    }

    /// Handles focus change events (e.g. window losing/gaining focus).
    pub fn handle_focus_change(&mut self, focused: bool) {
        if !focused {
            self.set_cursor_grab(false);
            ae_editor::interactions::handle_focus_lost(
                &mut self.editor,
                &mut self.gizmo_system,
                &mut self.ecs,
            );
            self.input.clear_pressed_keys();
        }
    }

    /// Handles non-input window events (CursorMoved for delta tracking, DroppedFile for import).
    /// Consumed window events are ignored to prevent clicks/drags from passing to the 3D scene.
    pub fn handle_window_event(&mut self, event: &WindowEvent, consumed: bool) {
        if consumed {
            return;
        }
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position.x, position.y);
            }
            WindowEvent::DroppedFile(path) => {
                crate::importer::handle_dropped_file(self, path.clone());
            }
            _ => {}
        }
    }

    /// Handles raw device events (such as hardware un-clamped MouseMotion) for rock solid cross-platform camera control.
    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        if let winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if self.editor.right_mouse_pressed || self.mode == EngineMode::Play {
                self.editor.mouse_delta.0 += *dx as f32;
                self.editor.mouse_delta.1 += *dy as f32;
            }
        }
    }
}