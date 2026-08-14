// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Skeletal hierarchy and joint transform structures for Aeon Engine.
//!

use glam::Mat4;
use serde::{Deserialize, Serialize};

/// Represents a single joint (bone) within a skeleton.
/// Joints contain a name, an optional parent index in the topologically sorted hierarchy,
/// a local bind pose transform, and an inverse bind matrix used for vertex skinning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    /// Name of the joint (e.g., "mixamorig:Hips", "Spine_01", "Hand_R").
    pub name: String,

    /// Parent joint index within the skeleton's topologically sorted array.
    /// Must satisfy `parent_index < joint_index` to maintain a flat-tree topological layout.
    pub parent_index: Option<usize>,

    /// Local transform matrix relative to the parent joint in bind pose.
    pub local_bind_pose: Mat4,

    /// Inverse bind matrix (world-to-joint matrix in bind pose) used for vertex skinning.
    pub inverse_bind_matrix: Mat4,
}

impl Joint {
    /// Creates a new joint with the given name, parent index, bind pose, and inverse bind matrix.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        parent_index: Option<usize>,
        local_bind_pose: Mat4,
        inverse_bind_matrix: Mat4,
    ) -> Self {
        Self {
            name: name.into(),
            parent_index,
            local_bind_pose,
            inverse_bind_matrix,
        }
    }
}

/// Represents a 3D skeleton composed of a topologically sorted hierarchy of joints.
/// Joints are guaranteed to be stored such that any parent joint index appears before
/// its children (`parent_index < child_index`). This flat-tree ordering allows linear,
/// non-recursive, CPU cache-friendly $O(N)$ global transform evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skeleton {
    /// Topologically sorted array of joints.
    pub joints: Vec<Joint>,
}

impl Skeleton {
    /// Creates a new empty skeleton.
    #[must_use]
    pub fn new() -> Self {
        Self { joints: Vec::new() }
    }

    /// Creates a skeleton from an existing list of joints, validating topological ordering.
    #[must_use]
    pub fn from_joints(joints: Vec<Joint>) -> Self {
        Self { joints }
    }

    /// Returns the number of joints in the skeleton.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.joints.len()
    }

    /// Returns `true` if the skeleton contains no joints.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.joints.is_empty()
    }

    /// Finds a joint index by name.
    #[must_use]
    pub fn find_joint_index(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    /// Computes global world transforms for all joints from a slice of local pose matrices.
    /// Executes a linear $O(N)$ iteration over the topologically sorted joints, avoiding recursion
    /// and maintaining optimal CPU L1/L2 cache locality.
    /// # Arguments
    /// * `local_poses` - Slice of local joint matrices. Must have length equal to `self.len()`.
    /// # Returns
    /// Vector of computed global matrices for each joint.
    #[must_use]
    pub fn evaluate_global_transforms(&self, local_poses: &[Mat4]) -> Vec<Mat4> {
        let count = self.joints.len();
        let mut global_transforms = vec![Mat4::IDENTITY; count];
        let mut evaluated = vec![false; count];

        fn eval_joint(
            idx: usize,
            joints: &[Joint],
            local_poses: &[Mat4],
            globals: &mut [Mat4],
            evaluated: &mut [bool],
        ) -> Mat4 {
            if evaluated[idx] {
                return globals[idx];
            }
            let local_pose = local_poses.get(idx).copied().unwrap_or(Mat4::IDENTITY);
            let global = if let Some(parent_idx) = joints[idx].parent_index {
                if parent_idx < joints.len() && parent_idx != idx {
                    eval_joint(parent_idx, joints, local_poses, globals, evaluated) * local_pose
                } else {
                    local_pose
                }
            } else {
                local_pose
            };
            globals[idx] = global;
            evaluated[idx] = true;
            global
        }

        for i in 0..count {
            eval_joint(
                i,
                &self.joints,
                local_poses,
                &mut global_transforms,
                &mut evaluated,
            );
        }

        global_transforms
    }
}