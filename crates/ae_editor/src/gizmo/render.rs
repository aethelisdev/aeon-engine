// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use cgmath::{InnerSpace, Matrix4, Quaternion, Rotation as _, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

use super::core::{ActiveAxis, GizmoMode, GizmoScreenParams, GizmoSystem};

/// GPU vertex for gizmo line/mesh rendering (position + color).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GizmoVertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: [f32; 3],
}

impl GizmoVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3  // color
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GizmoVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl GizmoSystem {
    /// Creates a new GizmoSystem with its own WGPU pipeline, vertex buffers,
    /// and uniform buffer. Generates all geometry upfront for Translate, Rotate, and Scale modes.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let vertices = Self::build_axis_vertices(1.0, false);
        let rotate_vertices = Self::build_rotation_vertices(1.0);
        let scale_vertices = Self::build_axis_vertices(1.0, true);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let rotate_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Rotate Vertex Buffer"),
            contents: bytemuck::cast_slice(&rotate_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let scale_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Scale Vertex Buffer"),
            contents: bytemuck::cast_slice(&scale_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let interaction_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Interaction Buffer"),
            size: (std::mem::size_of::<GizmoVertex>() * 4096) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ring_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo O-Ring Vertex Buffer"),
            size: (std::mem::size_of::<GizmoVertex>() * 2048) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Uniform Buffer"),
            size: 64, // Mat4x4
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gizmo Bind Group Layout"),
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
            label: Some("Gizmo Bind Group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/gizmo.wgsl").into()),
        });

        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gizmo Pipeline Layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let create_pipeline = |topology: wgpu::PrimitiveTopology| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if topology == wgpu::PrimitiveTopology::LineList {
                    "Gizmo Line Pipeline"
                } else {
                    "Gizmo Mesh Pipeline"
                }),
                layout: Some(&pll),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[GizmoVertex::layout()],
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
                    topology,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: wgpu::StencilState::default(),
                    bias: if topology == wgpu::PrimitiveTopology::TriangleList {
                        wgpu::DepthBiasState {
                            constant: -10000,
                            slope_scale: -1.0,
                            clamp: 0.0,
                        }
                    } else {
                        wgpu::DepthBiasState::default()
                    },
                }),
                multisample: wgpu::MultisampleState {
                    count: msaa_samples.max(1),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            })
        };

        let mesh_pipeline = create_pipeline(wgpu::PrimitiveTopology::TriangleList);
        let line_pipeline = create_pipeline(wgpu::PrimitiveTopology::LineList);

        Self {
            mode: GizmoMode::Translate,
            space: super::space::GizmoSpace::Local,
            active_axis: ActiveAxis::None,
            hovered_axis: ActiveAxis::None,
            is_dragging: false,
            drag_start_world: Vector3::new(0.0, 0.0, 0.0),
            drag_start_vector: Vector3::new(1.0, 0.0, 0.0),
            drag_current_vector: Vector3::new(1.0, 0.0, 0.0),
            drag_gizmo_pos: Vector3::new(0.0, 0.0, 0.0),
            drag_plane_normal: Vector3::new(0.0, 0.0, 1.0),
            drag_current_hit: Vector3::new(0.0, 0.0, 0.0),
            drag_scale: 1.0,
            drag_scale_factor: 1.0,
            cam_right: std::cell::Cell::new(Vector3::unit_x()),
            cam_up: std::cell::Cell::new(Vector3::unit_y()),
            entity_rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            vertex_buffer,
            rotate_vertex_buffer,
            scale_vertex_buffer,
            interaction_vertex_buffer,
            ring_vertex_buffer,
            uniform_buffer,
            bind_group,
            mesh_pipeline,
            line_pipeline,
            num_vertices: vertices.len() as u32,
            num_rotate_vertices: rotate_vertices.len() as u32,
            num_scale_vertices: scale_vertices.len() as u32,
        }
    }

    /// Legacy render API — opens its own render pass and calls `draw_in_render_pass`.
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        gizmo_pos: Vector3<f32>,
        camera_pos: Vector3<f32>,
        view_proj: Matrix4<f32>,
        screen: &GizmoScreenParams,
    ) {
        // Legacy render() API starts a render pass and calls the `draw_in_render_pass` function.
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Gizmo Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        let dist_cam = (camera_pos - gizmo_pos).magnitude();
        self.draw_in_render_pass(queue, &mut pass, gizmo_pos, dist_cam, view_proj, screen);
    }

    /// Draws the gizmo inside an existing render pass (no pass creation).
    /// Computes model matrix from gizmo position and screen-compensated scale,
    /// then issues draw calls for the active mode's geometry.
    pub fn draw_in_render_pass<'a>(
        &'a self,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'a>,
        gizmo_pos: Vector3<f32>,
        camera_distance: f32,
        view_proj: Matrix4<f32>,
        screen: &GizmoScreenParams,
    ) {
        // Axis length is dynamically calculated for constant screen size.
        let dist = camera_distance.max(1e-6); // Min clamp to avoid breaking perspective scale coefficient.
        let axis_len_world = screen.axis_length_world(dist); // Axis length in world space based on distance.
        let scale = axis_len_world; // Direct scale is sufficient because vertices are generated assuming len=1.

        // Model matrix: Scaled drawing at gizmo_pos.
        // Entity rotation is applied in Local mode, aligning the axes to the object.
        let rotation_matrix = if self.space == super::space::GizmoSpace::Local {
            Matrix4::from(self.entity_rotation)
        } else {
            Matrix4::identity()
        };
        let model =
            Matrix4::from_translation(gizmo_pos) * rotation_matrix * Matrix4::from_scale(scale);

        // MVP: moves model space to clip space using view_proj.
        let mvp: [[f32; 4]; 4] = (view_proj * model).into();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&mvp));

        // Gizmo pipeline: depth_compare=Always, so it doesn't get occluded by the scene.
        pass.set_bind_group(0, &self.bind_group, &[]);

        match self.mode {
            GizmoMode::Translate => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..self.num_vertices, 0..1);

                // Draw the O-ring (camera-facing 360-degree circle in the center) using mesh_pipeline!
                let ring = self.build_o_ring_mesh();
                if !ring.is_empty() {
                    queue.write_buffer(&self.ring_vertex_buffer, 0, bytemuck::cast_slice(&ring));
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
                    pass.draw(0..ring.len() as u32, 0..1);
                }

                let lines = self.build_interaction_lines();
                if !lines.is_empty() {
                    queue.write_buffer(
                        &self.interaction_vertex_buffer,
                        0,
                        bytemuck::cast_slice(&lines),
                    );
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(..));
                    pass.draw(0..lines.len() as u32, 0..1);
                }
            }
            GizmoMode::Rotate => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.rotate_vertex_buffer.slice(..));
                pass.draw(0..self.num_rotate_vertices, 0..1);

                // Draw the O-ring (camera-facing 360-degree circle in the center) using mesh_pipeline!
                let ring = self.build_o_ring_mesh();
                if !ring.is_empty() {
                    queue.write_buffer(&self.ring_vertex_buffer, 0, bytemuck::cast_slice(&ring));
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
                    pass.draw(0..ring.len() as u32, 0..1);
                }

                if self.is_dragging && self.active_axis != ActiveAxis::None {
                    let sv = self.drag_start_vector;
                    let cv = self.drag_current_vector;

                    let cross = sv.cross(cv);
                    let sign = cross.dot(self.drag_plane_normal).signum();
                    let raw_angle = sv.dot(cv).clamp(-1.0, 1.0).acos();
                    let angle = if !raw_angle.is_nan() {
                        if sign < 0.0 {
                            std::f32::consts::TAU - raw_angle
                        } else {
                            raw_angle
                        }
                    } else {
                        0.0
                    };

                    if angle > 0.005 {
                        let steps = (angle * 12.0).max(4.0) as u32;

                        let mut orth = self.drag_plane_normal.cross(sv).normalize();
                        if orth.magnitude2().is_nan() || orth.magnitude2() < 0.1 {
                            orth = cv;
                        }

                        let (sv_local, orth_local) =
                            if self.space == super::space::GizmoSpace::Local {
                                let inv_rot = self.entity_rotation.conjugate();
                                (inv_rot.rotate_vector(sv), inv_rot.rotate_vector(orth))
                            } else {
                                (sv, orth)
                            };

                        let color = match self.active_axis {
                            ActiveAxis::X => [1.0, 0.2, 0.2],
                            ActiveAxis::Y => [0.2, 1.0, 0.2],
                            ActiveAxis::Z => [0.2, 0.4, 1.0],
                            _ => [1.0, 1.0, 1.0],
                        };
                        let dark_col = [color[0] * 0.2, color[1] * 0.2, color[2] * 0.2];
                        let r = 0.95;

                        let mut pie_vertices = Vec::new();
                        for i in 0..steps {
                            let t1 = (i as f32) / (steps as f32) * angle;
                            let t2 = ((i + 1) as f32) / (steps as f32) * angle;

                            let p1 = sv_local * t1.cos() + orth_local * t1.sin();
                            let p2 = sv_local * t2.cos() + orth_local * t2.sin();

                            pie_vertices.extend_from_slice(&[
                                GizmoVertex {
                                    position: [0.0, 0.0, 0.0],
                                    color: dark_col,
                                },
                                GizmoVertex {
                                    position: (p1 * r).into(),
                                    color: dark_col,
                                },
                                GizmoVertex {
                                    position: (p2 * r).into(),
                                    color: dark_col,
                                },
                            ]);
                        }

                        if pie_vertices.len() <= 1000 {
                            queue.write_buffer(
                                &self.interaction_vertex_buffer,
                                0,
                                bytemuck::cast_slice(&pie_vertices),
                            );
                            pass.set_pipeline(&self.mesh_pipeline);
                            pass.set_vertex_buffer(
                                0,
                                self.interaction_vertex_buffer.slice(
                                    0..(std::mem::size_of::<GizmoVertex>() * pie_vertices.len())
                                        as wgpu::BufferAddress,
                                ),
                            );
                            pass.draw(0..pie_vertices.len() as u32, 0..1);
                        }
                    }
                }

                // Draw lines at all times!
                let lines = self.build_interaction_lines();
                if !lines.is_empty() {
                    let lines_offset =
                        (std::mem::size_of::<GizmoVertex>() * 2048) as wgpu::BufferAddress;
                    queue.write_buffer(
                        &self.interaction_vertex_buffer,
                        lines_offset,
                        bytemuck::cast_slice(&lines),
                    );
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(lines_offset..));
                    pass.draw(0..lines.len() as u32, 0..1);
                }
            }
            GizmoMode::Scale => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.scale_vertex_buffer.slice(..));
                pass.draw(0..self.num_scale_vertices, 0..1);

                // 1) Draw the O-ring using mesh_pipeline (TriangleList) over ring_vertex_buffer!
                let ring = self.build_o_ring_mesh();
                if !ring.is_empty() {
                    queue.write_buffer(&self.ring_vertex_buffer, 0, bytemuck::cast_slice(&ring));
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
                    pass.draw(0..ring.len() as u32, 0..1);
                }

                // 2) Draw guidelines and the red center point using line_pipeline!
                let lines = self.build_interaction_lines();
                if !lines.is_empty() {
                    queue.write_buffer(
                        &self.interaction_vertex_buffer,
                        0,
                        bytemuck::cast_slice(&lines),
                    );
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(..));
                    pass.draw(0..lines.len() as u32, 0..1);
                }
            }
        }
    }

    /// Prepares the gizmo GPU state for an upcoming `draw_overlay()` call.
    /// Writes the MVP uniform buffer based on gizmo position, camera distance,
    /// and screen params. Must be called BEFORE `draw_overlay()` in the same frame.
    pub fn prepare_overlay(
        &self,
        queue: &wgpu::Queue,
        gizmo_pos: Vector3<f32>,
        camera_distance: f32,
        view_proj: Matrix4<f32>,
        screen: &GizmoScreenParams,
        cam_right: Vector3<f32>,
        cam_up: Vector3<f32>,
    ) {
        self.cam_right.set(cam_right);
        self.cam_up.set(cam_up);

        let dist = camera_distance.max(1e-6);
        let axis_len_world = screen.axis_length_world(dist);
        let scale = axis_len_world;

        // Entity rotation is applied in Local mode, aligning the axes to the object.
        let rotation_matrix = if self.space == super::space::GizmoSpace::Local {
            Matrix4::from(self.entity_rotation)
        } else {
            Matrix4::identity()
        };
        let model =
            Matrix4::from_translation(gizmo_pos) * rotation_matrix * Matrix4::from_scale(scale);
        let mvp: [[f32; 4]; 4] = (view_proj * model).into();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&mvp));
    }
}

