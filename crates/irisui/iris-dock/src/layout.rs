// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Geometric layout computation for split containers, leaves, tab bars, and splitters.

use crate::tree::{DockNode, DockNodeId, DockTree, SplitDirection};
use iris_core::Rect;

/// Geometric layout info computed for a terminal leaf hosting tabbed panels.
#[derive(Debug, Clone)]
pub struct LeafLayoutInfo<'a, T> {
    /// Generational identifier of the leaf node.
    pub node_id: DockNodeId,
    /// Total bounding rectangle occupied by the entire leaf (tab bar + content).
    pub rect: Rect,
    /// Bounding rectangle occupied by the top tab strip.
    pub tab_bar_rect: Rect,
    /// Bounding rectangle available for the active panel's inner content.
    pub content_rect: Rect,
    /// References to the tabs hosted inside this leaf.
    pub tabs: &'a [T],
    /// Index of the active tab.
    pub active_tab: usize,
}

/// Geometric layout info computed for a draggable splitter divider between two panes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitterLayoutInfo {
    /// Identifier of the split container node owning this splitter.
    pub node_id: DockNodeId,
    /// Partition axis of the splitter.
    pub direction: SplitDirection,
    /// Interactive hit rectangle of the splitter divider.
    pub rect: Rect,
    /// Current division ratio.
    pub ratio: f32,
}

/// Complete geometric layout of all visible leaves and interactive splitters.
#[derive(Debug, Clone)]
pub struct ComputedDockLayout<'a, T> {
    /// Computed leaf layout regions.
    pub leaves: Vec<LeafLayoutInfo<'a, T>>,
    /// Computed draggable splitter boundaries.
    pub splitters: Vec<SplitterLayoutInfo>,
}

impl<'a, T> Default for ComputedDockLayout<'a, T> {
    fn default() -> Self {
        Self {
            leaves: Vec::new(),
            splitters: Vec::new(),
        }
    }
}

/// Computes pixel-precise bounding boxes for all docking tree leaves and splitters.
pub fn compute_dock_layout<'a, T>(
    tree: &'a DockTree<T>,
    available_rect: Rect,
    splitter_thickness: f32,
    tab_bar_height: f32,
) -> ComputedDockLayout<'a, T> {
    let mut layout = ComputedDockLayout::default();

    if let Some(root_id) = tree.root() {
        compute_recursive(
            tree,
            root_id,
            available_rect,
            splitter_thickness,
            tab_bar_height,
            &mut layout,
        );
    }

    layout
}

fn compute_recursive<'a, T>(
    tree: &'a DockTree<T>,
    node_id: DockNodeId,
    rect: Rect,
    splitter_thickness: f32,
    tab_bar_height: f32,
    layout: &mut ComputedDockLayout<'a, T>,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };

    match *node {
        DockNode::Leaf {
            ref tabs,
            active_tab,
        } => {
            let clamped_tab_height = tab_bar_height.min(rect.height);
            let tab_bar_rect = Rect::new(rect.x, rect.y, rect.width, clamped_tab_height);
            let content_rect = Rect::new(
                rect.x,
                rect.y + clamped_tab_height,
                rect.width,
                (rect.height - clamped_tab_height).max(0.0),
            );

            layout.leaves.push(LeafLayoutInfo {
                node_id,
                rect,
                tab_bar_rect,
                content_rect,
                tabs,
                active_tab,
            });
        }
        DockNode::Split {
            direction,
            ratio,
            first,
            second,
        } => match direction {
            SplitDirection::Horizontal => {
                let usable_width = (rect.width - splitter_thickness).max(0.0);
                let first_width = usable_width * ratio;
                let second_width = usable_width - first_width;

                let first_rect = Rect::new(rect.x, rect.y, first_width, rect.height);
                let splitter_rect = Rect::new(
                    rect.x + first_width,
                    rect.y,
                    splitter_thickness,
                    rect.height,
                );
                let second_rect = Rect::new(
                    rect.x + first_width + splitter_thickness,
                    rect.y,
                    second_width,
                    rect.height,
                );

                layout.splitters.push(SplitterLayoutInfo {
                    node_id,
                    direction,
                    rect: splitter_rect,
                    ratio,
                });

                compute_recursive(
                    tree,
                    first,
                    first_rect,
                    splitter_thickness,
                    tab_bar_height,
                    layout,
                );
                compute_recursive(
                    tree,
                    second,
                    second_rect,
                    splitter_thickness,
                    tab_bar_height,
                    layout,
                );
            }
            SplitDirection::Vertical => {
                let usable_height = (rect.height - splitter_thickness).max(0.0);
                let first_height = usable_height * ratio;
                let second_height = usable_height - first_height;

                let first_rect = Rect::new(rect.x, rect.y, rect.width, first_height);
                let splitter_rect = Rect::new(
                    rect.x,
                    rect.y + first_height,
                    rect.width,
                    splitter_thickness,
                );
                let second_rect = Rect::new(
                    rect.x,
                    rect.y + first_height + splitter_thickness,
                    rect.width,
                    second_height,
                );

                layout.splitters.push(SplitterLayoutInfo {
                    node_id,
                    direction,
                    rect: splitter_rect,
                    ratio,
                });

                compute_recursive(
                    tree,
                    first,
                    first_rect,
                    splitter_thickness,
                    tab_bar_height,
                    layout,
                );
                compute_recursive(
                    tree,
                    second,
                    second_rect,
                    splitter_thickness,
                    tab_bar_height,
                    layout,
                );
            }
        },
    }
}