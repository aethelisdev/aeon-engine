// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::resources;
use wgpu::util::DeviceExt;

/// Full-screen post-processing system managing Bloom (extract → blur → composite)
/// and MSAA resolve.
/// Owns intermediate render targets, bind groups, and the bloom parameter buffer.
pub struct PostProcessSystem {
    pub bloom_extract_pipeline: wgpu::RenderPipeline,
    pub bloom_blur_pipeline: wgpu::RenderPipeline,
    pub bloom_composite_pipeline: wgpu::RenderPipeline,

    pub bloom_bind_group_layout: wgpu::BindGroupLayout,
    pub bloom_params_bind_group_layout: wgpu::BindGroupLayout,
    pub bloom_params_buffer: wgpu::Buffer,
    pub bloom_params_bind_group: wgpu::BindGroup,

    pub scene_texture_view: wgpu::TextureView,
    pub bloom_texture_view_a: wgpu::TextureView,
    pub bloom_texture_view_b: wgpu::TextureView,

    pub msaa_samples: u32,
    pub multisampled_framebuffer: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,

    /// The actual driver-configured format of the intermediate scene render target.
    /// Queried dynamically from the created texture, this format is passed to other
    /// rendering pipelines to guarantee compatibility and prevent Vulkan mismatch panics.
    pub scene_format: wgpu::TextureFormat,

    pub eb_bg: Option<wgpu::BindGroup>,
    pub bb_bg: Option<wgpu::BindGroup>,
    pub fb_bg: Option<wgpu::BindGroup>,
    pub empty_bloom_bg: Option<wgpu::BindGroup>,
}

impl PostProcessSystem {
    /// Initializes the post-processing system with bloom pipelines, MSAA framebuffer,
    /// depth texture, and intermediate bloom targets.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        initial_msaa: u32,
        initial_bloom_intensity: f32,
    ) -> Self {
        let bloom_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let bloom_params_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom Params BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bloom_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bloom Params Buffer"),
            contents: bytemuck::cast_slice(&[initial_bloom_intensity]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bloom_params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Params Bind Group"),
            layout: &bloom_params_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bloom_params_buffer.as_entire_binding(),
            }],
        });

        let (bloom_extract_pipeline, bloom_blur_pipeline, bloom_composite_pipeline) =
            crate::render::pipelines::bloom::create_bloom_pipelines(
                device,
                config,
                &bloom_bind_group_layout,
                &bloom_params_bind_group_layout,
            );

        let multisampled_framebuffer =
            resources::create_target_view(device, config, initial_msaa.max(2), "MSAA FB");
        let (scene_texture, scene_texture_view) =
            resources::create_target_texture_and_view(device, config, 1, "Scene TX");
        let scene_format = scene_texture.format();
        let bloom_texture_view_a = resources::create_target_view(device, config, 1, "Bloom A");
        let bloom_texture_view_b = resources::create_target_view(device, config, 1, "Bloom B");
        let depth_texture_view =
            resources::create_depth_texture(device, config, initial_msaa, "depth_texture");

        let mut sys = Self {
            bloom_extract_pipeline,
            bloom_blur_pipeline,
            bloom_composite_pipeline,
            bloom_bind_group_layout,
            bloom_params_bind_group_layout,
            bloom_params_buffer,
            bloom_params_bind_group,
            scene_texture_view,
            bloom_texture_view_a,
            bloom_texture_view_b,
            msaa_samples: initial_msaa,
            multisampled_framebuffer,
            depth_texture_view,
            scene_format,
            eb_bg: None,
            bb_bg: None,
            fb_bg: None,
            empty_bloom_bg: None,
        };
        sys.update_bind_groups(device);
        sys
    }

    /// Rebuilds all render targets and bind groups after a surface resize or MSAA change.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        msaa_samples: u32,
    ) {
        self.msaa_samples = msaa_samples;
        if self.msaa_samples > 1 {
            self.multisampled_framebuffer =
                resources::create_target_view(device, config, self.msaa_samples, "MSAA FB");
        }
        self.depth_texture_view =
            resources::create_depth_texture(device, config, self.msaa_samples, "depth_texture");
        let (scene_texture, scene_texture_view) =
            resources::create_target_texture_and_view(device, config, 1, "Scene TX");
        self.scene_texture_view = scene_texture_view;
        self.scene_format = scene_texture.format();
        self.bloom_texture_view_a = resources::create_target_view(device, config, 1, "Bloom A");
        self.bloom_texture_view_b = resources::create_target_view(device, config, 1, "Bloom B");
        self.update_bind_groups(device);
    }

    fn update_bind_groups(&mut self, device: &wgpu::Device) {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        self.eb_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.scene_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        self.bb_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.bloom_texture_view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        self.fb_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.bloom_texture_view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        self.empty_bloom_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.bloom_texture_view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
    }

    /// Executes the full bloom pipeline (extract → blur → composite) or a direct
    /// composite pass when bloom is disabled.
    pub fn execute(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        bloom_enabled: bool,
    ) {
        let eb_bg = self.eb_bg.as_ref().unwrap();
        let fb_bg = self.fb_bg.as_ref().unwrap();
        let bb_bg = self.bb_bg.as_ref().unwrap();
        let empty_bloom_bg = self.empty_bloom_bg.as_ref().unwrap();

        if bloom_enabled {
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.bloom_texture_view_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.bloom_extract_pipeline);
                pass.set_bind_group(0, eb_bg, &[]);
                pass.draw(0..6, 0..1);
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.bloom_texture_view_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.bloom_blur_pipeline);
                pass.set_bind_group(0, bb_bg, &[]);
                pass.draw(0..6, 0..1);
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.bloom_composite_pipeline);
                pass.set_bind_group(0, eb_bg, &[]);
                pass.set_bind_group(1, fb_bg, &[]);
                pass.set_bind_group(2, &self.bloom_params_bind_group, &[]);
                pass.draw(0..6, 0..1);
            }
        } else {
            queue.write_buffer(
                &self.bloom_params_buffer,
                0,
                bytemuck::cast_slice(&[0.0_f32]),
            );
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.bloom_composite_pipeline);
                pass.set_bind_group(0, eb_bg, &[]);
                pass.set_bind_group(1, empty_bloom_bg, &[]);
                pass.set_bind_group(2, &self.bloom_params_bind_group, &[]);
                pass.draw(0..6, 0..1);
            }
        }
    }
}