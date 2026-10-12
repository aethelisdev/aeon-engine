// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical TreeView Keyboard Navigation & Auto-Scroll Engine
//!
//! Provides autonomous keyboard event handling for hierarchical tree navigation
//! (Arrow Up/Down, Left collapse/parent, Right expand/child, Home, End), modifier-aware
//! range extension, and $O(1)$ auto-scroll calculations.
//!
//! ## Architectural Guarantees
//! - **Zero Per-Frame Allocation**: Operates purely over borrowed slices of POD metadata
//!   ([`TreeItemInfo`]) and item identifiers.
//! - **Mathematical Hierarchy Invariants**: Left arrow collapses expanded branches or
//!   recursively jumps to the parent depth; Right arrow expands collapsed branches or
//!   steps into the first child node.
//! - **Zero-Jitter Viewport Clamping**: [`compute_tree_scroll_to_view`] ensures focused
//!   items are kept in view without unnecessary scroll oscillations.
//!

use super::model::TreeState;
use iris_core::event::{KeyCode, UiEvent};
use std::hash::Hash;

/// Lightweight POD metadata descriptor for a visible, flattened tree item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeItemInfo<Id> {
    /// Unique identifier of the tree item.
    pub id: Id,
    /// Nesting depth level of the item (0 = root).
    pub depth: u16,
    /// True if the item contains one or more child sub-branches.
    pub has_children: bool,
}

impl<Id> TreeItemInfo<Id> {
    /// Creates a new tree item info descriptor.
    #[inline]
    pub const fn new(id: Id, depth: u16, has_children: bool) -> Self {
        Self {
            id,
            depth,
            has_children,
        }
    }

    /// Creates a leaf tree item descriptor with no children.
    #[inline]
    pub const fn leaf(id: Id, depth: u16) -> Self {
        Self {
            id,
            depth,
            has_children: false,
        }
    }
}

/// Evaluated outcome of a tree keyboard navigation event.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TreeNavResponse<Id> {
    /// True if the keyboard event was recognized and consumed by the navigation handler.
    pub consumed: bool,
    /// True if a node's expanded state was toggled, requiring hierarchy re-flattening.
    pub layout_changed: bool,
    /// True if the selection set or primary focused node changed.
    pub selection_changed: bool,
    /// Identifier of the newly selected or focused item, if any.
    pub selected_id: Option<Id>,
    /// Index in the visible items list that should be brought into view via auto-scrolling.
    pub scroll_target_index: Option<usize>,
}

impl<Id> TreeNavResponse<Id> {
    /// Returns an unhandled response that was ignored.
    #[inline]
    pub const fn unhandled() -> Self {
        Self {
            consumed: false,
            layout_changed: false,
            selection_changed: false,
            selected_id: None,
            scroll_target_index: None,
        }
    }

    /// Constructs a handled response specifying state mutations.
    #[inline]
    pub const fn handled(
        layout_changed: bool,
        selection_changed: bool,
        selected_id: Option<Id>,
        scroll_target_index: Option<usize>,
    ) -> Self {
        Self {
            consumed: true,
            layout_changed,
            selection_changed,
            selected_id,
            scroll_target_index,
        }
    }
}

/// Computes the new vertical scroll offset required to bring `row_index` into the viewport.
///
/// Returns `Some(new_scroll_y)` if the item is currently outside the viewport boundaries,
/// or `None` if the item is already completely visible within `[current_scroll_y, current_scroll_y + viewport_height]`.
///
/// # Mathematical Invariants
/// - If the item's top edge is above the viewport (`row_top < current_scroll_y`),
///   scrolls upward so the item top aligns with the top of the viewport.
/// - If the item's bottom edge is below the viewport (`row_bottom > current_scroll_y + viewport_height`),
///   scrolls downward so the item bottom aligns with the bottom of the viewport.
/// - Clamps the resulting scroll position strictly to `[0.0, max_scroll_y]`.
#[inline]
pub fn compute_tree_scroll_to_view(
    row_index: usize,
    row_stride: f32,
    current_scroll_y: f32,
    viewport_height: f32,
    max_scroll_y: f32,
) -> Option<f32> {
    let row_top = row_index as f32 * row_stride;
    let row_bottom = row_top + row_stride;
    let vp_bottom = current_scroll_y + viewport_height;

    if row_top < current_scroll_y {
        Some(row_top.clamp(0.0, max_scroll_y.max(0.0)))
    } else if row_bottom > vp_bottom {
        let desired = row_bottom - viewport_height;
        Some(desired.clamp(0.0, max_scroll_y.max(0.0)))
    } else {
        None
    }
}

