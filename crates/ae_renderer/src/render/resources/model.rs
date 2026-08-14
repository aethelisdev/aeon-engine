// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::RenderState;
use crate::render::types::{ModelAsset, SkinVertex, Vertex};
use wgpu::util::DeviceExt;

impl RenderState {
    /// Uploads pre-parsed model data (vertices, indices) to GPU buffers and registers
    /// the asset in the manager with deduplication.
    pub fn upload_model_data(
        &self,
        assets: &mut crate::asset::AssetManager,
        data: crate::asset::ParsedModelData,
    ) -> (crate::asset::AssetHandle, [f32; 3], [f32; 3]) {
        // If it was loaded while we were parsing
        if let Some(&id) = assets.model_path_map.get(&data.canonical_path) {
            return (id, data.min, data.max);
        }

        let v_label = format!("{} Vertex Buffer", data.original_path);
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&v_label),
                contents: bytemuck::cast_slice(&data.all_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let i_label = format!("{} Index Buffer", data.original_path);
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&i_label),
                contents: bytemuck::cast_slice(&data.all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let default_texture = if let Some(cpu_tex) = data.embedded_texture {
            let tex_path = data.canonical_path.with_extension("embedded_tex");
            let tex_label = format!("{}_tex", data.original_path);
            Some(self.upload_cpu_texture_data(assets, tex_path, cpu_tex, &tex_label))
        } else {
            None
        };

        let source_path_str = data.canonical_path.to_string_lossy().to_string();
        let handle = assets.models.insert(ModelAsset {
            vertex_buffer,
            index_buffer,
            num_indices: data.all_indices.len() as u32,
            source_path: source_path_str,
            min: data.min,
            max: data.max,
            raw_vertices: data.raw_positions,
            raw_indices: data.all_indices,
            raw_skin_vertices: data.raw_skin_vertices,
            gpu_vertices: data.all_vertices,
            skeleton: data.skeleton,
            animations: data.animations,
            default_texture,
        });

        assets.model_path_map.insert(data.canonical_path, handle);

        (handle, data.min, data.max)
    }

    /// Synchronous model loader: parses GLTF file, extracts mesh data with AABB bounds,
    /// and uploads to GPU. Includes path deduplication.
    pub fn load_model(
        &self,
        assets: &mut crate::asset::AssetManager,
        path: &str,
    ) -> (crate::asset::AssetHandle, [f32; 3], [f32; 3]) {
        if !crate::asset::is_safe_path(path) {
            core::hint::cold_path();
            log::error!("[SECURITY ERROR] Blocked unsafe model load path: {}", path);
            return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
        }

        // --- DEDUPLICATION LOGIC ---
        let canonical_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                core::hint::cold_path();
                log::error!(
                    "[ERROR] Failed to canonicalize model path '{}': {}",
                    path,
                    e
                );
                return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
            }
        };

        if let Some(&id) = assets.model_path_map.get(&canonical_path) {
            log::info!(
                "Model already loaded, returning existing ID: {:?}",
                canonical_path
            );
            let (min, max) = assets
                .models
                .get(id)
                .map(|m| (m.min, m.max))
                .unwrap_or(([0.0; 3], [0.0; 3]));
            return (id, min, max);
        }

        let import_result = gltf::import(path);
        let (document, buffers, images) = match import_result {
            Ok(res) => res,
            Err(e) => {
                core::hint::cold_path();
                log::error!("Failed to load GLTF Model file {}: {:?}", path, e);
                return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
            }
        };

        let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);
        let embedded_texture = extract_gltf_embedded_texture(&document, &images);

        let (all_vertices, all_indices, raw_positions, raw_skin_vertices, min, max) =
            parse_gltf_scene_geometry(&document, &buffers, skeleton.is_some());

        let data = crate::asset::ParsedModelData {
            all_vertices,
            all_indices,
            raw_positions,
            raw_skin_vertices,
            min,
            max,
            canonical_path,
            original_path: path.to_owned(),
            final_name: String::new(),
            skeleton,
            animations,
            embedded_texture,
        };

        self.upload_model_data(assets, data)
    }
}

