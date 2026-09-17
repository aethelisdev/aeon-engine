// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Iris docking chrome layout calculation and retained node generation.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::{SplitDirection, compute_dock_layout};
use irisui::prelude::*;

use super::super::theme::*;
use super::types::*;

/// Builds Iris-rendered tab strips, panel backgrounds, and split dividers for the native tree.
/// The returned geometry is the authoritative panel-coordinate source used by the Iris overlay.
pub fn build_native_dock(
    tree: &mut UiTree,
    parent: WidgetId,
    layout_state: &PanelLayoutState,
    workspace_rect: Rect,
    cursor_pos: Point,
    is_cursor_occluded: bool,
) -> NativeDockFrame {
    let computed = compute_dock_layout(
        &layout_state.dock_state.tree,
        workspace_rect,
        SPLITTER_THICKNESS,
        NATIVE_DOCK_TAB_HEIGHT,
    );
    let mut frame = NativeDockFrame::default();
    let is_dragging_splitter = layout_state.dock_state.active_splitter.is_some();

    // 1. Render all leaf panel backgrounds and tab strips
    for leaf in &computed.leaves {
        if leaf.tabs.is_empty() {
            if layout_state.dock_state.active_drag.is_some() {
                add_rect_node(
                    tree,
                    parent,
                    leaf.rect,
                    "IrisDockPanelEmpty",
                    Style::new()
                        .background(Color::from_u8(16, 20, 28, 160))
                        .border(1.0, Color::from_u8(35, 42, 55, 120))
                        .clip_children(true),
                );
            }
            continue;
        }

        // Base panel background quad
        add_rect_node(
            tree,
            parent,
            leaf.rect,
            "IrisDockPanel",
            Style::new()
                .background(ELEVATION_1_PANEL)
                .clip_children(true),
        );

        // Tab strip header quad
        let strip_id = add_rect_node(
            tree,
            parent,
            leaf.tab_bar_rect,
            "IrisDockTabStrip",
            Style::new()
                .background(ELEVATION_2_HEADER)
                .clip_children(true),
        );

        // Continuous baseline divider running full width of the tab bar
        add_rect_node(
            tree,
            strip_id,
            Rect::new(
                leaf.tab_bar_rect.x,
                leaf.tab_bar_rect.bottom() - 1.0,
                leaf.tab_bar_rect.width,
                1.0,
            ),
            "IrisDockTabStripBaseline",
            Style::new().background(BORDER_MICRON),
        );

        // Pre-measure all tabs in the current leaf to determine layout tier
        let tab_count = leaf.tabs.len();
        let bar_width = leaf.tab_bar_rect.width;

        let measurements: Vec<(f32, Option<[f32; 4]>, String, bool)> = leaf
            .tabs
            .iter()
            .map(|panel| {
                let atlas_icon = panel.atlas_icon();
                let char_count = panel.title().chars().count();
                let is_closeable = *panel != PanelId::Viewport;
                let close_w = if is_closeable { 18.0 } else { 0.0 };
                let icon_w = if atlas_icon.is_some() { 18.0 } else { 0.0 };
                let title = if atlas_icon.is_some() {
                    panel.title().to_string()
                } else {
                    format!("{} {}", panel.icon(), panel.title())
                };
                let natural_w =
                    (16.0 + icon_w + (char_count as f32) * 6.8 + close_w + 14.0).clamp(52.0, 170.0);
                (natural_w, atlas_icon, title, is_closeable)
            })
            .collect();

        let total_natural_w: f32 = measurements.iter().map(|m| m.0).sum();

        // Determine hybrid layout tier: Natural vs Proportional Shrink vs Overflow Chevron
        let (has_chevron, visible_start, visible_end, tab_widths) = if total_natural_w <= bar_width
        {
            (
                false,
                0,
                tab_count,
                measurements.iter().map(|m| m.0).collect(),
            )
        } else if (tab_count as f32 * MIN_SHRUNK_TAB_WIDTH) <= bar_width {
            let shrink_ratio = (bar_width / total_natural_w.max(1.0)).clamp(0.0, 1.0);
            (
                false,
                0,
                tab_count,
                measurements
                    .iter()
                    .map(|m| (m.0 * shrink_ratio).max(MIN_SHRUNK_TAB_WIDTH))
                    .collect(),
            )
        } else {
            let available_for_tabs = (bar_width - CHEVRON_WIDTH).max(0.0);
            let max_visible =
                ((available_for_tabs / MIN_SHRUNK_TAB_WIDTH).floor() as usize).clamp(1, tab_count);
            let active_idx = leaf.active_tab.min(tab_count.saturating_sub(1));
            let (start, end) = if active_idx < max_visible {
                (0, max_visible)
            } else {
                let end = (active_idx + 1).min(tab_count);
                (end.saturating_sub(max_visible), end)
            };
            let count = (end - start).max(1);
            (
                true,
                start,
                end,
                vec![(available_for_tabs / count as f32).max(MIN_SHRUNK_TAB_WIDTH); count],
            )
        };

        // Tabs start flush at the left boundary of the tab bar
        let mut current_tab_x = leaf.tab_bar_rect.x;
        let max_tab_strip_right = if has_chevron {
            leaf.tab_bar_rect.right() - CHEVRON_WIDTH
        } else {
            leaf.tab_bar_rect.right()
        };

        for (offset, index) in (visible_start..visible_end).enumerate() {
            let panel = &leaf.tabs[index];
            let &(desired_natural_w, atlas_icon, ref title, is_closeable) = &measurements[index];
            let active = index == leaf.active_tab;

            let desired_w = tab_widths.get(offset).copied().unwrap_or(desired_natural_w);
            let tab_w = desired_w.min((max_tab_strip_right - current_tab_x).max(0.0));
            if tab_w <= 1.0 {
                continue;
            }

            let tab_rect = Rect::new(
                current_tab_x,
                leaf.tab_bar_rect.y,
                tab_w,
                leaf.tab_bar_rect.height,
            );
            let is_tab_hovered = !is_cursor_occluded && tab_rect.contains_point(cursor_pos);

            let bg_color = if active {
                ELEVATION_3_ACTIVE_PILL
            } else if is_tab_hovered {
                ELEVATION_3_HOVERED_PILL
            } else {
                ELEVATION_2_HEADER
            };

            // Tab studio background with rounded top corners, flat on bottom baseline
            let tab_id = add_rect_node(
                tree,
                strip_id,
                tab_rect,
                "IrisDockTab",
                Style::new()
                    .background(bg_color)
                    .corner_radii(CornerRadii::new(5.0, 5.0, 0.0, 0.0))
                    .clip_children(true),
            );

            // Active 2px cyan line anchored flush to the bottom baseline and spanning full tab width
            if active {
                add_rect_node(
                    tree,
                    tab_id,
                    Rect::new(tab_rect.x, tab_rect.bottom() - 2.0, tab_rect.width, 2.0),
                    "IrisDockTabActiveLine",
                    Style::new().background(ACCENT_CYAN),
                );
            }

            // Optional GPU atlas icon quad (16x16 crisp texels, blue when active, white when idle)
            if let Some(uv) = atlas_icon
                && tab_w >= 36.0
            {
                let tint = if active {
                    ACCENT_CYAN
                } else if is_tab_hovered {
                    Color::WHITE
                } else {
                    TEXT_REGULAR
                };
                add_icon_node(
                    tree,
                    tab_id,
                    Rect::new(tab_rect.x + 7.0, tab_rect.y + 5.0, 16.0, 16.0),
                    "IrisDockTabAtlasIcon",
                    uv,
                    tint,
                );
            }

            // Tab text label (clamped within remaining tab width)
            let text_offset_x = if atlas_icon.is_some() { 26.0 } else { 6.0 };
            let render_close = is_closeable && (active || is_tab_hovered || tab_w >= 84.0);
            let close_reserved = if render_close { 22.0 } else { 4.0 };
            let text_width = (tab_rect.width - text_offset_x - close_reserved).max(0.0);
            if text_width >= 8.0 {
                let text_rect = Rect::new(
                    tab_rect.x + text_offset_x,
                    tab_rect.y + 4.0,
                    text_width,
                    18.0,
                );
                let text_col = if active {
                    ACCENT_CYAN
                } else if is_tab_hovered {
                    TEXT_BRIGHT
                } else {
                    TEXT_MUTED
                };
                add_text_node(tree, tab_id, text_rect, title, text_col, TextAlign::Left);
            }

            // Tab close button (if closeable and tab width is sufficient)
            if render_close {
                let close_rect = Rect::new(
                    tab_rect.right() - 19.0,
                    tab_rect.y + 3.0,
                    15.0,
                    tab_rect.height - 6.0,
                );
                let is_close_hovered = !is_cursor_occluded && close_rect.contains_point(cursor_pos);
                if is_close_hovered {
                    add_rect_node(
                        tree,
                        tab_id,
                        close_rect,
                        "IrisDockTabCloseHoverPill",
                        Style::new()
                            .background(Color::rgba(0.9, 0.2, 0.2, 0.25))
                            .corner_radii(CornerRadii::all(3.0)),
                    );
                }
                let close_col = if is_close_hovered {
                    Color::rgba(1.0, 0.45, 0.45, 1.0)
                } else {
                    Color::rgba(0.65, 0.68, 0.75, 0.85)
                };
                add_text_node(tree, tab_id, close_rect, "✖", close_col, TextAlign::Center);
                frame.close_targets.push(NativeDockCloseTarget {
                    leaf: leaf.node_id,
                    tab_index: index,
                    rect: close_rect,
                });
            }

            frame.tab_targets.push(NativeDockTabTarget {
                leaf: leaf.node_id,
                tab_index: index,
                panel: *panel,
                rect: tab_rect,
                leaf_rect: leaf.rect,
            });

            current_tab_x += tab_w;
        }

        // Render tab overflow chevron button if tabs exceeded available tab bar width
        if has_chevron {
            let chevron_rect = Rect::new(
                leaf.tab_bar_rect.right() - CHEVRON_WIDTH,
                leaf.tab_bar_rect.y,
                CHEVRON_WIDTH,
                leaf.tab_bar_rect.height,
            );
            let is_chevron_hovered = !is_cursor_occluded && chevron_rect.contains_point(cursor_pos);
            let chevron_bg = if is_chevron_hovered {
                ELEVATION_3_HOVERED_PILL
            } else {
                ELEVATION_2_HEADER
            };
            add_rect_node(
                tree,
                strip_id,
                chevron_rect,
                "IrisDockTabChevron",
                Style::new().background(chevron_bg),
            );
            let chevron_text_col = if is_chevron_hovered {
                ACCENT_CYAN
            } else {
                TEXT_MUTED
            };
            add_text_node(
                tree,
                strip_id,
                chevron_rect,
                "▾",
                chevron_text_col,
                TextAlign::Center,
            );
            frame.chevron_targets.push(NativeDockChevronTarget {
                leaf: leaf.node_id,
                rect: chevron_rect,
            });
        }

        if let Some(panel) = leaf.tabs.get(leaf.active_tab) {
            frame.panel_rects.push((*panel, leaf.content_rect));
        }
    }

    // 2. Render all splitter dividers
    for splitter in computed.splitters {
        let is_splitter_active = layout_state
            .dock_state
            .active_splitter
            .as_ref()
            .is_some_and(|s| s.node_id == splitter.node_id);
        let is_hovered = !is_cursor_occluded
            && splitter.rect.contains_point(cursor_pos)
            && !is_dragging_splitter;
        let splitter_col = if is_splitter_active || is_hovered {
            SPLITTER_ACTIVE
        } else {
            SPLITTER_IDLE
        };

        let visual_rect = if splitter.direction == SplitDirection::Horizontal {
            Rect::new(
                splitter.rect.x,
                splitter.rect.y + NATIVE_DOCK_TAB_HEIGHT,
                splitter.rect.width,
                (splitter.rect.height - NATIVE_DOCK_TAB_HEIGHT).max(0.0),
            )
        } else {
            splitter.rect
        };

        add_rect_node(
            tree,
            parent,
            visual_rect,
            "IrisDockSplitter",
            Style::new().background(splitter_col),
        );
        let total_dim = if splitter.direction == SplitDirection::Horizontal {
            workspace_rect.width
        } else {
            workspace_rect.height
        }
        .max(1.0);
        frame.splitter_targets.push(NativeDockSplitterTarget {
            node: splitter.node_id,
            direction: splitter.direction,
            rect: splitter.rect,
            total_dimension: total_dim,
        });
    }

    frame
}

