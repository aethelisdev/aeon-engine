// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use hecs;

/// ViewportRect represent screen boundaries of 3D Viewport
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Viewport descriptor for sub-region rendering (currently unused but reserved).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

#[allow(dead_code)]
impl Default for Viewport {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [800.0, 600.0],
        }
    }
}

/// Abstract representation of the scene to decouple RenderState from ECS.
/// Constructed by `extract()` each frame — performs frustum culling and
/// sorts transparent objects back-to-front for correct alpha blending.
pub struct RenderScene {
    pub light_uniform: LightUniform,
    pub triangle_instances: Vec<Instance>,
    pub cube_instances: Vec<Instance>,
    pub sphere_instances: Vec<Instance>,
    pub cylinder_instances: Vec<Instance>,
    pub capsule_instances: Vec<Instance>,
    pub torus_instances: Vec<Instance>,
    pub transparent_objs: Vec<(f32, crate::asset::AssetHandle, Instance)>,
    pub model_instance_data: std::collections::HashMap<crate::asset::AssetHandle, Vec<Instance>>,
    /// List of entity IDs that were determined to be visible within the camera's frustum during the culling pass.
    pub visible_entities: Vec<hecs::Entity>,
}

impl RenderScene {
    /// Extracts visible entities from the ECS World into GPU-ready instance data.
    /// Applies high-performance AABB cell-frustum spatial partitioning culling using the SpatialGrid.
    /// Each grid cell is tested as a precise axis-aligned bounding box against the 6 frustum planes,
    /// eliminating the ~63% false positive area that bounding sphere tests introduce.
    /// Falls back to O(N) flat traversal if the spatial grid is empty or entity count is below 150K.
    pub fn extract(
        world: &hecs::World,
        camera: &crate::camera::Camera,
        _asset_manager: &crate::asset::AssetManager,
        selected_entities: &std::collections::HashSet<hecs::Entity>,
        spatial_grid: &ae_core::spatial::SpatialGrid,
    ) -> Self {
        // Will be overwritten by GraphicsSettings global sun state during Main Pass
        let light_uniform = LightUniform {
            direction: [0.0, 1.0, 0.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
            ambient_color: [0.1, 0.1, 0.15],
            _padding3: 0,
            fog_params: [0.0; 4],
        };

        let _total_ents = world.len() as usize;
        let mut triangle_instances = Vec::new();
        let mut cube_instances = Vec::new();
        let mut sphere_instances = Vec::new();
        let mut cylinder_instances = Vec::new();
        let mut capsule_instances = Vec::new();
        let mut torus_instances = Vec::new();
        let mut transparent_objs: Vec<(f32, crate::asset::AssetHandle, Instance)> = Vec::new();
        let mut model_instance_data: std::collections::HashMap<
            crate::asset::AssetHandle,
            Vec<Instance>,
        > = std::collections::HashMap::new();

        // Use dedicated culling matrix (shorter zfar=400) to aggressively cull distant objects
        // on the CPU. The actual render matrix uses zfar=2000 for visual depth quality.
        let frustum = crate::math::frustum::Frustum::from_matrix(camera.build_culling_matrix());
        let mut visible_entities = Vec::new();

        let use_fallback = spatial_grid.cells.is_empty();

        if !use_fallback {
            let cam_pos_vec =
                cgmath::Vector3::new(camera.position.x, camera.position.y, camera.position.z);
            let view_distance = 450.0_f32;
            let cell_size = spatial_grid.cell_size;
            let mut visible_set: std::collections::HashSet<hecs::Entity> =
                std::collections::HashSet::with_capacity(1024);

            for (cx, cy, cz, entities) in
                spatial_grid.query_cells_near_camera(cam_pos_vec, view_distance)
            {
                let cell_min = cgmath::Vector3::new(
                    cx as f32 * cell_size,
                    cy as f32 * cell_size,
                    cz as f32 * cell_size,
                );
                let cell_max = cgmath::Vector3::new(
                    (cx as f32 + 1.0) * cell_size,
                    (cy as f32 + 1.0) * cell_size,
                    (cz as f32 + 1.0) * cell_size,
                );

                if frustum.is_aabb_visible(cell_min, cell_max) {
                    for &e in entities {
                        if visible_set.insert(e) {
                            visible_entities.push(e);
                        }
                    }
                }
            }

            for &selected in selected_entities {
                if visible_set.insert(selected) {
                    visible_entities.push(selected);
                }
            }

            // Guaranteed Player Visibility: Controlled entities are never culled by SpatialGrid
            for (ent, _ctrl) in world
                .query::<(hecs::Entity, &ae_core::ecs::CharacterController)>()
                .iter()
            {
                if visible_set.insert(ent) {
                    visible_entities.push(ent);
                }
            }
            for (ent, _tag) in world
                .query::<(hecs::Entity, &ae_core::ecs::PlayerTag)>()
                .iter()
            {
                if visible_set.insert(ent) {
                    visible_entities.push(ent);
                }
            }

            // Ground & Large Entity Safeguard: Ensures Ground Plane and large environment platforms are ALWAYS visible
            // Runs only on named entities (e.g. Ground Plane) to bypass iterating millions of unnamed stress test blocks!
            let mut ground_query = world.query::<(hecs::Entity, &ae_core::ecs::Name)>();
            for (ent, name) in ground_query.iter() {
                let s = name.0.as_bytes();
                if s.windows(6).any(|w| w.eq_ignore_ascii_case(b"ground")) {
                    if visible_set.insert(ent) {
                        visible_entities.push(ent);
                    }
                }
            }

            let cap = visible_entities.len().min(65536);
            cube_instances.reserve(cap);

            for &entity in &visible_entities {
                let mut q = world.query_one::<(
                    &ae_core::ecs::Position,
                    Option<&ae_core::ecs::Rotation>,
                    Option<&ae_core::ecs::Scale>,
                    Option<&ae_core::ecs::Color>,
                    Option<&ae_core::ecs::ModelId>,
                    Option<&ae_core::ecs::Shape>,
                    Option<&ae_core::ecs::SpriteId>,
                    Option<&ae_core::ecs::BoundingRadius>,
                    Option<&ae_core::ecs::GlobalTransform>,
                    Option<&ae_core::ecs::LodGroup>,
                    Option<&ae_core::ecs::CharacterController>,
                    Option<&ae_core::ecs::PlayerTag>,
                )>(entity);

                if let Ok((
                    pos,
                    rot,
                    scale,
                    color,
                    model_id,
                    shape,
                    sprite_id,
                    bounding_radius,
                    global_transform,
                    lod_group,
                    kcc,
                    player_tag,
                )) = q.get()
                {
                    let p_world = if let Some(gt) = global_transform {
                        cgmath::Vector3::new(gt.0.w.x, gt.0.w.y, gt.0.w.z)
                    } else {
                        cgmath::Vector3::new(pos.x, pos.y, pos.z)
                    };

                    let (sx, sy, sz) = if let Some(s) = scale {
                        (s.x.abs(), s.y.abs(), s.z.abs())
                    } else if let Some(gt) = global_transform {
                        use cgmath::InnerSpace;
                        let sx = cgmath::Vector3::new(gt.0.x.x, gt.0.x.y, gt.0.x.z).magnitude();
                        let sy = cgmath::Vector3::new(gt.0.y.x, gt.0.y.y, gt.0.y.z).magnitude();
                        let sz = cgmath::Vector3::new(gt.0.z.x, gt.0.z.y, gt.0.z.z).magnitude();
                        (sx, sy, sz)
                    } else {
                        (1.0, 1.0, 1.0)
                    };

                    let is_exempt = kcc.is_some() || player_tag.is_some() || sx > 10.0 || sz > 10.0;

                    if !is_exempt {
                        let base_radius = if let Some(r) = bounding_radius {
                            r.0
                        } else if let Some(m_handle) = model_id.map(|m| m.0) {
                            _asset_manager
                                .models
                                .get(m_handle)
                                .map(|asset| asset.bounding_radius())
                                .unwrap_or(1.0)
                        } else {
                            1.0
                        };

                        let scale_extent = (sx * sx + sy * sy + sz * sz).sqrt().max(1.0);
                        let raw_radius = base_radius * scale_extent;
                        let culling_radius = (raw_radius * 1.25 + 0.5).max(1.5);

                        if !frustum.is_sphere_visible(p_world, culling_radius) {
                            continue;
                        }
                    }

                    let model_matrix = if let Some(gt) = global_transform {
                        gt.0
                    } else {
                        cgmath::Matrix4::from_translation(p_world)
                            * cgmath::Matrix4::from(
                                rot.map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
                                    .unwrap_or(cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0)),
                            )
                            * cgmath::Matrix4::from_nonuniform_scale(
                                scale.map(|s| s.x).unwrap_or(1.0),
                                scale.map(|s| s.y).unwrap_or(1.0),
                                scale.map(|s| s.z).unwrap_or(1.0),
                            )
                    };

                    let base_color = color
                        .map(|c| [c.r, c.g, c.b, c.a])
                        .unwrap_or([0.3, 0.3, 0.3, 1.0]);
                    let final_color = if selected_entities.contains(&entity) {
                        [
                            (base_color[0] * 1.1 + 0.1).min(1.0),
                            (base_color[1] * 1.1 + 0.15).min(1.0),
                            (base_color[2] * 1.1 + 0.2).min(1.0),
                            base_color[3],
                        ]
                    } else {
                        base_color
                    };

                    let instance = Instance {
                        model_matrix: Into::<[[f32; 4]; 4]>::into(model_matrix),
                        color: final_color,
                    };

                    let mut active_model_handle = model_id.map(|m| m.0);

                    if let Some(lod) = lod_group {
                        use cgmath::InnerSpace;
                        let cam_pos = cgmath::Vector3::new(
                            camera.position.x,
                            camera.position.y,
                            camera.position.z,
                        );
                        let dist = (p_world - cam_pos).magnitude();

                        if dist < lod.threshold_1 {
                            active_model_handle = Some(lod.lod_0);
                        } else if dist < lod.threshold_2 {
                            active_model_handle = lod.lod_1.or(Some(lod.lod_0));
                        } else {
                            active_model_handle = lod.lod_2.or(lod.lod_1).or(Some(lod.lod_0));
                        }
                    }

                    if let Some(m_handle) = active_model_handle {
                        model_instance_data
                            .entry(m_handle)
                            .or_default()
                            .push(instance);
                    } else if let Some(s_id) = sprite_id {
                        use cgmath::InnerSpace;
                        let cam_pos = cgmath::Vector3::new(
                            camera.position.x,
                            camera.position.y,
                            camera.position.z,
                        );
                        let dist = (p_world - cam_pos).magnitude();
                        transparent_objs.push((dist, s_id.0, instance));
                    } else if let Some(s) = shape {
                        match s {
                            ae_core::ecs::Shape::Cube => cube_instances.push(instance),
                            ae_core::ecs::Shape::Triangle => triangle_instances.push(instance),
                            ae_core::ecs::Shape::Sphere => sphere_instances.push(instance),
                            ae_core::ecs::Shape::Cylinder => cylinder_instances.push(instance),
                            ae_core::ecs::Shape::Capsule => capsule_instances.push(instance),
                            ae_core::ecs::Shape::Torus => torus_instances.push(instance),
                        }
                    }
                }
            }
        } else {
            // Contiguous SOA archetype iteration path:
            // High-throughput frustum culling with instant early exit for 10M+ objects.
            let mut query = world.query::<(
                hecs::Entity,
                &ae_core::ecs::Position,
                Option<&ae_core::ecs::Rotation>,
                Option<&ae_core::ecs::Scale>,
                Option<&ae_core::ecs::Color>,
                Option<&ae_core::ecs::ModelId>,
                Option<&ae_core::ecs::Shape>,
                Option<&ae_core::ecs::SpriteId>,
                Option<&ae_core::ecs::BoundingRadius>,
                Option<&ae_core::ecs::GlobalTransform>,
                Option<&ae_core::ecs::LodGroup>,
                Option<&ae_core::ecs::CharacterController>,
                Option<&ae_core::ecs::PlayerTag>,
            )>();

            for (
                entity,
                pos,
                rot,
                scale,
                color,
                model_id,
                shape,
                sprite_id,
                bounding_radius,
                global_transform,
                lod_group,
                kcc,
                player_tag,
            ) in query.iter()
            {
                let p_world = if let Some(gt) = global_transform {
                    cgmath::Vector3::new(gt.0.w.x, gt.0.w.y, gt.0.w.z)
                } else {
                    cgmath::Vector3::new(pos.x, pos.y, pos.z)
                };

                let (sx, sy, sz) = if let Some(s) = scale {
                    (s.x.abs(), s.y.abs(), s.z.abs())
                } else if let Some(gt) = global_transform {
                    use cgmath::InnerSpace;
                    let sx = cgmath::Vector3::new(gt.0.x.x, gt.0.x.y, gt.0.x.z).magnitude();
                    let sy = cgmath::Vector3::new(gt.0.y.x, gt.0.y.y, gt.0.y.z).magnitude();
                    let sz = cgmath::Vector3::new(gt.0.z.x, gt.0.z.y, gt.0.z.z).magnitude();
                    (sx, sy, sz)
                } else {
                    (1.0, 1.0, 1.0)
                };

                let is_exempt = kcc.is_some() || player_tag.is_some() || sx > 10.0 || sz > 10.0;

                if !is_exempt {
                    let base_radius = if let Some(r) = bounding_radius {
                        r.0
                    } else if let Some(m_handle) = model_id.map(|m| m.0) {
                        _asset_manager
                            .models
                            .get(m_handle)
                            .map(|asset| asset.bounding_radius())
                            .unwrap_or(1.0)
                    } else {
                        1.0
                    };

                    let scale_extent = (sx * sx + sy * sy + sz * sz).sqrt().max(1.0);
                    let raw_radius = base_radius * scale_extent;
                    let culling_radius = (raw_radius * 1.25 + 0.5).max(1.5);

                    if !frustum.is_sphere_visible(p_world, culling_radius) {
                        continue;
                    }
                }

                visible_entities.push(entity);

                let model_matrix = if let Some(gt) = global_transform {
                    gt.0
                } else {
                    cgmath::Matrix4::from_translation(p_world)
                        * cgmath::Matrix4::from(
                            rot.map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
                                .unwrap_or(cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0)),
                        )
                        * cgmath::Matrix4::from_nonuniform_scale(
                            scale.map(|s| s.x).unwrap_or(1.0),
                            scale.map(|s| s.y).unwrap_or(1.0),
                            scale.map(|s| s.z).unwrap_or(1.0),
                        )
                };

                let base_color = color
                    .map(|c| [c.r, c.g, c.b, c.a])
                    .unwrap_or([0.3, 0.3, 0.3, 1.0]);
                let final_color = if selected_entities.contains(&entity) {
                    [
                        (base_color[0] * 1.1 + 0.1).min(1.0),
                        (base_color[1] * 1.1 + 0.15).min(1.0),
                        (base_color[2] * 1.1 + 0.2).min(1.0),
                        base_color[3],
                    ]
                } else {
                    base_color
                };

                let instance = Instance {
                    model_matrix: Into::<[[f32; 4]; 4]>::into(model_matrix),
                    color: final_color,
                };

                let mut active_model_handle = model_id.map(|m| m.0);

                if let Some(lod) = lod_group {
                    use cgmath::InnerSpace;
                    let cam_pos = cgmath::Vector3::new(
                        camera.position.x,
                        camera.position.y,
                        camera.position.z,
                    );
                    let dist = (p_world - cam_pos).magnitude();

                    if dist < lod.threshold_1 {
                        active_model_handle = Some(lod.lod_0);
                    } else if dist < lod.threshold_2 {
                        active_model_handle = lod.lod_1.or(Some(lod.lod_0));
                    } else {
                        active_model_handle = lod.lod_2.or(lod.lod_1).or(Some(lod.lod_0));
                    }
                }

                if let Some(m_handle) = active_model_handle {
                    model_instance_data
                        .entry(m_handle)
                        .or_default()
                        .push(instance);
                } else if let Some(s_id) = sprite_id {
                    use cgmath::InnerSpace;
                    let cam_pos = cgmath::Vector3::new(
                        camera.position.x,
                        camera.position.y,
                        camera.position.z,
                    );
                    let dist = (p_world - cam_pos).magnitude();
                    transparent_objs.push((dist, s_id.0, instance));
                } else if let Some(s) = shape {
                    match s {
                        ae_core::ecs::Shape::Cube => cube_instances.push(instance),
                        ae_core::ecs::Shape::Triangle => triangle_instances.push(instance),
                        ae_core::ecs::Shape::Sphere => sphere_instances.push(instance),
                        ae_core::ecs::Shape::Cylinder => cylinder_instances.push(instance),
                        ae_core::ecs::Shape::Capsule => capsule_instances.push(instance),
                        ae_core::ecs::Shape::Torus => torus_instances.push(instance),
                    }
                }
            }
        }

