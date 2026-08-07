// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use wgpu::util::DeviceExt;

/// Outline uniforms passed into the composite WGSL shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OutlineUniforms {
    pub viewport_size: [f32; 2],
    pub _padding: [f32; 2],
    pub outline_color: [f32; 4],
}

/// Selection Outline rendering system managing mask generation and screen-space silhouette edge detection.
pub struct SelectionOutlinePass {
    pub mask_pipeline: wgpu::RenderPipeline,
    pub composite_pipeline: wgpu::RenderPipeline,
    pub outline_bind_group_layout: wgpu::BindGroupLayout,
    pub outline_uniform_buffer: wgpu::Buffer,
    pub mask_sampler: wgpu::Sampler,
    pub mask_texture: Option<wgpu::Texture>,
    pub mask_view: Option<wgpu::TextureView>,
    pub mask_depth_texture: Option<wgpu::Texture>,
    pub mask_depth_view: Option<wgpu::TextureView>,
    pub composite_bind_group: Option<wgpu::BindGroup>,
    pub width: u32,
    pub height: u32,
}

impl SelectionOutlinePass {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let outline_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Selection Outline Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let (mask_pipeline, composite_pipeline) = create_outline_pipelines(
            device,
            surface_format,
            camera_bgl,
            &outline_bind_group_layout,
        );

        let initial_uniforms = OutlineUniforms {
            viewport_size: [1280.0, 720.0],
            _padding: [0.0, 0.0],
            outline_color: [1.0, 0.55, 0.05, 0.95],
        };
        let outline_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Outline Uniform Buffer"),
            contents: bytemuck::cast_slice(&[initial_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mask_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Selection Mask Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            mask_pipeline,
            composite_pipeline,
            outline_bind_group_layout,
            outline_uniform_buffer,
            mask_sampler,
            mask_texture: None,
            mask_view: None,
            mask_depth_texture: None,
            mask_depth_view: None,
            composite_bind_group: None,
            width: 0,
            height: 0,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        if width == 0 || height == 0 || (self.width == width && self.height == height) {
            return;
        }
        self.width = width;
        self.height = height;

        let uniforms = OutlineUniforms {
            viewport_size: [width as f32, height as f32],
            _padding: [0.0, 0.0],
            outline_color: [1.0, 0.55, 0.05, 0.95],
        };
        queue.write_buffer(
            &self.outline_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        let mask_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Selection Mask Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mask_view = mask_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Selection Mask Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Outline Composite Bind Group"),
            layout: &self.outline_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.mask_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.outline_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        self.mask_texture = Some(mask_texture);
        self.mask_view = Some(mask_view);
        self.mask_depth_texture = Some(depth_texture);
        self.mask_depth_view = Some(depth_view);
        self.composite_bind_group = Some(composite_bind_group);
    }
}

/// Creates the Selection Outline pipelines for high-quality
/// screen-space silhouette edge detection.
pub(crate) fn create_outline_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    outline_bind_group_layout: &wgpu::BindGroupLayout,
) -> (wgpu::RenderPipeline, wgpu::RenderPipeline) {
    let mask_shader =
        device.create_shader_module(wgpu::include_wgsl!("../../shaders/selection_mask.wgsl"));
    let composite_shader =
        device.create_shader_module(wgpu::include_wgsl!("../../shaders/outline_composite.wgsl"));

    let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Selection Mask Pipeline Layout"),
        bind_group_layouts: &[Some(camera_bind_group_layout)],
        immediate_size: 0,
    });

    let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Selection Mask Pipeline"),
        layout: Some(&mask_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &mask_shader,
            entry_point: Some("vs_main"),
            buffers: &[
                Some(crate::render::Vertex::desc()),
                Some(crate::render::types::Instance::desc()),
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &mask_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let composite_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Outline Composite Pipeline Layout"),
            bind_group_layouts: &[Some(outline_bind_group_layout)],
            immediate_size: 0,
        });

    let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Outline Composite Pipeline"),
        layout: Some(&composite_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &composite_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &composite_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

    (mask_pipeline, composite_pipeline)
}