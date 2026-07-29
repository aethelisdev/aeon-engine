// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Creates the Bloom rendering pipelines modernized for Wgpu 23+ (v2026 stable).
/// Returns extract, blur, and composite pipelines for the 3-pass bloom effect.
pub(crate) fn create_bloom_pipelines(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    bloom_bind_group_layout: &wgpu::BindGroupLayout,
    bloom_params_bind_group_layout: &wgpu::BindGroupLayout,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
) {
    let bloom_shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/bloom.wgsl"));

    let bloom_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bloom Pipeline Layout"),
        bind_group_layouts: &[Some(bloom_bind_group_layout)],
        immediate_size: 0,
    });

    let bloom_composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bloom Composite Layout"),
        bind_group_layouts: &[
            Some(bloom_bind_group_layout),
            Some(bloom_bind_group_layout),
            Some(bloom_params_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let bloom_extract_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bloom Extract Pipeline"),
        layout: Some(&bloom_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &bloom_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &bloom_shader,
            entry_point: Some("fs_extract"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let bloom_blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bloom Blur Pipeline"),
        layout: Some(&bloom_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &bloom_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &bloom_shader,
            entry_point: Some("fs_blur"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let bloom_composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bloom Composite Pipeline"),
        layout: Some(&bloom_composite_layout),
        vertex: wgpu::VertexState {
            module: &bloom_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &bloom_shader,
            entry_point: Some("fs_composite"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    (
        bloom_extract_pipeline,
        bloom_blur_pipeline,
        bloom_composite_pipeline,
    )
}