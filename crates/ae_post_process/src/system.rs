// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Full-screen post-processing orchestrator owning MSAA, scene, depth, and bloom targets.

use crate::bloom::create_bloom_pipelines;
use crate::targets::{
    create_depth_texture, create_target_texture_and_view, create_target_view,
    sanitize_bloom_intensity, sanitize_msaa_samples,
};
use wgpu::util::DeviceExt;

/// Full-screen post-processing system managing Bloom (extract -> blur -> composite)
/// and MSAA scene/depth targets.
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

    /// Driver-configured format of the intermediate scene render target.
    pub scene_format: wgpu::TextureFormat,

    eb_bg: Option<wgpu::BindGroup>,
    bb_bg: Option<wgpu::BindGroup>,
    fb_bg: Option<wgpu::BindGroup>,
    empty_bloom_bg: Option<wgpu::BindGroup>,
}

fn create_texture_sampler_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn encode_fullscreen_pass(
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_groups: &[&wgpu::BindGroup],
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
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
    pass.set_pipeline(pipeline);
    for (slot, bg) in bind_groups.iter().enumerate() {
        pass.set_bind_group(slot as u32, *bg, &[]);
    }
    pass.draw(0..6, 0..1);
}

impl PostProcessSystem {
    /// Initializes bloom pipelines, MSAA framebuffer, depth texture, and intermediate bloom targets.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        initial_msaa: u32,
        initial_bloom_intensity: f32,
    ) -> Self {
        let msaa_samples = sanitize_msaa_samples(initial_msaa);
        let safe_intensity = sanitize_bloom_intensity(initial_bloom_intensity);

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
            contents: bytemuck::cast_slice(&[safe_intensity]),
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
            create_bloom_pipelines(
                device,
                config,
                &bloom_bind_group_layout,
                &bloom_params_bind_group_layout,
            );

        let multisampled_framebuffer =
            create_target_view(device, config, msaa_samples.max(2), "MSAA FB");
        let (scene_texture, scene_texture_view) =
            create_target_texture_and_view(device, config, 1, "Scene TX");
        let scene_format = scene_texture.format();
        let bloom_texture_view_a = create_target_view(device, config, 1, "Bloom A");
        let bloom_texture_view_b = create_target_view(device, config, 1, "Bloom B");
        let depth_texture_view =
            create_depth_texture(device, config, msaa_samples, "depth_texture");

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
            msaa_samples,
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

    /// Rebuilds intermediate render targets and bind groups after surface resize or MSAA change.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        msaa_samples: u32,
    ) {
        self.msaa_samples = sanitize_msaa_samples(msaa_samples);
        if self.msaa_samples > 1 {
            self.multisampled_framebuffer =
                create_target_view(device, config, self.msaa_samples, "MSAA FB");
        }
        self.depth_texture_view =
            create_depth_texture(device, config, self.msaa_samples, "depth_texture");
        let (scene_texture, scene_texture_view) =
            create_target_texture_and_view(device, config, 1, "Scene TX");
        self.scene_texture_view = scene_texture_view;
        self.scene_format = scene_texture.format();
        self.bloom_texture_view_a = create_target_view(device, config, 1, "Bloom A");
        self.bloom_texture_view_b = create_target_view(device, config, 1, "Bloom B");
        self.update_bind_groups(device);
    }

    /// Uploads the sanitized bloom intensity scalar to the GPU uniform buffer.
    pub fn set_bloom_intensity(&self, queue: &wgpu::Queue, intensity: f32) {
        let safe_intensity = sanitize_bloom_intensity(intensity);
        queue.write_buffer(
            &self.bloom_params_buffer,
            0,
            bytemuck::cast_slice(&[safe_intensity]),
        );
    }

    /// Returns `(color_view, resolve_target, depth_view)` for the main 3D forward pass.
    #[must_use]
    pub fn forward_targets(
        &self,
    ) -> (
        &wgpu::TextureView,
        Option<&wgpu::TextureView>,
        &wgpu::TextureView,
    ) {
        if self.msaa_samples > 1 {
            (
                &self.multisampled_framebuffer,
                Some(&self.scene_texture_view),
                &self.depth_texture_view,
            )
        } else {
            (&self.scene_texture_view, None, &self.depth_texture_view)
        }
    }

    fn update_bind_groups(&mut self, device: &wgpu::Device) {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        self.eb_bg = Some(create_texture_sampler_bind_group(
            device,
            &self.bloom_bind_group_layout,
            &self.scene_texture_view,
            &sampler,
        ));
        self.bb_bg = Some(create_texture_sampler_bind_group(
            device,
            &self.bloom_bind_group_layout,
            &self.bloom_texture_view_a,
            &sampler,
        ));
        self.fb_bg = Some(create_texture_sampler_bind_group(
            device,
            &self.bloom_bind_group_layout,
            &self.bloom_texture_view_b,
            &sampler,
        ));
        self.empty_bloom_bg = Some(create_texture_sampler_bind_group(
            device,
            &self.bloom_bind_group_layout,
            &self.bloom_texture_view_b,
            &sampler,
        ));
    }

    /// Executes the 3-pass bloom pipeline (extract -> blur -> composite) or direct tonemapping composite.
    pub fn execute(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        bloom_enabled: bool,
    ) {
        let (Some(eb_bg), Some(fb_bg), Some(bb_bg), Some(empty_bloom_bg)) = (
            self.eb_bg.as_ref(),
            self.fb_bg.as_ref(),
            self.bb_bg.as_ref(),
            self.empty_bloom_bg.as_ref(),
        ) else {
            return;
        };

        if bloom_enabled {
            encode_fullscreen_pass(
                encoder,
                &self.bloom_texture_view_a,
                &self.bloom_extract_pipeline,
                &[eb_bg],
            );
            encode_fullscreen_pass(
                encoder,
                &self.bloom_texture_view_b,
                &self.bloom_blur_pipeline,
                &[bb_bg],
            );
            encode_fullscreen_pass(
                encoder,
                surface_view,
                &self.bloom_composite_pipeline,
                &[eb_bg, fb_bg, &self.bloom_params_bind_group],
            );
        } else {
            self.set_bloom_intensity(queue, 0.0);
            encode_fullscreen_pass(
                encoder,
                surface_view,
                &self.bloom_composite_pipeline,
                &[eb_bg, empty_bloom_bg, &self.bloom_params_bind_group],
            );
        }
    }
}