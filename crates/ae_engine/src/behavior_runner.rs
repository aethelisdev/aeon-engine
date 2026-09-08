// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic Gameplay Behavior Execution Pipeline.
//!
//! Coordinates modular, data-driven entity execution systems directly across archetype storage.
//!

pub mod collision_bridge;
pub mod combat;
pub mod destructible;
pub mod moving_platform;
pub mod native_behavior;
pub mod rotator;
pub mod runner;
pub mod trigger_zone;

#[cfg(test)]
mod tests;

pub use runner::*;