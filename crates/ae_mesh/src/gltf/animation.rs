// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! glTF 2.0 skeletal armature, hierarchical node bind pose, and animation clip extraction.

/// Extracts an optional skeletal [`ae_animation::Skeleton`] and all embedded
/// [`ae_animation::AnimationClip`] channels from a parsed glTF document.
///
/// Supports both explicit `skin` armatures and non-skinned node hierarchy animations.
#[must_use]
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

        let child_to_parent = build_child_parent_map(&joint_nodes);

        let mut joints = Vec::with_capacity(joint_nodes.len());
        for (i, node) in joint_nodes.iter().enumerate() {
            let name = node
                .name()
                .map_or_else(|| format!("Joint_{i}"), str::to_owned);
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
        let all_nodes: Vec<gltf::Node> = document.nodes().collect();
        node_indices = all_nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.index(), idx))
            .collect();

        let child_to_parent = build_child_parent_map(&all_nodes);
        let count = all_nodes.len();
        let mut world_bind_matrices = vec![glam::Mat4::IDENTITY; count];
        let mut evaluated = vec![false; count];

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
            let name = node
                .name()
                .map_or_else(|| format!("Node_{i}"), str::to_owned);
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

    for (anim_idx, anim) in document.animations().enumerate() {
        let anim_name = anim
            .name()
            .map_or_else(|| format!("Animation_{anim_idx}"), str::to_owned);
        let mut max_duration = 0.0f32;
        let mut channels = Vec::new();

        for channel in anim.channels() {
            let target_node = channel.target().node().index();
            let Some(&joint_index) = node_indices.get(&target_node) else {
                continue;
            };
            let reader = channel.reader(|b| Some(&buffers[b.index()]));
            let Some(timestamps) = reader.read_inputs().map(Iterator::collect::<Vec<f32>>) else {
                continue;
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
                gltf::animation::Property::MorphTargetWeights => continue,
            };

            if let Some(outputs) = reader.read_outputs() {
                push_channel_outputs(
                    joint_index,
                    property,
                    interp,
                    &timestamps,
                    outputs,
                    &mut channels,
                );
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

fn build_child_parent_map(nodes: &[gltf::Node]) -> std::collections::HashMap<usize, usize> {
    let mut child_to_parent = std::collections::HashMap::new();
    for (parent_joint_idx, node) in nodes.iter().enumerate() {
        for child in node.children() {
            child_to_parent.insert(child.index(), parent_joint_idx);
        }
    }
    child_to_parent
}

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
    let world = if let Some(&parent_idx) = child_to_parent.get(&all_nodes[idx].index())
        && parent_idx < all_nodes.len()
        && parent_idx != idx
    {
        eval_world_bind(
            parent_idx,
            all_nodes,
            child_to_parent,
            world_matrices,
            evaluated,
        ) * local_pose
    } else {
        local_pose
    };
    world_matrices[idx] = world;
    evaluated[idx] = true;
    world
}

fn push_channel_outputs(
    joint_index: usize,
    target_property: ae_animation::TargetProperty,
    interp: ae_animation::Interpolation,
    timestamps: &[f32],
    outputs: gltf::animation::util::ReadOutputs<'_>,
    channels: &mut Vec<ae_animation::Channel>,
) {
    match outputs {
        gltf::animation::util::ReadOutputs::Translations(iter)
        | gltf::animation::util::ReadOutputs::Scales(iter) => {
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
                target_property,
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
                target_property,
                vector_track: None,
                rotation_track: Some(ae_animation::RotationTrack {
                    keyframes: kfs,
                    interpolation: interp,
                }),
            });
        }
        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {}
    }
}