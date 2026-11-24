// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Core Render Data Types & Subsystems
//!
//! Re-exports textures, 3D models, scene extraction, and viewport metrics.

pub mod assets;
pub mod scene;
pub mod viewport;

pub use assets::{ModelAsset, TextureAsset};
pub use scene::RenderScene;
pub use viewport::{FrameRenderStats, OverlayRenderer, RenderError};