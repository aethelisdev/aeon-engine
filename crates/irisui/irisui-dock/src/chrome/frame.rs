// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dock chrome hit-target descriptors, frame output geometry, and build parameters.

use crate::tree::{DockNodeId, DockTree};
use irisui_core::geometry::{Point, Rect};
use irisui_core::id::WidgetId;

pub use crate::area::splitter::{
    DEFAULT_SPLITTER_HIT_MARGIN, DEFAULT_SPLITTER_THICKNESS, DEFAULT_SPLITTER_TOTAL_HIT_ZONE,
    DockSplitterTarget, hit_splitter,
};
pub use crate::style::DockChromeStyle;

/// Native Iris tab hit target linked to a stable dock tree leaf and tab index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockTabTarget<T> {
    /// Leaf node that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Payload identifier of the tab.
    pub tab: T,
    /// Logical editor-space click bounds.
    pub rect: Rect,
    /// Bounding rectangle of the owning leaf pane.
    pub leaf_rect: Rect,
}

impl<T> DockTabTarget<T> {
    /// Checks whether the point is contained within this tab target's bounds.
    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains_point(point)
    }
}

/// Native Iris tab close button hit target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockCloseTarget {
    /// Leaf that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Logical editor-space click bounds.
    pub rect: Rect,
}

impl DockCloseTarget {
    /// Checks whether the point is contained within this close target's bounds.
    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains_point(point)
    }
}

/// Native Iris dock tab overflow chevron button hit target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockChevronTarget {
    /// Leaf that owns the overflowing tab bar.
    pub leaf: DockNodeId,
    /// Logical editor-space click bounds of the chevron button.
    pub rect: Rect,
}

impl DockChevronTarget {
    /// Checks whether the point is contained within this chevron target's bounds.
    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains_point(point)
    }
}

/// Native dock chrome geometry consumed by host panel builders and editor hit testing.
///
/// Rebuilt from the authoritative split tree every render frame.
#[derive(Debug, Clone)]
pub struct DockChromeFrame<T> {
    /// Content rectangle assigned to each currently active panel.
    pub panel_rects: Vec<(T, Rect)>,
    /// Generational widget node IDs assigned to each currently active panel leaf container.
    pub leaf_containers: Vec<(T, WidgetId)>,
    /// Click targets for rendered tab labels and their source leaf/index pairs.
    pub tab_targets: Vec<DockTabTarget<T>>,
    /// Click targets for tab close `✖` buttons.
    pub close_targets: Vec<DockCloseTarget>,
    /// Drag targets for rendered split dividers.
    pub splitter_targets: Vec<DockSplitterTarget>,
    /// Click targets for tab strip overflow chevron buttons (`▾`).
    pub chevron_targets: Vec<DockChevronTarget>,
    /// Bounding rectangles of active floating windows.
    pub floating_rects: Vec<Rect>,
}

impl<T> Default for DockChromeFrame<T> {
    fn default() -> Self {
        Self {
            panel_rects: Vec::new(),
            leaf_containers: Vec::new(),
            tab_targets: Vec::new(),
            close_targets: Vec::new(),
            splitter_targets: Vec::new(),
            chevron_targets: Vec::new(),
            floating_rects: Vec::new(),
        }
    }
}

impl<T: PartialEq + Copy> DockChromeFrame<T> {
    /// Returns the active content rectangle for a panel, if its tab is selected in a leaf.
    pub fn panel_rect(&self, target: T) -> Option<Rect> {
        self.panel_rects
            .iter()
            .find_map(|(candidate, rect)| (*candidate == target).then_some(*rect))
    }

    /// Returns the active leaf container widget node ID for a panel, if its tab is selected in a leaf.
    pub fn leaf_container(&self, target: T) -> Option<WidgetId> {
        self.leaf_containers
            .iter()
            .find_map(|(candidate, id)| (*candidate == target).then_some(*id))
    }

    /// Returns the tab associated with the specified leaf container widget node ID, if present.
    ///
    /// Enables reverse lookup from event hit targets back to the active panel or tab.
    pub fn tab_for_leaf_container(&self, id: WidgetId) -> Option<T> {
        self.leaf_containers
            .iter()
            .find_map(|(candidate, wid)| (*wid == id).then_some(*candidate))
    }

    /// Returns `true` if the specified widget node ID matches an active leaf container.
    #[inline]
    #[must_use]
    pub fn is_leaf_container(&self, id: WidgetId) -> bool {
        self.leaf_containers.iter().any(|(_, wid)| *wid == id)
    }

    /// Returns an immutable slice of all registered `(Tab, WidgetId)` leaf containers.
    #[inline]
    #[must_use]
    pub fn leaf_containers(&self) -> &[(T, WidgetId)] {
        &self.leaf_containers
    }
}

/// Context parameters passed into [`super::build_dock_chrome`].
///
/// Groups dimensional metrics, tree data, interaction states, and delegates into a single descriptor.
pub struct DockChromeParams<'a, T, V> {
    /// Generational split tree governing the pane hierarchy.
    pub dock_tree: &'a DockTree<T>,
    /// Bounding rectangle available for the entire dock workspace.
    pub workspace_rect: Rect,
    /// Current logical mouse cursor position.
    pub cursor_pos: Point,
    /// Whether the cursor is occluded by modals or foreground dropdown menus.
    pub is_cursor_occluded: bool,
    /// Whether an active divider partition is currently being dragged.
    pub is_dragging_splitter: bool,
    /// Identifier of the specific split node being dragged, if any.
    pub active_splitter_node: Option<DockNodeId>,
    /// Whether an individual tab is currently being dragged.
    pub is_dragging_tab: bool,
    /// Tab viewer lifecycle delegate providing titles, icons, and closeability.
    pub viewer: &'a V,
    /// Visual styling and dimensional metrics.
    pub style: &'a DockChromeStyle,
}