// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport 3D Billboard System
//!
//! Subsystem for collecting entity billboard badges, updating GPU buffers,
//! and rendering via the `OverlayRenderer` trait inside the 3D forward pass.

use super::pipeline::BillboardPipeline;
use super::types::{BillboardIconType, BillboardUniform, BillboardVertex};
use cgmath::Matrix4;
use hecs::Entity;

/// Parameters for preparing billboard overlays before render pass execution.
pub struct BillboardPrepareParams<'a> {
    /// WGPU command queue for buffer writes.
    pub queue: &'a wgpu::Queue,
    /// WGPU device for buffer allocation/resizing.
    pub device: &'a wgpu::Device,
    /// ECS world containing scene entities and components.
    pub world: &'a hecs::World,
    /// Combined camera view-projection matrix.
    pub view_proj: Matrix4<f32>,
    /// Viewport logical screen dimensions in pixels `[width, height]`.
    pub viewport_size: [f32; 2],
    /// Currently selected entities in the editor.
    pub selected_entities: &'a [Entity],
    /// Entity currently under cursor hover, if any.
    pub hovered_entity: Option<Entity>,
}

/// Standalone manager and renderer for 3D camera-facing viewport billboards.
pub struct BillboardSystem {
    /// Render pipeline and texture array bindings.
    pub pipeline: BillboardPipeline,
    /// Dynamic vertex buffer for instanced billboard quads.
    vertex_buffer: wgpu::Buffer,
    /// Camera view-projection uniform buffer.
    uniform_buffer: wgpu::Buffer,
    /// Camera uniform bind group.
    camera_bind_group: wgpu::BindGroup,
    /// Active vertex count for the current frame.
    vertex_count: u32,
    /// Maximum vertex capacity before buffer reallocation.
    buffer_capacity: usize,
}

impl BillboardSystem {
    /// Default vertex allocation capacity (128 quads * 6 vertices = 768 vertices).
    const INITIAL_CAPACITY: usize = 768;

    /// Creates a new `BillboardSystem` with GPU buffers and pipeline.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let pipeline = BillboardPipeline::new(device, queue, surface_format, msaa_samples);

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Billboard Camera Uniform Buffer"),
            size: std::mem::size_of::<BillboardUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Billboard Camera Bind Group"),
            layout: &pipeline.camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let buffer_capacity = Self::INITIAL_CAPACITY;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Billboard Dynamic Vertex Buffer"),
            size: (buffer_capacity * std::mem::size_of::<BillboardVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            camera_bind_group,
            vertex_count: 0,
            buffer_capacity,
        }
    }

    /// Rebuilds pipeline when MSAA or surface format changes.
    pub fn rebuild_pipeline(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) {
        self.pipeline
            .rebuild_pipeline(device, surface_format, msaa_samples);
    }

    /// Prepares billboard vertices from the ECS world and uploads to GPU memory.
    pub fn prepare_overlay(&mut self, params: BillboardPrepareParams<'_>) {
        let uniform = BillboardUniform {
            view_proj: params.view_proj.into(),
            viewport_size: [
                params.viewport_size[0].max(1.0),
                params.viewport_size[1].max(1.0),
            ],
            icon_size_px: 28.0,
            _pad: 0.0,
        };
        params
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let mut vertices: Vec<BillboardVertex> = Vec::new();

        for (ent, pos) in params
            .world
            .query::<(Entity, &ae_core::ecs::Position)>()
            .iter()
        {
            let is_selected = params.selected_entities.contains(&ent);

            // Skip hidden entities unless actively selected
            if params.world.get::<&ae_core::ecs::Hidden>(ent).is_ok() && !is_selected {
                continue;
            }

            let is_light = params.world.get::<&ae_core::ecs::Light>(ent).is_ok();
            let is_audio_source = params.world.get::<&ae_audio::AudioSource>(ent).is_ok();
            let is_audio_listener = params.world.get::<&ae_audio::AudioListener>(ent).is_ok();

            if !is_light && !is_audio_source && !is_audio_listener {
                continue;
            }

            let icon_type = if is_light {
                BillboardIconType::Light
            } else if is_audio_source {
                BillboardIconType::AudioSource
            } else {
                BillboardIconType::Camera
            };

            let is_hover = params.hovered_entity == Some(ent);

            let (bg_color, border_color, icon_tint) = if is_selected {
                (
                    [0.70, 0.45, 0.08, 0.90],
                    [1.0, 0.85, 0.20, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                )
            } else if is_hover {
                (
                    [0.25, 0.28, 0.38, 0.90],
                    [0.0, 0.85, 1.0, 0.90],
                    [0.0, 0.95, 1.0, 1.0],
                )
            } else {
                (
                    [0.08, 0.09, 0.13, 0.85],
                    [0.30, 0.35, 0.48, 0.70],
                    icon_type.default_tint(),
                )
            };

            let world_pos = [pos.x, pos.y, pos.z];
            let layer = icon_type.layer_index();

            // Two triangles for camera-facing quad
            let quad_verts = [
                // Triangle 1
                BillboardVertex {
                    world_pos,
                    quad_offset: [-1.0, -1.0],
                    uv: [0.0, 0.0],
                    layer,
                    bg_color,
                    border_color,
                    icon_tint,
                },
                BillboardVertex {
                    world_pos,
                    quad_offset: [1.0, -1.0],
                    uv: [1.0, 0.0],
                    layer,
                    bg_color,
                    border_color,
                    icon_tint,
                },
                BillboardVertex {
                    world_pos,
                    quad_offset: [1.0, 1.0],
                    uv: [1.0, 1.0],
                    layer,
                    bg_color,
                    border_color,
                    icon_tint,
                },
                // Triangle 2
                BillboardVertex {
                    world_pos,
                    quad_offset: [-1.0, -1.0],
                    uv: [0.0, 0.0],
                    layer,
                    bg_color,
                    border_color,
                    icon_tint,
                },
                BillboardVertex {
                    world_pos,
                    quad_offset: [1.0, 1.0],
                    uv: [1.0, 1.0],
                    layer,
                    bg_color,
                    border_color,
                    icon_tint,
                },
                BillboardVertex {
                    world_pos,
                    quad_offset: [-1.0, 1.0],
                    uv: [0.0, 1.0],
                    layer,
                    bg_color,
                    border_color,
                    icon_tint,
                },
            ];

            vertices.extend_from_slice(&quad_verts);
        }

        self.vertex_count = vertices.len() as u32;
        if vertices.is_empty() {
            return;
        }

        if vertices.len() > self.buffer_capacity {
            self.buffer_capacity = vertices.len().next_power_of_two();
            self.vertex_buffer = params.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Billboard Dynamic Vertex Buffer (Resized)"),
                size: (self.buffer_capacity * std::mem::size_of::<BillboardVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        params
            .queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }
}

/// Implements `OverlayRenderer` to integrate  with forward render pass.
impl ae_renderer::render::OverlayRenderer for BillboardSystem {
    fn draw_overlay<'a>(&'a self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.pipeline.texture_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}