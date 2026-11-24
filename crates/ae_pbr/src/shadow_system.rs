// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hardware Cascaded Shadow Map (CSM) GPU resources, cascade pass orchestration, and upload helpers.
//!
//! Consumes CPU cascade matrix projections from [`ae_light::compute_cascade_light_space`] and
//! drives depth-array passes through [`ShadowCascadePass`] without coupling to scene/asset storage.

use crate::shadow_pipeline::{create_shadow_cutout_pipeline, create_shadow_pipeline};
use crate::vertex::PBR_DEPTH_FORMAT;
use ae_light::{CascadeShadowConfig, LightSpaceUniform, compute_cascade_light_space};
use wgpu::util::DeviceExt;

/// Cascaded Shadow Map (CSM) GPU subsystem managing up to 4 depth-array cascades.
pub struct ShadowSystem {
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub shadow_cutout_pipeline: wgpu::RenderPipeline,

    pub shadow_depth_texture: wgpu::Texture,
    pub shadow_cascade_views: [wgpu::TextureView; 4],

    pub shadow_bind_group_layout: wgpu::BindGroupLayout,
    pub shadow_bind_group: wgpu::BindGroup,

    pub light_space_bgl_vs: wgpu::BindGroupLayout,
    pub light_space_uniform: LightSpaceUniform,
    pub light_space_buffer: wgpu::Buffer,

    pub shadow_matrix_buffers: [wgpu::Buffer; 4],
    pub shadow_pass_bind_groups: [wgpu::BindGroup; 4],
}

/// Zero-allocation pass interface handed to scene renderers for recording opaque and cutout shadow draws.
pub struct ShadowCascadePass<'a, 'enc> {
    pass: wgpu::RenderPass<'enc>,
    cutout_pipeline: &'a wgpu::RenderPipeline,
}

impl<'a, 'enc> ShadowCascadePass<'a, 'enc> {
    /// Draws a non-indexed vertex/instance range with the active opaque shadow pipeline.
    pub fn draw_vertices(
        &mut self,
        vertex_buffer: wgpu::BufferSlice<'a>,
        instance_buffer: wgpu::BufferSlice<'a>,
        vertices: std::ops::Range<u32>,
        instances: std::ops::Range<u32>,
    ) {
        self.pass.set_vertex_buffer(0, vertex_buffer);
        self.pass.set_vertex_buffer(1, instance_buffer);
        self.pass.draw(vertices, instances);
    }

    /// Draws an indexed vertex/instance range (`Uint32` indices) with the currently bound pipeline.
    pub fn draw_indexed(
        &mut self,
        vertex_buffer: wgpu::BufferSlice<'a>,
        index_buffer: wgpu::BufferSlice<'a>,
        instance_buffer: wgpu::BufferSlice<'a>,
        indices: std::ops::Range<u32>,
        instances: std::ops::Range<u32>,
    ) {
        self.pass.set_vertex_buffer(0, vertex_buffer);
        self.pass.set_vertex_buffer(1, instance_buffer);
        self.pass
            .set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
        self.pass.draw_indexed(indices, 0, instances);
    }

    /// Switches from the opaque depth pipeline to the alpha-tested cutout shadow pipeline.
    pub fn begin_cutout_stage(&mut self) {
        self.pass.set_pipeline(self.cutout_pipeline);
    }

    /// Binds a texture bind group at slot `1` for cutout alpha-mask sampling.
    pub fn set_cutout_texture(&mut self, bind_group: &'a wgpu::BindGroup) {
        self.pass.set_bind_group(1, bind_group, &[]);
    }
}

fn build_cascade_depth_resources(
    device: &wgpu::Device,
    resolution: u32,
) -> (
    wgpu::Texture,
    [wgpu::TextureView; 4],
    wgpu::TextureView,
    wgpu::Sampler,
) {
    let safe_res = resolution.max(1);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Shadow Depth Texture"),
        size: wgpu::Extent3d {
            width: safe_res,
            height: safe_res,
            depth_or_array_layers: 4,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: PBR_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let layer_labels = ["CSM0", "CSM1", "CSM2", "CSM3"];
    let cascade_views = std::array::from_fn(|idx| {
        texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(layer_labels[idx]),
            base_array_layer: idx as u32,
            array_layer_count: Some(1),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        })
    });

    let array_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Shadow Array View"),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Shadow Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });

    (texture, cascade_views, array_view, sampler)
}

fn create_shadow_sample_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    array_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    light_space_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Shadow Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(array_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: light_space_buffer.as_entire_binding(),
            },
        ],
    })
}