/// Handles keyboard directional navigation across a visible tree item list.
///
/// # Navigation Rules
/// - **`ArrowUp`**: Selects previous visible item; clamps at index 0.
/// - **`ArrowDown`**: Selects next visible item; clamps at `items.len() - 1`.
/// - **`ArrowRight`**:
///   - If current node has children and is collapsed, expands it (`layout_changed: true`).
///   - If current node has children and is already expanded, selects its first child (`index + 1`).
/// - **`ArrowLeft`**:
///   - If current node has children and is expanded, collapses it (`layout_changed: true`).
///   - If current node is already collapsed or a leaf node, jumps to its parent (previous item with lesser depth).
/// - **`Home`**: Selects first item (index 0).
/// - **`End`**: Selects last item (`items.len() - 1`).
/// - **`Shift` held**: Extends selection as a continuous range (`select_range`).
pub fn handle_tree_arrow_navigation<Id>(
    state: &mut TreeState<Id>,
    items: &[TreeItemInfo<Id>],
    key: KeyCode,
    shift_held: bool,
) -> TreeNavResponse<Id>
where
    Id: Eq + Hash + Clone,
{
    if items.is_empty() {
        return TreeNavResponse::unhandled();
    }

    let current_idx = state
        .primary_selected()
        .and_then(|anchor| items.iter().position(|item| &item.id == anchor));

    match key {
        KeyCode::ArrowUp => {
            let target_idx = match current_idx {
                Some(idx) if idx > 0 => idx - 1,
                Some(_) => 0,
                None => items.len() - 1,
            };
            apply_navigation_selection(state, items, target_idx, shift_held)
        }
        KeyCode::ArrowDown => {
            let target_idx = match current_idx {
                Some(idx) if idx + 1 < items.len() => idx + 1,
                Some(_) => items.len() - 1,
                None => 0,
            };
            apply_navigation_selection(state, items, target_idx, shift_held)
        }
        KeyCode::ArrowRight => {
            let Some(idx) = current_idx else {
                return apply_navigation_selection(state, items, 0, false);
            };
            let current_item = &items[idx];

            if current_item.has_children {
                if !state.is_expanded(&current_item.id) {
                    state.set_expanded(current_item.id.clone(), true);
                    TreeNavResponse::handled(true, false, Some(current_item.id.clone()), Some(idx))
                } else if idx + 1 < items.len() {
                    apply_navigation_selection(state, items, idx + 1, shift_held)
                } else {
                    TreeNavResponse::handled(false, false, Some(current_item.id.clone()), Some(idx))
                }
            } else {
                TreeNavResponse::handled(false, false, Some(current_item.id.clone()), Some(idx))
            }
        }
        KeyCode::ArrowLeft => {
            let Some(idx) = current_idx else {
                return apply_navigation_selection(state, items, 0, false);
            };
            let current_item = &items[idx];

            if current_item.has_children && state.is_expanded(&current_item.id) {
                state.set_expanded(current_item.id.clone(), false);
                TreeNavResponse::handled(true, false, Some(current_item.id.clone()), Some(idx))
            } else if current_item.depth > 0 {
                let target_depth = current_item.depth - 1;
                let parent_idx = items[..idx]
                    .iter()
                    .rposition(|item| item.depth == target_depth)
                    .unwrap_or(0);
                apply_navigation_selection(state, items, parent_idx, shift_held)
            } else {
                TreeNavResponse::handled(false, false, Some(current_item.id.clone()), Some(idx))
            }
        }
        KeyCode::Home => apply_navigation_selection(state, items, 0, shift_held),
        KeyCode::End => apply_navigation_selection(state, items, items.len() - 1, shift_held),
        _ => TreeNavResponse::unhandled(),
    }
}

/// Helper function that updates the selection on `state` and produces a `TreeNavResponse`.
fn apply_navigation_selection<Id>(
    state: &mut TreeState<Id>,
    items: &[TreeItemInfo<Id>],
    target_idx: usize,
    shift_held: bool,
) -> TreeNavResponse<Id>
where
    Id: Eq + Hash + Clone,
{
    let target_id = items[target_idx].id.clone();
    if shift_held {
        // Collect visible item IDs for contiguous range calculation
        let id_slice: Vec<Id> = items.iter().map(|item| item.id.clone()).collect();
        state.select_range(target_id.clone(), &id_slice);
    } else {
        state.select_single(target_id.clone());
    }

    TreeNavResponse::handled(false, true, Some(target_id), Some(target_idx))
}

