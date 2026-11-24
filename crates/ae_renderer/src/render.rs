// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rendering Engine Module — Manages WGPU device state, render passes, pipelines, materials, and post-processing.

pub mod engine;
pub mod primitives;
pub mod resources;
pub mod setup;
pub mod types;
pub mod viewport_texture;

pub use engine::*;
pub use types::*;
pub use viewport_texture::ViewportTexture;