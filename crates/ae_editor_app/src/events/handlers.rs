// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport camera navigation, object picking, hardware mouse grab, and resize handlers.

use crate::session::EditorSession;
use ae_core::modules::EngineMode;

impl EditorSession {
    /// Forwards mouse scroll deltas to the Edit-mode viewport camera zoom controller.
    pub fn handle_mouse_scroll(&mut self, scroll_amount: f32) {
        if self.window.mode != EngineMode::Edit {
            return;
        }
        let boost = self.core.input.is_shift_pressed();
        ae_camera_controller::apply_scroll_zoom(
            &mut self.render.camera,
            scroll_amount,
            boost,
            self.editor.config.camera_config(),
        );
    }

    /// Processes physical cursor motion for viewport gizmo dragging and camera look.
    pub fn handle_cursor_moved(&mut self) {
        let window_size = (
            self.render.render_state.size.width,
            self.render.render_state.size.height,
        );
        let scale_factor = self.ui.scale_factor();
        let last_viewport_rect = self.render.render_state.last_viewport_rect;
        ae_editor::interactions::handle_cursor_moved(
            &mut self.editor,
            &mut self.render.camera,
            &mut self.gizmos.transform,
            &mut self.core.world,
            ae_editor::interactions::CursorMoveParams {
                input: &self.core.input,
                window_size,
                last_viewport_rect,
                scale_factor,
                is_edit_mode: self.window.mode == EngineMode::Edit,
            },
        );
    }

    /// Routes mouse button events into Play cursor grab or Edit gizmo/picking handlers.
    pub fn handle_mouse_click(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.core.input.process_mouse_button_event(button, state);

        if self.window.mode == EngineMode::Play
            && !self.sim.state_manager.is_paused()
            && state == winit::event::ElementState::Pressed
        {
            self.set_cursor_grab(true);
        }

        if self.window.mode == EngineMode::Edit {
            let window_size = (
                self.render.render_state.size.width,
                self.render.render_state.size.height,
            );
            let scale_factor = self.ui.scale_factor();
            let ui_ref = &self.ui;
            let is_point_over_ui = move |pos| ui_ref.wants_pointer_at(pos);

            ae_editor::interactions::handle_mouse_click(
                &mut self.editor,
                &self.render.camera,
                &mut self.gizmos.transform,
                &mut self.core.world,
                ae_editor::interactions::MouseClickParams {
                    spatial_grid: &self.render.spatial_grid,
                    window_size,
                    last_viewport_rect: self.render.render_state.last_viewport_rect,
                    scale_factor,
                    is_point_over_ui: &is_point_over_ui,
                    is_edit_mode: true,
                    input: &self.core.input,
                    button,
                    state,
                },
            );
        }
    }

    /// Locks or releases hardware cursor visibility for Play mode FPS/TPS look.
    pub fn set_cursor_grab(&mut self, grabbed: bool) {
        let window = &self.render.render_state.window;
        if grabbed {
            if !self.window.is_cursor_grabbed {
                self.window.is_cursor_grabbed = true;
                window.set_cursor_visible(false);
                let grab_result = window
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
                if let Err(e) = grab_result {
                    log::warn!("Failed to lock mouse cursor: {:?}", e);
                    window.set_cursor_visible(true);
                    self.window.is_cursor_grabbed = false;
                } else {
                    let center_x = self.render.render_state.size.width as f64 / 2.0;
                    let center_y = self.render.render_state.size.height as f64 / 2.0;
                    let _ = window
                        .set_cursor_position(winit::dpi::PhysicalPosition::new(center_x, center_y));
                    self.core.input.set_cursor_pos(center_x, center_y);
                    self.core.input.clear_mouse_delta();
                }
            }
        } else if self.window.is_cursor_grabbed || self.window.mode == EngineMode::Edit {
            self.window.is_cursor_grabbed = false;
            window.set_cursor_visible(true);
            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            self.core.input.clear_mouse_delta();
        }
    }

    /// Resizes swapchain, camera aspect ratio, and UI canvas layout.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render
            .resize_dimensions(new_size.width, new_size.height);
        self.ui.on_window_resized(new_size.width, new_size.height);
    }

    /// Builds screen-space gizmo hit-test parameters for [`ae_gizmos`].
    #[must_use]
    pub fn gizmo_screen_params(&self) -> ae_gizmos::GizmoScreenParams {
        let window_size = (
            self.render.render_state.size.width,
            self.render.render_state.size.height,
        );
        ae_editor::interactions::gizmo_screen_params(&self.render.camera, window_size)
    }

    /// Releases mouse grabs and cancels ongoing gizmo/camera drags on window focus loss.
    pub fn handle_focus_change(&mut self, focused: bool) {
        if !focused {
            self.set_cursor_grab(false);
            ae_editor::interactions::handle_focus_lost(
                &mut self.editor,
                &mut self.gizmos.transform,
                &mut self.core.world,
            );
            self.core.input.clear_pressed_keys();
        }
    }

    /// Forwards raw hardware `MouseMotion` deltas for unclipped camera look.
    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        if let winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event
            && (self.editor.viewport_camera_look_active
                || (self.window.mode == EngineMode::Play
                    && self.window.is_cursor_grabbed
                    && !self.sim.state_manager.is_paused()
                    && !self.render.is_2d()))
        {
            self.core.input.add_mouse_delta(*dx as f32, *dy as f32);
        }
    }
}