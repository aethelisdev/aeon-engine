// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor UI Render Types and Parameters
//!
//! Provides the descriptor structures required for rendering a single frame of the Editor UI.

use crate::ui::types::EngineUiAction;
use winit::window::Window;

/// Parameters for rendering the entire Editor UI frame.
pub struct EditorUiRenderParams<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a Window,
    pub window_surface_view: &'a wgpu::TextureView,
    pub viewport_texture_view: Option<&'a wgpu::TextureView>,
    pub fps: f32,
    pub world: &'a hecs::World,
    pub mode: &'a ae_core::modules::EngineMode,
    pub undo_stack: &'a [ae_editor::undo_redo::Command],
    pub redo_stack: &'a [ae_editor::undo_redo::Command],
    pub graphics_settings: &'a ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping: &'a ae_editor::snapping::SnapSettings,
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    pub camera: &'a ae_renderer::camera::Camera,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub shaders: &'a ae_renderer::asset::AssetStorage<ae_renderer::asset::ShaderAsset>,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
}