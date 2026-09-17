// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Types, geometric targets, parameters, and constants for native Iris docking chrome.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::{DockNodeId, SplitDirection};
use irisui::prelude::*;

/// Height of each native docking tab strip in logical editor pixels.
pub const NATIVE_DOCK_TAB_HEIGHT: f32 = 26.0;
/// Width/Thickness of partition splitter lines in logical pixels.
pub const SPLITTER_THICKNESS: f32 = 3.0;
/// Minimum proportional width in logical pixels for docked tabs before triggering overflow handling.
pub const MIN_SHRUNK_TAB_WIDTH: f32 = 68.0;
/// Width in logical pixels reserved for the tab overflow chevron button.
pub const CHEVRON_WIDTH: f32 = 24.0;

/// Native dock frame geometry consumed by Iris panel builders and editor hit testing.
/// The frame is rebuilt from the authoritative Iris split tree every render frame. It keeps
/// panel content coordinates independent from any legacy immediate-mode dock renderer.
#[derive(Debug, Clone, Default)]
pub struct NativeDockFrame {
    /// Content rectangle assigned to each currently active panel.
    pub panel_rects: Vec<(PanelId, Rect)>,
    /// Click targets for Iris-rendered tab labels and their source leaf/index pairs.
    pub tab_targets: Vec<NativeDockTabTarget>,
    /// Click targets for tab close `✖` buttons.
    pub close_targets: Vec<NativeDockCloseTarget>,
    /// Drag targets for Iris-rendered split dividers.
    pub splitter_targets: Vec<NativeDockSplitterTarget>,
    /// Click targets for tab strip overflow chevron buttons (`▾`).
    pub chevron_targets: Vec<NativeDockChevronTarget>,
    /// Click targets for open overflow dropdown menu items.
    pub overflow_item_targets: Vec<NativeDockOverflowItemTarget>,
    /// Active bounding rectangle of the open tab overflow dropdown menu (if any), used for overlay hit testing.
    pub active_overflow_rect: Option<Rect>,
}

impl NativeDockFrame {
    /// Returns the active content rectangle for a panel, if its tab is selected in an Iris leaf.
    pub fn panel_rect(&self, panel: PanelId) -> Option<Rect> {
        self.panel_rects
            .iter()
            .find_map(|(candidate, rect)| (*candidate == panel).then_some(*rect))
    }
}

/// Native Iris tab hit target linked to a stable dock tree leaf and tab index.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockTabTarget {
    /// Leaf that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Panel kind.
    pub panel: PanelId,
    /// Logical editor-space click bounds.
    pub rect: Rect,
    /// Bounding rectangle of the owning leaf pane.
    pub leaf_rect: Rect,
}

/// Native Iris tab close button hit target.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockCloseTarget {
    /// Leaf that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Logical editor-space click bounds.
    pub rect: Rect,
}

/// Native Iris dock tab overflow chevron button hit target.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockChevronTarget {
    /// Leaf that owns the overflowing tab bar.
    pub leaf: DockNodeId,
    /// Logical editor-space click bounds of the chevron button.
    pub rect: Rect,
}

/// Native Iris dock tab overflow dropdown menu item target.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockOverflowItemTarget {
    /// Leaf owning the target tab.
    pub leaf: DockNodeId,
    /// Tab index within the leaf.
    pub tab_index: usize,
    /// Panel kind.
    pub panel: PanelId,
    /// Logical editor-space click bounds of the dropdown item.
    pub rect: Rect,
}

/// Native Iris splitter hit target linked to a stable split node.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockSplitterTarget {
    /// Split node that owns the divider.
    pub node: DockNodeId,
    /// Axis along which the divider moves.
    pub direction: SplitDirection,
    /// Logical editor-space drag bounds.
    pub rect: Rect,
    /// Parent-axis dimension used to normalize drag distance into a split ratio.
    pub total_dimension: f32,
}

/// Parameters passed to build the floating dock tab overflow dropdown menu.
pub struct NativeDockOverflowMenuParams<'a> {
    /// Generational identifier of the leaf node owning the overflowing tab bar.
    pub leaf_id: DockNodeId,
    /// Bounding rectangle of the chevron anchor button in logical pixels.
    pub anchor_rect: Rect,
    /// Authoritative editor panel layout state.
    pub layout_state: &'a PanelLayoutState,
    /// Current logical mouse cursor position.
    pub cursor_pos: Point,
    /// Whether the cursor is occluded by top-level modals or popups.
    pub is_cursor_occluded: bool,
}