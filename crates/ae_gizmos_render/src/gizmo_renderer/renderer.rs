// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! WGPU transform gizmo pipeline, GPU buffers, and [`ae_renderer::render::OverlayRenderer`]
//! implementation decoupled from CPU gizmo state in [`ae_gizmos::TransformGizmo`].

use ae_gizmos::{
    ActiveAxis, GizmoMode, GizmoScreenParams, GizmoSpace, GizmoVertex, TransformGizmo,
};
use cgmath::{InnerSpace, Matrix4, Rotation as _, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

const ROTATE_VERTEX_CAPACITY: usize = 12288;
const INTERACTION_VERTEX_CAPACITY: usize = 4096;
const PIE_VERTEX_OFFSET_COUNT: usize = 2048;
const PIE_MAX_VERTICES: usize = 1000;
const RING_VERTEX_CAPACITY: usize = 2048;

const GIZMO_ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
    0 => Float32x3, // position
    1 => Float32x4, // color (RGBA)
    2 => Float32x2  // uv for SDF anti-aliased circles
];

/// Returns the WGPU vertex buffer layout descriptor matching [`ae_gizmos::GizmoVertex`].
#[must_use]
pub fn gizmo_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GizmoVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &GIZMO_ATTRIBS,
    }
}

/// Parameters for preparing gizmo overlay uniforms and GPU vertex buffers.
pub struct GizmoOverlayPrepareParams<'a> {
    /// WGPU command queue used for uniform and dynamic vertex buffer uploads.
    pub queue: &'a wgpu::Queue,
    /// World-space gizmo origin pivot.
    pub gizmo_pos: Vector3<f32>,
    /// Positive camera-to-gizmo forward distance for screen-constant handle scaling.
    pub camera_distance: f32,
    /// Active camera view-projection matrix.
    pub view_proj: Matrix4<f32>,
    /// Viewport projection/pixel-size configuration.
    pub screen: &'a GizmoScreenParams,
    /// Orthonormal camera right axis.
    pub cam_right: Vector3<f32>,
    /// Orthonormal camera up axis.
    pub cam_up: Vector3<f32>,
    /// Normalized camera forward direction.
    pub cam_forward: Vector3<f32>,
    /// World-space camera eye position.
    pub cam_pos: Vector3<f32>,
}

/// GPU overlay renderer for [`TransformGizmo`].
///
/// Owns all WGPU pipelines, vertex buffers, and MVP uniform bindings while reading
/// interaction state from [`TransformGizmo`]. Uploads all per-frame dynamic geometry in
/// [`GizmoRenderer::prepare_overlay`] so [`ae_renderer::render::OverlayRenderer::draw_overlay`]
/// issues read-only render pass commands without CPU heap allocations.
pub struct GizmoRenderer {
    vertex_buffer: wgpu::Buffer,
    rotate_vertex_buffer: wgpu::Buffer,
    scale_vertex_buffer: wgpu::Buffer,
    interaction_vertex_buffer: wgpu::Buffer,
    ring_vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    mesh_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    num_vertices: u32,
    num_scale_vertices: u32,
    last_translate_state: (ActiveAxis, ActiveAxis),
    last_scale_state: (ActiveAxis, ActiveAxis),
    prepared_mode: GizmoMode,
    ring_draw_len: u32,
    rotate_draw_len: u32,
    pie_draw_len: u32,
    interaction_lines_len: u32,
}