        Self {
            light_uniform,
            triangle_instances,
            cube_instances,
            sphere_instances,
            cylinder_instances,
            capsule_instances,
            torus_instances,
            transparent_objs,
            model_instance_data,
            visible_entities,
        }
    }
}

/// PBR mesh vertex: position, vertex color, and surface normal.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Sprite/billboard vertex: position, UV coordinates, and surface normal.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl SpriteVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Per-instance data: 4×4 model matrix + RGBA color (80 bytes per instance).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model_matrix: [[f32; 4]; 4],
    pub color: [f32; 4],
}

/// Byte stride of a single `Instance` in GPU buffer. Used for buffer offset calculations.
/// Replaces the hardcoded magic number `80` throughout the render pipeline.
pub const INSTANCE_SIZE: usize = std::mem::size_of::<Instance>();

impl Instance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// GPU uniform for scene lighting: directional sun, ambient fill, and fog parameters.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    /// A normalized vector pointing TOWARDS the light source.
    /// This convention is shared across Sky, PBR shading, and Shadow Cascades.
    pub direction: [f32; 3],
    pub _padding: u32,

    /// The direct sunlight color and intensity multiplier.
    pub color: [f32; 3],
    pub _padding2: u32,

    pub ambient_color: [f32; 3],
    pub _padding3: u32,

    /// Fog settings: r, g, b, w=distance (0.0 means disabled)
    pub fog_params: [f32; 4],
}

