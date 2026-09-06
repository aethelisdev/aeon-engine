// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Geometric layout computation for split containers, leaves, tab bars, and splitters.

use crate::tab_bar::{TabBarLayoutInfo, compute_tab_bar_layout};
use crate::tab_viewer::TabViewer;
use crate::tree::{DockNode, DockNodeId, DockTree, SplitDirection};
use iris_core::Rect;

/// Geometric layout info computed for a terminal leaf hosting tabbed panels.
#[derive(Debug, Clone)]
pub struct LeafLayoutInfo<T> {
    /// Generational identifier of the leaf node.
    pub node_id: DockNodeId,
    /// Total bounding rectangle occupied by the entire leaf (tab bar + content).
    pub rect: Rect,
    /// Bounding rectangle occupied by the top tab strip.
    pub tab_bar_rect: Rect,
    /// Bounding rectangle available for the active panel's inner content.
    pub content_rect: Rect,
    /// Detailed geometric layout info for individual tabs and auxiliary buttons, if computed.
    pub tab_bar_layout: Option<TabBarLayoutInfo>,
    /// Cloned list of tabs hosted inside this leaf.
    pub tabs: Vec<T>,
    /// Index of the active tab.
    pub active_tab: usize,
    /// Whether this leaf is temporarily maximized to fill the entire dock area.
    pub is_maximized: bool,
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

/// Configuration options controlling docking layout calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockLayoutOptions {
    /// Divider thickness in pixels between split panes.
    pub splitter_thickness: f32,
    /// Default height allocated for leaf tab strips.
    pub tab_bar_height: f32,
    /// Identifier of a leaf temporarily maximized to fill the entire dock area.
    pub maximized_leaf: Option<DockNodeId>,
    /// Whether to collapse the tab bar to 0 height when a leaf hosts only 1 tab.
    pub auto_hide_single_tab_bar: bool,
}

impl Default for DockLayoutOptions {
    fn default() -> Self {
        Self {
            splitter_thickness: 4.0,
            tab_bar_height: 26.0,
            maximized_leaf: None,
            auto_hide_single_tab_bar: false,
        }
    }
}

/// Complete geometric layout of all visible leaves and interactive splitters.
#[derive(Debug, Clone)]
pub struct ComputedDockLayout<T> {
    /// Computed leaf layout regions.
    pub leaves: Vec<LeafLayoutInfo<T>>,
    /// Computed draggable splitter boundaries.
    pub splitters: Vec<SplitterLayoutInfo>,
}

impl<T> Default for ComputedDockLayout<T> {
    fn default() -> Self {
        Self {
            leaves: Vec::new(),
            splitters: Vec::new(),
        }
    }
}

/// Computed layout for an independent floating window and its hosted docking tree.
#[derive(Debug, Clone)]
pub struct ComputedFloatingLayout<T> {
    /// Window unique identifier.
    pub window_id: u64,
    /// Absolute outer window boundary.
    pub window_rect: Rect,
    /// Title bar boundary for dragging and control icons.
    pub title_bar_rect: Rect,
    /// Computed dock layout within the floating window's content area.
    pub dock_layout: ComputedDockLayout<T>,
}

/// Computes pixel-precise bounding boxes for all docking tree leaves and splitters using layout options.
pub fn compute_dock_layout_with_options<T: Clone>(
    tree: &DockTree<T>,
    available_rect: Rect,
    options: DockLayoutOptions,
) -> ComputedDockLayout<T> {
    let mut layout = ComputedDockLayout::default();

    if let Some(max_leaf) = options.maximized_leaf
        && let Some(DockNode::Leaf { tabs, active_tab }) = tree.get(max_leaf)
    {
        let eff_tab_height = if options.auto_hide_single_tab_bar && tabs.len() <= 1 {
            0.0
        } else {
            options.tab_bar_height.min(available_rect.height)
        };
        let tab_bar_rect = Rect::new(
            available_rect.x,
            available_rect.y,
            available_rect.width,
            eff_tab_height,
        );
        let content_rect = Rect::new(
            available_rect.x,
            available_rect.y + eff_tab_height,
            available_rect.width,
            (available_rect.height - eff_tab_height).max(0.0),
        );
        layout.leaves.push(LeafLayoutInfo {
            node_id: max_leaf,
            rect: available_rect,
            tab_bar_rect,
            content_rect,
            tab_bar_layout: None,
            tabs: tabs.clone(),
            active_tab: *active_tab,
            is_maximized: true,
        });
        return layout;
    }

    if let Some(root_id) = tree.root() {
        compute_recursive(tree, root_id, available_rect, options, &mut layout);
    }

    layout
}

