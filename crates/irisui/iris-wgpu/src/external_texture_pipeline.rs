// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU pipeline for compositing an externally owned single-layer `Texture2D` into Iris UI.
//!
//! This module deliberately remains separate from the texture-array pipeline. External render
//! targets use a `D2` texture view while icons and atlas-backed widgets use `D2Array`; keeping
//! their bind-group layouts distinct prevents view-dimension mismatches and leaves existing atlas
//! rendering unchanged. Instance-buffer capacity is retained between frames; uploads can still
//! require backend staging allocations, which must be measured separately from capacity growth.

use bytemuck::{Pod, Zeroable};
use iris_core::{Color, Rect};
use wgpu::util::DeviceExt;

/// Per-instance data for compositing one external two-dimensional texture into screen space.
/// The texture is sampled in its supplied linear view format. Tint colors are converted from the
/// public sRGB `Color` representation before upload, preserving correct output on sRGB surfaces.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ExternalTextureQuadInstance {
    /// Screen-space destination rectangle `[x, y, width, height]` in physical pixels.
    pub rect: [f32; 4],
    /// Source texture UV bounds `[min_u, min_v, max_u, max_v]`.
    pub uv_rect: [f32; 4],
    /// Linear RGBA multiplier applied after texture sampling.
    pub tint: [f32; 4],
    /// Screen-space clip bounds `[min_x, min_y, max_x, max_y]`; inverted bounds disable clipping.
    pub clip_rect: [f32; 4],
}

impl ExternalTextureQuadInstance {
    /// Creates a full-coverage external texture quad with an optional screen-space clip boundary.
    /// The destination rectangle must describe physical pixels in the same coordinate space as the
    /// prepared screen size. The supplied tint is converted to linear color because the fragment
    /// output is written into a linear rendering pipeline.
    pub fn new(rect: Rect, tint: Color, clip_rect: Option<Rect>) -> Self {
        Self::with_uv(rect, [0.0, 0.0, 1.0, 1.0], tint, clip_rect)
    }

