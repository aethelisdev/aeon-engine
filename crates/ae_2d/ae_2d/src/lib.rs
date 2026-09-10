// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon 2D (`ae_2d`)
//!
//! High-performance, modular 2D engine subsystem for Aeon Engine.
//! Provides instanced sprite rendering, 2D rigid body physics, frame-based flipbook animations,
//! and orthographic camera controls.
//!
//! ## Sub-Ecosystem Architecture
//! - [`core`]: 2D camera, transform, sorting keys, and dimension mode exclusion.
//! - [`renderer`]: WGPU-accelerated instanced sprite pipeline and dynamic batcher.
//! - [`physics`]: Rapier2D rigid body simulation, collision geometry, and raycasting.
//! - [`animation`]: Sprite sheet and flipbook sequence controllers.
//! - [`prelude`]: Commonly used 2D components, systems, and builders.
//!

pub use ae_2d_animation as animation;
pub use ae_2d_core as core;
pub use ae_2d_core::{camera, components, mode};
pub use ae_2d_physics as physics;
pub use ae_2d_render as renderer;

#[cfg(test)]
mod tests;

// Re-export common symbols for  backwards compatibility
pub use ae_2d_animation::SpriteAnimation;
pub use ae_2d_core::{ActiveDimensionMode, Camera2D, SortMode, SpriteRenderer, SpriteSortKey};
pub use ae_2d_physics::{BodyType2D, Collider2D, ColliderShape2D, Physics2DWorld, RigidBody2D};
pub use ae_2d_render::{
    Sprite2DPipeline, SpriteBatch, SpriteBatcher, SpriteInstance, SpriteVertex,
};

/// Convenient common imports for 2D development with Aeon Engine.
pub mod prelude {
    pub use ae_2d_animation::SpriteAnimation;
    pub use ae_2d_core::{ActiveDimensionMode, Camera2D, SortMode, SpriteRenderer, SpriteSortKey};
    pub use ae_2d_physics::{BodyType2D, Collider2D, ColliderShape2D, Physics2DWorld, RigidBody2D};
    pub use ae_2d_render::{
        Sprite2DPipeline, SpriteBatch, SpriteBatcher, SpriteInstance, SpriteVertex,
    };
}