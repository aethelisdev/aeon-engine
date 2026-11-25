// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Self-contained debug wireframe renderer and overlay implementation.

use ae_gizmos::{ConvexHullWireframe, DebugLineVertex, DebugShapes, build_convex_hull_wireframe};
use std::collections::HashMap;

/// Input parameters for [`DebugRenderer::collect_lines`].
pub struct DebugCollectParams<'a> {
    /// WGPU logical device used if vertex buffer growth is required.
    pub device: &'a wgpu::Device,
    /// WGPU submission queue receiving uniform and vertex buffer writes.
    pub queue: &'a wgpu::Queue,
    /// Active ECS world containing entities with `Collider` or `CharacterController`.
    pub world: &'a ae_ecs::World,
    /// Renderer asset manager providing cached physics mesh geometry for `Trimesh`/`ConvexHull`.
    pub asset_manager: &'a ae_renderer::asset::AssetManager,
    /// Active camera view-projection matrix uploaded each frame.
    pub view_proj: cgmath::Matrix4<f32>,
    /// Currently selected editor entities whose colliders are always visualized in Edit mode.
    pub selected_entities: &'a [ae_ecs::Entity],
    /// When `true`, visualizes all scene colliders; when `false`, draws selected entities only.
    pub show_all_colliders: bool,
}

const DEBUG_LINE_ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x3
];

/// Returns the WGPU vertex buffer layout matching [`ae_gizmos::DebugLineVertex`].
#[must_use]
pub fn debug_line_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<DebugLineVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &DEBUG_LINE_ATTRIBS,
    }
}

/// Self-contained debug wireframe renderer.
///
/// Owns its own WGPU pipeline, vertex buffer (auto-growing), uniform buffer,
/// and bind group. Implements `OverlayRenderer` trait for compositing.
pub struct DebugRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_count: u32,
    /// Maximum vertex capacity (auto-grows when needed).
    buffer_capacity: usize,
    /// Reusable CPU staging buffer preventing per-frame allocations during rebuilds.
    vertices_scratch: Vec<DebugLineVertex>,
    /// Structural/transform signature of the last uploaded wireframe geometry.
    cached_signature: Option<u64>,
    /// Per-mesh cached 3D convex hull boundary wireframes computed once per [`AssetHandle`].
    convex_hull_cache: HashMap<ae_renderer::asset::AssetHandle, ConvexHullWireframe>,
}

impl DebugRenderer {
    /// Creates a new DebugRenderer with its own pipeline and GPU resources.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Uniform Buffer"),
            size: 64, // mat4x4
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug BGL"),
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
            label: Some("Debug Bind Group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/debug_line.wgsl").into()),
        });

        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Pipeline Layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Line Pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(debug_line_vertex_layout())],
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
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
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
        });

        let initial_capacity = 4096;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Vertex Buffer"),
            size: (std::mem::size_of::<DebugLineVertex>() * initial_capacity)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            vertex_count: 0,
            buffer_capacity: initial_capacity,
            vertices_scratch: Vec::with_capacity(initial_capacity),
            cached_signature: None,
            convex_hull_cache: HashMap::new(),
        }
    }

    /// Rebuilds the debug line render pipeline when MSAA sample count or surface format changes.
    pub fn rebuild_pipeline(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/debug_line.wgsl").into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug BGL Layout"),
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

        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Pipeline Layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        self.pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Line Pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(debug_line_vertex_layout())],
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
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
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
        });
    }

    /// Updates the camera VP uniform and lazily regenerates collider wireframes only when
    /// entity transforms, colliders, selection, or mesh asset availability change.
    pub fn collect_lines(&mut self, params: DebugCollectParams<'_>) {
        let vp: [[f32; 4]; 4] = params.view_proj.into();
        params
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&vp));

        let signature = compute_debug_signature(
            params.world,
            Some(params.asset_manager),
            params.selected_entities,
            params.show_all_colliders,
        );
        if self.cached_signature == Some(signature) {
            return;
        }
        self.cached_signature = Some(signature);

        populate_debug_vertices(
            params.world,
            Some(params.asset_manager),
            params.selected_entities,
            params.show_all_colliders,
            &mut self.convex_hull_cache,
            &mut self.vertices_scratch,
        );

        self.vertex_count = self.vertices_scratch.len() as u32;
        if self.vertices_scratch.is_empty() {
            return;
        }

        if self.vertices_scratch.len() > self.buffer_capacity {
            self.buffer_capacity = self.vertices_scratch.len().next_power_of_two();
            self.vertex_buffer = params.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Debug Vertex Buffer"),
                size: (std::mem::size_of::<DebugLineVertex>() * self.buffer_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        params.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.vertices_scratch),
        );
    }
}