    /// Creates an external texture quad using explicit normalized source UV bounds.
    /// Callers can use this for cropped render targets or vertically flipped source views. UV
    /// validation remains the caller's responsibility because texture addressing policy belongs to
    /// the owner of the external render target. An absent clip is represented by inverted bounds,
    /// preserving every valid clip rectangle, including zero-sized and off-screen rectangles.
    pub fn with_uv(rect: Rect, uv_rect: [f32; 4], tint: Color, clip_rect: Option<Rect>) -> Self {
        let clip = clip_rect.map_or([1.0, 1.0, 0.0, 0.0], |clip| {
            [clip.x, clip.y, clip.right(), clip.bottom()]
        });

        Self {
            rect: rect.to_array(),
            uv_rect,
            tint: tint.to_linear().to_array(),
            clip_rect: clip,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ExternalTextureUniforms {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct UnitVertex {
    position: [f32; 2],
}

/// GPU resources used to draw a reusable external `Texture2D` quad in an Iris render pass.
/// The pipeline accepts a standard `TextureViewDimension::D2` binding and therefore cannot be
/// used for icon atlases. Call [`Self::prepare`] before [`Self::render`] on every frame that the
/// destination geometry or screen size changes. The texture bind group is intentionally supplied
/// by the caller so it can be recreated only when the external texture view changes.
pub struct ExternalTexturePipeline {
    pipeline: wgpu::RenderPipeline,
    globals_bind_group: wgpu::BindGroup,
    globals_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    instance_count: u32,
    screen_size: [f32; 2],
    uniforms_dirty: bool,
}

impl ExternalTexturePipeline {
    const UNIT_VERTICES: [UnitVertex; 4] = [
        UnitVertex {
            position: [0.0, 0.0],
        },
        UnitVertex {
            position: [1.0, 0.0],
        },
        UnitVertex {
            position: [1.0, 1.0],
        },
        UnitVertex {
            position: [0.0, 1.0],
        },
    ];
    const UNIT_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

    /// Creates the external texture pipeline for the specified UI surface format.
    /// The pipeline uses straight-alpha blending and a linear filtering sampler. Its bind-group
    /// layout requires a single-sampled, filterable `Texture2D` view, matching resolved render
    /// targets such as a 3D viewport color texture.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iris UI External Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/external_texture_quad.wgsl").into(),
            ),
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iris UI External Texture Globals Buffer"),
            size: std::mem::size_of::<ExternalTextureUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Iris UI External Texture Globals Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iris UI External Texture Globals Bind Group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Iris UI External Texture View Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Iris UI External Texture Linear Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Iris UI External Texture Pipeline Layout"),
            bind_group_layouts: &[Some(&globals_layout), Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnitVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ExternalTextureQuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Iris UI External Texture Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout), Some(instance_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI External Texture Unit Vertex Buffer"),
            contents: bytemuck::cast_slice(&Self::UNIT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI External Texture Unit Index Buffer"),
            contents: bytemuck::cast_slice(&Self::UNIT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            globals_bind_group,
            globals_buffer,
            texture_bind_group_layout,
            sampler,
            vertex_buffer,
            index_buffer,
            instance_buffer: None,
            instance_capacity: 0,
            instance_count: 0,
            screen_size: [0.0, 0.0],
            uniforms_dirty: true,
        }
    }

    /// Creates the texture bind group for one externally owned single-layer texture view.
    /// Recreate this bind group only when the source texture view changes, such as after a viewport
    /// resize. The same bind group may be reused across frames while the view remains valid.
    pub fn create_texture_bind_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iris UI External Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Uploads the screen dimensions and destination instance for the next render call.
    /// A single persistent GPU buffer is allocated on the first call and reused thereafter. Zero
    /// screen dimensions are clamped to one pixel to prevent shader division-by-zero during
    /// minimized-window frames.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        instance: ExternalTextureQuadInstance,
    ) {
        self.prepare_instances(device, queue, screen_size, std::slice::from_ref(&instance));
    }

    /// Uploads all external image destinations once before an ordered render pass.
    /// Capacity grows geometrically and is retained when the list shrinks. Empty lists disable
    /// draws without destroying buffers; zero screen dimensions are clamped to one pixel.
    pub fn prepare_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        instances: &[ExternalTextureQuadInstance],
    ) {
        self.instance_count = instances.len() as u32;
        let safe_screen_size = [screen_size[0].max(1.0), screen_size[1].max(1.0)];
        let size_changed = (self.screen_size[0] - safe_screen_size[0]).abs() > f32::EPSILON
            || (self.screen_size[1] - safe_screen_size[1]).abs() > f32::EPSILON;

        if size_changed || self.uniforms_dirty {
            let uniforms = ExternalTextureUniforms {
                screen_size: safe_screen_size,
                _padding: [0.0, 0.0],
            };
            queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&uniforms));
            self.screen_size = safe_screen_size;
            self.uniforms_dirty = false;
        }

        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Iris UI External Texture Instance Buffer"),
                size: (self.instance_capacity * std::mem::size_of::<ExternalTextureQuadInstance>())
                    as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if !instances.is_empty()
            && let Some(instance_buffer) = &self.instance_buffer
        {
            queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    /// Draws the most recently prepared external texture instance into the active render pass.
    /// Calling this before [`Self::prepare`] is safe and records no draw command. The caller owns
    /// render-pass ordering, allowing a future viewport compositor to place the image beneath Iris
    /// overlays without changing the existing texture-array command stream.
    pub fn render<'render>(
        &'render self,
        render_pass: &mut wgpu::RenderPass<'render>,
        texture_bind_group: &'render wgpu::BindGroup,
    ) {
        self.render_instance(render_pass, texture_bind_group, 0);
    }

    /// Draws a prepared destination using the supplied D2 binding, without changing scissor state.
    /// Out-of-range indices and calls before preparation are ignored. Every pipeline binding is
    /// restored, allowing safe interleaving with SDF and texture-array commands.
    pub fn render_instance<'render>(
        &'render self,
        render_pass: &mut wgpu::RenderPass<'render>,
        texture_bind_group: &'render wgpu::BindGroup,
        instance_index: u32,
    ) {
        if instance_index >= self.instance_count {
            return;
        }
        let Some(instance_buffer) = &self.instance_buffer else {
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
        render_pass.set_bind_group(1, texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, instance_index..instance_index + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_texture_quad_converts_tint_and_clip_bounds() {
        let tint = Color::rgba(0.5, 0.25, 0.75, 0.8);
        let instance = ExternalTextureQuadInstance::new(
            Rect::new(10.0, 20.0, 300.0, 180.0),
            tint,
            Some(Rect::new(12.0, 24.0, 280.0, 160.0)),
        );

        assert_eq!(instance.rect, [10.0, 20.0, 300.0, 180.0]);
        assert_eq!(instance.uv_rect, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(instance.tint, tint.to_linear().to_array());
        assert_eq!(instance.clip_rect, [12.0, 24.0, 292.0, 184.0]);
    }

    #[test]
    fn test_external_texture_quad_preserves_explicit_uv_bounds() {
        let instance = ExternalTextureQuadInstance::with_uv(
            Rect::new(0.0, 0.0, 64.0, 64.0),
            [0.2, 0.1, 0.8, 0.9],
            Color::WHITE,
            None,
        );

        assert_eq!(instance.uv_rect, [0.2, 0.1, 0.8, 0.9]);
        assert_eq!(instance.clip_rect, [1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_external_texture_quad_preserves_zero_sized_clip() {
        let instance = ExternalTextureQuadInstance::new(
            Rect::new(0.0, 0.0, 64.0, 64.0),
            Color::WHITE,
            Some(Rect::ZERO),
        );

        assert_eq!(instance.clip_rect, [0.0, 0.0, 0.0, 0.0]);
    }
}