impl GizmoRenderer {
    /// Creates a new `GizmoRenderer` with its own WGPU pipelines, uniform buffer,
    /// and pre-populated Translate/Scale axis vertex buffers.
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let vertices = TransformGizmo::build_axis_vertices(1.0, false);
        let scale_vertices = TransformGizmo::build_axis_vertices(1.0, true);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Translate Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let rotate_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Rotate Dynamic Vertex Buffer"),
            size: (std::mem::size_of::<GizmoVertex>() * ROTATE_VERTEX_CAPACITY)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scale_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Scale Vertex Buffer"),
            contents: bytemuck::cast_slice(&scale_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let interaction_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Interaction Buffer"),
            size: (std::mem::size_of::<GizmoVertex>() * INTERACTION_VERTEX_CAPACITY)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ring_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Dynamic Ring Vertex Buffer"),
            size: (std::mem::size_of::<GizmoVertex>() * RING_VERTEX_CAPACITY)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Uniform Buffer"),
            size: 64, // 4x4 matrix
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gizmo BGL"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gizmo Pipeline Layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/gizmo.wgsl").into()),
        });

        let create_pipeline = |topology: wgpu::PrimitiveTopology| -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Gizmo Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(gizmo_vertex_layout())],
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
            num_scale_vertices: scale_vertices.len() as u32,
            last_translate_state: (ActiveAxis::None, ActiveAxis::None),
            last_scale_state: (ActiveAxis::None, ActiveAxis::None),
            prepared_mode: GizmoMode::Select,
            ring_draw_len: 0,
            rotate_draw_len: 0,
            pie_draw_len: 0,
            interaction_lines_len: 0,
        }
    }

    /// Prepares GPU uniforms and uploads all dynamic vertex buffers for an upcoming
    /// [`ae_renderer::render::OverlayRenderer::draw_overlay`] call.
    ///
    /// Must be called BEFORE `draw_overlay()` in the same frame. Synchronizes the camera
    /// basis onto `gizmo` so view-aligned ring meshes match the current camera projection.
    pub fn prepare_overlay(
        &mut self,
        gizmo: &mut TransformGizmo,
        params: GizmoOverlayPrepareParams<'_>,
    ) {
        gizmo.set_camera_basis(
            params.cam_right,
            params.cam_up,
            params.cam_forward,
            params.cam_pos,
        );

        self.prepared_mode = gizmo.mode;
        self.ring_draw_len = 0;
        self.rotate_draw_len = 0;
        self.pie_draw_len = 0;
        self.interaction_lines_len = 0;

        if gizmo.mode == GizmoMode::Select {
            return;
        }

        let dist = params.camera_distance.max(1e-6);
        let scale = params.screen.axis_length_world(dist);

        // Entity rotation is applied in Local mode, aligning the axes to the object.
        let rotation_matrix = if gizmo.space == GizmoSpace::Local {
            Matrix4::from(gizmo.entity_rotation)
        } else {
            Matrix4::identity()
        };
        let model = Matrix4::from_translation(params.gizmo_pos)
            * rotation_matrix
            * Matrix4::from_scale(scale);
        let mvp: [[f32; 4]; 4] = (params.view_proj * model).into();
        params
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&mvp));

        let current_state = (gizmo.hovered_axis, gizmo.active_axis);
        if gizmo.mode == GizmoMode::Translate && self.last_translate_state != current_state {
            let verts = gizmo.build_dynamic_axis_vertices(1.0, false);
            params
                .queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));
            self.last_translate_state = current_state;
        } else if gizmo.mode == GizmoMode::Scale && self.last_scale_state != current_state {
            let verts = gizmo.build_dynamic_axis_vertices(1.0, true);
            params
                .queue
                .write_buffer(&self.scale_vertex_buffer, 0, bytemuck::cast_slice(&verts));
            self.last_scale_state = current_state;
        }

        // Upload O-ring vertices for all active transform modes
        let ring = gizmo.build_o_ring_mesh();
        let ring_len = ring.len().min(RING_VERTEX_CAPACITY);
        if ring_len > 0 {
            params.queue.write_buffer(
                &self.ring_vertex_buffer,
                0,
                bytemuck::cast_slice(&ring[..ring_len]),
            );
            self.ring_draw_len = ring_len as u32;
        }

        // Mode-specific dynamic overlays
        if gizmo.mode == GizmoMode::Rotate {
            let rotate_vertices = gizmo.build_dynamic_rotation_vertices(1.0);
            let write_len = rotate_vertices.len().min(ROTATE_VERTEX_CAPACITY);
            if write_len > 0 {
                params.queue.write_buffer(
                    &self.rotate_vertex_buffer,
                    0,
                    bytemuck::cast_slice(&rotate_vertices[..write_len]),
                );
                self.rotate_draw_len = write_len as u32;
            }

            let pie_vertices = build_rotation_pie_vertices(gizmo);
            if !pie_vertices.is_empty() && pie_vertices.len() <= PIE_MAX_VERTICES {
                params.queue.write_buffer(
                    &self.interaction_vertex_buffer,
                    0,
                    bytemuck::cast_slice(&pie_vertices),
                );
                self.pie_draw_len = pie_vertices.len() as u32;
            }

            let lines = gizmo.build_interaction_lines();
            let line_len = lines
                .len()
                .min(INTERACTION_VERTEX_CAPACITY - PIE_VERTEX_OFFSET_COUNT);
            if line_len > 0 {
                let lines_offset = (std::mem::size_of::<GizmoVertex>() * PIE_VERTEX_OFFSET_COUNT)
                    as wgpu::BufferAddress;
                params.queue.write_buffer(
                    &self.interaction_vertex_buffer,
                    lines_offset,
                    bytemuck::cast_slice(&lines[..line_len]),
                );
                self.interaction_lines_len = line_len as u32;
            }
        } else {
            let lines = gizmo.build_interaction_lines();
            let line_len = lines.len().min(INTERACTION_VERTEX_CAPACITY);
            if line_len > 0 {
                params.queue.write_buffer(
                    &self.interaction_vertex_buffer,
                    0,
                    bytemuck::cast_slice(&lines[..line_len]),
                );
                self.interaction_lines_len = line_len as u32;
            }
        }
    }
}