impl ShadowSystem {
    /// Initializes depth textures, samplers, per-cascade buffers, and shadow pipelines.
    pub fn new(
        device: &wgpu::Device,
        config: &CascadeShadowConfig,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let (shadow_depth_texture, shadow_cascade_views, array_view, sampler) =
            build_cascade_depth_resources(device, config.resolution.as_u32());

        let light_space_uniform = LightSpaceUniform {
            matrices: [[[0.0; 4]; 4]; 4],
            cascade_splits: [0.0; 4],
            shadow_bias: config.bias,
            pcf_radius: config.pcf.radius(),
            shadow_enabled: u32::from(config.enabled),
            cascade_count: config.cascade_count.clamp(1, 4),
        };
        let light_space_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Space Buffer"),
            contents: bytemuck::cast_slice(&[light_space_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shadow_bind_group = create_shadow_sample_bind_group(
            device,
            &shadow_bind_group_layout,
            &array_view,
            &sampler,
            &light_space_buffer,
        );

        let light_space_bgl_vs =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow VS BGL"),
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

        let mat_labels = [
            "Cascade Matrix Buffer 0",
            "Cascade Matrix Buffer 1",
            "Cascade Matrix Buffer 2",
            "Cascade Matrix Buffer 3",
        ];
        let bg_labels = [
            "Cascade Bind Group 0",
            "Cascade Bind Group 1",
            "Cascade Bind Group 2",
            "Cascade Bind Group 3",
        ];
        let shadow_matrix_buffers = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(mat_labels[i]),
                size: 64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        let shadow_pass_bind_groups = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(bg_labels[i]),
                layout: &light_space_bgl_vs,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_matrix_buffers[i].as_entire_binding(),
                }],
            })
        });

        let shadow_pipeline = create_shadow_pipeline(device, &light_space_bgl_vs);
        let shadow_cutout_pipeline =
            create_shadow_cutout_pipeline(device, &light_space_bgl_vs, texture_bind_group_layout);

        Self {
            shadow_pipeline,
            shadow_cutout_pipeline,
            shadow_depth_texture,
            shadow_cascade_views,
            shadow_bind_group_layout,
            shadow_bind_group,
            light_space_bgl_vs,
            light_space_uniform,
            light_space_buffer,
            shadow_matrix_buffers,
            shadow_pass_bind_groups,
        }
    }

    /// Recreates shadow depth textures and sampling bind group after a shadow resolution change.
    pub fn resize_targets(&mut self, device: &wgpu::Device, resolution: u32) {
        let (texture, cascade_views, array_view, sampler) =
            build_cascade_depth_resources(device, resolution);
        self.shadow_depth_texture = texture;
        self.shadow_cascade_views = cascade_views;
        self.shadow_bind_group = create_shadow_sample_bind_group(
            device,
            &self.shadow_bind_group_layout,
            &array_view,
            &sampler,
            &self.light_space_buffer,
        );
    }

    /// Computes light-space matrices via [`ae_light::compute_cascade_light_space`] and uploads to GPU buffers.
    pub fn update_cascades(
        &mut self,
        queue: &wgpu::Queue,
        config: &CascadeShadowConfig,
        camera: &ae_camera::Camera,
        light_dir: cgmath::Vector3<f32>,
    ) {
        let next_uniform = compute_cascade_light_space(config, camera, light_dir);
        for (i, matrix) in next_uniform
            .matrices
            .iter()
            .take(next_uniform.cascade_count as usize)
            .enumerate()
        {
            queue.write_buffer(
                &self.shadow_matrix_buffers[i],
                0,
                bytemuck::cast_slice(&[*matrix]),
            );
        }

        self.light_space_uniform = next_uniform;
        queue.write_buffer(
            &self.light_space_buffer,
            0,
            bytemuck::cast_slice(&[self.light_space_uniform]),
        );
    }

    /// Executes depth-only shadow passes across active cascades, delegating mesh draw submission to `record_cascade`.
    pub fn execute_pass<'a>(
        &'a self,
        encoder: &mut wgpu::CommandEncoder,
        enabled: bool,
        cascade_count: u32,
        has_instances: bool,
        mut record_cascade: impl FnMut(&mut ShadowCascadePass<'a, '_>),
    ) {
        if !enabled || !has_instances {
            return;
        }

        let cascades = cascade_count.clamp(1, 4) as usize;
        let pass_labels = [
            "Shadow Pass 0",
            "Shadow Pass 1",
            "Shadow Pass 2",
            "Shadow Pass 3",
        ];

        for cascade_idx in 0..cascades {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(pass_labels[cascade_idx]),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_cascade_views[cascade_idx],
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_pass_bind_groups[cascade_idx], &[]);
            let mut cascade_pass = ShadowCascadePass {
                pass,
                cutout_pipeline: &self.shadow_cutout_pipeline,
            };
            record_cascade(&mut cascade_pass);
        }
    }
}