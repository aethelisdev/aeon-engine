// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod bloom;
pub mod grid;
pub mod pbr;
pub mod shadow;
pub mod sky;
pub mod sprite;

/// Centralized pipeline registry holding all render pipelines.
/// Owns PBR, wireframe, sprite, grid, and sky pipelines. Supports full
/// rebuild when MSAA sample count changes at runtime.
pub struct PipelineManager {
    pub render_pipeline: wgpu::RenderPipeline,
    pub wireframe_pipeline: wgpu::RenderPipeline,
    pub sprite_pipeline: wgpu::RenderPipeline,
    pub grid_pipeline: wgpu::RenderPipeline,
    pub sky_pipeline: wgpu::RenderPipeline,
}

impl PipelineManager {
    /// Creates all render pipelines using the provided bind group layouts and MSAA count.
    pub fn new(
        device: &wgpu::Device,
        camera_bgl: &wgpu::BindGroupLayout,
        light_bgl: &wgpu::BindGroupLayout,
        shadow_bgl: &wgpu::BindGroupLayout,
        texture_bgl: &wgpu::BindGroupLayout,
        sky_bgl: &wgpu::BindGroupLayout,
        scene_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let (render_pipeline, wireframe_pipeline) = pbr::create_pbr_pipelines(
            device,
            camera_bgl,
            light_bgl,
            shadow_bgl,
            scene_format,
            msaa_samples,
        );
        let grid_pipeline = grid::create_grid_pipeline(
            device,
            camera_bgl,
            light_bgl,
            shadow_bgl,
            scene_format,
            msaa_samples,
        );
        let sprite_pipeline = sprite::create_sprite_pipeline(
            device,
            camera_bgl,
            texture_bgl,
            light_bgl,
            scene_format,
            msaa_samples,
        );
        let sky_pipeline =
            sky::create_sky_pipeline(device, camera_bgl, sky_bgl, scene_format, msaa_samples);

        Self {
            render_pipeline,
            wireframe_pipeline,
            sprite_pipeline,
            grid_pipeline,
            sky_pipeline,
        }
    }

    /// Destroys and rebuilds all pipelines with a new MSAA sample count.
    pub fn rebuild_for_msaa(
        &mut self,
        device: &wgpu::Device,
        camera_bgl: &wgpu::BindGroupLayout,
        light_bgl: &wgpu::BindGroupLayout,
        shadow_bgl: &wgpu::BindGroupLayout,
        texture_bgl: &wgpu::BindGroupLayout,
        sky_bgl: &wgpu::BindGroupLayout,
        scene_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) {
        let (rp, wp) = pbr::create_pbr_pipelines(
            device,
            camera_bgl,
            light_bgl,
            shadow_bgl,
            scene_format,
            msaa_samples,
        );
        self.render_pipeline = rp;
        self.wireframe_pipeline = wp;
        self.grid_pipeline = grid::create_grid_pipeline(
            device,
            camera_bgl,
            light_bgl,
            shadow_bgl,
            scene_format,
            msaa_samples,
        );
        self.sprite_pipeline = sprite::create_sprite_pipeline(
            device,
            camera_bgl,
            texture_bgl,
            light_bgl,
            scene_format,
            msaa_samples,
        );
        self.sky_pipeline =
            sky::create_sky_pipeline(device, camera_bgl, sky_bgl, scene_format, msaa_samples);
    }
}