// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dock Chrome Hit Target Extraction Engine
//!
//! Computes precise geometric hit targets for tab headers, close buttons,
//! overflow chevrons, and splitter dividers directly from the docking layout
//! without requiring retained UI widget tree allocations.

use crate::chrome::{
    DockChevronTarget, DockChromeFrame, DockCloseTarget, DockSplitterTarget, DockTabTarget,
};
use crate::layout::compute_dock_layout;
use crate::tab_viewer::TabViewer;
use crate::tree::{DockTree, SplitDirection};
use iris_core::geometry::Rect;

/// Extracts all interactive click and drag targets for the dock area.
///
/// Evaluates exact tab title measurements, proportional tab shrinkage,
/// overflow chevron thresholds, and splitter divider bounding geometries.
///
/// # Arguments
/// * `dock_tree` - The active binary split docking tree.
/// * `workspace_rect` - Bounding bounds available for the docked panels.
/// * `splitter_thickness` - Width/height of divider splitters in logical pixels.
/// * `tab_bar_height` - Height of leaf tab strips in logical pixels.
/// * `viewer` - Reference to the tab viewer providing tab titles and closeability.
pub fn compute_dock_interaction_targets<T: Clone + Copy, V: TabViewer<T>>(
    dock_tree: &DockTree<T>,
    workspace_rect: Rect,
    splitter_thickness: f32,
    tab_bar_height: f32,
    viewer: &V,
) -> DockChromeFrame<T> {
    let computed = compute_dock_layout(
        dock_tree,
        workspace_rect,
        splitter_thickness,
        tab_bar_height,
    );
    let mut frame = DockChromeFrame::default();

    // 1. Process all leaf panel tab headers, close buttons, and chevrons
    for leaf in &computed.leaves {
        if leaf.tabs.is_empty() {
            continue;
        }

        let tab_count = leaf.tabs.len();
        let bar_width = leaf.tab_bar_rect.width;

        let measurements: Vec<(f32, bool)> = leaf
            .tabs
            .iter()
            .map(|tab| {
                let atlas_icon = viewer.atlas_icon(tab);
                let title = if atlas_icon.is_some() {
                    viewer.raw_title(tab)
                } else {
                    viewer.title(tab)
                };
                let char_count = title.chars().count();
                let is_closeable = viewer.closeable(tab);
                let close_w = if is_closeable { 18.0f32 } else { 0.0f32 };
                let icon_w = if atlas_icon.is_some() {
                    18.0f32
                } else {
                    0.0f32
                };
                let natural_w: f32 =
                    (16.0f32 + icon_w + (char_count as f32) * 6.8f32 + close_w + 14.0f32)
                        .clamp(52.0f32, 170.0f32);
                (natural_w, is_closeable)
            })
            .collect();

        let total_natural_w: f32 = measurements.iter().map(|m| m.0).sum();
        const MIN_SHRUNK_TAB_WIDTH: f32 = 64.0;
        const CHEVRON_WIDTH: f32 = 28.0;

        let (has_chevron, visible_start, visible_end, tab_widths): (bool, usize, usize, Vec<f32>) =
            if total_natural_w <= bar_width {
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
                let max_visible = ((available_for_tabs / MIN_SHRUNK_TAB_WIDTH).floor() as usize)
                    .clamp(1, tab_count);
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

        let mut current_tab_x = leaf.tab_bar_rect.x;
        for (i, &w) in (visible_start..visible_end).zip(tab_widths.iter()) {
            let tab = &leaf.tabs[i];
            let is_closeable = measurements[i].1;
            let tab_rect = Rect::new(
                current_tab_x,
                leaf.tab_bar_rect.y,
                w,
                leaf.tab_bar_rect.height,
            );

            if is_closeable && w >= 84.0 {
                let close_rect = Rect::new(tab_rect.right() - 19.0, tab_rect.y + 3.0, 16.0, 16.0);
                frame.close_targets.push(DockCloseTarget {
                    leaf: leaf.node_id,
                    tab_index: i,
                    rect: close_rect,
                });
            }

            frame.tab_targets.push(DockTabTarget {
                leaf: leaf.node_id,
                tab_index: i,
                tab: *tab,
                rect: tab_rect,
                leaf_rect: leaf.rect,
            });

            current_tab_x += w;
        }

        if has_chevron {
            let chevron_rect = Rect::new(
                leaf.tab_bar_rect.right() - CHEVRON_WIDTH,
                leaf.tab_bar_rect.y,
                CHEVRON_WIDTH,
                leaf.tab_bar_rect.height,
            );
            frame.chevron_targets.push(DockChevronTarget {
                leaf: leaf.node_id,
                rect: chevron_rect,
            });
        }

        if let Some(panel) = leaf.tabs.get(leaf.active_tab) {
            frame.panel_rects.push((*panel, leaf.content_rect));
        }
    }

    // 2. Process all splitter dividers
    for splitter in computed.splitters {
        let total_dim = if splitter.direction == SplitDirection::Horizontal {
            workspace_rect.width
        } else {
            workspace_rect.height
        }
        .max(1.0);

        frame.splitter_targets.push(DockSplitterTarget {
            node: splitter.node_id,
            direction: splitter.direction,
            rect: splitter.rect,
            total_dimension: total_dim,
        });
    }

    frame
}