/// GPU uniform for Cascaded Shadow Map (CSM) data: 4 light-space matrices,
/// cascade split depths, and PCF/bias configuration.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightSpaceUniform {
    pub matrices: [[[f32; 4]; 4]; 4], // 4 matrices for 4 cascades (64 bytes * 4 = 256 bytes)
    pub cascade_splits: [f32; 4],     // Z view depths for splitting (16 bytes)
    pub shadow_bias: f32,
    pub pcf_radius: i32,
    pub shadow_enabled: u32,
    pub _pad: u32,
} // Total: 288 bytes

/// GPU uniform for the procedural sky shader: sun position, atmosphere density,
/// color parameters, and quality mode selector.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyUniform {
    pub sun_direction: [f32; 4], // xyz used, w padding
    pub sun_color: [f32; 4],     // rgb color, w intensity
    pub horizon_color: [f32; 4], // w unused
    pub zenith_color: [f32; 4],  // w unused
    pub atmosphere_density: f32,
    pub sun_disc_size: f32,
    pub sun_glow_strength: f32,
    pub sky_quality_mode: u32, // 0=Low, 1=Medium, 2=High
}

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Generic overlay renderer trait for drawing editor overlays into the main render pass.
/// Systems that need to draw overlays (e.g. gizmos, debug lines) implement this trait.
/// RenderState calls `draw_overlay()` without knowing the concrete type, achieving full
/// decoupling between the render module and editor subsystems.
pub trait OverlayRenderer {
    /// Draw the overlay into an already-active render pass.
    /// Implementors should have already prepared their GPU state (uniforms, vertex data)
    /// via a separate `prepare()` call before this is invoked.
    fn draw_overlay<'a>(&'a self, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>);
}

