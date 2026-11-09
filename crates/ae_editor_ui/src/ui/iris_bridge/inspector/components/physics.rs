// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Component Inspector Cards
//!
//! Provides handlers for Collider, RigidBody, and Physics Material cards.

pub mod collider;
pub mod material;
pub mod rigidbody;

pub use collider::{ColliderHandler, draw_collider_card};
pub use material::{PhysicsMaterialHandler, draw_physics_material_card};
pub use rigidbody::{RigidBodyHandler, draw_rigidbody_card};