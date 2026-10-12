// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical TreeView Data Model, Indentation & Connector Line Math
//!
//! Provides the mathematical geometry foundation, branch connector coordinates,
//! and generic state management for hierarchical tree views (e.g. scene hierarchies,
//! virtual folder trees, and AST structures).
//!
//! ## Architectural Guarantees
//! - **Zero Per-Frame Heap Allocation**: Connector calculation operates purely on stack
//!   geometries via POD structures and stack callbacks ([`for_each_tree_connector_line`]).
//! - **Mathematical Indentation Invariants**: Child node items are strictly indented
//!   relative to their ancestor depth, guaranteeing non-occluding alignment between
//!   horizontal connector arm tips and node icons.
//! - **Generic Identity State**: [`TreeState`] manages expansion and multi-selection
//!   state for any type implementing [`Eq`] + [`Hash`] + [`Clone`].
//!

use iris_core::Color;
use std::collections::HashSet;
use std::hash::Hash;

/// Standard step width in physical pixels added per nesting level depth (`14.0 px`).
pub const DEFAULT_TREE_INDENT_STEP: f32 = 14.0;

/// Base horizontal margin in physical pixels for root tree nodes (`6.0 px`).
pub const DEFAULT_TREE_BASE_PADDING: f32 = 6.0;

/// Length of the horizontal L-connector arm extending from parent stem (`8.0 px`).
pub const DEFAULT_TREE_LINE_ARM_LENGTH: f32 = 8.0;

/// Clearance between horizontal connector arm tip and the following node icon (`4.0 px`).
pub const DEFAULT_TREE_LINE_ARM_GAP: f32 = 4.0;

/// Default row height for hierarchical tree rows in physical pixels (`24.0 px`).
pub const DEFAULT_TREE_ROW_HEIGHT: f32 = 24.0;

/// Default stroke thickness for hierarchy connector lines in physical pixels (`1.2 px`).
pub const DEFAULT_TREE_LINE_THICKNESS: f32 = 1.2;

/// Default color applied to hierarchy connector branch lines.
pub const DEFAULT_TREE_LINE_COLOR: Color = Color::rgba(0.20, 0.55, 0.90, 0.85);

/// Geometry and styling configuration for tree view indentation and connector lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeIndentConfig {
    /// Horizontal indent step added for each level of nesting depth.
    pub indent_step: f32,
    /// Base horizontal left padding applied to root items.
    pub base_padding: f32,
    /// Length of the horizontal branch arm pointing towards the child node.
    pub line_arm_length: f32,
    /// Safety gap between connector tip and child icon/content.
    pub line_arm_gap: f32,
    /// Physical row height of the tree node item.
    pub row_height: f32,
    /// Stroke thickness of the vertical stem and horizontal arm.
    pub line_thickness: f32,
    /// Color tint applied to the branch connector lines.
    pub line_color: Color,
}

impl Default for TreeIndentConfig {
    #[inline]
    fn default() -> Self {
        Self {
            indent_step: DEFAULT_TREE_INDENT_STEP,
            base_padding: DEFAULT_TREE_BASE_PADDING,
            line_arm_length: DEFAULT_TREE_LINE_ARM_LENGTH,
            line_arm_gap: DEFAULT_TREE_LINE_ARM_GAP,
            row_height: DEFAULT_TREE_ROW_HEIGHT,
            line_thickness: DEFAULT_TREE_LINE_THICKNESS,
            line_color: DEFAULT_TREE_LINE_COLOR,
        }
    }
}

impl TreeIndentConfig {
    /// Creates a new tree indentation configuration with default engine values.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the horizontal indent step in physical pixels per depth level.
    #[inline]
    pub fn with_indent_step(mut self, step: f32) -> Self {
        self.indent_step = step.max(1.0);
        self
    }

    /// Overrides the base left padding in physical pixels.
    #[inline]
    pub fn with_base_padding(mut self, padding: f32) -> Self {
        self.base_padding = padding.max(0.0);
        self
    }

    /// Overrides the horizontal connector arm length in physical pixels.
    #[inline]
    pub fn with_line_arm_length(mut self, arm_length: f32) -> Self {
        self.line_arm_length = arm_length.max(0.0);
        self
    }

    /// Overrides the gap between connector tip and content in physical pixels.
    #[inline]
    pub fn with_line_arm_gap(mut self, gap: f32) -> Self {
        self.line_arm_gap = gap.max(0.0);
        self
    }