fn build_rotation_pie_vertices(gizmo: &TransformGizmo) -> Vec<GizmoVertex> {
    if !gizmo.is_dragging || gizmo.active_axis == ActiveAxis::None {
        return Vec::new();
    }

    let sv = gizmo.drag_start_vector;
    let cv = gizmo.drag_current_vector;
    let cross = sv.cross(cv);
    let sign = cross.dot(gizmo.drag_plane_normal).signum();
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

    if angle <= 0.005 {
        return Vec::new();
    }

    let steps = (angle * 12.0).max(4.0) as u32;
    let mut orth = gizmo.drag_plane_normal.cross(sv).normalize();
    if orth.magnitude2().is_nan() || orth.magnitude2() < 0.1 {
        orth = cv;
    }

    let (sv_local, orth_local) = if gizmo.space == GizmoSpace::Local {
        let inv_rot = gizmo.entity_rotation.conjugate();
        (inv_rot.rotate_vector(sv), inv_rot.rotate_vector(orth))
    } else {
        (sv, orth)
    };

    let pie_col = match gizmo.active_axis {
        ActiveAxis::X => [0.96, 0.12, 0.12, 0.45],
        ActiveAxis::Y => [0.22, 0.82, 0.22, 0.45],
        ActiveAxis::Z => [0.15, 0.45, 1.00, 0.45],
        ActiveAxis::Screen => [1.00, 0.90, 0.20, 0.45],
        _ => [1.00, 1.00, 1.00, 0.45],
    };
    let r = 0.95;

    let mut pie_vertices = Vec::with_capacity((steps * 3) as usize);
    for i in 0..steps {
        let t1 = (i as f32) / (steps as f32) * angle;
        let t2 = ((i + 1) as f32) / (steps as f32) * angle;

        let p1 = sv_local * t1.cos() + orth_local * t1.sin();
        let p2 = sv_local * t2.cos() + orth_local * t2.sin();

        pie_vertices.extend_from_slice(&[
            GizmoVertex {
                position: [0.0, 0.0, 0.0],
                color: pie_col,
                uv: [0.0, 0.0],
            },
            GizmoVertex {
                position: (p1 * r).into(),
                color: pie_col,
                uv: [0.0, 0.0],
            },
            GizmoVertex {
                position: (p2 * r).into(),
                color: pie_col,
                uv: [0.0, 0.0],
            },
        ]);
    }

    pie_vertices
}

