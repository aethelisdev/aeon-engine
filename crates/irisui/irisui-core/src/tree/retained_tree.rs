// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Retained Tree Expansion, Selection & Flattened Row Cache (`UiTree`)
//!
//! Stores per-tree expansion states (`expanded`/`collapsed`), single/multi/range selection
//! tracking, and revision-keyed flattened DFS row caches directly inside [`UiTree`] keyed by
//! semantic 64-bit tree tag so declarative panels never own boilerplate `TreeModel` or
//! `TreeState` synchronization state.

use std::collections::{HashMap, HashSet};

use super::arena::UiTree;

/// Lightweight flattened row record cached inside [`RetainedTreeState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetainedTreeRow {
    /// Deterministic 64-bit domain node key.
    pub key: u64,
    /// Visual nesting depth (`0` = root row).
    pub depth: u16,
    /// Whether this node has at least one child edge in the current tree graph.
    pub has_children: bool,
    /// Whether this node is currently expanded.
    pub is_expanded: bool,
    /// Master DFS index assigned across the unfiltered flattened order (used for stable tags).
    pub master_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RetainedFlattenKey {
    domain_revision: u64,
    source_len: usize,
    expansion_revision: u64,
    source_salt: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RetainedFilterKey {
    flatten_epoch: u64,
    source_salt: u64,
    query_hash: u64,
}

/// Persistent expansion, selection, and flattened DFS row cache keyed inside [`UiTree`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedTreeState {
    default_expanded: bool,
    seeded_initial: bool,
    expanded: HashSet<u64>,
    collapsed: HashSet<u64>,
    selected: HashSet<u64>,
    primary_selected: Option<u64>,
    expansion_revision: u64,
    flatten_epoch: u64,
    flatten_key: Option<RetainedFlattenKey>,
    filter_key: Option<RetainedFilterKey>,
    has_filter: bool,
    rows: Vec<RetainedTreeRow>,
    filtered_indices: Vec<usize>,
    visible_order: Vec<u64>,
}

impl RetainedTreeState {
    /// Constructs an empty [`RetainedTreeState`] with the specified default expansion mode.
    #[inline]
    #[must_use]
    pub fn new(default_expanded: bool) -> Self {
        Self {
            default_expanded,
            seeded_initial: false,
            expanded: HashSet::new(),
            collapsed: HashSet::new(),
            selected: HashSet::new(),
            primary_selected: None,
            expansion_revision: 0,
            flatten_epoch: 0,
            flatten_key: None,
            filter_key: None,
            has_filter: false,
            rows: Vec::new(),
            filtered_indices: Vec::new(),
            visible_order: Vec::new(),
        }
    }

    /// Seeds initial expanded keys once when the retained tree state is first initialized.
    pub fn seed_initial(&mut self, keys: &[u64]) {
        if !self.seeded_initial {
            self.seeded_initial = true;
            for &key in keys {
                self.set_expanded(key, true);
            }
        }
    }

    /// Returns whether `key` is currently expanded.
    #[inline]
    #[must_use]
    pub fn is_expanded(&self, key: u64) -> bool {
        if self.default_expanded {
            !self.collapsed.contains(&key)
        } else {
            self.expanded.contains(&key)
        }
    }

    /// Sets the expansion state for `key`, incrementing revision when changed.
    ///
    /// Returns `true` if the state changed.
    pub fn set_expanded(&mut self, key: u64, expanded: bool) -> bool {
        let changed = if self.default_expanded {
            if expanded {
                self.collapsed.remove(&key)
            } else {
                self.collapsed.insert(key)
            }
        } else if expanded {
            self.expanded.insert(key)
        } else {
            self.expanded.remove(&key)
        };
        if changed {
            self.expansion_revision = self.expansion_revision.wrapping_add(1);
        }
        changed
    }

    /// Toggles the expansion state of `key` and returns the resulting expanded value.
    pub fn toggle_expanded(&mut self, key: u64) -> bool {
        let next = !self.is_expanded(key);
        let _ = self.set_expanded(key, next);
        next
    }

