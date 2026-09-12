// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport 3D Billboard WGPU Pipeline
//!
//! Pipeline layout, shader compilation, and texture array atlas loading
//! for 3D billboard icon badge rendering.

use super::types::BillboardVertex;

/// Embedded 64x64x16 Editor Texture Array atlas containing viewport and entity icons.
const TOOLS_ICON_BYTES: &[u8] = include_bytes!("../../../../assets/icons/editor_atlas.png");

/// Manages WGPU pipeline and texture bindings for billboard badge rendering.
pub struct BillboardPipeline {
    /// Active render pipeline configured for alpha blending and MSAA.
    pub pipeline: wgpu::RenderPipeline,
    /// Bind group layout for camera uniform buffer.
    pub camera_bgl: wgpu::BindGroupLayout,
    /// Bind group layout for texture array atlas and sampler.
    pub texture_bgl: wgpu::BindGroupLayout,
    /// Bind group containing texture array atlas and sampler.
    pub texture_bind_group: wgpu::BindGroup,
    /// Texture array atlas view.
    pub texture_view: wgpu::TextureView,
}

impl BillboardPipeline {
    /// Creates a new `BillboardPipeline` for the specified surface format and sample count.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Billboard Camera BGL"),
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

        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Billboard Texture BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
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

        let (texture_view, texture_bind_group) =
            Self::create_texture_atlas(device, queue, &texture_bgl);

        let pipeline = Self::build_pipeline(
            device,
            surface_format,
            msaa_samples,
            &camera_bgl,
            &texture_bgl,
        );

        Self {
            pipeline,
            camera_bgl,
            texture_bgl,
            texture_bind_group,
            texture_view,
        }
    }

    /// Rebuilds the render pipeline when MSAA sample count or target format changes.
    pub fn rebuild_pipeline(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) {
        self.pipeline = Self::build_pipeline(
            device,
            surface_format,
            msaa_samples,
            &self.camera_bgl,
            &self.texture_bgl,
        );
    }

    /// Builds the internal WGPU render pipeline.
    fn build_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
        camera_bgl: &wgpu::BindGroupLayout,
        texture_bgl: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Billboard WGSL Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/billboard.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Billboard Pipeline Layout"),
            bind_group_layouts: &[Some(camera_bgl), Some(texture_bgl)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Billboard Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(BillboardVertex::layout())],
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
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
        })
    }

    /// Loads the embedded 64x64 icon atlas as an isolated 2D texture array into GPU memory.
    fn create_texture_atlas(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bgl: &wgpu::BindGroupLayout,
    ) -> (wgpu::TextureView, wgpu::BindGroup) {
        let img = image::load_from_memory(TOOLS_ICON_BYTES)
            .expect("Failed to decode embedded editor_atlas.png");
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let raw_rgba = rgba.as_raw();

        let tile_size = 64u32;
        let cols = width / tile_size;
        let rows = height / tile_size;
        let layer_count = (cols * rows).max(16);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Billboard Atlas Texture Array"),
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: layer_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Copy each 64x64 icon tile into its independent 2D array layer
        for r in 0..rows {
            for c in 0..cols {
                let layer = r * cols + c;
                if layer >= layer_count {
                    break;
                }

                let mut tile_pixels = Vec::with_capacity((tile_size * tile_size * 4) as usize);
                for y in 0..tile_size {
                    let src_y = r * tile_size + y;
                    let src_x_start = c * tile_size;
                    let start_idx = ((src_y * width + src_x_start) * 4) as usize;
                    let end_idx = start_idx + (tile_size * 4) as usize;
                    tile_pixels.extend_from_slice(&raw_rgba[start_idx..end_idx]);
                }

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &tile_pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(tile_size * 4),
                        rows_per_image: Some(tile_size),
                    },
                    wgpu::Extent3d {
                        width: tile_size,
                        height: tile_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Billboard Texture Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Billboard Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Billboard Texture Bind Group"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        (texture_view, bind_group)
    }
}