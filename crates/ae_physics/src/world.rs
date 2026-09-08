// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! AE Physics — Decoupled physics simulation world wrapper using Rapier3D.
//!

pub mod character;
pub mod core;
pub mod sync_ecs;
pub mod sync_physics;
#[cfg(test)]
pub mod tests;

pub use core::*;