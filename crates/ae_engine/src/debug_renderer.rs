// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Debug wireframe renderer for physics collider visualization.
/// Queries the ECS for entities with `Collider` + `Position` components,
/// generates wireframe line segments for each collider shape (Box, Sphere, Capsule),
/// and draws them as a green wireframe overlay using `OverlayRenderer`.
/// # Architecture
/// Self-contained module: owns its own WGPU pipeline, bind group, vertex buffer,
/// and uniform buffer. Does NOT depend on `PipelineManager` (same pattern as GizmoSystem).

/// Vertex for debug line rendering (position + color).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugLineVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl DebugLineVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugLineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

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
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/debug_line.wgsl").into()),
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

    /// Rebuilds the pipeline when MSAA sample count changes.
    pub fn rebuild_pipeline(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) {
        *self = Self::new(device, surface_format, msaa_samples);
    }

    /// Collects wireframe lines from all entities that have a `Collider` component.
    /// Writes the VP matrix uniform and uploads vertex data to the GPU.
    pub fn collect_lines(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &hecs::World,
        asset_manager: &ae_renderer::asset::AssetManager,
        view_proj: cgmath::Matrix4<f32>,
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
                    .unwrap_or(cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
                let s = scale
                    .map(|s| cgmath::Vector3::new(s.x, s.y, s.z))
                    .unwrap_or(cgmath::Vector3::new(1.0, 1.0, 1.0));

                cgmath::Matrix4::from_translation(p)
                    * cgmath::Matrix4::from(q)
                    * cgmath::Matrix4::from_nonuniform_scale(s.x, s.y, s.z)
            };

            match col.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => {
                    Self::generate_box_lines(&mut vertices, &model, half_extents, color);
                }
                ae_core::ecs::ColliderShape::Sphere { radius } => {
                    Self::generate_sphere_lines(&mut vertices, &model, radius, color);
                }
                ae_core::ecs::ColliderShape::Capsule {
                    half_height,
                    radius,
                } => {
                    Self::generate_capsule_lines(&mut vertices, &model, half_height, radius, color);
                }
                ae_core::ecs::ColliderShape::Trimesh => {
                    if let Some(m_id) = model_id {
                        if let Some((raw_vertices, raw_indices)) =
                            asset_manager.get_physics_mesh_data(m_id.0)
                        {
                            Self::generate_mesh_lines(
                                &mut vertices,
                                &model,
                                raw_vertices,
                                raw_indices,
                                color,
                            );
                        } else {
                            Self::generate_box_lines(&mut vertices, &model, [0.5, 0.5, 0.5], color);
                        }
                    } else {
                        Self::generate_box_lines(&mut vertices, &model, [0.5, 0.5, 0.5], color);
                    }
                }
                ae_core::ecs::ColliderShape::ConvexHull => {
                    if let Some(m_id) = model_id {
                        if let Some((raw_vertices, raw_indices)) =
                            asset_manager.get_physics_mesh_data(m_id.0)
                        {
                            Self::generate_mesh_lines(
                                &mut vertices,
                                &model,
                                raw_vertices,
                                raw_indices,
                                color,
                            );
                        } else {
                            Self::generate_box_lines(&mut vertices, &model, [0.5, 0.5, 0.5], color);
                        }
                    } else {
                        Self::generate_box_lines(&mut vertices, &model, [0.5, 0.5, 0.5], color);
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

    /// Generates 12-edge wireframe for a box collider.
    fn generate_box_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        he: [f32; 3],
        color: [f32; 3],
    ) {
        let corners = [
            [-he[0], -he[1], -he[2]],
            [he[0], -he[1], -he[2]],
            [he[0], he[1], -he[2]],
            [-he[0], he[1], -he[2]],
            [-he[0], -he[1], he[2]],
            [he[0], -he[1], he[2]],
            [he[0], he[1], he[2]],
            [-he[0], he[1], he[2]],
        ];

        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0), // front
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4), // back
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // connections
        ];

        for (a, b) in edges {
            let pa = Self::transform_point(model, corners[a]);
            let pb = Self::transform_point(model, corners[b]);
            verts.push(DebugLineVertex {
                position: pa,
                color,
            });
            verts.push(DebugLineVertex {
                position: pb,
                color,
            });
        }
    }

    /// Generates 3-ring wireframe for a sphere collider (XY, XZ, YZ planes).
    fn generate_sphere_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        radius: f32,
        color: [f32; 3],
    ) {
        let segments = 32;
        // XY ring
        Self::generate_circle(verts, model, radius, 0, 1, 2, segments, color);
        // XZ ring
        Self::generate_circle(verts, model, radius, 0, 2, 1, segments, color);
        // YZ ring
        Self::generate_circle(verts, model, radius, 1, 2, 0, segments, color);
    }

    /// Generates capsule wireframe: 2 hemisphere rings + 4 vertical lines.
    fn generate_capsule_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        half_height: f32,
        radius: f32,
        color: [f32; 3],
    ) {
        let segments = 24;

        // Top and bottom center circles (XZ plane)
        for offset_y in [half_height, -half_height] {
            for i in 0..segments {
                let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
                let p1 = [radius * a1.cos(), offset_y, radius * a1.sin()];
                let p2 = [radius * a2.cos(), offset_y, radius * a2.sin()];
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });
            }
        }

        // 4 vertical connecting lines
        for angle in [
            0.0_f32,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
            std::f32::consts::FRAC_PI_2 * 3.0,
        ] {
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            let top = Self::transform_point(model, [x, half_height, z]);
            let bot = Self::transform_point(model, [x, -half_height, z]);
            verts.push(DebugLineVertex {
                position: top,
                color,
            });
            verts.push(DebugLineVertex {
                position: bot,
                color,
            });
        }

        // Top hemisphere arc (XY plane)
        Self::generate_half_circle(verts, model, radius, half_height, true, 0, segments, color);
        // Top hemisphere arc (ZY plane)
        Self::generate_half_circle(verts, model, radius, half_height, true, 2, segments, color);
        // Bottom hemisphere arc (XY plane)
        Self::generate_half_circle(
            verts,
            model,
            radius,
            -half_height,
            false,
            0,
            segments,
            color,
        );
        // Bottom hemisphere arc (ZY plane)
        Self::generate_half_circle(
            verts,
            model,
            radius,
            -half_height,
            false,
            2,
            segments,
            color,
        );
    }

    /// Helper: generates a circle ring on a specified plane.
    fn generate_circle(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        radius: f32,
        axis_a: usize, // first in-plane axis
        axis_b: usize, // second in-plane axis
        axis_n: usize, // normal axis (set to 0)
        segments: usize,
        color: [f32; 3],
    ) {
        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
            let mut p1 = [0.0_f32; 3];
            let mut p2 = [0.0_f32; 3];
            p1[axis_a] = radius * a1.cos();
            p1[axis_b] = radius * a1.sin();
            p1[axis_n] = 0.0;
            p2[axis_a] = radius * a2.cos();
            p2[axis_b] = radius * a2.sin();
            p2[axis_n] = 0.0;
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p1),
                color,
            });
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p2),
                color,
            });
        }
    }

    /// Helper: generates a half-circle arc for capsule hemispheres.
    fn generate_half_circle(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        radius: f32,
        y_offset: f32,
        top: bool,
        horizontal_axis: usize, // 0=X, 2=Z
        segments: usize,
        color: [f32; 3],
    ) {
        let half_segments = segments / 2;
        let start_angle = if top { 0.0 } else { std::f32::consts::PI };
        let end_angle = if top {
            std::f32::consts::PI
        } else {
            std::f32::consts::TAU
        };

        for i in 0..half_segments {
            let t1 = start_angle + (i as f32) / (half_segments as f32) * (end_angle - start_angle);
            let t2 =
                start_angle + ((i + 1) as f32) / (half_segments as f32) * (end_angle - start_angle);

            let mut p1 = [0.0_f32; 3];
            let mut p2 = [0.0_f32; 3];
            p1[horizontal_axis] = radius * t1.cos();
            p1[1] = y_offset + radius * t1.sin();
            p2[horizontal_axis] = radius * t2.cos();
            p2[1] = y_offset + radius * t2.sin();

            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p1),
                color,
            });
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p2),
                color,
            });
        }
    }

    /// Transforms a local-space point by the model matrix and returns world-space `[f32; 3]`.
    fn transform_point(model: &cgmath::Matrix4<f32>, local: [f32; 3]) -> [f32; 3] {
        let v = model * cgmath::Vector4::new(local[0], local[1], local[2], 1.0);
        [v.x / v.w, v.y / v.w, v.z / v.w]
    }

    /// Generates wireframe lines representing a complex mesh collider.
    /// Iterates over the mesh indices to draw the edges of each triangle.
    fn generate_mesh_lines(
        vertices: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        raw_vertices: &[[f32; 3]],
        raw_indices: &[u32],
        color: [f32; 3],
    ) {
        for chunk in raw_indices.chunks_exact(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 < raw_vertices.len() && i1 < raw_vertices.len() && i2 < raw_vertices.len() {
                let p0 = raw_vertices[i0];
                let p1 = raw_vertices[i1];
                let p2 = raw_vertices[i2];

                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p0),
                    color,
                });
                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });

                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });
                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });

                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });
                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p0),
                    color,
                });
            }
        }
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