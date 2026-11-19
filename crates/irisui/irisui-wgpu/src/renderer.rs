// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Main GPU renderer managing WGPU SDF pipeline, buffers, and instanced draw passes.

use crate::quad::QuadInstance;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Uniform buffer structure sent to GPU representing screen dimensions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GlobalUniforms {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

/// Unit vertex for unit quad mesh.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct UnitVertex {
    position: [f32; 2],
}

/// Rendering mode controlling how [`crate::command::DrawCommand::DrawExternalTexture`] commands are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandListRenderMode {
    /// Draw all SDF quads, atlas textures, and external textures in exact Z-order.
    Full,
    /// Draw SDF quads and atlas textures while punching `alpha = 0` cutouts for external textures.
    StaticWithCutout,
    /// Draw exclusively registered external textures onto the active render target.
    ExternalTexturesOnly,
}

/// Main WGPU-based SDF Renderer for Iris UI.
pub struct IrisRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    instance_count: u32,
    screen_size: [f32; 2],
    uniform_buffer_dirty: bool,
    /// Dedicated texture quad pipeline for rendering images, icons, and embedded viewports.
    pub texture_pipeline: crate::texture_pipeline::TextureQuadPipeline,
    texture_instance_buffer: Option<wgpu::Buffer>,
    texture_instance_capacity: usize,
    uploaded_texture_quads: Vec<crate::texture_pipeline::TextureQuadInstance>,
    /// Active texture bind group used for textured quad draw commands.
    pub texture_bind_group: Option<wgpu::BindGroup>,
    /// Dedicated external texture pipeline for compositing host-owned 2D render targets.
    pub external_pipeline: crate::external_texture_pipeline::ExternalTexturePipeline,
    /// Persistent resource table managing external texture registrations and bind groups.
    pub external_textures: crate::external_textures::ExternalTextures,
    /// Hardware layer cache managing boundary-level GPU instance buffer slices and partial uploads.
    pub layer_cache: crate::layer_cache::BoundaryLayerCache,
}

impl IrisRenderer {
    /// Unit quad vertices [0.0 .. 1.0].
    const UNIT_VERTICES: &'static [UnitVertex] = &[
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

    /// Unit quad index list (2 triangles = 6 indices).
    const UNIT_INDICES: &'static [u16] = &[0, 1, 2, 0, 2, 3];

