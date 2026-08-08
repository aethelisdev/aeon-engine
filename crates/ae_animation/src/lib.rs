// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Skeletal Animation & Keyframe Subsystem (`ae_animation`)
//!
//! Provides skeletal hierarchy evaluation, keyframe track sampling, quaternion Slerp interpolation,
//! crossfade clip blending, and 16-byte aligned SSBO Storage Buffer vertex skinning matrix palette computations.
//!

pub mod clip;
pub mod player;
pub mod skeleton;
pub mod skinning;

#[cfg(test)]
mod tests;

pub use clip::{
    AnimationClip, Channel, Interpolation, Keyframe, RotationTrack, TargetProperty, VectorTrack,
};
pub use player::{AnimationPlayer, AnimationState};
pub use skeleton::{Joint, Skeleton};
pub use skinning::{BoneCapacityPreset, JointMatrix, SkinningPalette, compute_skinning_matrices};