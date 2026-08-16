// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GLTF skeletal armature, node hierarchy, and animation clip extraction.

/// Helper to extract skeleton joints and animation clips from glTF document.
/// Supports both explicit skin armatures and hierarchical scene node animations.
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

    // 1. Parse Skins (Skeletal Armatures)
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
    } else if document.animations().next().is_some() {
        // 1b. Node Hierarchy Animation (Scene nodes animated without skin armature)
        let all_nodes: Vec<gltf::Node> = document.nodes().collect();
        node_indices = all_nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.index(), idx))
            .collect();

        // Build child_node_index -> parent_joint_index map
        let mut child_to_parent = std::collections::HashMap::new();
        for (parent_joint_idx, node) in all_nodes.iter().enumerate() {
            for child in node.children() {
                child_to_parent.insert(child.index(), parent_joint_idx);
            }
        }

        // Calculate world bind matrices for each node to derive IBMs
        let count = all_nodes.len();
        let mut world_bind_matrices = vec![glam::Mat4::IDENTITY; count];
        let mut evaluated = vec![false; count];

        fn eval_world_bind(
            idx: usize,
            all_nodes: &[gltf::Node],
            child_to_parent: &std::collections::HashMap<usize, usize>,
            world_matrices: &mut [glam::Mat4],
            evaluated: &mut [bool],
        ) -> glam::Mat4 {
            if evaluated[idx] {
                return world_matrices[idx];
            }
            let local_pose = glam::Mat4::from_cols_array_2d(&all_nodes[idx].transform().matrix());
            let world = if let Some(&parent_idx) = child_to_parent.get(&all_nodes[idx].index()) {
                if parent_idx < all_nodes.len() && parent_idx != idx {
                    eval_world_bind(
                        parent_idx,
                        all_nodes,
                        child_to_parent,
                        world_matrices,
                        evaluated,
                    ) * local_pose
                } else {
                    local_pose
                }
            } else {
                local_pose
            };
            world_matrices[idx] = world;
            evaluated[idx] = true;
            world
        }

        for i in 0..count {
            eval_world_bind(
                i,
                &all_nodes,
                &child_to_parent,
                &mut world_bind_matrices,
                &mut evaluated,
            );
        }

        let mut joints = Vec::with_capacity(all_nodes.len());
        for (i, node) in all_nodes.iter().enumerate() {
            let name = node.name().unwrap_or(&format!("Node_{}", i)).to_string();
            let parent_index = child_to_parent.get(&node.index()).copied();
            let local_bind_pose = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
            let ibm = world_bind_matrices[i].inverse();

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
            let joint_index = match node_indices.get(&target_node) {
                Some(&idx) => idx,
                None => continue,
            };
            let reader = channel.reader(|b| Some(&buffers[b.index()]));
            let timestamps: Vec<f32> = match reader.read_inputs() {
                Some(iter) => iter.collect(),
                None => continue,
            };

            if let Some(&last_t) = timestamps.last()
                && last_t > max_duration
            {
                max_duration = last_t;
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