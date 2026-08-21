// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod bloom;
pub mod grid;
pub mod outline;
pub mod pbr;
pub mod shadow;
pub mod sky;
pub mod sprite;

/// Centralized pipeline registry holding all render pipelines.
/// Owns PBR, wireframe, sprite, grid, and sky pipelines. Supports full
/// rebuild when MSAA sample count changes at runtime.
pub struct PipelineManager {
    pub render_pipeline: wgpu::RenderPipeline,
    pub render_pipeline_cw: wgpu::RenderPipeline,
    pub cutout_pipeline: wgpu::RenderPipeline,
    pub cutout_pipeline_cw: wgpu::RenderPipeline,
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub wireframe_pipeline: wgpu::RenderPipeline,
    pub sprite_pipeline: wgpu::RenderPipeline,
    pub grid_pipeline: wgpu::RenderPipeline,
    pub sky_pipeline: wgpu::RenderPipeline,
}

impl PipelineManager {
    /// Creates all render pipelines using the provided bind group layouts and MSAA count.
    #[allow(clippy::too_many_arguments)]
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
        let pbr_pipes = pbr::create_pbr_pipelines(
            device,
            camera_bgl,
            light_bgl,
            shadow_bgl,
            texture_bgl,
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
            render_pipeline: pbr_pipes.render_pipeline,
            render_pipeline_cw: pbr_pipes.render_pipeline_cw,
            cutout_pipeline: pbr_pipes.cutout_pipeline,
            cutout_pipeline_cw: pbr_pipes.cutout_pipeline_cw,
            transparent_pipeline: pbr_pipes.transparent_pipeline,
            wireframe_pipeline: pbr_pipes.wireframe_pipeline,
            sprite_pipeline,
            grid_pipeline,
            sky_pipeline,
        }
    }

    /// Destroys and rebuilds all pipelines with a new MSAA sample count.
    #[allow(clippy::too_many_arguments)]
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
        let pbr_pipes = pbr::create_pbr_pipelines(
            device,
            camera_bgl,
            light_bgl,
            shadow_bgl,
            texture_bgl,
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

        self.render_pipeline = pbr_pipes.render_pipeline;
        self.render_pipeline_cw = pbr_pipes.render_pipeline_cw;
        self.cutout_pipeline = pbr_pipes.cutout_pipeline;
        self.cutout_pipeline_cw = pbr_pipes.cutout_pipeline_cw;
        self.transparent_pipeline = pbr_pipes.transparent_pipeline;
        self.wireframe_pipeline = pbr_pipes.wireframe_pipeline;
        self.grid_pipeline = grid_pipeline;
        self.sprite_pipeline = sprite_pipeline;
        self.sky_pipeline = sky_pipeline;
    }
}