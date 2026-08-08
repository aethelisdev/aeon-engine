// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Resources Sub-module — Manages GPU textures, render targets, and 3D GLTF model imports.
//!

pub mod model;
pub mod targets;
pub mod texture;

pub use model::*;
pub use targets::*;
pub use texture::*;