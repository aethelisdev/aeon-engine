// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone WGPU runtime HUD overlay renderer decoupled from `winit` and editor crates.
//!
//! Renders ECS HUD components and the interactive Pause Menu directly on top of the
//! standalone runtime swapchain surface via [`SimpleUiHost`].

use crate::iris_builder::{
    TAG_PLAY_EXIT, TAG_PLAY_RESUME, build_in_game_pause_menu, build_ingame_ecs_ui,
};
use irisui_core::MouseButton;
use irisui_widgets::SimpleUiHost;

/// Action returned by [`RuntimeHudOverlay::prepare_and_poll_action`] when a pause menu button is clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePauseAction {
    /// No pause menu button was clicked this frame.
    None,
    /// User clicked the Resume button.
    Resume,
    /// User clicked the Exit button.
    Exit,
}

/// Standalone Iris UI overlay renderer for exported / standalone runtime players.
pub struct RuntimeHudOverlay {
    host: SimpleUiHost,
}

impl RuntimeHudOverlay {
    /// Creates a new standalone HUD overlay renderer bound to the swapchain surface format.
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        physical_size: (u32, u32),
        scale_factor: f32,
    ) -> Self {
        Self {
            host: SimpleUiHost::new(device, target_format, physical_size, scale_factor),
        }
    }

    /// Updates target surface dimensions and DPI scale factor after window resize.
    #[inline]
    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.host.resize(physical_size, scale_factor);
    }

    /// Updates logical pointer coordinates and dispatches hover motion while paused.
    #[inline]
    pub fn handle_pointer_moved(&mut self, physical_x: f64, physical_y: f64, is_paused: bool) {
        let _ = self
            .host
            .handle_pointer_moved(physical_x, physical_y, is_paused);
    }

    /// Forwards primary left mouse press/release state to the pause overlay while paused.
    #[inline]
    pub fn handle_left_mouse_input(&mut self, pressed: bool, is_paused: bool) {
        if !is_paused {
            return;
        }
        let _ = self.host.handle_mouse_button(MouseButton::Left, pressed);
    }

    /// Polls any Pause Menu button click from the last event dispatch, rebuilds the HUD tree,
    /// computes layout, and compiles GPU draw commands.
    pub fn prepare_and_poll_action(
        &mut self,
        world: &ae_ecs::World,
        is_paused: bool,
    ) -> RuntimePauseAction {
        let action = if !is_paused {
            RuntimePauseAction::None
        } else if self.host.is_tag_clicked(TAG_PLAY_RESUME) {
            RuntimePauseAction::Resume
        } else if self.host.is_tag_clicked(TAG_PLAY_EXIT) {
            RuntimePauseAction::Exit
        } else {
            RuntimePauseAction::None
        };

        self.host.prepare_frame(|scope, ctx| {
            build_ingame_ecs_ui(scope, world, ctx.viewport_rect, ctx.cursor_pos);
            if is_paused {
                build_in_game_pause_menu(scope, ctx.viewport_rect, "Exit Game");
            }
        });

        action
    }

    /// Uploads text/quad resources and encodes the Iris UI HUD and Pause Menu pass onto `target_view`.
    #[inline]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        self.host
            .render_to_view(device, queue, encoder, target_view);
    }
}