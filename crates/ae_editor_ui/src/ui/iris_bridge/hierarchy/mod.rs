// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy (Outliner) Iris UI Module
//!
//! Orchestrates the 100% Iris UI GPU-accelerated Scene Hierarchy tree,
//! search bar filtering, cascading `➕` entity spawning menus, and right-click context actions.

pub mod add_menu;
pub mod context_menu;
pub mod footer;
pub mod header;
pub mod rows;
pub mod types;

pub use types::{
    AddSubmenuId, HierarchyAction, HierarchyPanelParams, HierarchyPanelTargets, HierarchyRow,
};

use add_menu::build_add_menu;
use context_menu::build_context_menu;
use footer::build_hierarchy_footer;
use header::build_hierarchy_header;
use irisui::prelude::*;
use rows::{build_hierarchy_rows, sync_hierarchy_rows};

/// Output node handles created during layout initialization of the Scene Hierarchy panel.
pub struct HierarchyPanelNodes {
    /// Root node of the hierarchy panel container.
    pub root_id: WidgetId,
}

/// Builds the complete Scene Hierarchy panel tree in the `UiTree`.
pub fn build_hierarchy_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) -> HierarchyPanelNodes {
    targets.panel_rect = params.panel_rect;

    // Panel Base Container
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("HierarchyPanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new().background(Color::rgba(0.06, 0.07, 0.09, 1.0));
    }
    let _ = tree.add_child(parent_id, root_id);

    // 1. Search Bar & Top Buttons Header
    let _header_nodes = build_hierarchy_header(tree, root_id, params, targets);

    // 2. Sync and Flatten ECS Hierarchy Tree Rows
    let total_entities = params.world.len() as usize;
    let mut flat_rows = Vec::with_capacity(total_entities.min(1024));
    sync_hierarchy_rows(params.world, &mut flat_rows);
    let total_objects = flat_rows.len();

    // 3. Scrollable Entity Rows
    build_hierarchy_rows(tree, root_id, &flat_rows, params, targets);

    // 4. Footer Status Line
    build_hierarchy_footer(tree, root_id, total_objects, params);

    // 5. Right-Click Entity Context Menu (if open)
    build_context_menu(tree, root_id, params, targets);

    // 6. Cascading `➕` Add Entity Dropdown Menu (if open)
    build_add_menu(tree, root_id, params, targets);

    HierarchyPanelNodes { root_id }
}

/// Handles interactive mouse clicks on the Scene Hierarchy panel.
pub fn handle_hierarchy_click(
    point: Point,
    button: MouseButton,
    targets: &HierarchyPanelTargets,
    out_actions: &mut Vec<HierarchyAction>,
) -> bool {
    // 1. Right-Click Context Menu Interaction
    if let Some((target_ent, menu_rect, del_rect, vis_rect)) = targets.active_context_menu {
        if menu_rect.contains_point(point) {
            if del_rect.contains_point(point) {
                out_actions.push(HierarchyAction::SelectEntity(Some(target_ent)));
                out_actions.push(HierarchyAction::DeleteSelected);
                out_actions.push(HierarchyAction::CloseContextMenu);
                return true;
            }
            if vis_rect.contains_point(point) {
                out_actions.push(HierarchyAction::ToggleVisibility(target_ent));
                out_actions.push(HierarchyAction::CloseContextMenu);
                return true;
            }
            return true;
        } else {
            out_actions.push(HierarchyAction::CloseContextMenu);
        }
    }

    // 2. Cascading Add Menu Submenu Interaction
    if let Some(sub_rect) = targets.active_submenu_rect
        && sub_rect.contains_point(point)
    {
        for (item_rect, action) in &targets.submenu_items {
            if item_rect.contains_point(point) {
                out_actions.push(action.clone());
                out_actions.push(HierarchyAction::CloseAddMenu);
                out_actions.push(HierarchyAction::CloseSubmenu);
                return true;
            }
        }
        return true;
    }

    // 3. Cascading Add Menu Root Card Interaction
    if let Some(card_rect) = targets.active_add_menu_rect
        && card_rect.contains_point(point)
    {
        for (item_rect, target_payload) in &targets.add_menu_items {
            if item_rect.contains_point(point) {
                match target_payload {
                    Ok(submenu_id) => {
                        out_actions.push(HierarchyAction::OpenSubmenu(*submenu_id));
                    }
                    Err(action) => {
                        out_actions.push(action.clone());
                        out_actions.push(HierarchyAction::CloseAddMenu);
                        out_actions.push(HierarchyAction::CloseSubmenu);
                    }
                }
                return true;
            }
        }
        return true;
    }

    if targets.active_add_menu_rect.is_some() {
        out_actions.push(HierarchyAction::CloseAddMenu);
        out_actions.push(HierarchyAction::CloseSubmenu);
    }

    // 4. Header `➕` Add Menu Button
    if targets.add_btn_rect.contains_point(point) {
        out_actions.push(HierarchyAction::OpenAddMenu(Some(point)));
        return true;
    }

    // 5. Header `🗑` Delete Selected Button
    if let Some(del_rect) = targets.delete_btn_rect
        && del_rect.contains_point(point)
    {
        out_actions.push(HierarchyAction::DeleteSelected);
        return true;
    }

    // 6. Search Bar Clear `✖` Button
    if let Some(clr_rect) = targets.search_clear_btn_rect
        && clr_rect.contains_point(point)
    {
        out_actions.push(HierarchyAction::ClearSearchQuery);
        return true;
    }

    // 7. Right-Click on Entity Row (Open Context Menu)
    if button == MouseButton::Right {
        for (ent, row_rect, _, _) in &targets.entity_rows {
            if row_rect.contains_point(point) {
                out_actions.push(HierarchyAction::SelectEntity(Some(*ent)));
                out_actions.push(HierarchyAction::OpenContextMenu(*ent, point));
                return true;
            }
        }
    }

    // 8. Left-Click on Entity Row / Eye Toggle Button
    if button == MouseButton::Left {
        for (ent, row_rect, eye_rect, _) in &targets.entity_rows {
            if eye_rect.contains_point(point) {
                out_actions.push(HierarchyAction::ToggleVisibility(*ent));
                return true;
            }
            if row_rect.contains_point(point) {
                out_actions.push(HierarchyAction::SelectEntity(Some(*ent)));
                return true;
            }
        }
    }

    targets.panel_rect.contains_point(point)
}

/// Handles interactive hover events on the Scene Hierarchy panel.
pub fn handle_hierarchy_hover(
    point: Point,
    targets: &HierarchyPanelTargets,
    out_actions: &mut Vec<HierarchyAction>,
) -> bool {
    // Check submenu hover switching if Add Menu is open
    if targets.active_add_menu_rect.is_some() {
        for (item_rect, target_payload) in &targets.add_menu_items {
            if item_rect.contains_point(point)
                && let Ok(submenu_id) = target_payload
            {
                out_actions.push(HierarchyAction::OpenSubmenu(*submenu_id));
                return true;
            }
        }
    }

    false
}