    /// Overrides the physical row height in physical pixels.
    #[inline]
    pub fn with_row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(1.0);
        self
    }

    /// Overrides the stroke thickness of connector lines in physical pixels.
    #[inline]
    pub fn with_line_thickness(mut self, thickness: f32) -> Self {
        self.line_thickness = thickness.max(0.5);
        self
    }

    /// Overrides the color tint of connector lines.
    #[inline]
    pub fn with_line_color(mut self, color: Color) -> Self {
        self.line_color = color;
        self
    }
}

/// Evaluated geometry measurements for a tree node row at a specific depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeConnectorGeometry {
    /// Zero-based depth level of the tree item (0 = root).
    pub depth: u16,
    /// Absolute horizontal coordinate where the horizontal connector arm ends.
    pub connector_tip_x: f32,
    /// Absolute left padding or offset where row content (icon, label) begins.
    pub content_start_x: f32,
}

impl TreeConnectorGeometry {
    /// Computes geometry measurements for a tree node at `depth` using `config`.
    ///
    /// # Invariants
    /// - For `depth == 0`, `connector_tip_x` is `0.0` and `content_start_x` equals `config.base_padding`.
    /// - For `depth > 0`, `content_start_x` is strictly `connector_tip_x + config.line_arm_gap`,
    ///   preventing line stems from penetrating into node content.
    #[inline]
    pub fn compute(depth: u16, config: &TreeIndentConfig) -> Self {
        if depth == 0 {
            Self {
                depth,
                connector_tip_x: 0.0,
                content_start_x: config.base_padding,
            }
        } else {
            let last_depth = (depth - 1) as f32;
            let stem_x = config.base_padding + 6.0 + (last_depth * config.indent_step);
            let connector_tip_x = stem_x + config.line_arm_length;
            let content_start_x = connector_tip_x + config.line_arm_gap;

            Self {
                depth,
                connector_tip_x,
                content_start_x,
            }
        }
    }
}

/// Computes the connector tip and content start geometry for backward compatibility.
///
/// Returns `(connector_tip_x, content_start_x)` in physical pixels.
#[inline]
pub fn compute_tree_connector_geometry(depth: u16, config: &TreeIndentConfig) -> (f32, f32) {
    let geom = TreeConnectorGeometry::compute(depth, config);
    (geom.connector_tip_x, geom.content_start_x)
}

/// Represents a vertical line segment connecting a parent branch to child levels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeLineVertical {
    /// Horizontal X offset relative to the tree row.
    pub x: f32,
    /// Top Y offset of the vertical segment (typically 0.0).
    pub y: f32,
    /// Height of the vertical segment in physical pixels.
    pub height: f32,
    /// Stroke thickness of the vertical line.
    pub thickness: f32,
}

/// Represents a horizontal L-branch arm pointing towards the immediate child item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeLineHorizontal {
    /// Horizontal start X offset of the arm segment.
    pub x: f32,
    /// Vertical center Y offset of the arm segment.
    pub y: f32,
    /// Width / length of the horizontal arm segment.
    pub width: f32,
    /// Stroke thickness of the horizontal line.
    pub thickness: f32,
}

/// Iterates over every branch connector line for a tree item at `depth` without heap allocation.
///
/// For each ancestor level `d` in `0..depth`, invokes `callback` with the corresponding
/// [`TreeLineVertical`] and optional [`TreeLineHorizontal`] (present only on the immediate parent level).
///
/// # Performance
/// Runs in strictly $O(\text{depth})$ with zero heap allocations, safe for per-frame render loops.
pub fn for_each_tree_connector_line<F>(depth: u16, config: &TreeIndentConfig, mut callback: F)
where
    F: FnMut(TreeLineVertical, Option<TreeLineHorizontal>),
{
    if depth == 0 {
        return;
    }

    for d in 0..depth {
        let is_last_level = d == depth - 1;
        let stem_x = config.base_padding + 6.0 + (d as f32 * config.indent_step);

        let v_height = if is_last_level {
            config.row_height * 0.5
        } else {
            config.row_height
        };

        let v_line = TreeLineVertical {
            x: stem_x,
            y: 0.0,
            height: v_height,
            thickness: config.line_thickness,
        };

        let h_line = if is_last_level {
            Some(TreeLineHorizontal {
                x: stem_x,
                y: (config.row_height * 0.5 - config.line_thickness * 0.5).round(),
                width: config.line_arm_length,
                thickness: config.line_thickness,
            })
        } else {
            None
        };

        callback(v_line, h_line);
    }
}

/// Generic state container managing expanded and selected nodes in a tree view.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeState<Id>
where
    Id: Eq + Hash + Clone,
{
    /// Set of item identifiers currently in an expanded branch state.
    pub expanded_nodes: HashSet<Id>,
    /// Set of item identifiers currently selected by the user.
    pub selected_nodes: HashSet<Id>,
    /// Primary or active item receiving keyboard focus / primary cursor.
    pub primary_selected: Option<Id>,
}

