// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified spatial, TRS matrix, hierarchy propagation, reparenting, and math interop crate.

pub mod conversions;
pub mod hierarchy;
pub mod matrix;
pub mod reparent;
pub mod spatial;

pub use cgmath;
pub use conversions::{ToCgmath, ToGlam, matrix4_to_glam_trs};
pub use glam;
pub use hierarchy::{integrate_unconstrained_velocities, update_hierarchy_transforms};
pub use matrix::{
    clamp_non_zero_scale_axis, compose_2d_model_matrix, compose_local_matrix, decompose_matrix,
    extract_world_position_and_scale, get_world_matrix, resolve_world_matrix,
    world_translation_for_entity,
};
pub use reparent::{parent_entity_keep_world_transform, unparent_entity_keep_world_transform};
pub use spatial::{AABB, SpatialGrid};