type EntityDebugComponents<'a> = (
    Option<&'a ae_plugin_api::Collider>,
    &'a ae_plugin_api::Position,
    Option<&'a ae_plugin_api::Rotation>,
    Option<&'a ae_plugin_api::Scale>,
    Option<&'a ae_plugin_api::GlobalTransform>,
    Option<&'a ae_plugin_api::ModelId>,
    Option<&'a ae_plugin_api::CharacterController>,
    Option<&'a ae_plugin_api::Hidden>,
);

struct EntityDebugRow<'a> {
    entity: ae_ecs::Entity,
    collider: Option<&'a ae_plugin_api::Collider>,
    pos: &'a ae_plugin_api::Position,
    rot: Option<&'a ae_plugin_api::Rotation>,
    scale: Option<&'a ae_plugin_api::Scale>,
    global_transform: Option<&'a ae_plugin_api::GlobalTransform>,
    model_id: Option<&'a ae_plugin_api::ModelId>,
    kcc: Option<&'a ae_plugin_api::CharacterController>,
    hidden: Option<&'a ae_plugin_api::Hidden>,
}

impl<'a> EntityDebugRow<'a> {
    #[inline]
    fn from_tuple(entity: ae_ecs::Entity, tuple: EntityDebugComponents<'a>) -> Self {
        Self {
            entity,
            collider: tuple.0,
            pos: tuple.1,
            rot: tuple.2,
            scale: tuple.3,
            global_transform: tuple.4,
            model_id: tuple.5,
            kcc: tuple.6,
            hidden: tuple.7,
        }
    }

    #[inline]
    fn has_debug_shape(&self) -> bool {
        self.collider.is_some() || self.kcc.is_some()
    }

    #[inline]
    fn world_matrix(&self) -> cgmath::Matrix4<f32> {
        ae_transform::resolve_world_matrix(
            self.global_transform,
            Some(self.pos),
            self.rot,
            self.scale,
        )
    }
}

#[inline]
fn fnv_mix(state: u64, value: u64) -> u64 {
    state.wrapping_mul(0x1000_0000_01B3).wrapping_add(value) ^ (value >> 17)
}

#[inline]
fn fnv_mix_f32(state: u64, value: f32) -> u64 {
    fnv_mix(state, u64::from(value.to_bits()))
}

fn hash_entity_row(
    row: &EntityDebugRow<'_>,
    asset_manager: Option<&ae_renderer::asset::AssetManager>,
    is_selected: bool,
) -> Option<u64> {
    if !row.has_debug_shape() {
        return None;
    }
    if row.hidden.is_some() && !is_selected {
        return None;
    }

    let mut h = fnv_mix(0xCBF2_9CE4_8422_2325, row.entity.to_bits().get());
    let mat: [[f32; 4]; 4] = row.world_matrix().into();
    for col in mat {
        for val in col {
            h = fnv_mix_f32(h, val);
        }
    }

    if let Some(col) = row.collider {
        h = fnv_mix(h, 1);
        match col.shape {
            ae_plugin_api::ColliderShape::Box { half_extents } => {
                h = fnv_mix(h, 10);
                for v in half_extents {
                    h = fnv_mix_f32(h, v);
                }
            }
            ae_plugin_api::ColliderShape::Sphere { radius } => {
                h = fnv_mix(h, 20);
                h = fnv_mix_f32(h, radius);
            }
            ae_plugin_api::ColliderShape::Capsule {
                half_height,
                radius,
                center_y,
            } => {
                h = fnv_mix(h, 30);
                h = fnv_mix_f32(h, half_height);
                h = fnv_mix_f32(h, radius);
                h = fnv_mix_f32(h, row.kcc.map_or(center_y, |k| k.center_y));
            }
            ae_plugin_api::ColliderShape::Trimesh => {
                h = fnv_mix(h, 40);
                h = hash_mesh_binding(h, row.model_id, asset_manager);
            }
            ae_plugin_api::ColliderShape::ConvexHull => {
                h = fnv_mix(h, 50);
                h = hash_mesh_binding(h, row.model_id, asset_manager);
            }
        }
    } else if let Some(ctrl) = row.kcc {
        h = fnv_mix(h, 2);
        h = fnv_mix_f32(h, ctrl.capsule_half_height());
        h = fnv_mix_f32(h, ctrl.radius);
        h = fnv_mix_f32(h, ctrl.center_y);
    }

    Some(h)
}