/// Thread-safe GLTF parser for async import pipeline. Extracts vertices, indices,
/// normals, colors, and computes AABB bounds with full scene graph node hierarchy transforms.
pub fn parse_gltf_file(
    path: &str,
    final_name: String,
) -> Result<crate::asset::ParsedModelData, String> {
    if !crate::asset::is_safe_path(path) {
        core::hint::cold_path();
        return Err(format!(
            "Security Error: Blocked unsafe GLTF path: {}",
            path
        ));
    }

    let canonical_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!(
                "Failed to canonicalize model path '{}': {}",
                path, e
            ));
        }
    };

    let import_result = gltf::import(path);
    let (document, buffers, images) = match import_result {
        Ok(res) => res,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!("Failed to load GLTF Model file {}: {:?}", path, e));
        }
    };

    let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);
    let embedded_texture = extract_gltf_embedded_texture(&document, &images);

    let (all_vertices, all_indices, raw_positions, raw_skin_vertices, min, max) =
        parse_gltf_scene_geometry(&document, &buffers, skeleton.is_some());

    Ok(crate::asset::ParsedModelData {
        all_vertices,
        all_indices,
        raw_positions,
        raw_skin_vertices,
        min,
        max,
        canonical_path,
        original_path: path.to_owned(),
        final_name,
        skeleton,
        animations,
        embedded_texture,
    })
}

/// Traverses GLTF scene node hierarchy and extracts all primitive geometry with world node transforms applied.
pub fn parse_gltf_scene_geometry(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    has_skeleton: bool,
) -> (
    Vec<Vertex>,
    Vec<u32>,
    Vec<[f32; 3]>,
    Vec<SkinVertex>,
    [f32; 3],
    [f32; 3],
) {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut raw_positions = Vec::new();
    let mut raw_skin_vertices = Vec::new();

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    let mut root_nodes: Vec<gltf::Node> = if let Some(scene) = document.default_scene() {
        scene.nodes().collect()
    } else {
        document.scenes().flat_map(|s| s.nodes()).collect()
    };

    if root_nodes.is_empty() {
        root_nodes = document.nodes().collect();
    }

    if !root_nodes.is_empty() {
        for root in &root_nodes {
            traverse_gltf_node(
                root,
                glam::Mat4::IDENTITY,
                buffers,
                &mut all_vertices,
                &mut all_indices,
                &mut raw_positions,
                &mut raw_skin_vertices,
                &mut min,
                &mut max,
                has_skeleton,
            );
        }
    } else {
        for mesh in document.meshes() {
            process_gltf_primitive_mesh(
                &mesh,
                glam::Mat4::IDENTITY,
                buffers,
                &mut all_vertices,
                &mut all_indices,
                &mut raw_positions,
                &mut raw_skin_vertices,
                &mut min,
                &mut max,
                has_skeleton,
            );
        }
    }

    if min[0] > max[0] {
        min = [-0.5; 3];
        max = [0.5; 3];
    }

    (
        all_vertices,
        all_indices,
        raw_positions,
        raw_skin_vertices,
        min,
        max,
    )
}

fn traverse_gltf_node(
    node: &gltf::Node,
    parent_transform: glam::Mat4,
    buffers: &[gltf::buffer::Data],
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u32>,
    raw_positions: &mut Vec<[f32; 3]>,
    raw_skin_vertices: &mut Vec<SkinVertex>,
    min: &mut [f32; 3],
    max: &mut [f32; 3],
    has_skeleton: bool,
) {
    let local_transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local_transform;

    if let Some(mesh) = node.mesh() {
        process_gltf_primitive_mesh(
            &mesh,
            world_transform,
            buffers,
            all_vertices,
            all_indices,
            raw_positions,
            raw_skin_vertices,
            min,
            max,
            has_skeleton,
        );
    }

    for child in node.children() {
        traverse_gltf_node(
            &child,
            world_transform,
            buffers,
            all_vertices,
            all_indices,
            raw_positions,
            raw_skin_vertices,
            min,
            max,
            has_skeleton,
        );
    }
}

