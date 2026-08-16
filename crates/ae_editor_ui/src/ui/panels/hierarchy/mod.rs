// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene Hierarchy / Outliner UI Panel Module.
//!
//! Provides the data-driven scene entity tree, search filtering, transform hierarchy visualization,
//! and object spawning / stress testing controls.
//!

pub mod tree;

pub use tree::{HierarchyCache, HierarchyRow};