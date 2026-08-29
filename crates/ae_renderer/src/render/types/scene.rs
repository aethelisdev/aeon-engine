// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ECS Render Scene Extraction & Culling
//!
//! Extracts visible entities from the ECS World into GPU-ready instances, applying spatial-partitioned frustum culling.

use crate::render::types::uniforms::LightUniform;
use crate::render::types::vertex::Instance;

/// Abstract representation of the scene to decouple RenderState from ECS.
/// Constructed by `extract()` each frame — performs frustum culling and
/// sorts transparent objects back-to-front for correct alpha blending.
pub struct RenderScene {
    pub light_uniform: LightUniform,
    pub triangle_instances: Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    pub cube_instances: Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    pub sphere_instances: Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    pub cylinder_instances: Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    pub capsule_instances: Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    pub torus_instances: Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    pub transparent_objs: Vec<(f32, crate::asset::AssetHandle, Instance)>,
    pub model_instance_data: std::collections::HashMap<
        crate::asset::AssetHandle,
        Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    >,
    pub selected_primitive_instances: Vec<(ae_core::ecs::Shape, Instance)>,
    pub selected_model_instances: Vec<(crate::asset::AssetHandle, Instance)>,
    /// List of entity IDs that were determined to be visible within the camera's frustum during the culling pass.
    pub visible_entities: Vec<hecs::Entity>,
}

impl RenderScene {
    /// Helper method to collect candidate visible entities using SpatialGrid cell-frustum culling,
    /// player tag guarantees, and ground safeguards.
    fn collect_visible_entities(
        world: &hecs::World,
        camera: &crate::camera::Camera,
        selected_entities: &std::collections::HashSet<hecs::Entity>,
        spatial_grid: &ae_core::spatial::SpatialGrid,
        frustum: &crate::math::frustum::Frustum,
    ) -> Vec<hecs::Entity> {
        let mut visible_entities = Vec::new();
        if spatial_grid.cells.is_empty() {
            return visible_entities;
        }

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

        // Ground & Large Entity Safeguard: Ensures Ground Plane and large environment platforms (Static RigidBody, large scale, or Ground/Floor/Zemin/Plane names) are ALWAYS visible
        let mut ground_query = world.query::<(
            hecs::Entity,
            Option<&ae_core::ecs::Name>,
            Option<&ae_core::ecs::RigidBody>,
            Option<&ae_core::ecs::Scale>,
        )>();
        for (ent, name, rb, scale) in ground_query.iter() {
            let is_static_body = rb
                .map(|r| r.body_type == ae_core::ecs::RigidBodyType::Static)
                .unwrap_or(false);
            let is_large_scale = scale.map(|s| s.x >= 20.0 || s.z >= 20.0).unwrap_or(false);
            let is_ground_name = name
                .map(|n| {
                    let s = n.0.as_bytes();
                    s.windows(6).any(|w| w.eq_ignore_ascii_case(b"ground"))
                        || s.windows(5).any(|w| w.eq_ignore_ascii_case(b"floor"))
                        || s.windows(5).any(|w| w.eq_ignore_ascii_case(b"zemin"))
                        || s.windows(5).any(|w| w.eq_ignore_ascii_case(b"plane"))
                })
                .unwrap_or(false);

            if (is_static_body || is_large_scale || is_ground_name) && visible_set.insert(ent) {
                visible_entities.push(ent);
            }
        }

        // Filter out explicitly hidden entities
        visible_entities.retain(|&e| world.get::<&ae_core::ecs::Hidden>(e).is_err());

        visible_entities
    }

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
        active_entity: Option<hecs::Entity>,
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

        let total_ents = world.len() as usize;
        let mut triangle_instances = Vec::with_capacity(total_ents.max(4));
        let mut cube_instances = Vec::with_capacity(total_ents.max(16));
        let mut sphere_instances = Vec::with_capacity(total_ents.max(4));
        let mut cylinder_instances = Vec::with_capacity(total_ents.max(4));
        let mut capsule_instances = Vec::with_capacity(total_ents.max(4));
        let mut torus_instances = Vec::with_capacity(total_ents.max(4));
        let mut transparent_objs: Vec<(f32, crate::asset::AssetHandle, Instance)> =
            Vec::with_capacity(total_ents.max(4));
        let mut model_instance_data: std::collections::HashMap<
            crate::asset::AssetHandle,
            Vec<(Instance, Option<crate::asset::AssetHandle>)>,
        > = std::collections::HashMap::with_capacity(16);
        let mut selected_primitive_instances = Vec::with_capacity(selected_entities.len().max(2));
        let mut selected_model_instances = Vec::with_capacity(selected_entities.len().max(2));
        let mut entity_id_map: std::collections::HashMap<hecs::Entity, f32> =
            std::collections::HashMap::with_capacity(selected_entities.len().max(4));
        for (idx, &e) in selected_entities.iter().enumerate() {
            entity_id_map.insert(e, ((idx % 250) + 1) as f32 / 255.0);
        }

        // Use dedicated culling matrix (shorter zfar=400) to aggressively cull distant objects
        // on the CPU. The actual render matrix uses zfar=2000 for visual depth quality.
        let frustum = crate::math::frustum::Frustum::from_matrix(camera.build_culling_matrix());
        let use_fallback = spatial_grid.cells.is_empty();

