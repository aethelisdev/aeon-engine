// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Static primitive vertex buffers and procedural 3D shape mesh generators.

/// Static primitive mesh vertex slices (`VERTICES`, `PYRAMID_VERTICES`, `CUBE_VERTICES`, `QUAD_VERTICES`, `GRID_QUAD_VERTICES`).
pub mod data;
/// Procedural parametric 3D mesh generators (sphere, cylinder, capsule, torus).
pub mod generators;

pub use data::{CUBE_VERTICES, GRID_QUAD_VERTICES, PYRAMID_VERTICES, QUAD_VERTICES, VERTICES};
pub use generators::{generate_capsule, generate_cylinder, generate_sphere, generate_torus};