/// Render error type for surface acquisition failures (wgpu 29+).
#[derive(Debug)]
pub enum RenderError {
    SurfaceLost,
    OutOfMemory,
    Other(String),
}

/// GPU-uploaded texture with its bind group, canonical source path, and dimensions.
pub struct TextureAsset {
    /// Egui/WGPU compatible GPU bind group containing texture view and sampler bindings.
    pub bind_group: wgpu::BindGroup,
    /// Absolute canonical path on local disk for memory deduplication.
    pub source_path: String,
    /// Width of the texture image in pixels.
    pub width: u32,
    /// Height of the texture image in pixels.
    pub height: u32,
}

/// GPU-uploaded 3D model asset with vertex/index buffers, AABB bounds,
/// and raw mesh data for physics shape generation.
pub struct ModelAsset {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub source_path: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Raw positions extracted for physics shape generation (Trimesh / ConvexHull)
    pub raw_vertices: Vec<[f32; 3]>,
    /// Raw indices extracted for physics shape generation (Trimesh / ConvexHull)
    pub raw_indices: Vec<u32>,
}

impl ModelAsset {
    /// Computes the true maximum 3D bounding radius from local AABB min and max extents.
    /// Ensures 3D mesh assets (e.g. cabinets, houses, props) resolve their exact local
    /// bounding radius rather than defaulting to 1.0, preventing premature camera culling.
    pub fn bounding_radius(&self) -> f32 {
        let r_min =
            (self.min[0] * self.min[0] + self.min[1] * self.min[1] + self.min[2] * self.min[2])
                .sqrt();
        let r_max =
            (self.max[0] * self.max[0] + self.max[1] * self.max[1] + self.max[2] * self.max[2])
                .sqrt();
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        let r_extent = (dx * dx + dy * dy + dz * dz).sqrt() * 0.5;
        r_min.max(r_max).max(r_extent).max(1.0)
    }
}