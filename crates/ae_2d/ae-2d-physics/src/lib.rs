// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae-2d-physics` - High-Performance 2D Physics Simulation Subsystem for Aeon Engine.
//!
//! Provides Rapier2D-powered rigid body dynamics, collision detection, and spatial queries.
//! - [`Physics2DWorld`]: Central physics orchestration container managing pipelines, islands, and solvers.
//! - [`RigidBody2D`] & [`BodyType2D`]: Dynamic, kinematic, and static body representation.
//! - [`Collider2D`] & [`ColliderShape2D`]: Box, circle, and capsule collision primitives.
//!

pub mod body;
pub mod collider;
pub mod world;

#[cfg(test)]
mod tests;

pub use body::{BodyType2D, RigidBody2D};
pub use collider::{Collider2D, ColliderShape2D};
pub use world::Physics2DWorld;