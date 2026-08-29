// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Core Render Data Types & Subsystems
//!
//! Re-exports vertex layouts, GPU uniforms, textures, 3D models, scene extraction, and viewport metrics.

pub mod assets;
pub mod scene;
pub mod uniforms;
pub mod vertex;
pub mod viewport;

//  re-exports across engine and renderer
pub use assets::{ModelAsset, ModelSubmesh, SubmeshAlphaMode, TextureAsset};
pub use scene::RenderScene;
pub use uniforms::{DEPTH_FORMAT, LightSpaceUniform, LightUniform, SkyUniform};
pub use vertex::{INSTANCE_SIZE, Instance, SkinVertex, SpriteVertex, Vertex};
pub use viewport::{FrameRenderStats, OverlayRenderer, RenderError, Viewport, ViewportRect};