/// Implements the generic `OverlayRenderer` trait so that `RenderState` can draw the
/// gizmo without depending on `GizmoSystem` directly. The engine calls `prepare_overlay()`
/// before passing this as `&dyn OverlayRenderer` to the render method.
impl ae_renderer::render::OverlayRenderer for GizmoSystem {
    fn draw_overlay<'a>(&'a self, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_bind_group(0, &self.bind_group, &[]);

        match self.mode {
            GizmoMode::Translate => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..self.num_vertices, 0..1);

                // Draw the O-ring (camera-facing 360-degree circle in the center) using mesh_pipeline!
                let ring = self.build_o_ring_mesh();
                if !ring.is_empty() {
                    queue.write_buffer(&self.ring_vertex_buffer, 0, bytemuck::cast_slice(&ring));
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
                    pass.draw(0..ring.len() as u32, 0..1);
                }

                let lines = self.build_interaction_lines();
                if !lines.is_empty() {
                    queue.write_buffer(
                        &self.interaction_vertex_buffer,
                        0,
                        bytemuck::cast_slice(&lines),
                    );
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(..));
                    pass.draw(0..lines.len() as u32, 0..1);
                }
            }
            GizmoMode::Rotate => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.rotate_vertex_buffer.slice(..));
                pass.draw(0..self.num_rotate_vertices, 0..1);

                // Draw the O-ring (camera-facing 360-degree circle in the center) using mesh_pipeline!
                let ring = self.build_o_ring_mesh();
                if !ring.is_empty() {
                    queue.write_buffer(&self.ring_vertex_buffer, 0, bytemuck::cast_slice(&ring));
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
                    pass.draw(0..ring.len() as u32, 0..1);
                }

                if self.is_dragging && self.active_axis != ActiveAxis::None {
                    let sv = self.drag_start_vector;
                    let cv = self.drag_current_vector;

                    let cross = sv.cross(cv);
                    let sign = cross.dot(self.drag_plane_normal).signum();
                    let raw_angle = sv.dot(cv).clamp(-1.0, 1.0).acos();
                    let angle = if !raw_angle.is_nan() {
                        if sign < 0.0 {
                            std::f32::consts::TAU - raw_angle
                        } else {
                            raw_angle
                        }
                    } else {
                        0.0
                    };

                    if angle > 0.005 {
                        let steps = (angle * 12.0).max(4.0) as u32;

                        let mut orth = self.drag_plane_normal.cross(sv).normalize();
                        if orth.magnitude2().is_nan() || orth.magnitude2() < 0.1 {
                            orth = cv;
                        }

                        let (sv_local, orth_local) =
                            if self.space == super::space::GizmoSpace::Local {
                                let inv_rot = self.entity_rotation.conjugate();
                                (inv_rot.rotate_vector(sv), inv_rot.rotate_vector(orth))
                            } else {
                                (sv, orth)
                            };

                        let color = match self.active_axis {
                            ActiveAxis::X => [1.0, 0.2, 0.2],
                            ActiveAxis::Y => [0.2, 1.0, 0.2],
                            ActiveAxis::Z => [0.2, 0.4, 1.0],
                            _ => [1.0, 1.0, 1.0],
                        };
                        let dark_col = [color[0] * 0.2, color[1] * 0.2, color[2] * 0.2];
                        let r = 0.95;

                        let mut pie_vertices = Vec::new();
                        for i in 0..steps {
                            let t1 = (i as f32) / (steps as f32) * angle;
                            let t2 = ((i + 1) as f32) / (steps as f32) * angle;

                            let p1 = sv_local * t1.cos() + orth_local * t1.sin();
                            let p2 = sv_local * t2.cos() + orth_local * t2.sin();

                            pie_vertices.extend_from_slice(&[
                                GizmoVertex {
                                    position: [0.0, 0.0, 0.0],
                                    color: dark_col,
                                },
                                GizmoVertex {
                                    position: (p1 * r).into(),
                                    color: dark_col,
                                },
                                GizmoVertex {
                                    position: (p2 * r).into(),
                                    color: dark_col,
                                },
                            ]);
                        }

                        if pie_vertices.len() <= 1000 {
                            queue.write_buffer(
                                &self.interaction_vertex_buffer,
                                0,
                                bytemuck::cast_slice(&pie_vertices),
                            );
                            pass.set_pipeline(&self.mesh_pipeline);
                            pass.set_vertex_buffer(
                                0,
                                self.interaction_vertex_buffer.slice(
                                    0..(std::mem::size_of::<GizmoVertex>() * pie_vertices.len())
                                        as wgpu::BufferAddress,
                                ),
                            );
                            pass.draw(0..pie_vertices.len() as u32, 0..1);
                        }
                    }
                }

                // Always draw guidelines/interaction lines!
                let lines = self.build_interaction_lines();
                if !lines.is_empty() {
                    let lines_offset =
                        (std::mem::size_of::<GizmoVertex>() * 2048) as wgpu::BufferAddress;
                    queue.write_buffer(
                        &self.interaction_vertex_buffer,
                        lines_offset,
                        bytemuck::cast_slice(&lines),
                    );
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(lines_offset..));
                    pass.draw(0..lines.len() as u32, 0..1);
                }
            }
            GizmoMode::Scale => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.scale_vertex_buffer.slice(..));
                pass.draw(0..self.num_scale_vertices, 0..1);

                // 1) Draw the O-ring using mesh_pipeline (TriangleList) via ring_vertex_buffer!
                let ring = self.build_o_ring_mesh();
                if !ring.is_empty() {
                    queue.write_buffer(&self.ring_vertex_buffer, 0, bytemuck::cast_slice(&ring));
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
                    pass.draw(0..ring.len() as u32, 0..1);
                }

                // 2) Draw the guidelines and red center point using line_pipeline!
                let lines = self.build_interaction_lines();
                if !lines.is_empty() {
                    queue.write_buffer(
                        &self.interaction_vertex_buffer,
                        0,
                        bytemuck::cast_slice(&lines),
                    );
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(..));
                    pass.draw(0..lines.len() as u32, 0..1);
                }
            }
        }
    }
}