/// Computes pixel-precise bounding boxes for all docking tree leaves and splitters.
pub fn compute_dock_layout<T: Clone>(
    tree: &DockTree<T>,
    available_rect: Rect,
    splitter_thickness: f32,
    tab_bar_height: f32,
) -> ComputedDockLayout<T> {
    let options = DockLayoutOptions {
        splitter_thickness,
        tab_bar_height,
        maximized_leaf: None,
        auto_hide_single_tab_bar: false,
    };
    compute_dock_layout_with_options(tree, available_rect, options)
}

/// Computes pixel-precise bounding boxes for all docking tree leaves, tab bars, and splitters using a [`TabViewer`] and custom [`DockLayoutOptions`].
pub fn compute_dock_layout_advanced<T: Clone, V: TabViewer<T>>(
    tree: &DockTree<T>,
    available_rect: Rect,
    options: DockLayoutOptions,
    viewer: &V,
) -> ComputedDockLayout<T> {
    let mut layout = ComputedDockLayout::default();

    if let Some(max_leaf) = options.maximized_leaf
        && let Some(DockNode::Leaf { tabs, active_tab }) = tree.get(max_leaf)
    {
        let eff_tab_height = if options.auto_hide_single_tab_bar && tabs.len() <= 1 {
            0.0
        } else {
            options.tab_bar_height.min(available_rect.height)
        };
        let tab_bar_rect = Rect::new(
            available_rect.x,
            available_rect.y,
            available_rect.width,
            eff_tab_height,
        );
        let content_rect = Rect::new(
            available_rect.x,
            available_rect.y + eff_tab_height,
            available_rect.width,
            (available_rect.height - eff_tab_height).max(0.0),
        );
        let tab_bar_layout = if eff_tab_height > 0.0 {
            Some(compute_tab_bar_layout(
                tab_bar_rect,
                tabs,
                *active_tab,
                viewer,
                0.0,
                false,
            ))
        } else {
            None
        };
        layout.leaves.push(LeafLayoutInfo {
            node_id: max_leaf,
            rect: available_rect,
            tab_bar_rect,
            content_rect,
            tab_bar_layout,
            tabs: tabs.clone(),
            active_tab: *active_tab,
            is_maximized: true,
        });
        return layout;
    }

    if let Some(root_id) = tree.root() {
        compute_recursive_viewer(tree, root_id, available_rect, options, viewer, &mut layout);
    }

    layout
}

/// Computes pixel-precise bounding boxes for all docking tree leaves, tab bars, and splitters using a [`TabViewer`].
pub fn compute_dock_layout_with_viewer<T: Clone, V: TabViewer<T>>(
    tree: &DockTree<T>,
    available_rect: Rect,
    splitter_thickness: f32,
    tab_bar_height: f32,
    viewer: &V,
) -> ComputedDockLayout<T> {
    let options = DockLayoutOptions {
        splitter_thickness,
        tab_bar_height,
        maximized_leaf: None,
        auto_hide_single_tab_bar: false,
    };
    compute_dock_layout_advanced(tree, available_rect, options, viewer)
}

