// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use std::sync::Arc;
use winit::window::Window;

use crate::render::types::ViewportRect;
use crate::render::viewport_texture::ViewportTexture;

/// Options containing viewport visualization flags.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    pub grid_enabled: bool,
    pub wireframe_enabled: bool,
}

/// Central render state owning all WGPU resources: device, queue, surface,
/// pipeline manager, uniforms, geometry system, post-processing, and shadow cascades.
pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub viewport_texture: Option<ViewportTexture>,

    pub pipelines: crate::render::pipelines::PipelineManager,
    pub uniforms: crate::render::uniforms::SceneUniforms,
    pub geometry: crate::render::primitives::GeometrySystem,
    pub default_white_texture: crate::render::TextureAsset,

    pub post_process: crate::render::post_process::PostProcessSystem,
    pub shadow: crate::render::shadow::ShadowSystem,
    pub outline: crate::render::pipelines::outline::SelectionOutlinePass,

    // Settings
    pub graphics_settings: crate::graphics_settings::GraphicsSettings,
    /// Cached 3D viewport rect from the last completed egui frame.
    /// Updated every frame after UI renders. Used for mouse-in-viewport detection.
    pub last_viewport_rect: ViewportRect,
    /// Cached list of present modes supported by the surface on this GPU adapter.
    pub supported_present_modes: Vec<wgpu::PresentMode>,
    /// Wall-clock seconds spent blocking inside `get_current_texture()` + `present()`.
    /// On Windows DX12 when DXGI ALLOW_TEARING is unavailable, these calls block at
    /// VSync rate (~16.7ms at 60Hz). This value is subtracted from `time.delta_time`
    /// when calculating the FPS counter so that Uncapped mode displays the engine's
    /// true compute throughput rather than the display refresh rate.
    pub last_present_wait_secs: f32,
}