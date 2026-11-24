// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! PBR opaque, cutout, transparent, and wireframe WGPU render pipeline builders.

use crate::vertex::{Instance, PBR_DEPTH_FORMAT, vertex_buffer_layout};

/// Holds all 3D PBR pipeline variants (CCW, CW mirrored, cutout, transparent, wireframe).
pub(crate) struct PbrPipelines {
    pub render_pipeline: wgpu::RenderPipeline,
    pub render_pipeline_cw: wgpu::RenderPipeline,
    pub cutout_pipeline: wgpu::RenderPipeline,
    pub cutout_pipeline_cw: wgpu::RenderPipeline,
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub wireframe_pipeline: wgpu::RenderPipeline,
}

/// Bind group layouts and render target state required to compile [`PbrPipelines`].
pub(crate) struct PbrPipelineParams<'a> {
    pub camera_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub light_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub shadow_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub scene_format: wgpu::TextureFormat,
    pub msaa_samples: u32,
}

struct PbrVariantSpec<'a> {
    label: &'a str,
    fragment_entry: &'a str,
    blend: wgpu::BlendState,
    front_face: wgpu::FrontFace,
    cull_mode: Option<wgpu::Face>,
    polygon_mode: wgpu::PolygonMode,
    depth_write_enabled: bool,
    depth_bias: wgpu::DepthBiasState,
}

fn build_pbr_variant(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    scene_format: wgpu::TextureFormat,
    msaa_samples: u32,
    spec: PbrVariantSpec<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(spec.label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(vertex_buffer_layout()), Some(Instance::desc())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(spec.fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format: scene_format,
                blend: Some(spec.blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: spec.front_face,
            cull_mode: spec.cull_mode,
            polygon_mode: spec.polygon_mode,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: PBR_DEPTH_FORMAT,
            depth_write_enabled: Some(spec.depth_write_enabled),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: spec.depth_bias,
        }),
        multisample: wgpu::MultisampleState {
            count: msaa_samples,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}

/// Compiles all 6 PBR rendering pipeline variants from `shaders/shader.wgsl`.
pub(crate) fn create_pbr_pipelines(
    device: &wgpu::Device,
    params: &PbrPipelineParams<'_>,
) -> PbrPipelines {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("PBR Pipeline Layout"),
        bind_group_layouts: &[
            Some(params.camera_bind_group_layout),
            Some(params.light_bind_group_layout),
            Some(params.shadow_bind_group_layout),
            Some(params.texture_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let build = |spec: PbrVariantSpec<'_>| {
        build_pbr_variant(
            device,
            &layout,
            &shader,
            params.scene_format,
            params.msaa_samples,
            spec,
        )
    };

    PbrPipelines {
        render_pipeline: build(PbrVariantSpec {
            label: "PBR Pipeline (CCW)",
            fragment_entry: "fs_main",
            blend: wgpu::BlendState::REPLACE,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState::default(),
        }),
        render_pipeline_cw: build(PbrVariantSpec {
            label: "PBR Pipeline (CW Mirrored)",
            fragment_entry: "fs_main",
            blend: wgpu::BlendState::REPLACE,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState::default(),
        }),
        cutout_pipeline: build(PbrVariantSpec {
            label: "PBR Cutout Pipeline (CCW)",
            fragment_entry: "fs_cutout",
            blend: wgpu::BlendState::REPLACE,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState::default(),
        }),
        cutout_pipeline_cw: build(PbrVariantSpec {
            label: "PBR Cutout Pipeline (CW Mirrored)",
            fragment_entry: "fs_cutout",
            blend: wgpu::BlendState::REPLACE,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState::default(),
        }),
        transparent_pipeline: build(PbrVariantSpec {
            label: "PBR Transparent Pipeline",
            fragment_entry: "fs_transparent",
            blend: wgpu::BlendState::ALPHA_BLENDING,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            depth_write_enabled: false,
            depth_bias: wgpu::DepthBiasState::default(),
        }),
        wireframe_pipeline: build(PbrVariantSpec {
            label: "Wireframe Pipeline",
            fragment_entry: "fs_main",
            blend: wgpu::BlendState::ALPHA_BLENDING,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Line,
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState {
                constant: -2,
                slope_scale: -2.0,
                clamp: 0.0,
            },
        }),
    }
}