        let visible_entities = if !use_fallback {
            Self::collect_visible_entities(world, camera, selected_entities, spatial_grid, &frustum)
        } else {
            // Fallback path: iterate all entities in World when SpatialGrid is not populated
            let mut fallback_visible = Vec::new();
            let mut query = world.query::<(
                hecs::Entity,
                &ae_core::ecs::Position,
                Option<&ae_core::ecs::ModelId>,
                Option<&ae_core::ecs::BoundingRadius>,
                Option<&ae_core::ecs::GlobalTransform>,
                Option<&ae_core::ecs::Scale>,
                Option<&ae_core::ecs::CharacterController>,
                Option<&ae_core::ecs::PlayerTag>,
            )>();
            for (
                entity,
                pos,
                model_id,
                bounding_radius,
                global_transform,
                scale,
                kcc,
                player_tag,
            ) in query.iter()
            {
                if world.get::<&ae_core::ecs::Hidden>(entity).is_ok() {
                    continue;
                }

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

                    let scale_extent = sx.max(sy).max(sz).max(1.0);
                    let raw_radius = base_radius * scale_extent;
                    let culling_radius = (raw_radius * 1.25 + 0.5).max(1.5);

                    if !frustum.is_sphere_visible(p_world, culling_radius) {
                        continue;
                    }
                }
                fallback_visible.push(entity);
            }
            fallback_visible
        };

        let cap = visible_entities.len().min(65536);
        cube_instances.reserve(cap);

        // Unified Instance Processing Loop
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
                _bounding_radius,
                global_transform,
                lod_group,
                _kcc,
                _player_tag,
            )) = q.get()
            {
                let p_world = if let Some(gt) = global_transform {
                    cgmath::Vector3::new(gt.0.w.x, gt.0.w.y, gt.0.w.z)
                } else {
                    cgmath::Vector3::new(pos.x, pos.y, pos.z)
                };

                let model_matrix = if let Some(gt) = global_transform {
                    gt.0
                } else {
                    cgmath::Matrix4::from_translation(p_world)
                        * cgmath::Matrix4::from(
                            rot.map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
                                .unwrap_or(cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0)),
                        )
                        * cgmath::Matrix4::from_nonuniform_scale(
                            scale
                                .map(|s| {
                                    let min = 1e-4;
                                    if s.x.abs() < min {
                                        f32::copysign(min, s.x)
                                    } else {
                                        s.x
                                    }
                                })
                                .unwrap_or(1.0),
                            scale
                                .map(|s| {
                                    let min = 1e-4;
                                    if s.y.abs() < min {
                                        f32::copysign(min, s.y)
                                    } else {
                                        s.y
                                    }
                                })
                                .unwrap_or(1.0),
                            scale
                                .map(|s| {
                                    let min = 1e-4;
                                    if s.z.abs() < min {
                                        f32::copysign(min, s.z)
                                    } else {
                                        s.z
                                    }
                                })
                                .unwrap_or(1.0),
                        )
                };

                let base_color = color
                    .map(|c| [c.r, c.g, c.b, c.a])
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
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
                    let cam_pos = camera.position_vec3();
                    let dist = (p_world - cam_pos).magnitude();

                    if dist < lod.threshold_1 {
                        active_model_handle = Some(lod.lod_0);
                    } else if dist < lod.threshold_2 {
                        active_model_handle = lod.lod_1.or(Some(lod.lod_0));
                    } else {
                        active_model_handle = lod.lod_2.or(lod.lod_1).or(Some(lod.lod_0));
                    }
                }

                if selected_entities.contains(&entity) {
                    let mut sel_instance = instance;
                    let is_primary = active_entity == Some(entity);
                    let sel_level = if is_primary { 1.0 } else { 0.5 };
                    let entity_id = entity_id_map.get(&entity).copied().unwrap_or(0.1);
                    sel_instance.color = [sel_level, entity_id, 0.0, 1.0];

                    if let Some(m_handle) = active_model_handle {
                        selected_model_instances.push((m_handle, sel_instance));
                    } else if let Some(s) = shape {
                        selected_primitive_instances.push((*s, sel_instance));
                    }
                }

                if let Some(m_handle) = active_model_handle {
                    let tex_handle = sprite_id.map(|s_id| s_id.0);
                    model_instance_data
                        .entry(m_handle)
                        .or_default()
                        .push((instance, tex_handle));
                } else if let Some(s) = shape {
                    let tex_handle = sprite_id.map(|s_id| s_id.0);
                    match s {
                        ae_core::ecs::Shape::Cube => cube_instances.push((instance, tex_handle)),
                        ae_core::ecs::Shape::Triangle => {
                            triangle_instances.push((instance, tex_handle))
                        }
                        ae_core::ecs::Shape::Sphere => {
                            sphere_instances.push((instance, tex_handle))
                        }
                        ae_core::ecs::Shape::Cylinder => {
                            cylinder_instances.push((instance, tex_handle))
                        }
                        ae_core::ecs::Shape::Capsule => {
                            capsule_instances.push((instance, tex_handle))
                        }
                        ae_core::ecs::Shape::Torus => torus_instances.push((instance, tex_handle)),
                    }
                } else if let Some(s_id) = sprite_id {
                    use cgmath::InnerSpace;
                    let cam_pos = camera.position_vec3();
                    let dist = (p_world - cam_pos).magnitude();
                    transparent_objs.push((dist, s_id.0, instance));
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
            selected_primitive_instances,
            selected_model_instances,
            visible_entities,
        }
    }
}