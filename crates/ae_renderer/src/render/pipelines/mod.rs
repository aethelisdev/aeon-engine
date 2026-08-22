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

/// Parameters required for configuring and building all render pipelines.
pub struct PipelineConfigParams<'a> {
    pub camera_bgl: &'a wgpu::BindGroupLayout,
    pub light_bgl: &'a wgpu::BindGroupLayout,
    pub shadow_bgl: &'a wgpu::BindGroupLayout,
    pub texture_bgl: &'a wgpu::BindGroupLayout,
    pub sky_bgl: &'a wgpu::BindGroupLayout,
    pub scene_format: wgpu::TextureFormat,
    pub msaa_samples: u32,
}

impl PipelineManager {
    /// Creates all render pipelines using the provided bind group layouts and MSAA count.
    pub fn new(device: &wgpu::Device, params: &PipelineConfigParams<'_>) -> Self {
        let pbr_params = pbr::PbrPipelineParams {
            camera_bind_group_layout: params.camera_bgl,
            light_bind_group_layout: params.light_bgl,
            shadow_bind_group_layout: params.shadow_bgl,
            texture_bind_group_layout: params.texture_bgl,
            scene_format: params.scene_format,
            msaa_samples: params.msaa_samples,
        };
        let pbr_pipes = pbr::create_pbr_pipelines(device, &pbr_params);
        let grid_pipeline = grid::create_grid_pipeline(
            device,
            params.camera_bgl,
            params.light_bgl,
            params.shadow_bgl,
            params.scene_format,
            params.msaa_samples,
        );
        let sprite_pipeline = sprite::create_sprite_pipeline(
            device,
            params.camera_bgl,
            params.texture_bgl,
            params.light_bgl,
            params.scene_format,
            params.msaa_samples,
        );
        let sky_pipeline = sky::create_sky_pipeline(
            device,
            params.camera_bgl,
            params.sky_bgl,
            params.scene_format,
            params.msaa_samples,
        );

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
    pub fn rebuild_for_msaa(&mut self, device: &wgpu::Device, params: &PipelineConfigParams<'_>) {
        let pbr_params = pbr::PbrPipelineParams {
            camera_bind_group_layout: params.camera_bgl,
            light_bind_group_layout: params.light_bgl,
            shadow_bind_group_layout: params.shadow_bgl,
            texture_bind_group_layout: params.texture_bgl,
            scene_format: params.scene_format,
            msaa_samples: params.msaa_samples,
        };
        let pbr_pipes = pbr::create_pbr_pipelines(device, &pbr_params);
        let grid_pipeline = grid::create_grid_pipeline(
            device,
            params.camera_bgl,
            params.light_bgl,
            params.shadow_bgl,
            params.scene_format,
            params.msaa_samples,
        );
        let sprite_pipeline = sprite::create_sprite_pipeline(
            device,
            params.camera_bgl,
            params.texture_bgl,
            params.light_bgl,
            params.scene_format,
            params.msaa_samples,
        );
        let sky_pipeline = sky::create_sky_pipeline(
            device,
            params.camera_bgl,
            params.sky_bgl,
            params.scene_format,
            params.msaa_samples,
        );

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