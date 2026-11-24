// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Depth-only and alpha-tested cutout shadow map WGPU pipelines (`shaders/shadow.wgsl`).

use crate::vertex::{Instance, PBR_DEPTH_FORMAT, vertex_buffer_layout};

fn shadow_depth_stencil_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: PBR_DEPTH_FORMAT,
        depth_write_enabled: Some(true),
        depth_compare: Some(wgpu::CompareFunction::Less),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState {
            constant: 2,
            slope_scale: 2.0,
            clamp: 0.0,
        },
    }
}

/// Creates the opaque depth-only directional shadow pipeline.
pub(crate) fn create_shadow_pipeline(
    device: &wgpu::Device,
    light_space_bgl_vs: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shadow.wgsl"));
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow Pipeline Layout"),
        bind_group_layouts: &[Some(light_space_bgl_vs)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_shadow"),
            buffers: &[Some(vertex_buffer_layout()), Some(Instance::desc())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            ..Default::default()
        },
        depth_stencil: Some(shadow_depth_stencil_state()),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Creates the alpha-tested cutout shadow rendering pipeline (`fs_shadow_cutout`).
pub(crate) fn create_shadow_cutout_pipeline(
    device: &wgpu::Device,
    light_space_bgl_vs: &wgpu::BindGroupLayout,
    texture_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shadow.wgsl"));
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow Cutout Pipeline Layout"),
        bind_group_layouts: &[Some(light_space_bgl_vs), Some(texture_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Cutout Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_shadow_cutout"),
            buffers: &[Some(vertex_buffer_layout()), Some(Instance::desc())],
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
            cull_mode: None,
            unclipped_depth: false,
            ..Default::default()
        },
        depth_stencil: Some(shadow_depth_stencil_state()),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}