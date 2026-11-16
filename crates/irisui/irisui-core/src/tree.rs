// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generational arena-based UI tree manager guaranteeing 100% safe memory operations.

pub mod arena;
pub mod dirty;
pub mod hit_test;
pub mod query;
pub mod state;

#[cfg(test)]
mod tests;

pub use arena::UiTree;
pub use hit_test::HitTargetInfo;