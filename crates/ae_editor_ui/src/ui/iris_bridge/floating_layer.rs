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

use super::theme::*;

/// Checks if a panel currently resides within any active floating window.
pub fn is_panel_in_floating_window(layout_state: &PanelLayoutState, panel: PanelId) -> bool {
    layout_state
        .dock_state
        .floating_windows
        .iter()
        .any(|w| w.tree.all_tabs().contains(&panel))
}

/// Returns the Iris content rectangle assigned to an active panel in a floating window.
/// Floating panel builders use this rectangle instead of the retired host renderer's bounds,
/// ensuring their content begins below the native title and tab strip rather than covering it.
pub fn active_panel_content_rect(layout_state: &PanelLayoutState, panel: PanelId) -> Option<Rect> {
    const TITLE_BAR_HEIGHT: f32 = 26.0;

    layout_state
        .dock_state
        .floating_windows
        .iter()
        .find_map(|window| {
            for (_, node) in window.tree.iter() {
                if let DockNode::Leaf { tabs, active_tab } = node
                    && tabs.get(*active_tab).is_some_and(|active| *active == panel)
                {
                    return Some(Rect::new(
                        window.rect.x,
                        window.rect.y + TITLE_BAR_HEIGHT,
                        window.rect.width,
                        (window.rect.height - TITLE_BAR_HEIGHT).max(0.0),
                    ));
                }
            }
            None
        })
}

/// Resolves the content rectangle of the 3D viewport when detached in a floating window.
pub fn resolve_floating_viewport_rect(layout_state: &PanelLayoutState) -> Option<Rect> {
    active_panel_content_rect(layout_state, PanelId::Viewport)
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
            node.set_role(WidgetRole::FloatingWindow);
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
                .background(ELEVATION_4_POPUP)
                .border(1.0, BORDER_ELEVATED)
                .border_radius(19.0)
                .box_shadow(0.0, 8.0, 28.0, Color::rgba(0.0, 0.0, 0.0, 0.75));
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
                .background(ELEVATION_2_HEADER)
                .border_radius(19.0);
        }
        let _ = tree.add_child(win_container, bar_id);

        // 4. Tab Bar Buttons (Dock-back `⤢` and Close `✖`)
        let dock_btn_rect = Rect::new(bar_rect.right() - 56.0, bar_rect.y + 2.0, 20.0, 22.0);
        let is_dock_hovered = dock_btn_rect.contains_point(cursor_pos);
        let dock_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(dock_btn_id) {
            node.set_name("FloatingWindowDockBtn");
            node.computed_rect = dock_btn_rect;
            node.text = Some("⤢".to_string());
            node.font_size = 12.0;
            node.text_color = if is_dock_hovered {
                Color::WHITE
            } else {
                ACCENT_CYAN
            };
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(win_container, dock_btn_id);

        let close_btn_rect = Rect::new(bar_rect.right() - 32.0, bar_rect.y + 2.0, 20.0, 22.0);
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
                TEXT_MUTED
            };
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(win_container, close_btn_id);

        // 5. Header Baseline Divider
        let baseline_id = tree.create_node();
        if let Some(line) = tree.get_mut(baseline_id) {
            line.set_name("FloatingWindowHeaderBaseline");
            line.computed_rect =
                Rect::new(bar_rect.x, bar_rect.bottom() - 1.0, bar_rect.width, 1.0);
            line.style = Style::new().background(BORDER_MICRON);
        }
        let _ = tree.add_child(win_container, baseline_id);

        // 6. Render Tab Titles & Active Highlights
        let mut current_tab_x = bar_rect.x;
        for (_leaf_id, node) in win.tree.iter() {
            if let DockNode::Leaf { tabs, active_tab } = node {
                for (tab_idx, panel) in tabs.iter().enumerate() {
                    let is_active = tab_idx == *active_tab;
                    let atlas_icon = panel.atlas_icon();
                    let char_count = panel.title().chars().count();
                    let icon_w = if atlas_icon.is_some() { 18.0 } else { 0.0 };
                    let title = if atlas_icon.is_some() {
                        panel.title().to_string()
                    } else {
                        format!("{} {}", panel.icon(), panel.title())
                    };
                    let tab_w =
                        (16.0 + icon_w + (char_count as f32) * 6.8 + 14.0).clamp(52.0, 160.0);
                    let tab_rect = Rect::new(current_tab_x, bar_rect.y, tab_w, TAB_BAR_HEIGHT);
                    let is_tab_hovered = tab_rect.contains_point(cursor_pos);

                    let tab_node_id = tree.create_node();
                    if let Some(tab_node) = tree.get_mut(tab_node_id) {
                        tab_node.set_name("FloatingWindowTab");
                        tab_node.computed_rect = tab_rect;
                        let bg_color = if is_active {
                            ELEVATION_3_ACTIVE_PILL
                        } else if is_tab_hovered {
                            ELEVATION_3_HOVERED_PILL
                        } else {
                            ELEVATION_2_HEADER
                        };
                        tab_node.style = Style::new()
                            .background(bg_color)
                            .corner_radii(CornerRadii::new(5.0, 5.0, 0.0, 0.0));
                    }
                    let _ = tree.add_child(win_container, tab_node_id);

                    if is_active {
                        let active_line_id = tree.create_node();
                        if let Some(line) = tree.get_mut(active_line_id) {
                            line.set_name("FloatingWindowTabActiveLine");
                            line.computed_rect =
                                Rect::new(tab_rect.x, tab_rect.bottom() - 2.0, tab_rect.width, 2.0);
                            line.style = Style::new().background(ACCENT_CYAN);
                        }
                        let _ = tree.add_child(win_container, active_line_id);
                    }

                    if let Some(uv) = atlas_icon {
                        let icon_node = tree.create_node();
                        if let Some(node) = tree.get_mut(icon_node) {
                            node.set_name("FloatingWindowTabIcon");
                            node.computed_rect =
                                Rect::new(tab_rect.x + 7.0, tab_rect.y + 5.0, 16.0, 16.0);
                            node.set_texture_uv(uv);
                            let tint = if is_active {
                                ACCENT_CYAN
                            } else if is_tab_hovered {
                                Color::WHITE
                            } else {
                                TEXT_REGULAR
                            };
                            node.set_texture_tint(tint);
                        }
                        let _ = tree.add_child(win_container, icon_node);
                    }

                    let text_offset_x = if atlas_icon.is_some() {
                        7.0 + 16.0 + 5.0
                    } else {
                        6.0
                    };
                    let text_id = tree.create_node();
                    if let Some(text_node) = tree.get_mut(text_id) {
                        text_node.set_name("FloatingWindowTabTitle");
                        text_node.computed_rect = Rect::new(
                            tab_rect.x + text_offset_x,
                            tab_rect.y + 4.0,
                            tab_rect.width - text_offset_x - 6.0,
                            18.0,
                        );
                        text_node.text = Some(title);
                        text_node.font_size = 12.0;
                        text_node.text_color = if is_active {
                            ACCENT_CYAN
                        } else if is_tab_hovered {
                            TEXT_BRIGHT
                        } else {
                            TEXT_MUTED
                        };
                        text_node.text_align = TextAlign::Left;
                    }
                    let _ = tree.add_child(win_container, text_id);

                    current_tab_x += tab_w;
                }

                if *active_tab < tabs.len() {
                    panel_containers.push((tabs[*active_tab], win_container));
                }
            }
        }
    }

    (rects, panel_containers)
}