/// Adds an absolutely positioned native docking rectangle to the retained Iris tree.
pub(crate) fn add_rect_node(
    tree: &mut UiTree,
    parent: WidgetId,
    rect: Rect,
    name: &str,
    style: Style,
) -> WidgetId {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name(name);
        node.computed_rect = rect;
        node.set_style(style);
    }
    let _ = tree.add_child(parent, id);
    id
}

/// Adds a native docking tab label to the retained Iris tree.
pub(crate) fn add_text_node(
    tree: &mut UiTree,
    parent: WidgetId,
    rect: Rect,
    text: &str,
    color: Color,
    alignment: TextAlign,
) {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name("IrisDockTabLabel");
        node.computed_rect = rect;
        node.set_text(text);
        node.set_text_properties(12.0, 16.0, color, alignment);
        node.set_style(Style::new().clip_children(true));
    }
    let _ = tree.add_child(parent, id);
}

/// Adds a textured GPU atlas icon node to the retained Iris tree.
pub(crate) fn add_icon_node(
    tree: &mut UiTree,
    parent: WidgetId,
    rect: Rect,
    name: &str,
    uv: [f32; 4],
    tint: Color,
) {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name(name);
        node.set_texture_uv(uv);
        node.computed_rect = rect;
        node.set_texture_tint(tint);
    }
    let _ = tree.add_child(parent, id);
}