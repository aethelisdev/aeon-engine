// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 3D Model Loading and GPU Asset Management Sub-module.
//!
//! Handles glTF/GLB import, scene graph traversal, skeletal armatures,
//! animations, and GPU buffer uploads.
//!

pub mod animation;
pub mod geometry;
pub mod loader;
pub mod textures;

pub use animation::*;
pub use geometry::*;
pub use loader::*;
pub use textures::*;