fn hash_mesh_binding(
    mut state: u64,
    model_id: Option<&ae_plugin_api::ModelId>,
    asset_manager: Option<&ae_renderer::asset::AssetManager>,
) -> u64 {
    let Some(m_id) = model_id else {
        return fnv_mix(state, 0);
    };
    let mut hasher = FnvHasher(state);
    std::hash::Hash::hash(&m_id.0, &mut hasher);
    state = hasher.0;
    if let Some((verts, indices)) = asset_manager.and_then(|am| am.get_physics_mesh_data(m_id.0)) {
        state = fnv_mix(state, verts.len() as u64 + 1);
        state = fnv_mix(state, indices.len() as u64 + 1);
    }
    state
}

struct FnvHasher(u64);

impl std::hash::Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = fnv_mix(self.0, u64::from(byte));
        }
    }
}

fn compute_debug_signature(
    world: &ae_ecs::World,
    asset_manager: Option<&ae_renderer::asset::AssetManager>,
    selected_entities: &[ae_ecs::Entity],
    show_all_colliders: bool,
) -> u64 {
    let mut state = fnv_mix(0x517C_C1B7_2722_0A95, u64::from(show_all_colliders));
    if !show_all_colliders {
        for &entity in selected_entities {
            if let Ok(tuple) = world.query_one::<EntityDebugComponents<'_>>(entity).get() {
                let row = EntityDebugRow::from_tuple(entity, tuple);
                if let Some(h) = hash_entity_row(&row, asset_manager, true) {
                    state = fnv_mix(state, h);
                }
            }
        }
        return state;
    }

    let mut acc = 0_u64;
    let mut count = 0_u64;
    let mut query = world.query::<(ae_ecs::Entity, EntityDebugComponents<'_>)>();
    for (entity, tuple) in query.iter() {
        let row = EntityDebugRow::from_tuple(entity, tuple);
        let is_selected = selected_entities.contains(&entity);
        if let Some(h) = hash_entity_row(&row, asset_manager, is_selected) {
            acc ^= h;
            count += 1;
        }
    }
    fnv_mix(fnv_mix(state, acc), count)
}

fn populate_debug_vertices(
    world: &ae_ecs::World,
    asset_manager: Option<&ae_renderer::asset::AssetManager>,
    selected_entities: &[ae_ecs::Entity],
    show_all_colliders: bool,
    convex_hull_cache: &mut HashMap<ae_renderer::asset::AssetHandle, ConvexHullWireframe>,
    out_vertices: &mut Vec<DebugLineVertex>,
) {
    out_vertices.clear();
    if !show_all_colliders {
        for &entity in selected_entities {
            if let Ok(tuple) = world.query_one::<EntityDebugComponents<'_>>(entity).get() {
                let row = EntityDebugRow::from_tuple(entity, tuple);
                emit_entity_debug_lines(&row, asset_manager, true, convex_hull_cache, out_vertices);
            }
        }
        return;
    }

    let mut query = world.query::<(ae_ecs::Entity, EntityDebugComponents<'_>)>();
    for (entity, tuple) in query.iter() {
        let row = EntityDebugRow::from_tuple(entity, tuple);
        let is_selected = selected_entities.contains(&entity);
        emit_entity_debug_lines(
            &row,
            asset_manager,
            is_selected,
            convex_hull_cache,
            out_vertices,
        );
    }
}