fn process_gltf_primitive_mesh(
    mesh: &gltf::Mesh,
    world_transform: glam::Mat4,
    buffers: &[gltf::buffer::Data],
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u32>,
    raw_positions: &mut Vec<[f32; 3]>,
    raw_skin_vertices: &mut Vec<SkinVertex>,
    min: &mut [f32; 3],
    max: &mut [f32; 3],
    has_skeleton: bool,
) {
    let normal_matrix = glam::Mat3::from_mat4(world_transform);

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let mut pos_iter = match reader.read_positions() {
            Some(iter) => iter,
            None => continue,
        };
        let mut norm_iter = reader.read_normals();
        let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());
        let mut tex_coord_iter = reader.read_tex_coords(0).map(|tc| tc.into_f32());

        let mut joint_iter = reader.read_joints(0).map(|j| j.into_u16());
        let mut weight_iter = reader.read_weights(0).map(|w| w.into_f32());

        let is_skinned = joint_iter.is_some() && weight_iter.is_some() && has_skeleton;

        let start_vertex = all_vertices.len() as u32;

        while let Some(raw_pos) = pos_iter.next() {
            let raw_norm = norm_iter
                .as_mut()
                .and_then(|n| n.next())
                .unwrap_or([0.0, 1.0, 0.0]);
            let color = col_iter
                .as_mut()
                .and_then(|c| c.next())
                .unwrap_or([1.0, 1.0, 1.0]);
            let uv = tex_coord_iter
                .as_mut()
                .and_then(|tc| tc.next())
                .unwrap_or([0.0, 0.0]);

            let (final_pos, final_norm) = if is_skinned {
                (raw_pos, raw_norm)
            } else {
                let p4 = world_transform * glam::Vec4::new(raw_pos[0], raw_pos[1], raw_pos[2], 1.0);
                let n3 = (normal_matrix * glam::Vec3::new(raw_norm[0], raw_norm[1], raw_norm[2]))
                    .normalize_or_zero();
                ([p4.x, p4.y, p4.z], [n3.x, n3.y, n3.z])
            };

            let (j_indices, j_weights, has_skin) =
                if let (Some(j_it), Some(w_it)) = (joint_iter.as_mut(), weight_iter.as_mut()) {
                    let raw_j = j_it.next().unwrap_or([0, 0, 0, 0]);
                    let raw_w = w_it.next().unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    (
                        [
                            raw_j[0] as u32,
                            raw_j[1] as u32,
                            raw_j[2] as u32,
                            raw_j[3] as u32,
                        ],
                        raw_w,
                        true,
                    )
                } else {
                    ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0], false)
                };

            for i in 0..3 {
                if final_pos[i] < min[i] {
                    min[i] = final_pos[i];
                }
                if final_pos[i] > max[i] {
                    max[i] = final_pos[i];
                }
            }

            all_vertices.push(Vertex {
                position: final_pos,
                color,
                normal: final_norm,
                uv,
            });
            raw_positions.push(final_pos);

            if has_skin && has_skeleton {
                raw_skin_vertices.push(SkinVertex {
                    bind_position: final_pos,
                    bind_normal: final_norm,
                    joint_indices: j_indices,
                    joint_weights: j_weights,
                });
            }
        }

        if let Some(indices) = reader.read_indices() {
            for idx in indices.into_u32() {
                all_indices.push(start_vertex + idx);
            }
        }
    }
}

/// Extracts primary RGBA diffuse texture from glTF embedded images, prioritizing materials with base_color_texture.
pub fn extract_gltf_embedded_texture(
    document: &gltf::Document,
    images: &[gltf::image::Data],
) -> Option<ae_texture::CpuTextureData> {
    if images.is_empty() {
        return None;
    }

    let mut target_image_idx = None;
    for material in document.materials() {
        if let Some(pbr) = material.pbr_metallic_roughness().base_color_texture() {
            let src_idx = pbr.texture().source().index();
            if src_idx < images.len() {
                target_image_idx = Some(src_idx);
                break;
            }
        }
    }

    let img = if let Some(idx) = target_image_idx {
        &images[idx]
    } else {
        images.iter().max_by_key(|img| img.width * img.height)?
    };

    let rgba_bytes = match img.format {
        gltf::image::Format::R8G8B8A8 => img.pixels.clone(),
        gltf::image::Format::R8G8B8 => {
            let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
            for chunk in img.pixels.chunks_exact(3) {
                out.push(chunk[0]);
                out.push(chunk[1]);
                out.push(chunk[2]);
                out.push(255);
            }
            out
        }
        gltf::image::Format::R8 => {
            let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
            for &b in &img.pixels {
                out.push(b);
                out.push(b);
                out.push(b);
                out.push(255);
            }
            out
        }
        gltf::image::Format::R8G8 => {
            let mut out = Vec::with_capacity((img.width * img.height * 4) as usize);
            for chunk in img.pixels.chunks_exact(2) {
                out.push(chunk[0]);
                out.push(chunk[1]);
                out.push(0);
                out.push(255);
            }
            out
        }
        _ => img.pixels.clone(),
    };

    if rgba_bytes.len() == (img.width * img.height * 4) as usize && img.width > 0 && img.height > 0
    {
        Some(ae_texture::CpuTextureData::new(
            img.width,
            img.height,
            rgba_bytes,
            ae_texture::ColorSpace::Srgb,
            "embedded_model_texture",
        ))
    } else {
        None
    }
}

