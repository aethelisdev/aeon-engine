// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction, Viewport, and Hardware Device Event Handlers.
//!
//! Provides granular event processing logic for 3D camera navigation,
//! entity picking raycasts, hardware mouse locking, and viewport resizing.

use crate::engine::AeEngine;
use ae_core::modules::EngineMode;

impl AeEngine {
    /// Forwards mouse scroll deltas to the camera interaction subsystem.
    ///
    /// Drives smooth zoom and dolly movements for the active 3D perspective camera
    /// based on mouse wheel rotational steps or trackpad offsets.
    pub fn handle_mouse_scroll(&mut self, scroll_amount: f32) {
        if self.mode != EngineMode::Edit {
            return;
        }
        let boost = self.input.is_shift_pressed();
        ae_camera_controller::apply_scroll_zoom(
            &mut self.camera,
            scroll_amount,
            boost,
            self.editor.config.camera_config(),
        );
    }

    /// Processes 2D cursor movement in physical screen coordinates.
    ///
    /// Translates coordinates through the HighDPI scale factor and forwards
    /// them to gizmo manipulation, orbit navigation, and object picking.
    pub fn handle_cursor_moved(&mut self) {
        let window_size = (self.render_state.size.width, self.render_state.size.height);
        let scale_factor = self.ui.scale_factor();
        let last_viewport_rect = self.render_state.last_viewport_rect;
        ae_editor::interactions::handle_cursor_moved(
            &mut self.editor,
            &mut self.camera,
            &mut self.transform_gizmo,
            &mut self.world,
            ae_editor::interactions::CursorMoveParams {
                input: &self.input,
                window_size,
                last_viewport_rect,
                scale_factor,
                is_edit_mode: self.mode == EngineMode::Edit,
            },
        );
    }

    /// Forwards mouse button press and release events to the interaction subsystem.
    ///
    /// In Play mode, acquires pointer capture when clicked. In Edit mode, coordinates
    /// gizmo handle activation, entity selection, and viewport orbit triggers.
    pub fn handle_mouse_click(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.input.process_mouse_button_event(button, state);

        if self.mode == EngineMode::Play
            && !self.state_manager.is_paused()
            && state == winit::event::ElementState::Pressed
        {
            self.set_cursor_grab(true);
        }

        if self.mode == EngineMode::Edit {
            let window_size = (self.render_state.size.width, self.render_state.size.height);
            let scale_factor = self.ui.scale_factor();
            let ui_ref = &self.ui;
            let is_point_over_ui = move |pos| ui_ref.wants_pointer_at(pos);

            ae_editor::interactions::handle_mouse_click(
                &mut self.editor,
                &self.camera,
                &mut self.transform_gizmo,
                &mut self.world,
                ae_editor::interactions::MouseClickParams {
                    spatial_grid: &self.spatial_grid,
                    window_size,
                    last_viewport_rect: self.render_state.last_viewport_rect,
                    scale_factor,
                    is_point_over_ui: &is_point_over_ui,
                    is_edit_mode: true,
                    input: &self.input,
                    button,
                    state,
                },
            );
        }
    }

    /// Configures mouse cursor locking and visibility for Play Mode vs Edit Mode transitions.
    ///
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
                } else {
                    // Warp cursor to the center of the window so it is safely away from any menu buttons
                    let w = self.render_state.size.width as f64;
                    let h = self.render_state.size.height as f64;
                    let center_x = w / 2.0;
                    let center_y = h / 2.0;
                    let _ = window
                        .set_cursor_position(winit::dpi::PhysicalPosition::new(center_x, center_y));
                    self.input.set_cursor_pos(center_x, center_y);
                    self.input.clear_mouse_delta();
                }
            }
        } else if self.is_cursor_grabbed || self.mode == EngineMode::Edit {
            self.is_cursor_grabbed = false;
            window.set_cursor_visible(true);
            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            self.input.clear_mouse_delta();
        }
    }

    /// Handles window resize by updating the render surface and camera aspect ratio.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render_state.resize(new_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
        self.ui.on_window_resized(new_size.width, new_size.height);
    }

    /// Creates a ray from 2D viewport mouse coordinates for entity picking.
    #[must_use]
    pub fn create_ray(&self, mx: f32, my: f32) -> Option<ae_editor::picking::Ray> {
        let window_size = (self.render_state.size.width, self.render_state.size.height);
        ae_editor::interactions::create_ray(&self.camera, window_size, mx, my)
    }

    /// Returns screen space transformation parameters for rendering transformation gizmos.
    #[must_use]
    pub fn gizmo_screen_params(&self) -> ae_gizmos::GizmoScreenParams {
        let window_size = (self.render_state.size.width, self.render_state.size.height);
        ae_editor::interactions::gizmo_screen_params(&self.camera, window_size)
    }

    /// Handles focus change events (e.g. window losing/gaining focus).
    pub fn handle_focus_change(&mut self, focused: bool) {
        if !focused {
            self.set_cursor_grab(false);
            ae_editor::interactions::handle_focus_lost(
                &mut self.editor,
                &mut self.transform_gizmo,
                &mut self.world,
            );
            self.input.clear_pressed_keys();
        }
    }

    /// Handles raw device events (such as hardware un-clamped MouseMotion) for camera control.
    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        if let winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event
            && (self.editor.viewport_camera_look_active
                || (self.mode == EngineMode::Play
                    && self.is_cursor_grabbed
                    && !self.state_manager.is_paused()
                    && !self.dimension_mode.is_2d()))
        {
            self.input.add_mouse_delta(*dx as f32, *dy as f32);
        }
    }
}