impl<Id> Default for TreeState<Id>
where
    Id: Eq + Hash + Clone,
{
    #[inline]
    fn default() -> Self {
        Self {
            expanded_nodes: HashSet::new(),
            selected_nodes: HashSet::new(),
            primary_selected: None,
        }
    }
}

impl<Id> TreeState<Id>
where
    Id: Eq + Hash + Clone,
{
    /// Creates an empty tree state with no expanded or selected nodes.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a tree state with pre-expanded items.
    #[inline]
    pub fn with_expanded(items: impl IntoIterator<Item = Id>) -> Self {
        Self {
            expanded_nodes: items.into_iter().collect(),
            selected_nodes: HashSet::new(),
            primary_selected: None,
        }
    }

    /// Checks whether an item with identifier `id` is currently expanded.
    #[inline]
    pub fn is_expanded(&self, id: &Id) -> bool {
        self.expanded_nodes.contains(id)
    }

    /// Sets the expanded status of an item with identifier `id`.
    ///
    /// Returns `true` if the state changed, or `false` if it was already in that state.
    pub fn set_expanded(&mut self, id: Id, expanded: bool) -> bool {
        if expanded {
            self.expanded_nodes.insert(id)
        } else {
            self.expanded_nodes.remove(&id)
        }
    }

    /// Toggles the expanded status of an item with identifier `id`.
    ///
    /// Returns the new expansion status (`true` if now expanded, `false` if collapsed).
    pub fn toggle_expanded(&mut self, id: &Id) -> bool {
        if self.expanded_nodes.contains(id) {
            self.expanded_nodes.remove(id);
            false
        } else {
            self.expanded_nodes.insert(id.clone());
            true
        }
    }

    /// Expands all nodes in the given sequence.
    pub fn expand_all(&mut self, ids: impl IntoIterator<Item = Id>) {
        for id in ids {
            self.expanded_nodes.insert(id);
        }
    }

    /// Collapses all currently expanded nodes.
    #[inline]
    pub fn collapse_all(&mut self) {
        self.expanded_nodes.clear();
    }

    /// Checks whether an item with identifier `id` is currently selected.
    #[inline]
    pub fn is_selected(&self, id: &Id) -> bool {
        self.selected_nodes.contains(id)
    }

    /// Returns the total number of selected nodes.
    #[inline]
    pub fn selected_count(&self) -> usize {
        self.selected_nodes.len()
    }

    /// Clears any existing selection and sets `id` as the sole selected and primary item.
    pub fn select_single(&mut self, id: Id) {
        self.selected_nodes.clear();
        self.selected_nodes.insert(id.clone());
        self.primary_selected = Some(id);
    }

    /// Toggles the selection state of `id` without modifying other selected nodes (Ctrl-click).
    ///
    /// Returns `true` if now selected, `false` if deselected.
    pub fn select_toggle(&mut self, id: Id) -> bool {
        if self.selected_nodes.contains(&id) {
            self.selected_nodes.remove(&id);
            if self.primary_selected.as_ref() == Some(&id) {
                self.primary_selected = self.selected_nodes.iter().next().cloned();
            }
            false
        } else {
            self.primary_selected = Some(id.clone());
            self.selected_nodes.insert(id);
            true
        }
    }

    /// Extends selection as a continuous range between the current primary anchor and `target_id` (Shift-click).
    ///
    /// Clears any prior selection outside the contiguous range. If no anchor exists or if items are
    /// not found in `visible_order`, falls back to [`Self::select_single`].
    ///
    /// Returns `true` if the selection changed.
    pub fn select_range(&mut self, target_id: Id, visible_order: &[Id]) -> bool {
        let Some(ref anchor) = self.primary_selected else {
            self.select_single(target_id);
            return true;
        };

        let start_pos = visible_order.iter().position(|item| item == anchor);
        let end_pos = visible_order.iter().position(|item| item == &target_id);

        match (start_pos, end_pos) {
            (Some(start), Some(end)) => {
                let (min, max) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                self.selected_nodes.clear();
                for item in &visible_order[min..=max] {
                    self.selected_nodes.insert(item.clone());
                }
                true
            }
            _ => {
                self.select_single(target_id);
                true
            }
        }
    }

    /// Additively expands selection across the range between anchor and `target_id` without clearing prior selections (Ctrl-Shift-click).
    ///
    /// Returns `true` if any new items were selected.
    pub fn select_range_additive(&mut self, target_id: Id, visible_order: &[Id]) -> bool {
        let Some(ref anchor) = self.primary_selected else {
            self.selected_nodes.insert(target_id.clone());
            self.primary_selected = Some(target_id);
            return true;
        };

        let start_pos = visible_order.iter().position(|item| item == anchor);
        let end_pos = visible_order.iter().position(|item| item == &target_id);

        match (start_pos, end_pos) {
            (Some(start), Some(end)) => {
                let (min, max) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                let mut changed = false;
                for item in &visible_order[min..=max] {
                    changed |= self.selected_nodes.insert(item.clone());
                }
                changed
            }
            _ => self.selected_nodes.insert(target_id),
        }
    }

    /// Clears all selected nodes and primary selection.
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
        self.primary_selected = None;
    }

    /// Returns a reference to the primary selected item identifier, if any.
    #[inline]
    pub fn primary_selected(&self) -> Option<&Id> {
        self.primary_selected.as_ref()
    }

    /// Overrides the primary selected item identifier.
    #[inline]
    pub fn set_primary_selected(&mut self, id: Option<Id>) {
        self.primary_selected = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_indent_and_connector_math() {
        let config = TreeIndentConfig::new()
            .with_indent_step(14.0)
            .with_base_padding(6.0)
            .with_line_arm_length(8.0)
            .with_line_arm_gap(4.0)
            .with_row_height(24.0);

        // Root depth (depth = 0)
        let root_geom = TreeConnectorGeometry::compute(0, &config);
        assert_eq!(root_geom.depth, 0);
        assert_eq!(root_geom.connector_tip_x, 0.0);
        assert_eq!(root_geom.content_start_x, 6.0);

        // Depths 1 through 6
        for depth in 1..=6 {
            let geom = TreeConnectorGeometry::compute(depth, &config);
            assert_eq!(geom.depth, depth);

            // Mathematical invariant: gap between arm tip and content start must strictly equal line_arm_gap
            let gap = geom.content_start_x - geom.connector_tip_x;
            assert!(
                (gap - config.line_arm_gap).abs() < f32::EPSILON,
                "Gap must strictly equal line_arm_gap at depth {depth}"
            );
            assert!(
                geom.content_start_x > geom.connector_tip_x,
                "Content must start strictly after connector line tip at depth {depth}"
            );

            // Indentation increase must match indent_step across successive depths
            if depth > 1 {
                let prev_geom = TreeConnectorGeometry::compute(depth - 1, &config);
                let step_delta = geom.connector_tip_x - prev_geom.connector_tip_x;
                assert!(
                    (step_delta - config.indent_step).abs() < f32::EPSILON,
                    "Step delta between depth {depth} and {} must equal indent_step",
                    depth - 1
                );
            }
        }
    }

    #[test]
    fn test_tree_state_initialization() {
        let mut state: TreeState<u64> = TreeState::new();
        assert_eq!(state.selected_count(), 0);
        assert!(!state.is_expanded(&1));
        assert!(!state.is_selected(&1));
        assert!(state.primary_selected().is_none());

        // Toggle expand
        assert!(state.toggle_expanded(&10));
        assert!(state.is_expanded(&10));
        assert!(!state.toggle_expanded(&10));
        assert!(!state.is_expanded(&10));

        // Selection
        state.select_single(42);
        assert_eq!(state.selected_count(), 1);
        assert!(state.is_selected(&42));
        assert_eq!(state.primary_selected(), Some(&42));

        // Toggle select another item
        assert!(state.select_toggle(84));
        assert_eq!(state.selected_count(), 2);
        assert!(state.is_selected(&84));
        assert_eq!(state.primary_selected(), Some(&84));

        // Deselect first item
        assert!(!state.select_toggle(42));
        assert_eq!(state.selected_count(), 1);
        assert!(!state.is_selected(&42));
        assert_eq!(state.primary_selected(), Some(&84));

        // Clear selection
        state.clear_selection();
        assert_eq!(state.selected_count(), 0);
        assert!(state.primary_selected().is_none());
    }

    #[test]
    fn test_tree_connector_lines_callback_stack_zero_allocation() {
        let config = TreeIndentConfig::new().with_row_height(24.0);

        let mut vertical_count = 0;
        let mut horizontal_count = 0;

        for_each_tree_connector_line(3, &config, |v_line, h_line| {
            vertical_count += 1;
            assert!(v_line.height > 0.0);
            assert_eq!(v_line.thickness, config.line_thickness);

            if let Some(h) = h_line {
                horizontal_count += 1;
                assert_eq!(h.width, config.line_arm_length);
                assert_eq!(h.thickness, config.line_thickness);
            }
        });

        // Depth 3 has 3 vertical stems and exactly 1 horizontal connector arm
        assert_eq!(vertical_count, 3);
        assert_eq!(horizontal_count, 1);
    }
}