    /// Creates a new `IrisRenderer` instance for the given WGPU device and target surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iris UI SDF Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sdf_quad.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iris UI Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Iris UI Bind Group Layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iris UI Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Iris UI Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnitVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Iris UI SDF Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout), Some(QuadInstance::desc())],
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
            label: Some("Iris UI Unit Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(Self::UNIT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Iris UI Unit Quad Index Buffer"),
            contents: bytemuck::cast_slice(Self::UNIT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_pipeline =
            crate::texture_pipeline::TextureQuadPipeline::new(device, target_format);

        let external_pipeline =
            crate::external_texture_pipeline::ExternalTexturePipeline::new(device, target_format);
        let external_textures = crate::external_textures::ExternalTextures::default();

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            instance_buffer: None,
            instance_capacity: 0,
            instance_count: 0,
            screen_size: [0.0, 0.0],
            uniform_buffer_dirty: true,
            texture_pipeline,
            texture_instance_buffer: None,
            texture_instance_capacity: 0,
            uploaded_texture_quads: Vec::new(),
            texture_bind_group: None,
            external_pipeline,
            external_textures,
            layer_cache: crate::layer_cache::BoundaryLayerCache::new(),
        }
    }

    /// Sets the active texture bind group to be used for textured quad draw commands.
    #[inline]
    pub fn set_texture_bind_group(&mut self, bind_group: Option<wgpu::BindGroup>) {
        self.texture_bind_group = bind_group;
    }

    /// Prepares buffers and uploads instance data to the GPU before rendering.
    #[inline]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        quads: &[QuadInstance],
    ) {
        self.prepare_with_dirty_boundaries(device, queue, screen_size, quads, &[], None);
    }

    /// Prepares buffers and uploads instance data to the GPU, supporting partial uploads for dirty boundaries.
    pub fn prepare_with_dirty_boundaries(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        quads: &[QuadInstance],
        boundary_slices: &[(irisui_core::WidgetId, crate::command::BoundarySlice)],
        dirty_boundaries: Option<&std::collections::HashSet<irisui_core::WidgetId>>,
    ) {
        let size_changed = (self.screen_size[0] - screen_size[0]).abs() > 0.001
            || (self.screen_size[1] - screen_size[1]).abs() > 0.001;

        if size_changed || self.uniform_buffer_dirty {
            let uniforms = GlobalUniforms {
                screen_size,
                _padding: [0.0, 0.0],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
            self.screen_size = screen_size;
            self.uniform_buffer_dirty = false;
        }

        self.instance_count = quads.len() as u32;

        if self.instance_count == 0 {
            return;
        }

        self.layer_cache.update_instance_buffer(
            crate::layer_cache::InstanceBufferTarget {
                device,
                queue,
                buffer: &mut self.instance_buffer,
                capacity: &mut self.instance_capacity,
            },
            quads,
            boundary_slices,
            dirty_boundaries,
        );
    }

    /// Prepares buffers and uploads instance data from a `DrawCommandList` to the GPU.
    #[inline]
    pub fn prepare_command_list(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        command_list: &crate::command::DrawCommandList,
    ) {
        self.prepare_command_list_with_dirty_boundaries(
            device,
            queue,
            screen_size,
            command_list,
            None,
        );
    }

    /// Prepares buffers and uploads instance data from a `DrawCommandList` with boundary slicing and partial GPU updates.
    ///
    /// When `dirty_boundaries` is provided and the slice layout has not shifted, only the dirty boundary
    /// sub-ranges are uploaded via `queue.write_buffer`, leaving clean panels untouched in GPU VRAM ($0$ transfer).
    pub fn prepare_command_list_with_dirty_boundaries(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: [f32; 2],
        command_list: &crate::command::DrawCommandList,
        dirty_boundaries: Option<&std::collections::HashSet<irisui_core::WidgetId>>,
    ) {
        self.prepare_with_dirty_boundaries(
            device,
            queue,
            screen_size,
            &command_list.quads,
            &command_list.boundary_slices,
            dirty_boundaries,
        );

        self.texture_pipeline.update_globals(queue, screen_size);

        if !command_list.texture_quads.is_empty() {
            let needed_capacity = command_list.texture_quads.len();
            let mut buffer_reallocated = false;
            if self.texture_instance_capacity < needed_capacity {
                let new_capacity = (needed_capacity * 2).max(16);
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Iris UI Instanced Texture Quad Buffer"),
                    size: (new_capacity
                        * std::mem::size_of::<crate::texture_pipeline::TextureQuadInstance>())
                        as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.texture_instance_buffer = Some(buffer);
                self.texture_instance_capacity = new_capacity;
                buffer_reallocated = true;
            }

            if let Some(ref buffer) = self.texture_instance_buffer
                && (buffer_reallocated
                    || self.uploaded_texture_quads.as_slice()
                        != command_list.texture_quads.as_slice())
            {
                queue.write_buffer(buffer, 0, bytemuck::cast_slice(&command_list.texture_quads));
                self.uploaded_texture_quads.clear();
                self.uploaded_texture_quads
                    .extend_from_slice(&command_list.texture_quads);
            }
        }

        self.external_pipeline.prepare_instances(
            device,
            queue,
            screen_size,
            &command_list.external_texture_quads,
        );
    }

    /// Executes sequential draw commands and updates hardware scissor rects in exact Z-order.
    pub fn render_command_list<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        command_list: &'rp crate::command::DrawCommandList,
        screen_size: (u32, u32),
    ) {
        self.render_command_list_with_mode(
            render_pass,
            command_list,
            screen_size,
            CommandListRenderMode::Full,
        );
    }

    /// Executes sequential draw commands using the specified [`CommandListRenderMode`].
    pub fn render_command_list_with_mode<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        command_list: &'rp crate::command::DrawCommandList,
        screen_size: (u32, u32),
        mode: CommandListRenderMode,
    ) {
        #[derive(Copy, Clone, PartialEq, Eq)]
        enum ActivePipeline {
            None,
            Sdf,
            Texture,
            ExternalTexture,
        }

        let mut active_pipe = ActivePipeline::None;
        let mut active_scissor: Option<(u32, u32, u32, u32)> = None;
        let commands = &command_list.commands;
        let mut idx = 0;

        while idx < commands.len() {
            match commands[idx] {
                crate::command::DrawCommand::DrawSdfQuads { start, count } => {
                    if mode != CommandListRenderMode::ExternalTexturesOnly
                        && count > 0
                        && let Some(ref instance_buf) = self.instance_buffer
                    {
                        if active_pipe != ActivePipeline::Sdf {
                            render_pass.set_pipeline(&self.pipeline);
                            render_pass.set_bind_group(0, &self.bind_group, &[]);
                            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                            render_pass.set_vertex_buffer(1, instance_buf.slice(..));
                            render_pass.set_index_buffer(
                                self.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            active_pipe = ActivePipeline::Sdf;
                        }
                        render_pass.draw_indexed(0..6, 0, start..(start + count));
                    }
                    idx += 1;
                }
                crate::command::DrawCommand::SetScissor {
                    x,
                    y,
                    width,
                    height,
                } => {
                    let clamped_w = width.min(screen_size.0.saturating_sub(x)).max(1);
                    let clamped_h = height.min(screen_size.1.saturating_sub(y)).max(1);
                    if x < screen_size.0 && y < screen_size.1 {
                        let target = (x, y, clamped_w, clamped_h);
                        if active_scissor != Some(target) {
                            render_pass.set_scissor_rect(x, y, clamped_w, clamped_h);
                            active_scissor = Some(target);
                        }
                    }
                    idx += 1;
                }
                crate::command::DrawCommand::ResetScissor => {
                    let target = (0, 0, screen_size.0.max(1), screen_size.1.max(1));
                    if active_scissor != Some(target) {
                        render_pass.set_scissor_rect(0, 0, target.2, target.3);
                        active_scissor = Some(target);
                    }
                    idx += 1;
                }
                crate::command::DrawCommand::DrawTexture { instance_index } => {
                    let (end, next_idx) =
                        consecutive_texture_batch_end(commands, idx + 1, instance_index + 1);
                    idx = next_idx;
                    if mode != CommandListRenderMode::ExternalTexturesOnly
                        && let (Some(tex_buf), Some(tex_bg)) =
                            (&self.texture_instance_buffer, &self.texture_bind_group)
                    {
                        if active_pipe != ActivePipeline::Texture {
                            render_pass.set_pipeline(self.texture_pipeline.pipeline());
                            render_pass.set_bind_group(
                                0,
                                self.texture_pipeline.bind_group_globals(),
                                &[],
                            );
                            render_pass.set_bind_group(1, tex_bg, &[]);
                            render_pass.set_vertex_buffer(
                                0,
                                self.texture_pipeline.vertex_buffer().slice(..),
                            );
                            render_pass.set_vertex_buffer(1, tex_buf.slice(..));
                            render_pass.set_index_buffer(
                                self.texture_pipeline.index_buffer().slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            active_pipe = ActivePipeline::Texture;
                        }
                        render_pass.draw_indexed(0..6, 0, instance_index..end);
                    }
                }
                crate::command::DrawCommand::DrawExternalTexture { id, instance_index } => {
                    match mode {
                        CommandListRenderMode::Full
                        | CommandListRenderMode::ExternalTexturesOnly => {
                            if let Some(bind_group) = self.external_textures.get(id) {
                                self.external_pipeline.render_instance(
                                    render_pass,
                                    bind_group,
                                    instance_index,
                                );
                                active_pipe = ActivePipeline::ExternalTexture;
                            }
                        }
                        CommandListRenderMode::StaticWithCutout => {
                            if self.external_textures.get(id).is_some() {
                                self.external_pipeline
                                    .render_cutout_instance(render_pass, instance_index);
                                active_pipe = ActivePipeline::ExternalTexture;
                            }
                        }
                    }
                    idx += 1;
                }
            }
        }
    }

    /// Compiles draw commands when needed and uploads only dirty boundary slices (or all quads on
    /// `force_full_upload`), then clears `params.tree.dirty_boundaries()`.
    pub fn sync_and_prepare_tree(&mut self, params: IrisTreePrepareParams<'_>) {
        let _ = compile_tree_for_frame(
            params.tree,
            params.options,
            params.command_list,
            params.force_full_upload,
        );
        let dirty_boundaries = if params.force_full_upload {
            None
        } else {
            Some(params.tree.dirty_boundaries())
        };
        self.prepare_command_list_with_dirty_boundaries(
            params.device,
            params.queue,
            params.screen_size,
            params.command_list,
            dirty_boundaries,
        );
        params.tree.clear_dirty_boundaries();
    }
}

/// Parameters for [`IrisRenderer::sync_and_prepare_tree`].
pub struct IrisTreePrepareParams<'a> {
    /// Target WGPU device.
    pub device: &'a wgpu::Device,
    /// Target WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// Logical screen size `[width, height]`.
    pub screen_size: [f32; 2],
    /// Retained UI tree whose dirty boundaries and primitives are compiled and cleared.
    pub tree: &'a mut irisui_core::UiTree,
    /// Persistent draw command list updated in place when recompilation is needed.
    pub command_list: &'a mut crate::command::DrawCommandList,
    /// Tree compiler interaction/caret options.
    pub options: &'a mut crate::tree_compiler::TreeCompilerOptions<'a>,
    /// When `true`, forces full command recompilation and full GPU vertex upload.
    pub force_full_upload: bool,
}

