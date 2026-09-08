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
pub mod types;

pub use panel::{
    HierarchyPanelNodes, build_hierarchy_overlays, build_hierarchy_panel, handle_hierarchy_click,
    handle_hierarchy_hover,
};
pub use types::{
    AddSubmenuId, HierarchyAction, HierarchyPanelParams, HierarchyPanelTargets, HierarchyRow,
};