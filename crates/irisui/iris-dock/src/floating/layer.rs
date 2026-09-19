// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native UI tree node hierarchy generation for floating window surfaces.
//!
//! Spawns window backdrop cards, drop shadows, rounded corners, tab pill strips,
//! dock-back and close control buttons, and active leaf tab highlights.

use crate::floating::model::{FloatingWindow, FloatingWindowStyle};
use crate::tab_viewer::TabViewer;
use crate::tree::DockNode;
use iris_core::color::Color;
use iris_core::geometry::{CornerRadii, Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Builds the complete native Iris UI floating window widget hierarchy in the UI tree.
///
/// For each floating window:
/// 1. Creates a container node with `WidgetRole::FloatingWindow` (ensuring correct z-ordering in `UiLayer::Floating`).
/// 2. Adds the base background quad with drop shadow, border, and rounded corners.
/// 3. Adds the title bar header quad.
/// 4. Adds dock-back `⤢` and close `✖` control buttons.
/// 5. Adds header baseline divider.
/// 6. Renders tab pill items with icons, titles, and active indicator line.
///
/// Returns:
/// - List of window bounding rectangles (for hardware occlusion culling).
/// - Mapping from active `Tab` to its floating window container [`WidgetId`].
pub fn build_floating_windows_layer<Tab: Clone, V: TabViewer<Tab>>(
    tree: &mut UiTree,
    root_id: WidgetId,
    windows: &[FloatingWindow<Tab>],
    viewer: &V,
    cursor_pos: Point,
    style: &FloatingWindowStyle,
) -> (Vec<Rect>, Vec<(Tab, WidgetId)>) {
    let mut rects = Vec::with_capacity(windows.len());
    let mut active_tab_containers = Vec::new();

    for win in windows {
        let win_rect = win.rect;
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
                .background(style.background)
                .border(style.border_width, style.border_color)
                .border_radius(style.corner_radius)
                .box_shadow(
                    0.0,
                    style.shadow_offset_y,
                    style.shadow_blur,
                    style.shadow_color,
                );
        }
        let _ = tree.add_child(win_container, bg_id);

        // 3. Tab Bar Header Quad
        let bar_rect = Rect::new(
            win.rect.x,
            win.rect.y,
            win.rect.width,
            style.title_bar_height,
        );
        let bar_id = tree.create_node();
        if let Some(node) = tree.get_mut(bar_id) {
            node.set_name("FloatingWindowTabBar");
            node.computed_rect = bar_rect;
            node.style = Style::new()
                .background(style.title_bar_background)
                .border_radius(style.corner_radius);
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
                style.accent_color
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
                style.close_btn_hover
            } else {
                style.close_btn_idle
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
            line.style = Style::new().background(style.baseline_color);
        }
        let _ = tree.add_child(win_container, baseline_id);

        // 6. Render Tab Titles & Active Highlights
        let mut current_tab_x = bar_rect.x;
        for (_leaf_id, node) in win.tree.iter() {
            if let DockNode::Leaf { tabs, active_tab } = node {
                for (tab_idx, tab) in tabs.iter().enumerate() {
                    let is_active = tab_idx == *active_tab;
                    let atlas_icon = viewer.atlas_icon(tab);
                    let title = if atlas_icon.is_some() {
                        viewer.raw_title(tab)
                    } else {
                        viewer.title(tab)
                    };
                    let char_count = title.chars().count();
                    let icon_w = if atlas_icon.is_some() { 18.0 } else { 0.0 };
                    let tab_w =
                        (16.0 + icon_w + (char_count as f32) * 6.8 + 14.0).clamp(52.0, 160.0);
                    let tab_rect =
                        Rect::new(current_tab_x, bar_rect.y, tab_w, style.title_bar_height);
                    let is_tab_hovered = tab_rect.contains_point(cursor_pos);

                    let tab_node_id = tree.create_node();
                    if let Some(tab_node) = tree.get_mut(tab_node_id) {
                        tab_node.set_name("FloatingWindowTab");
                        tab_node.computed_rect = tab_rect;
                        let bg_color = if is_active {
                            style.tab_active_bg
                        } else if is_tab_hovered {
                            style.tab_hover_bg
                        } else {
                            style.tab_idle_bg
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
                            line.style = Style::new().background(style.accent_color);
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
                                style.accent_color
                            } else if is_tab_hovered {
                                Color::WHITE
                            } else {
                                style.text_muted
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
                            (tab_rect.width - text_offset_x - 6.0).max(0.0),
                            18.0,
                        );
                        text_node.text = Some(title);
                        text_node.font_size = 12.0;
                        text_node.text_color = if is_active {
                            style.text_active
                        } else if is_tab_hovered {
                            style.text_hover
                        } else {
                            style.text_muted
                        };
                        text_node.text_align = TextAlign::Left;
                    }
                    let _ = tree.add_child(win_container, text_id);

                    current_tab_x += tab_w;
                }

                if *active_tab < tabs.len() {
                    active_tab_containers.push((tabs[*active_tab].clone(), win_container));
                }
            }
        }
    }

    (rects, active_tab_containers)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyViewer;
    impl TabViewer<&'static str> for DummyViewer {
        fn title(&self, tab: &&'static str) -> String {
            tab.to_string()
        }
    }

    #[test]
    fn test_build_floating_windows_layer() {
        let window = FloatingWindow::new(
            1,
            "Console",
            Rect::new(50.0, 50.0, 300.0, 200.0),
            vec!["LogTab"],
        );
        let windows = vec![window];
        let viewer = DummyViewer;

        let mut tree = UiTree::new();
        let root = tree.create_node();
        let (rects, containers) = build_floating_windows_layer(
            &mut tree,
            root,
            &windows,
            &viewer,
            Point::new(0.0, 0.0),
            &FloatingWindowStyle::default(),
        );

        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], Rect::new(50.0, 50.0, 300.0, 200.0));
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].0, "LogTab");
    }
}