/// Recompiles `tree` into `command_list` only when `force_recompile` is `true` or
/// `tree.has_dirty_boundaries()` holds. Returns `true` if `command_list` was recompiled.
pub fn compile_tree_for_frame(
    tree: &irisui_core::UiTree,
    options: &mut crate::tree_compiler::TreeCompilerOptions<'_>,
    command_list: &mut crate::command::DrawCommandList,
    force_recompile: bool,
) -> bool {
    if !force_recompile && !tree.has_dirty_boundaries() {
        return false;
    }
    let Some(root_id) = tree.root() else {
        command_list.clear();
        return false;
    };
    command_list.clear();
    crate::tree_compiler::compile_tree_draw_commands_into(tree, root_id, options, command_list);
    true
}

#[inline]
fn consecutive_texture_batch_end(
    commands: &[crate::command::DrawCommand],
    mut idx: usize,
    mut end: u32,
) -> (u32, usize) {
    while idx < commands.len() {
        if let crate::command::DrawCommand::DrawTexture {
            instance_index: next_index,
        } = commands[idx]
            && next_index == end
        {
            end += 1;
            idx += 1;
        } else {
            break;
        }
    }
    (end, idx)
}

#[cfg(test)]
mod tests {
    use super::{compile_tree_for_frame, consecutive_texture_batch_end};
    use crate::command::{DrawCommand, DrawCommandList};
    use crate::tree_compiler::TreeCompilerOptions;
    use irisui_core::{Color, DirtyFlags, Rect, Style, UiTree};

