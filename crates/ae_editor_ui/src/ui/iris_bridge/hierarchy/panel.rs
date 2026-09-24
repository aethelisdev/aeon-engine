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
use super::types::{HIERARCHY_TAG_PANEL_ROOT, HierarchyPanelParams, HierarchyRow};
use crate::ui::iris_bridge::theme::*;
use irisui::prelude::*;

/// Builds the complete Scene Hierarchy panel tree purely using [`UiScope`].
///
/// Returns the computed maximum vertical scroll extent in physical pixels.
pub fn build_hierarchy_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    rows_cache: &mut Vec<HierarchyRow>,
) -> f32 {
    let mut scope = UiScope::with_tagged_interactions(tree, parent_id, &[], params.hovered_tag);

    let root_style = Style::new()
        .flex_col()
        .background(ELEVATION_1_PANEL)
        .border(1.0, Color::rgba(0.12, 0.13, 0.16, 0.90))
        .clip_children(true);

    let mut max_scroll = 0.0;

    scope.container_tagged(
        "HierarchyPanelRoot",
        root_style,
        WidgetRole::Default,
        HIERARCHY_TAG_PANEL_ROOT,
        |panel_scope| {
            // 1. Header Toolbar (Search Box, Add Button, Delete Button)
            build_hierarchy_header(panel_scope, params);

            // 2. Sync and Flatten ECS Hierarchy Tree Rows into persistent cache
            sync_hierarchy_rows(params.world, params.collapsed_entities, rows_cache);
            let total_objects = rows_cache.len();

            // 3. Virtualized & Frustum-Culled Entity Rows Viewport
            max_scroll = build_hierarchy_rows(panel_scope, rows_cache, params);

            // 4. Footer Status Line (Object Count & Selection Telemetry)
            build_hierarchy_footer(panel_scope, total_objects, params);

            // 5. Finalize flexbox layout to prevent dock leaf collapse
            panel_scope.finish_layout(params.panel_rect);
        },
    );

    max_scroll
}

/// Builds floating overlays for the Scene Hierarchy panel (Context Menu and Add Menu).
///
/// Ensures menus are attached to the root overlay layer on top of all docked panels,
/// preventing any bleed-through or clipping by neighboring dock tabs.
pub fn build_hierarchy_overlays(
    tree: &mut UiTree,
    overlay_root: WidgetId,
    params: &HierarchyPanelParams<'_>,
) {
    // 1. Right-Click Entity Context Menu (if open)
    let _ = build_context_menu(tree, overlay_root, params);

    // 2. Cascading `➕` Add Entity Dropdown Menu (if open)
    let _ = build_add_menu(tree, overlay_root, params);
}