/// Implements `OverlayRenderer` so `RenderState` draws prepared gizmo buffers without allocations.
impl ae_renderer::render::OverlayRenderer for GizmoRenderer {
    fn draw_overlay<'a>(&'a self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        if self.prepared_mode == GizmoMode::Select {
            return;
        }

        pass.set_bind_group(0, &self.bind_group, &[]);

        match self.prepared_mode {
            GizmoMode::Select => {}
            GizmoMode::Translate => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..self.num_vertices, 0..1);

                if self.ring_draw_len > 0 {
                    let ring_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.ring_draw_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(0..ring_bytes));
                    pass.draw(0..self.ring_draw_len, 0..1);
                }

                if self.interaction_lines_len > 0 {
                    let line_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.interaction_lines_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(0..line_bytes));
                    pass.draw(0..self.interaction_lines_len, 0..1);
                }
            }
            GizmoMode::Rotate => {
                if self.rotate_draw_len > 0 {
                    let rot_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.rotate_draw_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.rotate_vertex_buffer.slice(0..rot_bytes));
                    pass.draw(0..self.rotate_draw_len, 0..1);
                }

                if self.ring_draw_len > 0 {
                    let ring_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.ring_draw_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(0..ring_bytes));
                    pass.draw(0..self.ring_draw_len, 0..1);
                }

                if self.pie_draw_len > 0 {
                    let pie_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.pie_draw_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(0..pie_bytes));
                    pass.draw(0..self.pie_draw_len, 0..1);
                }

                if self.interaction_lines_len > 0 {
                    let lines_offset = (std::mem::size_of::<GizmoVertex>()
                        * PIE_VERTEX_OFFSET_COUNT)
                        as wgpu::BufferAddress;
                    let line_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.interaction_lines_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(
                        0,
                        self.interaction_vertex_buffer
                            .slice(lines_offset..lines_offset + line_bytes),
                    );
                    pass.draw(0..self.interaction_lines_len, 0..1);
                }
            }
            GizmoMode::Scale => {
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, self.scale_vertex_buffer.slice(..));
                pass.draw(0..self.num_scale_vertices, 0..1);

                if self.ring_draw_len > 0 {
                    let ring_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.ring_draw_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(0..ring_bytes));
                    pass.draw(0..self.ring_draw_len, 0..1);
                }

                if self.interaction_lines_len > 0 {
                    let line_bytes = (std::mem::size_of::<GizmoVertex>()
                        * self.interaction_lines_len as usize)
                        as wgpu::BufferAddress;
                    pass.set_pipeline(&self.line_pipeline);
                    pass.set_vertex_buffer(0, self.interaction_vertex_buffer.slice(0..line_bytes));
                    pass.draw(0..self.interaction_lines_len, 0..1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gizmo_vertex_layout_stride_and_fields() {
        let layout = gizmo_vertex_layout();
        assert_eq!(
            layout.array_stride,
            std::mem::size_of::<GizmoVertex>() as u64
        );
        assert_eq!(layout.array_stride, (3 + 4 + 2) * 4);
        assert_eq!(layout.attributes.len(), 3);
    }

    #[test]
    fn test_rotation_pie_vertices_empty_when_not_dragging_or_tiny_angle() {
        let mut gizmo = TransformGizmo::new();
        gizmo.is_dragging = false;
        gizmo.active_axis = ActiveAxis::X;
        assert!(build_rotation_pie_vertices(&gizmo).is_empty());

        gizmo.is_dragging = true;
        gizmo.drag_start_vector = Vector3::new(1.0, 0.0, 0.0);
        gizmo.drag_current_vector = Vector3::new(1.0, 0.0, 0.0);
        assert!(build_rotation_pie_vertices(&gizmo).is_empty());
    }
}