    #[test]
    fn test_consecutive_texture_batch_end_coalesces_contiguous_runs() {
        let cmds = [
            DrawCommand::DrawTexture { instance_index: 0 },
            DrawCommand::DrawTexture { instance_index: 1 },
            DrawCommand::DrawTexture { instance_index: 2 },
            DrawCommand::DrawTexture { instance_index: 4 },
            DrawCommand::ResetScissor,
        ];
        assert_eq!(consecutive_texture_batch_end(&cmds, 1, 1), (3, 3));
        assert_eq!(consecutive_texture_batch_end(&cmds, 4, 5), (5, 4));
    }

    #[test]
    fn test_compile_tree_for_frame_respects_dirty_boundaries_and_force() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 200.0, 100.0);
            node.style = Style::new().background(Color::RED);
        }
        tree.clear_dirty_boundaries();

        let mut cmd = DrawCommandList::new();
        let mut opts = TreeCompilerOptions::default();

        // Clean tree and no force -> skipped
        assert!(!compile_tree_for_frame(&tree, &mut opts, &mut cmd, false));
        assert!(cmd.quads.is_empty());

        // Forced recompile -> compiles
        assert!(compile_tree_for_frame(&tree, &mut opts, &mut cmd, true));
        assert!(!cmd.quads.is_empty());

        // Dirty boundary -> recompiles even when force == false
        tree.set_boundary(root, true);
        tree.mark_dirty(root, DirtyFlags::PAINT);
        assert!(compile_tree_for_frame(&tree, &mut opts, &mut cmd, false));
    }
}