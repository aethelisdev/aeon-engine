// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy (Outliner) Iris UI Module
//!
//! Orchestrates the 100% Iris UI GPU-accelerated Scene Hierarchy tree,
//! search bar filtering, cascading `➕` entity spawning menus, and right-click context actions.

pub mod add_menu;
pub mod builder;
pub mod footer;
pub mod header;
pub mod panel;
pub mod rows;
#[cfg(test)]
pub mod tests;
#[cfg(test)]
pub mod tests_add_menu;
#[cfg(test)]
pub mod tests_navigation;
pub mod types;

pub use add_menu::{draw_hierarchy_add_menu, hierarchy_add_menu_entries};
pub use panel::build_hierarchy_panel;
pub use rows::SceneHierarchySource;
pub use types::{
    AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_EYE_BASE,
    HIERARCHY_TAG_FOLDOUT_BASE, HIERARCHY_TAG_PANEL_ROOT, HIERARCHY_TAG_ROW_BASE,
    HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT, HIERARCHY_TAG_VIEWPORT,
    HierarchyPanelParams, HierarchyPanelState, HierarchyRow, HierarchyState, make_eye_tag,
    make_foldout_tag, make_row_tag,
};