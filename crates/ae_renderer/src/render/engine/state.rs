// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use std::sync::Arc;
use winit::window::Window;

use super::frame::forward::{PreparedModelBatch, TransparentSubmeshDraw};
use crate::render::types::ViewportRect;
use crate::render::viewport_texture::ViewportTexture;

/// Persistent per-frame scratch buffers reused across renders to eliminate heap allocations in the hot path.
#[derive(Default)]
pub struct RenderFrameScratch {
    /// Combined flat instance data uploaded to GPU instance buffer.
    pub all_instances: Vec<crate::render::types::Instance>,
    /// Flattened model instance items referencing textures and reflection states.
    pub model_instances: Vec<crate::render::types::RenderInstanceItem>,
    /// Active 3D model batches prepared for the current frame.
    pub model_batches: Vec<PreparedModelBatch>,
    /// Instance buffer start offsets mapped by model asset handle.
    pub model_starts: std::collections::HashMap<crate::asset::AssetHandle, usize>,
    /// Sorted transparent model submesh draw operations.
    pub transparent_model_draws: Vec<TransparentSubmeshDraw>,
}

impl RenderFrameScratch {
    /// Clears all scratch buffers while retaining allocated heap capacities.
    pub fn clear(&mut self) {
        self.all_instances.clear();
        self.model_instances.clear();
        self.model_batches.clear();
        self.model_starts.clear();
        self.transparent_model_draws.clear();
    }
}

/// Options containing viewport visualization flags.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    pub grid_enabled: bool,
    pub wireframe_enabled: bool,
    pub scale_factor: f32,
    /// Indicates whether the active viewport is running in 2D dimension mode.
    ///
    /// When `true`, completely bypasses all 3D shadow cascades, 3D PBR forward passes,
    /// and 3D silhouette outlines, delegating rendering to the isolated 2D pipeline.
    pub is_2d_mode: bool,
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
    /// Cached 3D viewport rect from the last completed UI frame.
    /// Updated every frame after UI renders. Used for mouse-in-viewport detection.
    pub last_viewport_rect: ViewportRect,
    /// Cached list of present modes supported by the surface on this GPU adapter.
    pub supported_present_modes: Vec<wgpu::PresentMode>,
    /// Wall-clock seconds spent blocking inside `get_current_texture()` + `present()`.
    ///
    /// On Windows DX12 when DXGI ALLOW_TEARING is unavailable, these calls block at
    /// VSync rate (~16.7ms at 60Hz). This value is subtracted from `time.delta_time`
    /// when calculating the FPS counter so that Uncapped mode displays the engine's
    /// true compute throughput rather than the display refresh rate.
    pub last_present_wait_secs: f32,
    /// Live rendering and geometry metrics collected during the last frame.
    pub last_render_stats: crate::render::types::FrameRenderStats,
    /// Measured CPU command buffer recording and pass orchestration timings for the last completed frame.
    pub last_recording_timings: ae_core::telemetry::RenderPassRecordingTimings,
    /// Physical GPU adapter information (name, backend, device type, driver).
    pub adapter_info: wgpu::AdapterInfo,
    /// Persistent frame scratch buffers for zero-allocation rendering in hot paths.
    pub frame_scratch: RenderFrameScratch,
}

impl RenderState {
    /// Computes a granular breakdown of allocated Video RAM (VRAM) across graphics subsystems.
    ///
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetHandle;
    use crate::render::types::Instance;

    #[test]
    fn test_render_frame_scratch_clear_retains_capacity() {
        let mut scratch = RenderFrameScratch::default();
        let mut sm = slotmap::SlotMap::<AssetHandle, ()>::with_key();
        let handles: Vec<AssetHandle> = (0..100).map(|_| sm.insert(())).collect();

        for (i, &handle) in handles.iter().enumerate() {
            scratch.all_instances.push(Instance::default());
            scratch.model_batches.push(PreparedModelBatch {
                handle,
                instance_start: i,
                instance_count: 1,
                buffer_offset: i,
            });
            scratch.model_starts.insert(handle, i);
            scratch
                .transparent_model_draws
                .push(TransparentSubmeshDraw {
                    distance_sq: (i as f32) * 2.0,
                    handle,
                    submesh_index: 0,
                    buffer_offset: i,
                    instance_index: 0,
                    override_tex: None,
                });
        }

        assert_eq!(scratch.all_instances.len(), 100);
        assert_eq!(scratch.model_batches.len(), 100);
        assert_eq!(scratch.model_starts.len(), 100);
        assert_eq!(scratch.transparent_model_draws.len(), 100);

        let cap_instances = scratch.all_instances.capacity();
        let cap_batches = scratch.model_batches.capacity();
        let cap_starts = scratch.model_starts.capacity();
        let cap_draws = scratch.transparent_model_draws.capacity();

        assert!(cap_instances >= 100);
        assert!(cap_batches >= 100);
        assert!(cap_starts >= 100);
        assert!(cap_draws >= 100);

        scratch.clear();

        assert_eq!(scratch.all_instances.len(), 0);
        assert_eq!(scratch.model_batches.len(), 0);
        assert_eq!(scratch.model_starts.len(), 0);
        assert_eq!(scratch.transparent_model_draws.len(), 0);

        assert_eq!(scratch.all_instances.capacity(), cap_instances);
        assert_eq!(scratch.model_batches.capacity(), cap_batches);
        assert_eq!(scratch.model_starts.capacity(), cap_starts);
        assert_eq!(scratch.transparent_model_draws.capacity(), cap_draws);
    }
}