fn emit_entity_debug_lines(
    row: &EntityDebugRow<'_>,
    asset_manager: Option<&ae_renderer::asset::AssetManager>,
    is_selected: bool,
    convex_hull_cache: &mut HashMap<ae_renderer::asset::AssetHandle, ConvexHullWireframe>,
    vertices: &mut Vec<DebugLineVertex>,
) {
    if !row.has_debug_shape() || (row.hidden.is_some() && !is_selected) {
        return;
    }
    let color = [0.0, 1.0, 0.4];
    let model = row.world_matrix();

    if let Some(col) = row.collider {
        match col.shape {
            ae_plugin_api::ColliderShape::Box { half_extents } => {
                DebugShapes::generate_box_lines(vertices, &model, half_extents, color);
            }
            ae_plugin_api::ColliderShape::Sphere { radius } => {
                DebugShapes::generate_sphere_lines(vertices, &model, radius, color);
            }
            ae_plugin_api::ColliderShape::Capsule {
                half_height,
                radius,
                center_y,
            } => {
                let offset_y = row.kcc.map_or(center_y, |ctrl| ctrl.center_y);
                let capsule_model = model
                    * cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, offset_y, 0.0));
                DebugShapes::generate_capsule_lines(
                    vertices,
                    &capsule_model,
                    half_height,
                    radius,
                    color,
                );
            }
            ae_plugin_api::ColliderShape::Trimesh => {
                if let Some((raw_vertices, raw_indices)) = row
                    .model_id
                    .and_then(|m| asset_manager.and_then(|am| am.get_physics_mesh_data(m.0)))
                {
                    DebugShapes::generate_mesh_lines(
                        vertices,
                        &model,
                        raw_vertices,
                        raw_indices,
                        color,
                    );
                } else {
                    DebugShapes::generate_box_lines(vertices, &model, [0.5, 0.5, 0.5], color);
                }
            }
            ae_plugin_api::ColliderShape::ConvexHull => {
                if let Some(m_id) = row.model_id
                    && let Some((raw_vertices, _)) =
                        asset_manager.and_then(|am| am.get_physics_mesh_data(m_id.0))
                {
                    let hull = convex_hull_cache
                        .entry(m_id.0)
                        .or_insert_with(|| build_convex_hull_wireframe(raw_vertices));
                    if hull.edges.is_empty() {
                        DebugShapes::generate_box_lines(vertices, &model, [0.5, 0.5, 0.5], color);
                    } else {
                        DebugShapes::generate_convex_hull_lines(vertices, &model, hull, color);
                    }
                } else {
                    DebugShapes::generate_box_lines(vertices, &model, [0.5, 0.5, 0.5], color);
                }
            }
        }
    } else if let Some(ctrl) = row.kcc {
        let capsule_model = model
            * cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, ctrl.center_y, 0.0));
        DebugShapes::generate_capsule_lines(
            vertices,
            &capsule_model,
            ctrl.capsule_half_height(),
            ctrl.radius,
            color,
        );
    }
}

/// Implements `OverlayRenderer` so `RenderState` can draw debug wireframes
/// without knowing the concrete `DebugRenderer` type.
impl ae_renderer::render::OverlayRenderer for DebugRenderer {
    fn draw_overlay<'a>(&'a self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_only_vs_show_all_and_hidden_filtering() {
        let mut world = ae_ecs::World::new();
        let selected = world.spawn((
            ae_plugin_api::Position::default(),
            ae_plugin_api::Collider::default(),
        ));
        let _unselected = world.spawn((
            ae_plugin_api::Position::new(5.0, 0.0, 0.0),
            ae_plugin_api::Collider::default(),
        ));
        let hidden = world.spawn((
            ae_plugin_api::Position::new(10.0, 0.0, 0.0),
            ae_plugin_api::Collider::default(),
            ae_plugin_api::Hidden,
        ));

        let mut cache = HashMap::new();
        let mut verts = Vec::new();

        // No selection + show_all=false must emit zero vertices
        populate_debug_vertices(&world, None, &[], false, &mut cache, &mut verts);
        assert!(verts.is_empty());

        // Default mode draws strictly selected entities (1 box = 24 vertices)
        populate_debug_vertices(&world, None, &[selected], false, &mut cache, &mut verts);
        assert_eq!(verts.len(), 24);

        // show_all=true draws visible colliders (selected + unselected = 48), skips hidden unselected
        populate_debug_vertices(&world, None, &[selected], true, &mut cache, &mut verts);
        assert_eq!(verts.len(), 48);

        // Selecting a Hidden entity still visualizes its collider wireframe
        populate_debug_vertices(&world, None, &[hidden], false, &mut cache, &mut verts);
        assert_eq!(verts.len(), 24);
    }

    #[test]
    fn test_signature_tracks_selection_toggle_and_transform_changes() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((
            ae_plugin_api::Position::default(),
            ae_plugin_api::Collider::default(),
        ));

        let sig_a = compute_debug_signature(&world, None, &[entity], false);
        let sig_b = compute_debug_signature(&world, None, &[entity], false);
        assert_eq!(sig_a, sig_b);

        let sig_all = compute_debug_signature(&world, None, &[entity], true);
        assert_ne!(sig_a, sig_all);

        let sig_empty = compute_debug_signature(&world, None, &[], false);
        assert_ne!(sig_a, sig_empty);

        let mut pos = world
            .get_copied::<ae_plugin_api::Position>(entity)
            .expect("position");
        pos.x = 3.5;
        world.insert_one(entity, pos).expect("insert pos");
        let sig_moved = compute_debug_signature(&world, None, &[entity], false);
        assert_ne!(sig_a, sig_moved);
    }
}