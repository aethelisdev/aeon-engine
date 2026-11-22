// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Swappable Hierarchical Tree Data Source & Cached Flattening Model
//!
//! Decouples domain tree storage (ECS scenes, filesystem folders, AST nodes, UI outlines)
//! from Iris UI's layout-friendly pre-order DFS row flattening, revision invalidation caching,
//! and case-insensitive search filtering.
//!
//! Any consumer implementing [`TreeDataSource`] gains $O(1)$ cache hits across unchanged frames,
//! persistent scratch buffer reuse, and zero-allocation range selection order slices without
//! leaking domain concepts into `irisui-widgets`.

use super::model::TreeState;
use std::collections::HashMap;
use std::hash::Hash;

/// Collector sink populated by [`TreeDataSource::collect_nodes`] with raw `(node_id, parent_id)` edges.
///
/// Reuses internal hash maps and vectors across frames inside [`TreeModel`] to avoid per-sync
/// heap reallocations.
#[derive(Debug, Clone)]
pub struct TreeNodeCollector<Id>
where
    Id: Eq + Hash,
{
    pub(crate) all_nodes: Vec<Id>,
    pub(crate) parent_map: HashMap<Id, Id>,
    pub(crate) children_map: HashMap<Id, Vec<Id>>,
    pub(crate) root_ids: Vec<Id>,
}

impl<Id> Default for TreeNodeCollector<Id>
where
    Id: Eq + Hash,
{
    #[inline]
    fn default() -> Self {
        Self {
            all_nodes: Vec::new(),
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
            root_ids: Vec::new(),
        }
    }
}

impl<Id> TreeNodeCollector<Id>
where
    Id: Copy + Eq + Hash + Ord,
{
    /// Reserves capacity hint for expected node count before collection begins.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.all_nodes.reserve(additional);
    }

    /// Registers a visible domain node identifier and its optional parent identifier.
    ///
    /// Parent edges referencing nodes omitted from [`TreeDataSource::collect_nodes`] are
    /// promoted gracefully to root level during graph resolution.
    #[inline]
    pub fn push_node(&mut self, id: Id, parent: Option<Id>) {
        self.all_nodes.push(id);
        if let Some(parent_id) = parent {
            self.parent_map.insert(id, parent_id);
            self.children_map.entry(parent_id).or_default().push(id);
        }
    }

    pub(crate) fn clear_buffers(&mut self) {
        self.all_nodes.clear();
        self.parent_map.clear();
        for children in self.children_map.values_mut() {
            children.clear();
        }
        self.root_ids.clear();
    }
}

/// Abstract domain adapter decoupling tree data backends from Iris UI flattening, caching, and search.
///
/// Consumers implement this lightweight trait for their storage backend (such as an ECS world,
/// asset directory index, or UI canvas tree), making tree data sources cleanly swappable.
pub trait TreeDataSource {
    /// Stable node identifier (`Copy`, hashable, and deterministically orderable).
    type Id: Copy + Eq + Hash + Ord;

    /// Returns a stable opaque 64-bit key uniquely identifying `id` for retained tree storage.
    fn node_key(&self, id: Self::Id) -> u64;

    /// Resolves the typed domain identifier corresponding to an opaque 64-bit `key`, if valid.
    fn id_from_key(&self, key: u64) -> Option<Self::Id>;

    /// Emits every visible node and its optional parent relationship into `collector`.
    fn collect_nodes(&self, collector: &mut TreeNodeCollector<Self::Id>);

    /// Writes searchable display text for `id` into `out` during filter evaluations.
    ///
    /// Implementations should push directly into `out` rather than allocating temporary `String`s.
    fn search_text(&self, id: Self::Id, out: &mut String);
}