    /// Returns the monotonic expansion revision incremented on every expand/collapse toggle.
    #[inline]
    #[must_use]
    pub const fn expansion_revision(&self) -> u64 {
        self.expansion_revision
    }

    /// Returns whether `key` is currently part of the active selection set.
    #[inline]
    #[must_use]
    pub fn is_selected(&self, key: u64) -> bool {
        self.selected.contains(&key)
    }

    /// Returns the primary anchor selection key, if any.
    #[inline]
    #[must_use]
    pub const fn primary_selected(&self) -> Option<u64> {
        self.primary_selected
    }

    /// Returns the number of currently selected node keys.
    #[inline]
    #[must_use]
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Clears prior selection and selects only `key` as primary.
    pub fn select_single(&mut self, key: u64) {
        self.selected.clear();
        self.selected.insert(key);
        self.primary_selected = Some(key);
    }

    /// Processes a primary row click with optional `ctrl` and `shift` range selection modifiers.
    ///
    /// Returns `true` if the selection changed.
    pub fn handle_row_click(&mut self, key: u64, ctrl: bool, shift: bool) -> bool {
        let prev_primary = self.primary_selected;
        let prev_len = self.selected.len();
        if shift && let Some(anchor) = self.primary_selected {
            let anchor_idx = self.visible_order.iter().position(|&k| k == anchor);
            let target_idx = self.visible_order.iter().position(|&k| k == key);
            if let (Some(a), Some(t)) = (anchor_idx, target_idx) {
                let (lo, hi) = if a <= t { (a, t) } else { (t, a) };
                if !ctrl {
                    self.selected.clear();
                }
                for &item in &self.visible_order[lo..=hi] {
                    self.selected.insert(item);
                }
                return prev_primary != self.primary_selected || prev_len != self.selected.len();
            }
        }
        if ctrl {
            if !self.selected.remove(&key) {
                self.selected.insert(key);
            }
            self.primary_selected = Some(key);
            return true;
        }
        let unchanged = prev_len == 1 && prev_primary == Some(key) && self.selected.contains(&key);
        self.select_single(key);
        !unchanged
    }

    /// Returns `true` when flattened DFS rows already match `(domain_rev, source_len, source_salt)`.
    #[inline]
    #[must_use]
    pub fn flatten_cache_hits(
        &self,
        domain_revision: u64,
        source_len: usize,
        source_salt: u64,
    ) -> bool {
        self.flatten_key
            == Some(RetainedFlattenKey {
                domain_revision,
                source_len,
                expansion_revision: self.expansion_revision,
                source_salt,
            })
    }

    /// Clears cached flattened rows and advances the internal flatten epoch before rebuilding.
    pub fn begin_flatten_rebuild(&mut self) {
        self.rows.clear();
        self.visible_order.clear();
        self.flatten_epoch = self.flatten_epoch.wrapping_add(1);
    }

    /// Appends one flattened row record to both `rows` and `visible_order`.
    #[inline]
    pub fn push_flattened_row(&mut self, row: RetainedTreeRow) {
        self.visible_order.push(row.key);
        self.rows.push(row);
    }

    /// Seals the flatten cache key after populating all flattened DFS rows.
    pub fn finish_flatten_rebuild(
        &mut self,
        domain_revision: u64,
        source_len: usize,
        source_salt: u64,
    ) {
        self.flatten_key = Some(RetainedFlattenKey {
            domain_revision,
            source_len,
            expansion_revision: self.expansion_revision,
            source_salt,
        });
    }

    /// Returns `true` when cached filter indices already match `(source_salt, query_hash)`.
    #[inline]
    #[must_use]
    pub fn filter_cache_hits(&self, source_salt: u64, query_hash: u64) -> bool {
        self.filter_key
            == Some(RetainedFilterKey {
                flatten_epoch: self.flatten_epoch,
                source_salt,
                query_hash,
            })
    }

    /// Resets filtered row indices prior to evaluating a new search query.
    pub fn begin_filter(&mut self, has_filter: bool) {
        self.has_filter = has_filter;
        self.filtered_indices.clear();
    }

