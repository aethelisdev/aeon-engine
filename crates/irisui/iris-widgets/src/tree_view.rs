// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical TreeView Widget Root Module
//!
//! Exposes hierarchical tree data structures, branch line math, node indentation,
//! and selection state management for scene hierarchies, folder trees, and AST inspectors.
//!

pub mod model;
pub mod navigation;
pub mod selection;

pub use model::{
    DEFAULT_TREE_BASE_PADDING, DEFAULT_TREE_INDENT_STEP, DEFAULT_TREE_LINE_ARM_GAP,
    DEFAULT_TREE_LINE_ARM_LENGTH, DEFAULT_TREE_LINE_THICKNESS, DEFAULT_TREE_ROW_HEIGHT,
    TreeConnectorGeometry, TreeIndentConfig, TreeLineHorizontal, TreeLineVertical, TreeState,
    compute_tree_connector_geometry,
};

pub use navigation::{
    TreeItemInfo, TreeNavResponse, compute_tree_scroll_to_view, handle_tree_arrow_navigation,
    handle_tree_navigation_event,
};

pub use selection::{
    TREE_CHEVRON_ARROW_COLLAPSED, TREE_CHEVRON_ARROW_EXPANDED, TREE_CHEVRON_COLLAPSED,
    TREE_CHEVRON_EXPANDED, TreeClickTarget, TreeRowStyle, TreeSelectionModifiers,
    TreeSelectionResponse, evaluate_tree_click, handle_tree_foldout_click, handle_tree_row_click,
    handle_tree_selection_event, tree_chevron_arrow_glyph, tree_chevron_glyph,
};