/// Evaluates an incoming [`UiEvent`] against the tree item list.
///
/// Consumes `KeyboardInput` key-down events for directional navigation keys.
pub fn handle_tree_navigation_event<Id>(
    state: &mut TreeState<Id>,
    items: &[TreeItemInfo<Id>],
    event: &UiEvent,
    shift_held: bool,
) -> TreeNavResponse<Id>
where
    Id: Eq + Hash + Clone,
{
    match event {
        UiEvent::KeyboardInput {
            key,
            is_pressed: true,
            ..
        } => handle_tree_arrow_navigation(state, items, *key, shift_held),
        _ => TreeNavResponse::unhandled(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_arrow_navigation() {
        let mut state = TreeState::<u32>::new();
        let items = vec![
            TreeItemInfo::new(1, 0, true),    // root 1 (has children)
            TreeItemInfo::new(11, 1, false),  // child 11
            TreeItemInfo::new(12, 1, true),   // child 12 (has children)
            TreeItemInfo::new(121, 2, false), // grandchild 121
            TreeItemInfo::new(2, 0, false),   // root 2 (leaf)
        ];

        // 1. Initial ArrowDown from unselected state selects first item (1)
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowDown, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(1));
        assert_eq!(resp.scroll_target_index, Some(0));
        assert_eq!(state.primary_selected(), Some(&1));

        // 2. ArrowDown moves to child 11
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowDown, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(11));
        assert_eq!(resp.scroll_target_index, Some(1));

        // 3. ArrowUp moves back to root 1
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowUp, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(1));
        assert_eq!(resp.scroll_target_index, Some(0));

        // 4. ArrowRight on collapsed root 1 expands it
        assert!(!state.is_expanded(&1));
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowRight, false);
        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert!(state.is_expanded(&1));

        // 5. ArrowRight on already expanded root 1 steps to first child (11)
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowRight, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(11));

        // 6. ArrowLeft on child 11 jumps back to parent (1)
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowLeft, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(1));

        // 7. ArrowLeft on expanded parent 1 collapses it
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowLeft, false);
        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert!(!state.is_expanded(&1));

        // 8. End key jumps to last item (2)
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::End, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(2));
        assert_eq!(resp.scroll_target_index, Some(4));

        // 9. Home key jumps to first item (1)
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::Home, false);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(1));
        assert_eq!(resp.scroll_target_index, Some(0));

        // 10. Shift + ArrowDown extends range selection
        let resp = handle_tree_arrow_navigation(&mut state, &items, KeyCode::ArrowDown, true);
        assert!(resp.consumed);
        assert_eq!(resp.selected_id, Some(11));
        assert_eq!(state.selected_count(), 2);
        assert!(state.is_selected(&1));
        assert!(state.is_selected(&11));
    }

    #[test]
    fn test_tree_auto_scroll_to_selected() {
        let row_stride = 24.0;
        let viewport_h = 100.0;
        let max_scroll = 500.0;

        // Case 1: Item 0 (0.0..24.0) while scroll_y = 50.0 -> Needs to scroll up to 0.0
        let scroll = compute_tree_scroll_to_view(0, row_stride, 50.0, viewport_h, max_scroll);
        assert_eq!(scroll, Some(0.0));

        // Case 2: Item 2 (48.0..72.0) while scroll_y = 0.0 -> Already visible (0.0..100.0)
        let scroll = compute_tree_scroll_to_view(2, row_stride, 0.0, viewport_h, max_scroll);
        assert_eq!(scroll, None);

        // Case 3: Item 5 (120.0..144.0) while scroll_y = 0.0 -> Below viewport (0.0..100.0)
        // Desired bottom aligns with viewport bottom: 144.0 - 100.0 = 44.0
        let scroll = compute_tree_scroll_to_view(5, row_stride, 0.0, viewport_h, max_scroll);
        assert_eq!(scroll, Some(44.0));

        // Case 4: Clamping at max_scroll
        let scroll = compute_tree_scroll_to_view(100, row_stride, 0.0, viewport_h, 200.0);
        assert_eq!(scroll, Some(200.0));
    }
}