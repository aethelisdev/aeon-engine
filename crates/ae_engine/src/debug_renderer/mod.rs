// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// Debug wireframe renderer for physics collider visualization and 3D selection outline highlights.
/// Queries the ECS for entities with `Collider` + `Position` components,
/// generates wireframe line segments for each collider shape (Box, Sphere, Capsule),
/// and draws them as a green wireframe overlay using `OverlayRenderer`.
/// # Architecture
/// Self-contained module: owns its own WGPU pipeline, bind group, vertex buffer,
/// and uniform buffer. Does NOT depend on `PipelineManager` (same pattern as GizmoSystem).
pub mod shapes;
pub mod vertex;

use shapes::DebugShapes;
use vertex::DebugLineVertex;

/// Self-contained debug wireframe renderer.
/// Owns its own WGPU pipeline, vertex buffer (auto-growing), uniform buffer,
/// and bind group. Implements `OverlayRenderer` trait for compositing.
pub struct DebugRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_count: u32,
    /// Maximum vertex capacity (auto-grows when needed).
    buffer_capacity: usize,
}

impl DebugRenderer {
    /// Creates a new DebugRenderer with its own pipeline and GPU resources.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Uniform Buffer"),
            size: 64, // mat4x4
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug Bind Group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/debug_line.wgsl").into()),
        });

        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Pipeline Layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Line Pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[DebugLineVertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: msaa_samples.max(1),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let initial_capacity = 4096;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Vertex Buffer"),
            size: (std::mem::size_of::<DebugLineVertex>() * initial_capacity)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            vertex_count: 0,
            buffer_capacity: initial_capacity,
        }
    }

    /// Rebuilds the debug line render pipeline when MSAA sample count or surface format changes.
    pub fn rebuild_pipeline(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/debug_line.wgsl").into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug BGL Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Pipeline Layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        self.pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Line Pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[DebugLineVertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: msaa_samples.max(1),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
    }

    /// Queries physics colliders and active selection highlights to generate wireframe lines.
    /// Writes the VP matrix uniform and uploads vertex data to the GPU.
    pub fn collect_lines(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &hecs::World,
        asset_manager: &ae_renderer::asset::AssetManager,
        view_proj: cgmath::Matrix4<f32>,
        _selected_entities: &[hecs::Entity],
    ) {
        let vp: [[f32; 4]; 4] = view_proj.into();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&vp));

        let mut vertices: Vec<DebugLineVertex> = Vec::new();
        let color = [0.0, 1.0, 0.4]; // Green wireframe

        let mut query = world.query::<(
            &ae_core::ecs::Collider,
            &ae_core::ecs::Position,
            Option<&ae_core::ecs::Rotation>,
            Option<&ae_core::ecs::Scale>,
            Option<&ae_core::ecs::GlobalTransform>,
            Option<&ae_core::ecs::ModelId>,
        )>();

        for (col, pos, rot, scale, global_transform, model_id) in query.iter() {
            let model = if let Some(gt) = global_transform {
                gt.0
            } else {
                let p = cgmath::Vector3::new(pos.x, pos.y, pos.z);
                let q = rot
                    .map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
                    .unwrap_or_else(|| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
                let s = scale
                    .map(|s| cgmath::Vector3::new(s.x, s.y, s.z))
                    .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

                cgmath::Matrix4::from_translation(p)
                    * cgmath::Matrix4::from(q)
                    * cgmath::Matrix4::from_nonuniform_scale(s.x, s.y, s.z)
            };

            match col.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => {
                    DebugShapes::generate_box_lines(&mut vertices, &model, half_extents, color);
                }
                ae_core::ecs::ColliderShape::Sphere { radius } => {
                    DebugShapes::generate_sphere_lines(&mut vertices, &model, radius, color);
                }
                ae_core::ecs::ColliderShape::Capsule {
                    half_height,
                    radius,
                } => {
                    DebugShapes::generate_capsule_lines(
                        &mut vertices,
                        &model,
                        half_height,
                        radius,
                        color,
                    );
                }
                ae_core::ecs::ColliderShape::Trimesh => {
                    if let Some(m_id) = model_id {
                        if let Some((raw_vertices, raw_indices)) =
                            asset_manager.get_physics_mesh_data(m_id.0)
                        {
                            DebugShapes::generate_mesh_lines(
                                &mut vertices,
                                &model,
                                raw_vertices,
                                raw_indices,
                                color,
                            );
                        } else {
                            DebugShapes::generate_box_lines(
                                &mut vertices,
                                &model,
                                [0.5, 0.5, 0.5],
                                color,
                            );
                        }
                    } else {
                        DebugShapes::generate_box_lines(
                            &mut vertices,
                            &model,
                            [0.5, 0.5, 0.5],
                            color,
                        );
                    }
                }
                ae_core::ecs::ColliderShape::ConvexHull => {
                    if let Some(m_id) = model_id {
                        if let Some((raw_vertices, raw_indices)) =
                            asset_manager.get_physics_mesh_data(m_id.0)
                        {
                            DebugShapes::generate_mesh_lines(
                                &mut vertices,
                                &model,
                                raw_vertices,
                                raw_indices,
                                color,
                            );
                        } else {
                            DebugShapes::generate_box_lines(
                                &mut vertices,
                                &model,
                                [0.5, 0.5, 0.5],
                                color,
                            );
                        }
                    } else {
                        DebugShapes::generate_box_lines(
                            &mut vertices,
                            &model,
                            [0.5, 0.5, 0.5],
                            color,
                        );
                    }
                }
            }
        }

        self.vertex_count = vertices.len() as u32;

        if vertices.is_empty() {
            return;
        }

        // Grow buffer if needed
        if vertices.len() > self.buffer_capacity {
            self.buffer_capacity = vertices.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Debug Vertex Buffer"),
                size: (std::mem::size_of::<DebugLineVertex>() * self.buffer_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }
}

/// Implements `OverlayRenderer` so `RenderState` can draw debug wireframes
/// without knowing the concrete `DebugRenderer` type.
impl ae_renderer::render::OverlayRenderer for DebugRenderer {
    fn draw_overlay<'a>(&'a self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}