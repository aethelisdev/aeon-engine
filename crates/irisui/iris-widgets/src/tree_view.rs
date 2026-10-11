// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical TreeView Widget Root Module
//!
//! Exposes hierarchical tree data structures, branch line math, node indentation,
//! and selection state management for scene hierarchies, folder trees, and AST inspectors.
//!

pub mod model;

pub use model::{
    DEFAULT_TREE_BASE_PADDING, DEFAULT_TREE_INDENT_STEP, DEFAULT_TREE_LINE_ARM_GAP,
    DEFAULT_TREE_LINE_ARM_LENGTH, DEFAULT_TREE_LINE_THICKNESS, DEFAULT_TREE_ROW_HEIGHT,
    TreeConnectorGeometry, TreeIndentConfig, TreeLineHorizontal, TreeLineVertical, TreeState,
    compute_tree_connector_geometry,
};