// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Panel Orchestrator & Viewport Lifecycle
//!
//! Assembles the Scene Hierarchy root container, header toolbar, scrollable DFS
//! entity rows, and footer status bar purely using [`UiScope`] and declarative flexbox primitives.
//!

use super::add_menu::build_add_menu;
use super::context_menu::build_context_menu;
use super::footer::build_hierarchy_footer;
use super::header::build_hierarchy_header;
use super::rows::{build_hierarchy_rows, sync_hierarchy_rows};
use super::types::{HIERARCHY_TAG_PANEL_ROOT, HierarchyAction, HierarchyPanelParams, HierarchyRow};
use irisui::prelude::*;

/// Builds the complete Scene Hierarchy panel tree purely using [`UiScope`].
pub fn build_hierarchy_panel(
    scope: &mut UiScope<'_>,
    params: &HierarchyPanelParams<'_>,
    rows_cache: &mut Vec<HierarchyRow>,
    actions: &mut Vec<HierarchyAction>,
    tree_state: &mut TreeState<hecs::Entity>,
) {
    scope
        .panel("HierarchyPanelRoot")
        .tagged(HIERARCHY_TAG_PANEL_ROOT)
        .bounds(params.panel_rect)
        .background_themed(|t| t.panel_bg)
        .show(|panel_scope| {
            // 1. Header Toolbar (Search Box, Add Button, Delete Button)
            build_hierarchy_header(panel_scope, params, actions);

            // 2. Sync and Flatten ECS Hierarchy Tree Rows into persistent cache
            sync_hierarchy_rows(params.world, tree_state, rows_cache);
            let total_objects = rows_cache.len();

            // 3. Virtualized & Frustum-Culled Entity Rows Viewport
            build_hierarchy_rows(panel_scope, rows_cache, params, actions, tree_state);

            // 4. Footer Status Line (Object Count & Selection Telemetry)
            build_hierarchy_footer(panel_scope, total_objects, params);
        });
}

/// Builds floating overlays for the Scene Hierarchy panel (Context Menu and Add Menu).
///
/// Ensures menus are attached to the root overlay layer on top of all docked panels,
/// preventing any bleed-through or clipping by neighboring dock tabs.
///
/// Returns all rendered popup rectangles for precise overlay boundary tracking.
pub fn build_hierarchy_overlays(
    tree: &mut UiTree,
    overlay_root: WidgetId,
    params: &HierarchyPanelParams<'_>,
    actions: &mut Vec<HierarchyAction>,
) -> Vec<Rect> {
    let mut rects = Vec::new();
    // 1. Right-Click Entity Context Menu (if open)
    if let Some(ctx_rect) = build_context_menu(tree, overlay_root, params, actions) {
        rects.push(ctx_rect);
    }

    // 2. Cascading `➕` Add Entity Dropdown Menu (if open)
    let add_rects = build_add_menu(tree, overlay_root, params, actions);
    rects.extend(add_rects);
    rects
}