/// Helper to extract skeleton joints and animation clips from glTF document.
pub fn parse_gltf_skin_and_animations(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> (
    Option<ae_animation::Skeleton>,
    Vec<ae_animation::AnimationClip>,
) {
    let mut skeleton = None;
    let mut animations = Vec::new();
    let mut node_indices = std::collections::HashMap::new();

    // 1. Parse Skins
    if let Some(skin) = document.skins().next() {
        let reader = skin.reader(|b| Some(&buffers[b.index()]));
        let ibms: Vec<glam::Mat4> = reader
            .read_inverse_bind_matrices()
            .map(|iter| iter.map(|m| glam::Mat4::from_cols_array_2d(&m)).collect())
            .unwrap_or_default();

        let joint_nodes: Vec<gltf::Node> = skin.joints().collect();
        node_indices = joint_nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.index(), idx))
            .collect();

        // Build child_node_index -> parent_joint_index map
        let mut child_to_parent = std::collections::HashMap::new();
        for (parent_joint_idx, node) in joint_nodes.iter().enumerate() {
            for child in node.children() {
                child_to_parent.insert(child.index(), parent_joint_idx);
            }
        }

        let mut joints = Vec::with_capacity(joint_nodes.len());
        for (i, node) in joint_nodes.iter().enumerate() {
            let name = node.name().unwrap_or(&format!("Joint_{}", i)).to_string();
            let parent_index = child_to_parent.get(&node.index()).copied();
            let local_bind_pose = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
            let ibm = ibms.get(i).copied().unwrap_or(glam::Mat4::IDENTITY);

            joints.push(ae_animation::Joint::new(
                name,
                parent_index,
                local_bind_pose,
                ibm,
            ));
        }

        if !joints.is_empty() {
            skeleton = Some(ae_animation::Skeleton::from_joints(joints));
        }
    }

    // 2. Parse Animations
    for (anim_idx, anim) in document.animations().enumerate() {
        let anim_name = anim
            .name()
            .unwrap_or(&format!("Animation_{}", anim_idx))
            .to_string();
        let mut max_duration = 0.0f32;
        let mut channels = Vec::new();

        for channel in anim.channels() {
            let target_node = channel.target().node().index();
            let joint_index = *node_indices.get(&target_node).unwrap_or(&target_node);
            let reader = channel.reader(|b| Some(&buffers[b.index()]));
            let timestamps: Vec<f32> = match reader.read_inputs() {
                Some(iter) => iter.collect(),
                None => continue,
            };

            if let Some(&last_t) = timestamps.last() {
                if last_t > max_duration {
                    max_duration = last_t;
                }
            }

            let interp = match channel.sampler().interpolation() {
                gltf::animation::Interpolation::Step => ae_animation::Interpolation::Step,
                gltf::animation::Interpolation::Linear => ae_animation::Interpolation::Linear,
                gltf::animation::Interpolation::CubicSpline => {
                    ae_animation::Interpolation::CubicSpline
                }
            };

            let property = match channel.target().property() {
                gltf::animation::Property::Translation => ae_animation::TargetProperty::Translation,
                gltf::animation::Property::Rotation => ae_animation::TargetProperty::Rotation,
                gltf::animation::Property::Scale => ae_animation::TargetProperty::Scale,
                _ => continue,
            };

            if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(iter) => {
                        let kfs: Vec<_> = timestamps
                            .iter()
                            .zip(iter)
                            .map(|(&time, val)| ae_animation::Keyframe {
                                time,
                                value: glam::Vec3::from_array(val),
                            })
                            .collect();
                        channels.push(ae_animation::Channel {
                            joint_index,
                            target_property: property,
                            vector_track: Some(ae_animation::VectorTrack {
                                keyframes: kfs,
                                interpolation: interp,
                            }),
                            rotation_track: None,
                        });
                    }
                    gltf::animation::util::ReadOutputs::Rotations(iter) => {
                        let kfs: Vec<_> = timestamps
                            .iter()
                            .zip(iter.into_f32())
                            .map(|(&time, val)| ae_animation::Keyframe {
                                time,
                                value: glam::Quat::from_array(val),
                            })
                            .collect();
                        channels.push(ae_animation::Channel {
                            joint_index,
                            target_property: property,
                            vector_track: None,
                            rotation_track: Some(ae_animation::RotationTrack {
                                keyframes: kfs,
                                interpolation: interp,
                            }),
                        });
                    }
                    gltf::animation::util::ReadOutputs::Scales(iter) => {
                        let kfs: Vec<_> = timestamps
                            .iter()
                            .zip(iter)
                            .map(|(&time, val)| ae_animation::Keyframe {
                                time,
                                value: glam::Vec3::from_array(val),
                            })
                            .collect();
                        channels.push(ae_animation::Channel {
                            joint_index,
                            target_property: property,
                            vector_track: Some(ae_animation::VectorTrack {
                                keyframes: kfs,
                                interpolation: interp,
                            }),
                            rotation_track: None,
                        });
                    }
                    _ => {}
                }
            }
        }

        if !channels.is_empty() {
            let mut clip = ae_animation::AnimationClip::new(anim_name, max_duration);
            clip.channels = channels;
            animations.push(clip);
        }
    }

    (skeleton, animations)
}