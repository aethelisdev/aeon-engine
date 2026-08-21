// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Creates the Shadow rendering pipeline modernized for Wgpu 23+ (v2026 stable).
/// Depth-only pass with bias for shadow acne prevention.
pub(crate) fn create_shadow_pipeline(
    device: &wgpu::Device,
    light_space_bgl_vs: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/shadow.wgsl"));

    let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow Pipeline Layout"),
        bind_group_layouts: &[Some(light_space_bgl_vs)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Pipeline"),
        layout: Some(&shadow_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_shadow"),
            buffers: &[
                Some(crate::render::types::Vertex::desc()),
                Some(crate::render::types::Instance::desc()),
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: None, // Shadow pass is depth-only
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // Shadow bias to avoid acne
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Creates the Alpha-Tested Cutout Shadow rendering pipeline.
/// Executes `fs_shadow_cutout` to discard transparent texels during shadow pass,
/// enabling pixel-perfect shadows for foliage, hair, clothing, and decals.
pub(crate) fn create_shadow_cutout_pipeline(
    device: &wgpu::Device,
    light_space_bgl_vs: &wgpu::BindGroupLayout,
    texture_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/shadow.wgsl"));

    let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow Cutout Pipeline Layout"),
        bind_group_layouts: &[Some(light_space_bgl_vs), Some(texture_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Cutout Pipeline"),
        layout: Some(&shadow_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_shadow_cutout"),
            buffers: &[
                Some(crate::render::types::Vertex::desc()),
                Some(crate::render::types::Instance::desc()),
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_shadow_cutout"),
            targets: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // Double-sided for cutout cards/foliage/decals
            unclipped_depth: false,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // Shadow bias to avoid acne
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}