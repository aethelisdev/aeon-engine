// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window layer coordinator for Iris UI.
//!
//! Orchestrates solid, opaque background quad generation for detached floating panels,
//! builds native tab bars and titles, tracks active floating window rectangles to
//! support hardware occlusion culling, and resolves whether specific panels reside
//! in docked leaves or independent floating surfaces.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::DockNode;
use irisui::prelude::*;

/// Checks if a panel currently resides within any active floating window.
pub fn is_panel_in_floating_window(layout_state: &PanelLayoutState, panel: PanelId) -> bool {
    layout_state
        .dock_state
        .floating_windows
        .iter()
        .any(|w| w.tree.all_tabs().contains(&panel))
}

/// Builds the complete native Iris UI floating window hierarchy in the UI tree.
/// For each floating window:
/// 1. Creates a container node named `FloatingWindow_{win.id}`.
/// 2. Adds an opaque background quad with outer drop shadow and border (`#0e1016`).
/// 3. Adds the native 26px tab bar header quad (`#0f0f14`).
/// 4. Adds tab items with icons, titles, and active indicator line.
/// 5. Adds window control buttons (Dock-back `⤢` and Close `✖`).
/// Returns:
/// - List of floating window bounding rectangles for hardware occlusion culling.
/// - Active panel mapping `(PanelId, WidgetId)` where `WidgetId` is the floating window container.
pub fn build_floating_windows(
    tree: &mut UiTree,
    root_id: WidgetId,
    layout_state: &PanelLayoutState,
    cursor_pos: Point,
) -> (Vec<Rect>, Vec<(PanelId, WidgetId)>) {
    let mut rects = Vec::with_capacity(layout_state.dock_state.floating_windows.len());
    let mut panel_containers = Vec::new();

    for win in &layout_state.dock_state.floating_windows {
        let win_rect = Rect::new(win.rect.x, win.rect.y, win.rect.width, win.rect.height);
        rects.push(win_rect);

        // 1. Floating Window Container Node
        let win_container = tree.create_node();
        if let Some(node) = tree.get_mut(win_container) {
            node.set_name(format!("FloatingWindow_{}", win.id));
            node.computed_rect = win_rect;
            node.style = Style::new().width(win.rect.width).height(win.rect.height);
        }
        let _ = tree.add_child(root_id, win_container);

        // 2. Base Background Quad with Shadow and Border
        let bg_id = tree.create_node();
        if let Some(node) = tree.get_mut(bg_id) {
            node.set_name("FloatingWindowBase");
            node.computed_rect = win_rect;
            node.style = Style::new()
                .background(Color::rgba(0.055, 0.063, 0.086, 1.0))
                .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.90))
                .border_radius(6.0)
                .box_shadow(0.0, 4.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
        }
        let _ = tree.add_child(win_container, bg_id);

        // 3. Tab Bar Header Quad (26px)
        const TAB_BAR_HEIGHT: f32 = 26.0;
        let bar_rect = Rect::new(win.rect.x, win.rect.y, win.rect.width, TAB_BAR_HEIGHT);
        let bar_id = tree.create_node();
        if let Some(node) = tree.get_mut(bar_id) {
            node.set_name("FloatingWindowTabBar");
            node.computed_rect = bar_rect;
            node.style = Style::new()
                .background(Color::rgba(0.059, 0.059, 0.078, 1.0))
                .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.80));
        }
        let _ = tree.add_child(win_container, bar_id);

        // 4. Tab Bar Buttons (Dock-back `⤢` and Close `✖`)
        let dock_btn_rect = Rect::new(bar_rect.right() - 48.0, bar_rect.y + 2.0, 20.0, 22.0);
        let is_dock_hovered = dock_btn_rect.contains_point(cursor_pos);
        let dock_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(dock_btn_id) {
            node.set_name("FloatingWindowDockBtn");
            node.computed_rect = dock_btn_rect;
            node.text = Some("⤢".to_string());
            node.font_size = 12.0;
            node.text_color = if is_dock_hovered {
                Color::rgba(0.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.0, 0.898, 1.0, 0.85)
            };
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(win_container, dock_btn_id);

        let close_btn_rect = Rect::new(bar_rect.right() - 24.0, bar_rect.y + 2.0, 20.0, 22.0);
        let is_close_hovered = close_btn_rect.contains_point(cursor_pos);
        let close_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(close_btn_id) {
            node.set_name("FloatingWindowCloseBtn");
            node.computed_rect = close_btn_rect;
            node.text = Some("✖".to_string());
            node.font_size = 10.0;
            node.text_color = if is_close_hovered {
                Color::rgba(1.0, 0.35, 0.35, 1.0)
            } else {
                Color::rgba(0.65, 0.68, 0.75, 0.85)
            };
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(win_container, close_btn_id);

        // 5. Render Tab Titles & Active Highlights
        let mut current_tab_x = bar_rect.x + 4.0;
        for (_leaf_id, node) in win.tree.iter() {
            if let DockNode::Leaf { tabs, active_tab } = node {
                for (tab_idx, panel) in tabs.iter().enumerate() {
                    let is_active = tab_idx == *active_tab;
                    let title = format!("{} {}", panel.icon(), panel.title());
                    let tab_w = ((title.len() as f32) * 7.5 + 28.0).clamp(60.0, 180.0);
                    let tab_rect = Rect::new(current_tab_x, bar_rect.y, tab_w, TAB_BAR_HEIGHT);
                    let is_tab_hovered = tab_rect.contains_point(cursor_pos);

                    let tab_node_id = tree.create_node();
                    if let Some(tab_node) = tree.get_mut(tab_node_id) {
                        tab_node.set_name("FloatingWindowTabPill");
                        tab_node.computed_rect = tab_rect;
                        let bg_color = if is_active {
                            Color::rgba(0.086, 0.094, 0.118, 1.0)
                        } else if is_tab_hovered {
                            Color::rgba(0.080, 0.088, 0.110, 1.0)
                        } else {
                            Color::rgba(0.063, 0.067, 0.086, 1.0)
                        };
                        tab_node.style = Style::new().background(bg_color).border_radius(4.0);
                    }
                    let _ = tree.add_child(win_container, tab_node_id);

                    if is_active {
                        let active_line_id = tree.create_node();
                        if let Some(line) = tree.get_mut(active_line_id) {
                            line.set_name("FloatingWindowTabActiveLine");
                            line.computed_rect =
                                Rect::new(tab_rect.x, tab_rect.bottom() - 2.0, tab_rect.width, 2.0);
                            line.style = Style::new().background(Color::rgba(0.0, 0.898, 1.0, 1.0));
                        }
                        let _ = tree.add_child(win_container, active_line_id);
                    }

                    let text_id = tree.create_node();
                    if let Some(text_node) = tree.get_mut(text_id) {
                        text_node.set_name("FloatingWindowTabTitle");
                        text_node.computed_rect = Rect::new(
                            tab_rect.x + 8.0,
                            tab_rect.y + 4.0,
                            tab_rect.width - 16.0,
                            18.0,
                        );
                        text_node.text = Some(title);
                        text_node.font_size = 12.0;
                        text_node.text_color = if is_active {
                            Color::rgba(0.0, 0.898, 1.0, 1.0)
                        } else if is_tab_hovered {
                            Color::rgba(0.95, 0.96, 1.0, 1.0)
                        } else {
                            Color::rgba(0.70, 0.73, 0.80, 1.0)
                        };
                        text_node.text_align = TextAlign::Left;
                    }
                    let _ = tree.add_child(win_container, text_id);

                    current_tab_x += tab_w + 2.0;
                }

                if *active_tab < tabs.len() {
                    panel_containers.push((tabs[*active_tab], win_container));
                }
            }
        }
    }

    (rects, panel_containers)
}