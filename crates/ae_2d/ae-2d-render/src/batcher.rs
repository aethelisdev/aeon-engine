// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::mem;

use cgmath::Matrix4;
use wgpu::util::DeviceExt;

use crate::instance::{SpriteInstance, SpriteVertex};
use crate::pipeline::Sprite2DPipeline;
use ae_2d_core::SpriteSortKey;

/// Default quad vertices centered at origin with standard normalized UVs.
const QUAD_VERTICES: [SpriteVertex; 4] = [
    SpriteVertex {
        position: [-0.5, -0.5],
        uv: [0.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, 0.5],
        uv: [1.0, 0.0],
    },
    SpriteVertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
];

/// Triangle index indices forming two triangles for the quad.
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

/// Represents a single render batch sharing the same texture bind group.
pub struct SpriteBatch {
    /// Texture bind group index or key.
    pub texture_id: u32,

    /// Start instance index in the instance buffer.
    pub start_instance: u32,

    /// Number of instances in this batch.
    pub instance_count: u32,
}

/// High-performance instanced 2D Sprite batcher.
/// Collects sprite draw commands, sorts them using 64-bit sort keys, packs instance buffers,
/// and issues batched draw calls with zero steady-state heap allocations in per-frame hot loops.
pub struct SpriteBatcher {
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_buffer_capacity: usize,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    /// Reusable CPU scratchpad for sorting keys to avoid allocations in hot loop.
    scratch_keys: Vec<(SpriteSortKey, SpriteInstance)>,

    /// Reusable batch ranges.
    batches: Vec<SpriteBatch>,

    /// Default 1x1 white texture and bind group for untextured tinted sprites.
    pub white_texture_bind_group: wgpu::BindGroup,
}

impl SpriteBatcher {
    /// Initial instance buffer capacity (instances).
    pub const INITIAL_CAPACITY: usize = 1024;

    /// Creates a new `SpriteBatcher` with preallocated quad geometry and dynamic instance buffers.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, pipeline: &Sprite2DPipeline) -> Self {
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite2D Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite2D Quad Index Buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer_capacity = Self::INITIAL_CAPACITY;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite2D Instance Buffer"),
            size: (instance_buffer_capacity * mem::size_of::<SpriteInstance>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite2D Camera Uniform Buffer"),
            size: mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite2D Camera BindGroup"),
            layout: &pipeline.camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // 1x1 solid white fallback texture
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sprite2D White Fallback Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            size,
        );

        let white_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sprite2D Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let white_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite2D White BindGroup"),
            layout: &pipeline.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            quad_vertex_buffer,
            quad_index_buffer,
            instance_buffer,
            instance_buffer_capacity,
            camera_buffer,
            camera_bind_group,
            scratch_keys: Vec::with_capacity(Self::INITIAL_CAPACITY),
            batches: Vec::with_capacity(32),
            white_texture_bind_group,
        }
    }

    /// Clears the CPU scratchpad for beginning a new frame.
    /// Reuses vector allocation capacity without deallocating.
    pub fn begin_frame(&mut self) {
        self.scratch_keys.clear();
        self.batches.clear();
    }

    /// Submits a sprite instance into the batcher.
    pub fn push_instance(&mut self, key: SpriteSortKey, instance: SpriteInstance) {
        self.scratch_keys.push((key, instance));
    }

    /// Updates the camera view-projection uniform buffer.
    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: &Matrix4<f32>) {
        let raw: [[f32; 4]; 4] = (*view_proj).into();
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&raw));
    }

    /// Prepares GPU buffers and creates batches by sorting submitted sprite instances.
    /// Automatically reallocates GPU instance buffer if sprite count exceeds current capacity.
    pub fn finish_and_upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.scratch_keys.is_empty() {
            return;
        }

        // Sort instances by their 64-bit key (layer, order/Y, texture)
        self.scratch_keys.sort_by_key(|(key, _)| *key);

        let total_instances = self.scratch_keys.len();

        // Reallocate GPU instance buffer if needed
        if total_instances > self.instance_buffer_capacity {
            self.instance_buffer_capacity = total_instances.next_power_of_two();
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sprite2D Instance Buffer (Resized)"),
                size: (self.instance_buffer_capacity * mem::size_of::<SpriteInstance>())
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Build continuous batch ranges and pack raw instance bytes
        let mut packed_instances = Vec::with_capacity(total_instances);
        let mut current_texture_id = (self.scratch_keys[0].0.0 & 0xFFFF_FFFF) as u32;
        let mut start_idx = 0u32;
        let mut count = 0u32;

        for (idx, (key, instance)) in self.scratch_keys.iter().enumerate() {
            let tex_id = (key.0 & 0xFFFF_FFFF) as u32;
            if tex_id != current_texture_id {
                self.batches.push(SpriteBatch {
                    texture_id: current_texture_id,
                    start_instance: start_idx,
                    instance_count: count,
                });
                current_texture_id = tex_id;
                start_idx = idx as u32;
                count = 0;
            }
            packed_instances.push(*instance);
            count += 1;
        }

        if count > 0 {
            self.batches.push(SpriteBatch {
                texture_id: current_texture_id,
                start_instance: start_idx,
                instance_count: count,
            });
        }

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&packed_instances),
        );
    }

    /// Renders all batches into the provided render pass.
    /// The `lookup_texture` closure retrieves the texture bind group corresponding to a texture id.
    pub fn render<'a, F>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        pipeline: &'a Sprite2DPipeline,
        lookup_texture: F,
    ) where
        F: Fn(u32) -> Option<&'a wgpu::BindGroup>,
    {
        if self.batches.is_empty() {
            return;
        }

        rpass.set_pipeline(&pipeline.pipeline);
        rpass.set_bind_group(0, &self.camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rpass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for batch in &self.batches {
            let bind_group =
                lookup_texture(batch.texture_id).unwrap_or(&self.white_texture_bind_group);

            rpass.set_bind_group(1, bind_group, &[]);
            rpass.draw_indexed(
                0..6,
                0,
                batch.start_instance..(batch.start_instance + batch.instance_count),
            );
        }
    }
}