// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bloom extract, blur, and additive tonemapping composite WGPU pipeline creation.

fn build_fullscreen_bloom_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    fragment_entry: &str,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
    })
}

/// Creates extract, blur, and composite pipelines for the 3-pass bloom effect.
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
    let bloom_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/bloom.wgsl"));

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

    let bloom_extract_pipeline = build_fullscreen_bloom_pipeline(
        device,
        "Bloom Extract Pipeline",
        &bloom_pipeline_layout,
        &bloom_shader,
        "fs_extract",
        config.format,
    );
    let bloom_blur_pipeline = build_fullscreen_bloom_pipeline(
        device,
        "Bloom Blur Pipeline",
        &bloom_pipeline_layout,
        &bloom_shader,
        "fs_blur",
        config.format,
    );
    let bloom_composite_pipeline = build_fullscreen_bloom_pipeline(
        device,
        "Bloom Composite Pipeline",
        &bloom_composite_layout,
        &bloom_shader,
        "fs_composite",
        config.format,
    );

    (
        bloom_extract_pipeline,
        bloom_blur_pipeline,
        bloom_composite_pipeline,
    )
}