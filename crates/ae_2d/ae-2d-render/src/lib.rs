// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae-2d-render` - Hardware-Accelerated WGPU 2D Sprite Batching and Rendering Subsystem.
//!
//! Provides high-throughput instanced quad rendering for 2D sprites, tiles, and visual entities.
//! - [`SpriteBatcher`]: Manages dynamic vertex/instance buffers, sorting submissions, and batch splitting by texture.
//! - [`Sprite2DPipeline`]: Compiles and manages the WGSL 2D instancing shader, depth-stencil blend states, and uniform layouts.
//! - [`SpriteInstance`] & [`SpriteVertex`]: Memory-compact, GPU-aligned pod structs for zero-overhead quad submission.
//!

pub mod batcher;
pub mod instance;
pub mod pipeline;

#[cfg(test)]
mod tests;

pub use batcher::{SpriteBatch, SpriteBatcher};
pub use instance::{SpriteInstance, SpriteVertex};
pub use pipeline::Sprite2DPipeline;