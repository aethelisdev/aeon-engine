// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI WGPU Backend (`iris-wgpu`)
//!
//! GPU SDF rendering pipeline for Iris UI with sub-pixel antialiasing,
//! rounded rectangles, inner/outer borders, and gaussian drop shadows.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod command;
pub mod external_texture_pipeline;
pub mod external_textures;
pub mod quad;
pub mod renderer;
pub mod texture_pipeline;

pub use command::{DrawCommand, DrawCommandList};
pub use external_texture_pipeline::{ExternalTexturePipeline, ExternalTextureQuadInstance};
pub use external_textures::ExternalTextures;
pub use quad::QuadInstance;
pub use renderer::IrisRenderer;
pub use texture_pipeline::{TextureQuadInstance, TextureQuadPipeline};

#[cfg(test)]
mod tests;