    /// Records a matching master row index under the active search filter.
    #[inline]
    pub fn push_filtered_index(&mut self, master_idx: usize) {
        self.filtered_indices.push(master_idx);
    }

    /// Seals the filter cache key after completing a filter scan.
    pub fn finish_filter(&mut self, source_salt: u64, query_hash: u64) {
        self.filter_key = Some(RetainedFilterKey {
            flatten_epoch: self.flatten_epoch,
            source_salt,
            query_hash,
        });
    }

    /// Returns immutable access to all flattened unfiltered rows.
    #[inline]
    #[must_use]
    pub fn rows(&self) -> &[RetainedTreeRow] {
        &self.rows
    }

    /// Returns total unfiltered flattened row count.
    #[inline]
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns whether an active search query filter is constraining visible rows.
    #[inline]
    #[must_use]
    pub const fn has_active_filter(&self) -> bool {
        self.has_filter
    }

    /// Returns visible row count after active search filtering.
    #[inline]
    #[must_use]
    pub fn visible_count(&self) -> usize {
        if self.has_filter {
            self.filtered_indices.len()
        } else {
            self.rows.len()
        }
    }

    /// Resolves `(master_idx, RetainedTreeRow)` for the given virtualized visible row index.
    #[inline]
    #[must_use]
    pub fn visible_row(&self, visible_idx: usize) -> Option<RetainedTreeRow> {
        if self.has_filter {
            let &master_idx = self.filtered_indices.get(visible_idx)?;
            self.rows.get(master_idx).copied()
        } else {
            self.rows.get(visible_idx).copied()
        }
    }
}

/// Ensures `retained_trees` has an initialized entry for `tree_tag`, seeding `initial_expanded` once.
#[inline]
pub(crate) fn ensure_retained_tree<'a>(
    retained_trees: &'a mut HashMap<u64, RetainedTreeState>,
    tree_tag: u64,
    default_expanded: bool,
    initial_expanded: &[u64],
) -> &'a mut RetainedTreeState {
    let state = retained_trees
        .entry(tree_tag)
        .or_insert_with(|| RetainedTreeState::new(default_expanded));
    state.seed_initial(initial_expanded);
    state
}

impl UiTree {
    /// Returns mutable access to the retained tree state associated with `tree_tag`.
    ///
    /// Initializes expansion defaults (`default_expanded`, `initial_expanded`) on first access.
    #[inline]
    pub fn retained_tree_mut(
        &mut self,
        tree_tag: u64,
        default_expanded: bool,
        initial_expanded: &[u64],
    ) -> &mut RetainedTreeState {
        ensure_retained_tree(
            &mut self.retained_trees,
            tree_tag,
            default_expanded,
            initial_expanded,
        )
    }

    /// Returns immutable access to the retained tree state associated with `tree_tag`, if created.
    #[inline]
    #[must_use]
    pub fn retained_tree(&self, tree_tag: u64) -> Option<&RetainedTreeState> {
        self.retained_trees.get(&tree_tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retained_tree_state_expansion_selection_and_cache_guards() {
        let mut state = RetainedTreeState::new(false);
        state.seed_initial(&[10]);
        assert!(state.is_expanded(10));
        assert!(!state.is_expanded(20));

        assert!(state.toggle_expanded(20));
        assert!(state.is_expanded(20));
        assert_eq!(state.expansion_revision(), 2);

        state.begin_flatten_rebuild();
        state.push_flattened_row(RetainedTreeRow {
            key: 10,
            depth: 0,
            has_children: true,
            is_expanded: true,
            master_idx: 0,
        });
        state.push_flattened_row(RetainedTreeRow {
            key: 20,
            depth: 1,
            has_children: false,
            is_expanded: false,
            master_idx: 1,
        });
        state.finish_flatten_rebuild(1, 2, 99);
        assert!(state.flatten_cache_hits(1, 2, 99));
        assert!(!state.flatten_cache_hits(2, 2, 99));

        assert!(state.handle_row_click(10, false, false));
        assert!(state.handle_row_click(20, false, true));
        assert_eq!(state.selected_count(), 2);
        assert!(state.is_selected(10));
        assert!(state.is_selected(20));
    }
}