// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone WGPU runtime HUD overlay renderer decoupled from `winit` and editor crates.
//!
//! Renders ECS HUD components and the interactive Pause Menu directly on top of the
//! standalone runtime swapchain surface.

use crate::iris_builder::{
    TAG_PLAY_EXIT, TAG_PLAY_RESUME, build_in_game_pause_menu, build_ingame_ecs_ui,
};
use ae_renderer::render::{IrisRenderPassParams, iris_render_pass};
use irisui_core::{MouseButton, Point, Rect, Size, UiEvent, UiTree};
use irisui_layout::LayoutEngine;
use irisui_text::{
    TextCollectionOptions, TextRenderer, TextSystem, collect_text_sections_with_options,
};
use irisui_wgpu::{
    DrawCommandList, IrisRenderer, TreeCompilerOptions, compile_tree_draw_commands_into,
};
use irisui_widgets::UiScope;

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
    tree: UiTree,
    layout_engine: LayoutEngine,
    renderer: IrisRenderer,
    text_system: TextSystem,
    text_renderer: Option<TextRenderer>,
    command_list: DrawCommandList,
    cursor_pos: Point,
    target_format: wgpu::TextureFormat,
    physical_size: (u32, u32),
    scale_factor: f32,
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
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::default(),
            cursor_pos: Point::new(0.0, 0.0),
            target_format,
            physical_size,
            scale_factor: sanitize_scale_factor(scale_factor),
        }
    }

    /// Updates target surface dimensions and DPI scale factor after window resize.
    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.physical_size = physical_size;
        if scale_factor.is_finite() && scale_factor > 0.1 {
            self.scale_factor = scale_factor;
        }
    }

    /// Updates logical pointer coordinates and dispatches hover motion while paused.
    pub fn handle_pointer_moved(&mut self, physical_x: f64, physical_y: f64, is_paused: bool) {
        if !physical_x.is_finite() || !physical_y.is_finite() {
            return;
        }
        self.cursor_pos = Point::new(
            physical_x as f32 / self.scale_factor,
            physical_y as f32 / self.scale_factor,
        );
        if is_paused {
            let _ = self.tree.dispatch_event(&UiEvent::MouseMove {
                point: self.cursor_pos,
            });
        }
    }

    /// Forwards primary left mouse press/release state to the pause overlay while paused.
    pub fn handle_left_mouse_input(&mut self, pressed: bool, is_paused: bool) {
        if !is_paused {
            return;
        }
        let ui_event = if pressed {
            UiEvent::MouseDown {
                button: MouseButton::Left,
                point: self.cursor_pos,
            }
        } else {
            UiEvent::MouseUp {
                button: MouseButton::Left,
                point: self.cursor_pos,
            }
        };
        let _ = self.tree.dispatch_event(&ui_event);
    }

    /// Polls any Pause Menu button click from the last event dispatch, rebuilds the HUD tree,
    /// computes Taffy layout, and uploads text/quad GPU buffers.
    pub fn prepare_and_poll_action(
        &mut self,
        world: &ae_ecs::World,
        is_paused: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> RuntimePauseAction {
        let action = if is_paused {
            if self.tree.event_state.frame_clicked_tag == Some(TAG_PLAY_RESUME) {
                RuntimePauseAction::Resume
            } else if self.tree.event_state.frame_clicked_tag == Some(TAG_PLAY_EXIT) {
                RuntimePauseAction::Exit
            } else {
                RuntimePauseAction::None
            }
        } else {
            RuntimePauseAction::None
        };

        self.tree.clear_frame_clicks();
        self.tree.clear();
        self.command_list.clear();

        let (logical_w, logical_h) = logical_dimensions(self.physical_size, self.scale_factor);
        if let Ok(root) = self.tree.create_root() {
            let viewport_rect = Rect::new(0.0, 0.0, logical_w, logical_h);
            let mut scope = UiScope::new(&mut self.tree, root);
            build_ingame_ecs_ui(&mut scope, world, viewport_rect, self.cursor_pos);
            if is_paused {
                build_in_game_pause_menu(&mut scope, viewport_rect, "Exit Game");
            }

            let _ = self
                .layout_engine
                .compute_layout(&mut self.tree, Size::new(logical_w, logical_h));

            let mut options = TreeCompilerOptions::new()
                .with_clip(Some(viewport_rect))
                .with_hovered_id(self.tree.event_state.hovered);
            compile_tree_draw_commands_into(&self.tree, root, &mut options, &mut self.command_list);

            if self.text_renderer.is_none() {
                self.text_renderer = Some(TextRenderer::new(device, queue, self.target_format));
            }
            let text_options = TextCollectionOptions {
                hovered_id: self.tree.event_state.hovered,
                ..Default::default()
            };
            let sections = collect_text_sections_with_options(&self.tree, &text_options);
            if let Some(txt_renderer) = &mut self.text_renderer {
                txt_renderer.prepare(
                    device,
                    queue,
                    &mut self.text_system,
                    self.physical_size,
                    self.scale_factor,
                    &sections,
                );
            }
        }

        action
    }

    /// Encodes the Iris UI HUD and Pause Menu pass onto `target_view`.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        if self.command_list.quads.is_empty() && self.command_list.texture_quads.is_empty() {
            return;
        }
        let (lw, lh) = logical_dimensions(self.physical_size, self.scale_factor);
        iris_render_pass(IrisRenderPassParams {
            device,
            queue,
            encoder,
            target_view,
            renderer: &mut self.renderer,
            command_list: &self.command_list,
            text_renderer: self.text_renderer.as_ref(),
            volatile_text_renderer: None,
            screen_size: (lw as u32, lh as u32),
            dirty_boundaries: None,
        });
        self.tree.clear_dirty_boundaries();
    }
}

#[inline]
#[must_use]
fn sanitize_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.1 {
        scale_factor
    } else {
        1.0
    }
}

#[inline]
#[must_use]
fn logical_dimensions(physical_size: (u32, u32), scale_factor: f32) -> (f32, f32) {
    let scale = sanitize_scale_factor(scale_factor);
    let w = (physical_size.0 as f32 / scale).round().max(1.0);
    let h = (physical_size.1 as f32 / scale).round().max(1.0);
    (w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_factor_and_logical_dimension_guards() {
        assert_eq!(sanitize_scale_factor(f32::NAN), 1.0);
        assert_eq!(sanitize_scale_factor(0.0), 1.0);
        assert_eq!(sanitize_scale_factor(2.0), 2.0);
        assert_eq!(logical_dimensions((0, 0), f32::INFINITY), (1.0, 1.0));
        assert_eq!(logical_dimensions((1920, 1080), 2.0), (960.0, 540.0));
    }
}