/// Computes geometric layouts for a list of floating windows.
pub fn compute_floating_layouts<T: Clone, V: TabViewer<T>>(
    windows: &[crate::floating::FloatingWindow<T>],
    title_bar_height: f32,
    options: DockLayoutOptions,
    viewer: &V,
) -> Vec<ComputedFloatingLayout<T>> {
    let mut layouts = Vec::with_capacity(windows.len());
    for win in windows {
        if win.is_minimized {
            continue;
        }
        let title_bar_rect = win.title_bar_rect(title_bar_height);
        let content_rect = win.content_rect(title_bar_height);
        let dock_layout = compute_dock_layout_advanced(&win.tree, content_rect, options, viewer);
        layouts.push(ComputedFloatingLayout {
            window_id: win.id,
            window_rect: win.rect,
            title_bar_rect,
            dock_layout,
        });
    }
    layouts
}

fn compute_recursive<T: Clone>(
    tree: &DockTree<T>,
    node_id: DockNodeId,
    rect: Rect,
    options: DockLayoutOptions,
    layout: &mut ComputedDockLayout<T>,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };

    match *node {
        DockNode::Leaf {
            ref tabs,
            active_tab,
        } => {
            let clamped_tab_height = if options.auto_hide_single_tab_bar && tabs.len() <= 1 {
                0.0
            } else {
                options.tab_bar_height.min(rect.height)
            };
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
                tab_bar_layout: None,
                tabs: tabs.clone(),
                active_tab,
                is_maximized: false,
            });
        }
        DockNode::Split {
            direction,
            ratio,
            first,
            second,
        } => match direction {
            SplitDirection::Horizontal => {
                let usable_width = (rect.width - options.splitter_thickness).max(0.0);
                let first_width = usable_width * ratio;
                let second_width = usable_width - first_width;

                let first_rect = Rect::new(rect.x, rect.y, first_width, rect.height);
                let splitter_rect = Rect::new(
                    rect.x + first_width,
                    rect.y,
                    options.splitter_thickness,
                    rect.height,
                );
                let second_rect = Rect::new(
                    rect.x + first_width + options.splitter_thickness,
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

                compute_recursive(tree, first, first_rect, options, layout);
                compute_recursive(tree, second, second_rect, options, layout);
            }
            SplitDirection::Vertical => {
                let usable_height = (rect.height - options.splitter_thickness).max(0.0);
                let first_height = usable_height * ratio;
                let second_height = usable_height - first_height;

                let first_rect = Rect::new(rect.x, rect.y, rect.width, first_height);
                let splitter_rect = Rect::new(
                    rect.x,
                    rect.y + first_height,
                    rect.width,
                    options.splitter_thickness,
                );
                let second_rect = Rect::new(
                    rect.x,
                    rect.y + first_height + options.splitter_thickness,
                    rect.width,
                    second_height,
                );

                layout.splitters.push(SplitterLayoutInfo {
                    node_id,
                    direction,
                    rect: splitter_rect,
                    ratio,
                });

                compute_recursive(tree, first, first_rect, options, layout);
                compute_recursive(tree, second, second_rect, options, layout);
            }
        },
    }
}