/// Pre-flattened lightweight POD representation of one visible node row inside [`TreeModel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeRowEntry<Id> {
    /// Stable node identifier from [`TreeDataSource::Id`].
    pub id: Id,
    /// Indentation nesting depth (`0` = root item).
    pub depth: u16,
    /// `true` if this node has at least one child edge in the current tree graph.
    pub has_children: bool,
    /// `true` if this node is currently expanded in [`TreeState`].
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlattenCacheKey {
    domain_revision: u64,
    source_len: usize,
    expansion_revision: u64,
    source_salt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilterCacheKey {
    flatten_epoch: u64,
    source_salt: u64,
    query: String,
}

/// Generic cached pre-order DFS tree model supporting revision-keyed invalidation and search filtering.
///
/// Stores persistent scratch buffers internally so idle frames hit an $O(1)$ cache guard and filter
/// queries reuse pre-allocated string buffers without per-row heap churn.
#[derive(Debug, Clone)]
pub struct TreeModel<Id>
where
    Id: Copy + Eq + Hash + Ord,
{
    collector: TreeNodeCollector<Id>,
    rows: Vec<TreeRowEntry<Id>>,
    visible_order: Vec<Id>,
    filtered_indices: Vec<usize>,
    active_query: String,
    flatten_epoch: u64,
    last_flatten_key: Option<FlattenCacheKey>,
    last_filter_key: Option<FilterCacheKey>,
    query_lower: String,
    scratch_text: String,
}

impl<Id> Default for TreeModel<Id>
where
    Id: Copy + Eq + Hash + Ord,
{
    #[inline]
    fn default() -> Self {
        Self {
            collector: TreeNodeCollector::default(),
            rows: Vec::new(),
            visible_order: Vec::new(),
            filtered_indices: Vec::new(),
            active_query: String::new(),
            flatten_epoch: 0,
            last_flatten_key: None,
            last_filter_key: None,
            query_lower: String::new(),
            scratch_text: String::new(),
        }
    }
}

impl<Id> TreeModel<Id>
where
    Id: Copy + Eq + Hash + Ord,
{
    /// Constructs an empty cached tree model with reusable internal scratch storage.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidates cached revision keys so the next [`Self::sync`] call unconditionally rebuilds rows.
    #[inline]
    pub fn invalidate(&mut self) {
        self.last_flatten_key = None;
        self.last_filter_key = None;
    }

    /// Synchronizes flattened DFS rows and search filter indices against `source`.
    ///
    /// Skips tree traversal completely in $O(1)$ time when `(domain_revision, source_len,
    /// tree_state.expansion_revision(), source_salt)` matches the previous sync. Swapping
    /// `source` instances should pass a distinct `source_salt` or call [`Self::invalidate`].
    ///
    /// Returns `true` if a tree flatten rebuild executed, or `false` on a cache hit.
    pub fn sync<S: TreeDataSource<Id = Id>>(
        &mut self,
        source: &S,
        tree_state: &TreeState<Id>,
        domain_revision: u64,
        source_len: usize,
        search_query: &str,
    ) -> bool {
        let source_salt = std::ptr::from_ref(source) as usize as u64;
        let current_key = FlattenCacheKey {
            domain_revision,
            source_len,
            expansion_revision: tree_state.expansion_revision(),
            source_salt,
        };

        let rebuilt_rows = if self.last_flatten_key == Some(current_key) {
            false
        } else {
            self.rebuild_rows(source, tree_state);
            self.last_flatten_key = Some(current_key);
            true
        };

        self.sync_filter(source, source_salt, search_query);
        rebuilt_rows
    }

    /// Unconditionally rebuilds flattened rows from `source`, bypassing revision caches.
    pub fn sync_unconditional<S: TreeDataSource<Id = Id>>(
        &mut self,
        source: &S,
        tree_state: &TreeState<Id>,
    ) {
        self.rebuild_rows(source, tree_state);
        self.last_flatten_key = None;
        self.last_filter_key = None;
    }

    /// Returns total flattened row count prior to search filtering.
    #[inline]
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns all cached flattened DFS rows.
    #[inline]
    #[must_use]
    pub fn rows(&self) -> &[TreeRowEntry<Id>] {
        &self.rows
    }

    /// Returns flattened node identifiers in display order for $O(1)$ range selection slicing.
    #[inline]
    #[must_use]
    pub fn visible_order(&self) -> &[Id] {
        &self.visible_order
    }

    /// Returns `true` when a non-empty trimmed search filter query is active.
    #[inline]
    #[must_use]
    pub fn has_active_filter(&self) -> bool {
        !self.active_query.is_empty()
    }

    /// Returns the count of visible rows after applying active search filtering.
    #[inline]
    #[must_use]
    pub fn visible_count(&self) -> usize {
        if self.active_query.is_empty() {
            self.rows.len()
        } else {
            self.filtered_indices.len()
        }
    }

    /// Resolves `(master_row_index, &TreeRowEntry)` for a virtualized visible row index.
    #[inline]
    #[must_use]
    pub fn visible_row(&self, visible_idx: usize) -> Option<(usize, &TreeRowEntry<Id>)> {
        if self.active_query.is_empty() {
            self.rows.get(visible_idx).map(|row| (visible_idx, row))
        } else {
            let &master_idx = self.filtered_indices.get(visible_idx)?;
            self.rows.get(master_idx).map(|row| (master_idx, row))
        }
    }

    fn rebuild_rows<S: TreeDataSource<Id = Id>>(&mut self, source: &S, tree_state: &TreeState<Id>) {
        self.rows.clear();
        self.visible_order.clear();
        self.collector.clear_buffers();
        self.flatten_epoch = self.flatten_epoch.wrapping_add(1);

        source.collect_nodes(&mut self.collector);
        let node_count = self.collector.all_nodes.len();
        if node_count == 0 {
            return;
        }

        self.rows
            .reserve(node_count.saturating_sub(self.rows.len()));
        if self.collector.parent_map.is_empty() {
            for &id in &self.collector.all_nodes {
                self.rows.push(TreeRowEntry {
                    id,
                    depth: 0,
                    has_children: false,
                    is_expanded: true,
                });
            }
            self.rows.sort_by_key(|r| r.id);
        } else {
            for children in self.collector.children_map.values_mut() {
                children.sort();
            }
            for &id in &self.collector.all_nodes {
                let has_live_parent = self
                    .collector
                    .parent_map
                    .get(&id)
                    .is_some_and(|parent| self.collector.all_nodes.contains(parent));
                if !has_live_parent {
                    self.collector.root_ids.push(id);
                }
            }
            self.collector.root_ids.sort();

            let root_count = self.collector.root_ids.len();
            for idx in 0..root_count {
                let root_id = self.collector.root_ids[idx];
                self.push_dfs_tree(root_id, 0, tree_state);
            }
        }

        self.visible_order.extend(self.rows.iter().map(|r| r.id));
    }

    fn push_dfs_tree(&mut self, id: Id, depth: u16, tree_state: &TreeState<Id>) {
        let child_count = self.collector.children_map.get(&id).map_or(0, Vec::len);
        let has_children = child_count > 0;
        let is_expanded = tree_state.is_expanded(&id);

        self.rows.push(TreeRowEntry {
            id,
            depth,
            has_children,
            is_expanded,
        });

        if is_expanded && has_children {
            for idx in 0..child_count {
                let child_id = self.collector.children_map.get(&id).map(|v| v[idx]);
                if let Some(child) = child_id {
                    self.push_dfs_tree(child, depth + 1, tree_state);
                }
            }
        }
    }

    fn sync_filter<S: TreeDataSource<Id = Id>>(
        &mut self,
        source: &S,
        source_salt: u64,
        search_query: &str,
    ) {
        let trimmed = search_query.trim();
        if self.last_filter_key.as_ref().is_some_and(|key| {
            key.flatten_epoch == self.flatten_epoch
                && key.source_salt == source_salt
                && key.query == trimmed
        }) {
            return;
        }

        self.active_query.clear();
        self.active_query.push_str(trimmed);
        self.filtered_indices.clear();

        if trimmed.is_empty() {
            self.last_filter_key = Some(FilterCacheKey {
                flatten_epoch: self.flatten_epoch,
                source_salt,
                query: String::new(),
            });
            return;
        }

        write_lowercase(trimmed, &mut self.query_lower);
        for (master_idx, row) in self.rows.iter().enumerate() {
            self.scratch_text.clear();
            source.search_text(row.id, &mut self.scratch_text);
            self.scratch_text.make_ascii_lowercase();
            if self.scratch_text.contains(&self.query_lower) {
                self.filtered_indices.push(master_idx);
            }
        }

        self.last_filter_key = Some(FilterCacheKey {
            flatten_epoch: self.flatten_epoch,
            source_salt,
            query: trimmed.to_string(),
        });
    }
}

fn write_lowercase(src: &str, dst: &mut String) {
    dst.clear();
    for ch in src.chars() {
        for lc in ch.to_lowercase() {
            dst.push(lc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTreeSource<'a> {
        items: &'a [(u32, Option<u32>, &'a str)],
    }

    impl<'a> TreeDataSource for MockTreeSource<'a> {
        type Id = u32;

        fn node_key(&self, id: Self::Id) -> u64 {
            u64::from(id)
        }

        fn id_from_key(&self, key: u64) -> Option<Self::Id> {
            u32::try_from(key).ok()
        }

        fn collect_nodes(&self, collector: &mut TreeNodeCollector<Self::Id>) {
            collector.reserve(self.items.len());
            for &(id, parent, _) in self.items {
                collector.push_node(id, parent);
            }
        }

        fn search_text(&self, id: Self::Id, out: &mut String) {
            if let Some(&(_, _, name)) = self.items.iter().find(|&&(nid, _, _)| nid == id) {
                out.push_str(name);
            }
        }
    }

    struct AlternateSource;

    impl TreeDataSource for AlternateSource {
        type Id = u32;

        fn node_key(&self, id: Self::Id) -> u64 {
            u64::from(id)
        }

        fn id_from_key(&self, key: u64) -> Option<Self::Id> {
            u32::try_from(key).ok()
        }

        fn collect_nodes(&self, collector: &mut TreeNodeCollector<Self::Id>) {
            collector.push_node(99, None);
        }

        fn search_text(&self, _id: Self::Id, out: &mut String) {
            out.push_str("AlternateRoot");
        }
    }

    #[test]
    fn test_tree_model_dfs_collapse_and_revision_cache() {
        let items = [
            (1, None, "Root"),
            (2, Some(1), "Branch"),
            (3, Some(2), "Leaf"),
        ];
        let src = MockTreeSource { items: &items };
        let mut state = TreeState::expanded_by_default();
        let mut model = TreeModel::new();

        assert!(model.sync(&src, &state, 1, items.len(), ""));
        assert_eq!(model.visible_count(), 3);
        assert_eq!(model.rows()[0].depth, 0);
        assert_eq!(model.rows()[1].depth, 1);
        assert_eq!(model.rows()[2].depth, 2);

        // Identical revision + len + expansion + source pointer -> cache hit
        assert!(!model.sync(&src, &state, 1, items.len(), ""));

        // Search filter reuses flattened rows without re-running DFS
        assert!(!model.sync(&src, &state, 1, items.len(), "bra"));
        assert_eq!(model.visible_count(), 1);
        assert_eq!(model.visible_row(0).map(|(_, r)| r.id), Some(2));

        // Collapsing Branch increments expansion_revision and hides Leaf
        state.set_expanded(2, false);
        assert!(model.sync(&src, &state, 1, items.len(), ""));
        assert_eq!(model.visible_count(), 2);

        // Swapping to an alternate TreeDataSource updates 
        let alt = AlternateSource;
        assert!(model.sync(&alt, &state, 1, 1, ""));
        assert_eq!(model.visible_count(), 1);
        assert_eq!(model.rows()[0].id, 99);
    }
}