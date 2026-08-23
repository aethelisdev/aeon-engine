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
    pub scale_factor: f32,
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
    /// Live rendering and geometry metrics collected during the last frame.
    pub last_render_stats: crate::render::types::FrameRenderStats,
    /// Measured GPU pass execution timings for the last completed frame.
    pub last_gpu_pass_timings: ae_core::telemetry::GpuPassTimings,
    /// Physical GPU adapter information (name, backend, device type, driver).
    pub adapter_info: wgpu::AdapterInfo,
}

impl RenderState {
    /// Computes a granular breakdown of allocated Video RAM (VRAM) across graphics subsystems.
    /// Calculates texture memory, static/dynamic vertex and index buffer footprints,
    /// uniform buffers, instance buffers, and render target allocations in megabytes.
    pub fn get_vram_breakdown(
        &self,
        asset_manager: &crate::asset::AssetManager,
    ) -> ae_core::telemetry::VramStats {
        // 1. Textures VRAM
        let mut texture_bytes: usize = 4; // default white texture
        for (_, tex) in asset_manager.textures.iter() {
            texture_bytes += (tex.width * tex.height * 4) as usize;
        }
        if let Some(vt) = &self.viewport_texture {
            texture_bytes += (vt.width * vt.height * 4) as usize;
        }

        // 2. Mesh & Index Buffers VRAM
        let vertex_size = std::mem::size_of::<crate::render::types::Vertex>();
        let index_size = std::mem::size_of::<u32>();

        let mut mesh_index_bytes: usize = 0;
        // Primitive static vertex buffers
        mesh_index_bytes += self.geometry.triangle_num_vertices as usize * vertex_size;
        mesh_index_bytes += 36 * vertex_size; // Cube
        mesh_index_bytes += 6 * vertex_size; // Grid
        mesh_index_bytes += 6 * vertex_size; // Quad
        mesh_index_bytes += self.geometry.sphere_num_vertices as usize * vertex_size;
        mesh_index_bytes += self.geometry.cylinder_num_vertices as usize * vertex_size;
        mesh_index_bytes += self.geometry.capsule_num_vertices as usize * vertex_size;
        mesh_index_bytes += self.geometry.torus_num_vertices as usize * vertex_size;

        // Model assets
        for (_, model) in asset_manager.models.iter() {
            mesh_index_bytes += model.raw_vertices.len() * vertex_size;
            mesh_index_bytes += model.raw_indices.len() * index_size;
        }

        // 3. Dynamic Uniform & Target VRAM
        let mut dynamic_uniform_bytes: usize = 0;
        // Dynamic instance buffer
        dynamic_uniform_bytes += self.geometry.instance_buffer_capacity
            * std::mem::size_of::<crate::render::types::Instance>();
        // Uniform buffers (Camera, Light, Sky, Bloom, Outline)
        dynamic_uniform_bytes += 1024; // Scene uniforms
        // Shadow depth texture (2048x2048 D32Float = 16MB) + shadow cascade uniform
        dynamic_uniform_bytes += 2048 * 2048 * 4 + 512;
        // Post-process framebuffer targets & MSAA
        let vp_w = self.size.width.max(1) as usize;
        let vp_h = self.size.height.max(1) as usize;
        dynamic_uniform_bytes += vp_w * vp_h * 4 * 2; // Scene texture + Depth texture
        if self.graphics_settings.msaa_samples > 1 {
            dynamic_uniform_bytes += vp_w * vp_h * 4 * self.graphics_settings.msaa_samples as usize;
        }
        dynamic_uniform_bytes += (vp_w * vp_h * 4) / 2; // Bloom downsample mip chain

        let bytes_to_mb = 1.0 / (1024.0 * 1024.0);
        let texture_vram_mb = texture_bytes as f32 * bytes_to_mb;
        let mesh_index_vram_mb = mesh_index_bytes as f32 * bytes_to_mb;
        let dynamic_uniform_vram_mb = dynamic_uniform_bytes as f32 * bytes_to_mb;
        let total_vram_mb = texture_vram_mb + mesh_index_vram_mb + dynamic_uniform_vram_mb;

        ae_core::telemetry::VramStats {
            texture_vram_mb,
            mesh_index_vram_mb,
            dynamic_uniform_vram_mb,
            total_vram_mb,
        }
    }
}