// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU textured quad pipeline for rendering 2D texture array views and editor icons in Iris UI.

use bytemuck::{Pod, Zeroable};
use iris_core::{Color, Rect};
use wgpu::util::DeviceExt;

/// Uniform buffer structure sent to GPU representing screen dimensions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GlobalUniforms {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

/// Unit vertex for unit quad mesh.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct UnitVertex {
    position: [f32; 2],
}

/// Instance data for textured quad drawing across 2D texture array layers.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TextureQuadInstance {
    /// Bounding rectangle in pixels `[x, y, width, height]`.
    pub rect: [f32; 4],
    /// UV coordinate rectangle and layer index `[min_u, min_v, max_u, layer_index]`.
    pub uv_rect: [f32; 4],
    /// Color multiplier `[r, g, b, a]`.
    pub tint: [f32; 4],
    /// Scissor clipping rectangle `[min_x, min_y, max_x, max_y]`.
    pub clip_rect: [f32; 4],
}

/// Dedicated WGPU pipeline for rendering 2D textured rectangles and engine viewports.
pub struct TextureQuadPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_globals: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    bind_group_layout_tex: wgpu::BindGroupLayout,
}

impl TextureQuadPipeline {
    /// Unit quad vertices [0.0 .. 1.0].
    const UNIT_VERTICES: &'static [UnitVertex] = &[
        UnitVertex {
            position: [0.0, 0.0],
        },
        UnitVertex {
            position: [1.0, 0.0],
        },
        UnitVertex {
            position: [1.0, 1.0],
        },
        UnitVertex {
            position: [0.0, 1.0],
        },
    ];

    /// Unit quad index list (2 triangles = 6 indices).
    const UNIT_INDICES: &'static [u16] = &[0, 1, 2, 0, 2, 3];

    /// Creates a new `TextureQuadPipeline` for the specified target surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iris UI Texture Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/texture_quad.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iris UI Texture Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout_globals =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Iris UI Texture Globals Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group_globals = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iris UI Texture Globals Bind Group"),
            layout: &bind_group_layout_globals,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let bind_group_layout_tex =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Iris UI Texture View Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
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
                ],
            });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Iris UI Texture Bilinear Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Iris UI Texture Pipeline Layout"),
            bind_group_layouts: &[
                Some(&bind_group_layout_globals),
                Some(&bind_group_layout_tex),
            ],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnitVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureQuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Iris UI Texture Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout), Some(instance_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI Texture Vertex Buffer"),
            contents: bytemuck::cast_slice(Self::UNIT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI Texture Index Buffer"),
            contents: bytemuck::cast_slice(Self::UNIT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            bind_group_globals,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            sampler,
            bind_group_layout_tex,
        }
    }

    /// Creates a new texture bind group binding the specified texture view.
    pub fn create_texture_bind_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iris UI Texture Bind Group"),
            layout: &self.bind_group_layout_tex,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Updates screen dimensions uniform buffer.
    pub fn update_globals(&self, queue: &wgpu::Queue, screen_size: [f32; 2]) {
        let uniforms = GlobalUniforms {
            screen_size,
            _padding: [0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Prepares a single-instance buffer for the specified texture quad parameters.
    pub fn create_instance_buffer(
        &self,
        device: &wgpu::Device,
        rect: Rect,
        uv_rect: [f32; 4],
        tint: Color,
        clip_rect: Option<Rect>,
    ) -> wgpu::Buffer {
        let clip_arr = match clip_rect {
            Some(c) => [c.x, c.y, c.x + c.width, c.y + c.height],
            None => [0.0, 0.0, 0.0, 0.0],
        };

        let instance = TextureQuadInstance {
            rect: [rect.x, rect.y, rect.width, rect.height],
            uv_rect,
            tint: [tint.r, tint.g, tint.b, tint.a],
            clip_rect: clip_arr,
        };

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI Texture Instance Buffer"),
            contents: bytemuck::bytes_of(&instance),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    /// Records texture quad drawing commands into the active render pass.
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        tex_bind_group: &'a wgpu::BindGroup,
        instance_buffer: &'a wgpu::Buffer,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group_globals, &[]);
        render_pass.set_bind_group(1, tex_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    /// Returns a reference to the underlying textured quad render pipeline.
    #[inline]
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// Returns a reference to the globals bind group containing screen dimensions.
    #[inline]
    pub fn bind_group_globals(&self) -> &wgpu::BindGroup {
        &self.bind_group_globals
    }

    /// Returns a reference to the static unit quad vertex buffer.
    #[inline]
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// Returns a reference to the static unit quad index buffer.
    #[inline]
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
}