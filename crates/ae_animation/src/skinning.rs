// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Vertex skinning matrix palette computations and SSBO Storage Buffer alignment for Aeon Engine.
//!

use crate::skeleton::Skeleton;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use serde::{Deserialize, Serialize};

/// Standard bone capacity presets for hardware and performance targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoneCapacityPreset {
    /// Mobile Mesh Section limit (75 bones).
    MobileMeshSection = 75,
    /// Standard Desktop UBO limit (256 bones).
    StandardDesktop = 256,
    /// Unlimited Dynamic SSBO Storage Buffer limit (65,536 bones).
    UnlimitedSsbo = 65536,
}

/// Quad-Word (16-byte aligned) GPU joint matrix structure for SSBO Storage Buffers.
/// Wraps a $4 \times 4$ column-major float matrix in `#[repr(C)]` layout for direct zero-copy
/// `bytemuck::cast_slice` upload to `var<storage, read> joint_matrices: array<mat4x4<f32>>;`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct JointMatrix {
    /// 4x4 matrix stored as 4 column vectors of 4 floats each.
    pub cols: [[f32; 4]; 4],
}

impl Default for JointMatrix {
    fn default() -> Self {
        Self::from_mat4(Mat4::IDENTITY)
    }
}

impl JointMatrix {
    /// Creates a GPU joint matrix from a `glam::Mat4`.
    #[inline]
    #[must_use]
    pub fn from_mat4(mat: Mat4) -> Self {
        Self {
            cols: mat.to_cols_array_2d(),
        }
    }

    /// Converts back to a `glam::Mat4`.
    #[inline]
    #[must_use]
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.cols)
    }
}

/// A dynamic collection of final joint skinning matrices ready for GPU Storage Buffer upload.
/// Computes `skinning_matrix = global_transform * inverse_bind_matrix` for every joint in the skeleton.
#[derive(Debug, Clone, Default)]
pub struct SkinningPalette {
    /// Dynamic slice of 16-byte aligned GPU joint matrices.
    pub matrices: Vec<JointMatrix>,
}

impl SkinningPalette {
    /// Creates a new empty skinning palette.
    #[must_use]
    pub fn new() -> Self {
        Self {
            matrices: Vec::new(),
        }
    }

    /// Creates a skinning palette pre-allocated for a specific bone capacity preset.
    #[must_use]
    pub fn with_capacity_preset(preset: BoneCapacityPreset) -> Self {
        Self {
            matrices: Vec::with_capacity(preset as usize),
        }
    }

    /// Returns the raw byte slice for direct zero-copy WGPU Storage Buffer upload via `queue.write_buffer`.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.matrices)
    }

    /// Returns the number of joint matrices in the palette.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.matrices.len()
    }

    /// Returns `true` if the palette contains no matrices.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.matrices.is_empty()
    }
}

/// Computes final vertex skinning matrices for a skeleton given evaluated global joint transforms.
/// For each joint $i$: $\text{SkinningMatrix}_i = \text{GlobalTransform}_i \times \text{InverseBindMatrix}_i$
/// # Arguments
/// * `skeleton` - Reference to the target skeleton.
/// * `global_transforms` - Evaluated global world matrices for each joint.
/// # Returns
/// Computed `SkinningPalette` ready for SSBO Storage Buffer upload.
#[must_use]
pub fn compute_skinning_matrices(
    skeleton: &Skeleton,
    global_transforms: &[Mat4],
) -> SkinningPalette {
    let count = skeleton.joints.len().min(global_transforms.len());
    let mut matrices = Vec::with_capacity(count);

    for i in 0..count {
        let joint = &skeleton.joints[i];
        let global = global_transforms[i];
        let skin_mat = global * joint.inverse_bind_matrix;
        matrices.push(JointMatrix::from_mat4(skin_mat));
    }

    SkinningPalette { matrices }
}