// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Main GPU renderer managing WGPU SDF pipeline, buffers, and instanced draw passes.

use crate::quad::QuadInstance;
use bytemuck::{Pod, Zeroable};
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

/// Main WGPU-based SDF Renderer for Iris UI.
pub struct IrisRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    instance_count: u32,
    screen_size: [f32; 2],
}

impl IrisRenderer {
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

    /// Creates a new `IrisRenderer` instance for the given WGPU device and target surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iris UI SDF Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sdf_quad.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iris UI Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Iris UI Bind Group Layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iris UI Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Iris UI Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnitVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Iris UI SDF Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout), Some(QuadInstance::desc())],
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
            label: Some("Iris UI Unit Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(Self::UNIT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI Unit Quad Index Buffer"),
            contents: bytemuck::cast_slice(Self::UNIT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            instance_buffer: None,
            instance_capacity: 0,
            instance_count: 0,
            screen_size: [1.0, 1.0],
        }
    }

    /// Prepares buffers and uploads instance data to the GPU before rendering.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        quads: &[QuadInstance],
    ) {
        self.screen_size = screen_size;
        self.instance_count = quads.len() as u32;

        if self.instance_count == 0 {
            return;
        }

        // Update uniforms
        let uniforms = GlobalUniforms {
            screen_size,
            _padding: [0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Ensure instance buffer has enough capacity
        let needed_capacity = quads.len();
        if self.instance_capacity < needed_capacity {
            let new_capacity = (needed_capacity * 2).max(128);
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Iris UI Instanced Quad Buffer"),
                size: (new_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_buffer = Some(buffer);
            self.instance_capacity = new_capacity;
        }

        // Upload instance data
        if let Some(ref buffer) = self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(quads));
        }
    }

    /// Prepares buffers and uploads instance data from a `DrawCommandList` to the GPU.
    pub fn prepare_command_list(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        command_list: &crate::command::DrawCommandList,
    ) {
        self.prepare(device, queue, screen_size, &command_list.quads);
    }

    /// Executes sequential draw commands and updates hardware scissor rects in exact Z-order.
    pub fn render_command_list<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        command_list: &'rp crate::command::DrawCommandList,
        screen_size: (u32, u32),
    ) {
        let Some(ref instance_buf) = self.instance_buffer else {
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buf.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for cmd in &command_list.commands {
            match *cmd {
                crate::command::DrawCommand::DrawSdfQuads { start, count } => {
                    if count > 0 {
                        render_pass.draw_indexed(0..6, 0, start..(start + count));
                    }
                }
                crate::command::DrawCommand::SetScissor {
                    x,
                    y,
                    width,
                    height,
                } => {
                    let clamped_w = width.min(screen_size.0.saturating_sub(x)).max(1);
                    let clamped_h = height.min(screen_size.1.saturating_sub(y)).max(1);
                    if x < screen_size.0 && y < screen_size.1 {
                        render_pass.set_scissor_rect(x, y, clamped_w, clamped_h);
                    }
                }
                crate::command::DrawCommand::ResetScissor => {
                    render_pass.set_scissor_rect(0, 0, screen_size.0.max(1), screen_size.1.max(1));
                }
                crate::command::DrawCommand::DrawTexture { .. } => {}
            }
        }
    }
}