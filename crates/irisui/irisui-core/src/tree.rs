// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generational arena-based UI tree manager guaranteeing 100% safe memory operations.

pub mod arena;
pub mod dirty;
pub mod focus_sync;
pub mod hit_test;
pub mod live_text;
pub mod query;
pub mod retained_tree;
pub mod scroll_sync;
pub mod state;

#[cfg(test)]
mod tests;

pub use arena::{RetainedRevisionStatus, UiTree};
pub use hit_test::HitTargetInfo;
pub use retained_tree::{RetainedTreeRow, RetainedTreeState};