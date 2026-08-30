// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Component Inspector Cards
//!
//! Provides handlers for `🛡️ Collider`, `⚙ RigidBody`, and `🧱 Physics Material`
//! along with common UI widget and numeric card rendering helpers.

pub mod collider;
pub mod helpers;
pub mod material;
pub mod rigidbody;

pub use collider::ColliderHandler;
pub use helpers::*;
pub use material::PhysicsMaterialHandler;
pub use rigidbody::RigidBodyHandler;