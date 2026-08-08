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

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&(data.original_path.clone() + " Vertex Buffer")),
                contents: bytemuck::cast_slice(&data.all_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&(data.original_path.clone() + " Index Buffer")),
                contents: bytemuck::cast_slice(&data.all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

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
            skeleton: data.skeleton.clone(),
            animations: data.animations.clone(),
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
        let (document, buffers, _) = match import_result {
            Ok(res) => res,
            Err(e) => {
                core::hint::cold_path();
                log::error!("Failed to load GLTF Model file {}: {:?}", path, e);
                return (crate::asset::AssetHandle::default(), [0.0; 3], [0.0; 3]);
            }
        };

        let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut raw_positions = Vec::new();
        let mut raw_skin_vertices = Vec::new();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for mesh in document.meshes() {
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

                let start_vertex = all_vertices.len() as u32;

                while let Some(pos) = pos_iter.next() {
                    let normal = norm_iter
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

                    let (j_indices, j_weights, has_skin) = if let (Some(j_it), Some(w_it)) =
                        (joint_iter.as_mut(), weight_iter.as_mut())
                    {
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
                        if pos[i] < min[i] {
                            min[i] = pos[i];
                        }
                        if pos[i] > max[i] {
                            max[i] = pos[i];
                        }
                    }

                    all_vertices.push(Vertex {
                        position: pos,
                        color,
                        normal,
                        uv,
                    });
                    raw_positions.push(pos);

                    if has_skin && skeleton.is_some() {
                        raw_skin_vertices.push(SkinVertex {
                            bind_position: pos,
                            bind_normal: normal,
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

        let data = crate::asset::ParsedModelData {
            all_vertices,
            all_indices: all_indices.clone(),
            raw_positions,
            raw_skin_vertices,
            min,
            max,
            canonical_path,
            original_path: path.to_owned(),
            final_name: String::new(),
            skeleton,
            animations,
        };

        self.upload_model_data(assets, data)
    }
}

/// Thread-safe GLTF parser for async import pipeline. Extracts vertices, indices,
/// normals, colors, and computes AABB bounds without GPU access.
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
    let (document, buffers, _) = match import_result {
        Ok(res) => res,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!("Failed to load GLTF Model file {}: {:?}", path, e));
        }
    };

    let (skeleton, animations) = parse_gltf_skin_and_animations(&document, &buffers);

    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut raw_positions = Vec::new();
    let mut raw_skin_vertices = Vec::new();

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    for mesh in document.meshes() {
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

            let start_vertex = all_vertices.len() as u32;

            while let Some(pos) = pos_iter.next() {
                let normal = norm_iter
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
                    if pos[i] < min[i] {
                        min[i] = pos[i];
                    }
                    if pos[i] > max[i] {
                        max[i] = pos[i];
                    }
                }

                all_vertices.push(Vertex {
                    position: pos,
                    color,
                    normal,
                    uv,
                });
                raw_positions.push(pos);

                if has_skin && skeleton.is_some() {
                    raw_skin_vertices.push(SkinVertex {
                        bind_position: pos,
                        bind_normal: normal,
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
    })
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

        let mut joints = Vec::with_capacity(joint_nodes.len());
        for (i, node) in joint_nodes.iter().enumerate() {
            let name = node.name().unwrap_or(&format!("Joint_{}", i)).to_string();
            let parent_index = node
                .children()
                .find_map(|child| node_indices.get(&child.index()).copied());
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