fn compute_recursive_viewer<T: Clone, V: TabViewer<T>>(
    tree: &DockTree<T>,
    node_id: DockNodeId,
    rect: Rect,
    options: DockLayoutOptions,
    viewer: &V,
    layout: &mut ComputedDockLayout<T>,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };

    match *node {
        DockNode::Leaf {
            ref tabs,
            active_tab,
        } => {
            let clamped_tab_height = if options.auto_hide_single_tab_bar && tabs.len() <= 1 {
                0.0
            } else {
                options.tab_bar_height.min(rect.height)
            };
            let tab_bar_rect = Rect::new(rect.x, rect.y, rect.width, clamped_tab_height);
            let content_rect = Rect::new(
                rect.x,
                rect.y + clamped_tab_height,
                rect.width,
                (rect.height - clamped_tab_height).max(0.0),
            );

            let tab_bar_layout = if clamped_tab_height > 0.0 {
                Some(compute_tab_bar_layout(
                    tab_bar_rect,
                    tabs,
                    active_tab,
                    viewer,
                    0.0,
                    true,
                ))
            } else {
                None
            };

            layout.leaves.push(LeafLayoutInfo {
                node_id,
                rect,
                tab_bar_rect,
                content_rect,
                tab_bar_layout,
                tabs: tabs.clone(),
                active_tab,
                is_maximized: false,
            });
        }
        DockNode::Split {
            direction,
            ratio,
            first,
            second,
        } => match direction {
            SplitDirection::Horizontal => {
                let usable_width = (rect.width - options.splitter_thickness).max(0.0);
                let first_width = usable_width * ratio;
                let second_width = usable_width - first_width;

                let first_rect = Rect::new(rect.x, rect.y, first_width, rect.height);
                let splitter_rect = Rect::new(
                    rect.x + first_width,
                    rect.y,
                    options.splitter_thickness,
                    rect.height,
                );
                let second_rect = Rect::new(
                    rect.x + first_width + options.splitter_thickness,
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

                compute_recursive_viewer(tree, first, first_rect, options, viewer, layout);
                compute_recursive_viewer(tree, second, second_rect, options, viewer, layout);
            }
            SplitDirection::Vertical => {
                let usable_height = (rect.height - options.splitter_thickness).max(0.0);
                let first_height = usable_height * ratio;
                let second_height = usable_height - first_height;

                let first_rect = Rect::new(rect.x, rect.y, rect.width, first_height);
                let splitter_rect = Rect::new(
                    rect.x,
                    rect.y + first_height,
                    rect.width,
                    options.splitter_thickness,
                );
                let second_rect = Rect::new(
                    rect.x,
                    rect.y + first_height + options.splitter_thickness,
                    rect.width,
                    second_height,
                );

                layout.splitters.push(SplitterLayoutInfo {
                    node_id,
                    direction,
                    rect: splitter_rect,
                    ratio,
                });

                compute_recursive_viewer(tree, first, first_rect, options, viewer, layout);
                compute_recursive_viewer(tree, second, second_rect, options, viewer, layout);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab_viewer::SimpleTabViewer;

    #[test]
    fn test_compute_dock_layout_maximization() {
        let mut tree = DockTree::new();
        let leaf_a = tree.create_leaf(vec!["Tab A"]);
        tree.set_root(leaf_a);
        let (leaf_first, _leaf_second) = tree
            .split(leaf_a, SplitDirection::Horizontal, 0.5, vec!["Tab B"])
            .expect("Split succeeds");

        let available = Rect::new(0.0, 0.0, 800.0, 600.0);
        let viewer = SimpleTabViewer;

        // Without maximization: 2 leaves, 1 splitter
        let normal = compute_dock_layout_with_viewer(&tree, available, 4.0, 26.0, &viewer);
        assert_eq!(normal.leaves.len(), 2);
        assert_eq!(normal.splitters.len(), 1);
        assert!(!normal.leaves[0].is_maximized);

        // With maximization: leaf_a takes 100% of available rect, 0 splitters
        let opt = DockLayoutOptions {
            splitter_thickness: 4.0,
            tab_bar_height: 26.0,
            maximized_leaf: Some(leaf_first),
            auto_hide_single_tab_bar: false,
        };
        let maxed = compute_dock_layout_advanced(&tree, available, opt, &viewer);
        assert_eq!(maxed.leaves.len(), 1);
        assert_eq!(maxed.splitters.len(), 0);
        assert!(maxed.leaves[0].is_maximized);
        assert_eq!(maxed.leaves[0].rect, available);
    }

    #[test]
    fn test_auto_hide_single_tab_bar() {
        let mut tree = DockTree::new();
        let leaf = tree.create_leaf(vec!["SingleTab"]);
        tree.set_root(leaf);

        let available = Rect::new(0.0, 0.0, 800.0, 600.0);
        let viewer = SimpleTabViewer;

        let opt = DockLayoutOptions {
            splitter_thickness: 4.0,
            tab_bar_height: 26.0,
            maximized_leaf: None,
            auto_hide_single_tab_bar: true,
        };
        let layout = compute_dock_layout_advanced(&tree, available, opt, &viewer);
        assert_eq!(layout.leaves.len(), 1);
        assert_eq!(layout.leaves[0].tab_bar_rect.height, 0.0);
        assert_eq!(layout.leaves[0].content_rect.height, 600.0);
    }
}