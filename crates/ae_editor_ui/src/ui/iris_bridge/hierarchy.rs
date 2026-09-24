// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy (Outliner) Iris UI Module
//!
//! Orchestrates the 100% Iris UI GPU-accelerated Scene Hierarchy tree,
//! search bar filtering, cascading `➕` entity spawning menus, and right-click context actions.
//!

pub mod add_menu;
pub mod context_menu;
pub mod footer;
pub mod header;
pub mod panel;
pub mod rows;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use panel::{build_hierarchy_overlays, build_hierarchy_panel};
pub use types::{
    AddSubmenuId, HIERARCHY_CTX_DELETE, HIERARCHY_CTX_VISIBILITY, HIERARCHY_TAG_ADD_BUTTON,
    HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_EYE_PREFIX, HIERARCHY_TAG_FOLDOUT_PREFIX,
    HIERARCHY_TAG_INDEX_MASK, HIERARCHY_TAG_PANEL_ROOT, HIERARCHY_TAG_ROW_PREFIX,
    HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT, HierarchyAction, HierarchyPanelParams,
    HierarchyPanelState, HierarchyRow, is_hierarchy_tag, make_eye_tag, make_foldout_tag,
    make_row_tag, parse_eye_tag, parse_foldout_tag, parse_row_tag,
};