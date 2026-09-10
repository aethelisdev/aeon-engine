// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use irisui::prelude::*;

use crate::icons::{ICON_FOLDER, ICON_PLUS, ICON_WIREFRAME, ICON_WORLD};
use crate::ui::types::{LauncherTab, LauncherUiState};

/// Builds the left sidebar navigation hierarchy into the UI tree.
/// Assembles the logo badge, navigation tab buttons ("Recent Projects" and "New Project"),
/// hover/active state visual feedback, and version footer metadata.
pub fn build_sidebar(
    tree: &mut UiTree,
    root_id: WidgetId,
    state: &LauncherUiState,
    cursor_pos: Point,
    content_h: f32,
    sidebar_w: f32,
) -> WidgetId {
    let sidebar_id = tree.create_node();
    if let Some(node) = tree.get_mut(sidebar_id) {
        node.set_name("LauncherSidebar");
        node.computed_rect = Rect::new(0.0, 0.0, sidebar_w, content_h);
        node.style = Style::new()
            .background(Color::hex("#12141a"))
            .border(1.0, Color::hex("#1e222d"));
    }
    let _ = tree.add_child(root_id, sidebar_id);

    // Sidebar Header: Logo + "Aeon Engine"
    let logo_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(logo_card_id) {
        node.set_name("SidebarHeader");
        node.computed_rect = Rect::new(16.0, 18.0, sidebar_w - 32.0, 42.0);
    }
    let _ = tree.add_child(sidebar_id, logo_card_id);

    let logo_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(logo_icon_id) {
        node.set_name("SidebarLogoIcon");
        node.computed_rect = Rect::new(16.0, 26.0, 20.0, 20.0);
        node.set_texture_uv(ICON_WORLD);
        node.set_texture_tint(Color::hex("#00e5ff"));
    }
    let _ = tree.add_child(sidebar_id, logo_icon_id);

    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("SidebarTitle");
        node.set_text("AEON HUB");
        node.font_size = 14.0;
        node.text_color = Color::hex("#ffffff");
        node.computed_rect = Rect::new(44.0, 27.0, 150.0, 20.0);
    }
    let _ = tree.add_child(sidebar_id, title_id);

    // Sidebar Navigation Buttons
    let mut btn_y = 80.0;

    // Tab 1: Recent Projects (uses ICON_FOLDER quad)
    let is_recent_active = state.active_tab == LauncherTab::RecentProjects;
    let recent_rect = Rect::new(12.0, btn_y, sidebar_w - 24.0, 36.0);
    let is_recent_hovered = recent_rect.contains_point(cursor_pos);

    let recent_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(recent_btn_id) {
        node.set_name("NavRecentProjects");
        node.computed_rect = recent_rect;
        let bg = if is_recent_active {
            Color::hex("#1a202c")
        } else if is_recent_hovered {
            Color::hex("#161a22")
        } else {
            Color::TRANSPARENT
        };
        node.style = Style::new().background(bg).border_radius(6.0);
    }
    let _ = tree.add_child(sidebar_id, recent_btn_id);

    let recent_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(recent_icon_id) {
        node.computed_rect = Rect::new(24.0, btn_y + 10.0, 16.0, 16.0);
        node.set_texture_uv(ICON_FOLDER);
        let tint = if is_recent_active {
            Color::hex("#ffd166")
        } else {
            Color::hex("#8a8f9d")
        };
        node.set_texture_tint(tint);
    }
    let _ = tree.add_child(sidebar_id, recent_icon_id);

    let recent_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(recent_lbl_id) {
        node.set_text("Recent Projects");
        node.font_size = 12.0;
        node.text_color = if is_recent_active {
            Color::hex("#ffffff")
        } else {
            Color::hex("#a0a5b5")
        };
        node.computed_rect = Rect::new(48.0, btn_y + 9.0, 140.0, 18.0);
    }
    let _ = tree.add_child(sidebar_id, recent_lbl_id);

    btn_y += 44.0;

    // Tab 2: New Project (uses ICON_PLUS quad)
    let is_new_active = state.active_tab == LauncherTab::NewProject;
    let new_rect = Rect::new(12.0, btn_y, sidebar_w - 24.0, 36.0);
    let is_new_hovered = new_rect.contains_point(cursor_pos);

    let new_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(new_btn_id) {
        node.set_name("NavNewProject");
        node.computed_rect = new_rect;
        let bg = if is_new_active {
            Color::hex("#1a202c")
        } else if is_new_hovered {
            Color::hex("#161a22")
        } else {
            Color::TRANSPARENT
        };
        node.style = Style::new().background(bg).border_radius(6.0);
    }
    let _ = tree.add_child(sidebar_id, new_btn_id);

    let new_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(new_icon_id) {
        node.computed_rect = Rect::new(24.0, btn_y + 10.0, 16.0, 16.0);
        node.set_texture_uv(ICON_PLUS);
        let tint = if is_new_active {
            Color::hex("#00e5ff")
        } else {
            Color::hex("#8a8f9d")
        };
        node.set_texture_tint(tint);
    }
    let _ = tree.add_child(sidebar_id, new_icon_id);

    let new_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(new_lbl_id) {
        node.set_text("New Project");
        node.font_size = 12.0;
        node.text_color = if is_new_active {
            Color::hex("#ffffff")
        } else {
            Color::hex("#a0a5b5")
        };
        node.computed_rect = Rect::new(48.0, btn_y + 9.0, 140.0, 18.0);
    }
    let _ = tree.add_child(sidebar_id, new_lbl_id);

    // Sidebar Footer: Version & Wireframe Icon (uses ICON_WIREFRAME quad)
    let version_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(version_icon_id) {
        node.computed_rect = Rect::new(16.0, content_h - 32.0, 14.0, 14.0);
        node.set_texture_uv(ICON_WIREFRAME);
        node.set_texture_tint(Color::hex("#525866"));
    }
    let _ = tree.add_child(sidebar_id, version_icon_id);

    let ver_id = tree.create_node();
    if let Some(node) = tree.get_mut(ver_id) {
        node.set_text("Aeon Core v0.9.0");
        node.font_size = 10.0;
        node.text_color = Color::hex("#525866");
        node.computed_rect = Rect::new(36.0, content_h - 33.0, 160.0, 16.0);
    }
    let _ = tree.add_child(sidebar_id, ver_id);

    sidebar_id
}