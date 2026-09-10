// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec2;
use irisui::prelude::*;

use crate::project::ProjectRegistry;
use crate::ui::new_project::build_new_project_view;
use crate::ui::recent_projects::build_recent_projects_view;
use crate::ui::sidebar::build_sidebar;
use crate::ui::types::{LauncherAction, LauncherTab, LauncherUiState, ViewLayoutContext};

/// Builds the complete launcher user interface layout into the provided `UiTree`.
/// Sets up the root workspace container, delegating sidebar creation to `build_sidebar`
/// and active tab content rendering to `build_recent_projects_view` or `build_new_project_view`.
pub fn build_launcher_ui(
    tree: &mut UiTree,
    state: &LauncherUiState,
    registry: &ProjectRegistry,
    screen_size: Vec2,
    cursor_pos: Point,
) -> LauncherAction {
    let mut pending_action = LauncherAction::None;

    let root_id = match tree.root() {
        Some(id) => id,
        None => {
            let id = tree.create_node();
            let _ = tree.set_root(id);
            id
        }
    };
    if let Some(root_node) = tree.get_mut(root_id) {
        root_node.computed_rect = Rect::new(0.0, 0.0, screen_size.x, screen_size.y);
        root_node.style = Style::new().background(Color::hex("#0e1016"));
    }

    let sidebar_w = 220.0;
    let content_w = screen_size.x - sidebar_w;
    let content_h = screen_size.y;

    // ── 1. Left Sidebar ──────────────────────────────────────────────────────────
    build_sidebar(tree, root_id, state, cursor_pos, content_h, sidebar_w);

    // ── 2. Content Area ──────────────────────────────────────────────────────────
    let content_area_id = tree.create_node();
    if let Some(node) = tree.get_mut(content_area_id) {
        node.set_name("LauncherContentArea");
        node.computed_rect = Rect::new(sidebar_w, 0.0, content_w, content_h);
    }
    let _ = tree.add_child(root_id, content_area_id);

    let ctx = ViewLayoutContext {
        start_x: sidebar_w,
        content_w,
        content_h,
        cursor_pos,
        action: &mut pending_action,
    };

    match state.active_tab {
        LauncherTab::RecentProjects => {
            build_recent_projects_view(tree, content_area_id, ctx, registry);
        }
        LauncherTab::NewProject => {
            build_new_project_view(tree, content_area_id, ctx, state);
        }
    }

    pending_action
}