// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Retained Virtualized TreeView Builder (Zero Consumer Model/State)
//!
//! Stores expansion state, multi/range selection, and flattened DFS row caches directly inside
//! [`irisui_core::UiTree`] keyed by `viewport_tag`, freeing panels from owning boilerplate
//! [`TreeModel`](crate::tree_view::TreeModel) or [`TreeState`](crate::tree_view::TreeState)
//! fields.

use irisui_core::{Color, RetainedTreeRow, RetainedTreeState};

use super::context_menu::ContextMenuExt;
use super::core::UiScope;
use super::tree_view::TreeRowProps;
use super::tree_view_virtual::{TreeSelectionEvent, VirtualTreeConfig};
use crate::declarative::types::hash_label;
use crate::declarative::widget::virtual_list::VirtualListResponse;
use crate::tree_view::data_source::{TreeDataSource, TreeNodeCollector, TreeRowEntry};

/// Default no-op selection handler type for [`RetainedVirtualTreeBuilder`].
pub type RetainedDefaultSelectionFn<Ctx, Id> = fn(&mut Ctx, TreeSelectionEvent<Id>);
/// Default no-op context menu builder type for [`RetainedVirtualTreeBuilder`].
pub type RetainedDefaultContextMenuFn<Ctx, Id> = fn(&mut UiScope<'_>, &mut Ctx, Id);

/// Default [`RetainedVirtualTreeBuilder`] type prior to registering selection/menu callbacks.
pub type DefaultRetainedVirtualTreeBuilder<'s, 'a, 'c, S, Ctx> = RetainedVirtualTreeBuilder<
    's,
    'a,
    'c,
    S,
    Ctx,
    RetainedDefaultSelectionFn<Ctx, <S as TreeDataSource>::Id>,
    RetainedDefaultContextMenuFn<Ctx, <S as TreeDataSource>::Id>,
>;

fn noop_selection<Ctx, Id>(_ctx: &mut Ctx, _event: TreeSelectionEvent<Id>) {}
fn noop_context_menu<Ctx, Id>(_menu: &mut UiScope<'_>, _ctx: &mut Ctx, _id: Id) {}

/// Summary returned by [`RetainedVirtualTreeBuilder::render_row`] exposing virtual list metrics
/// alongside cached unfiltered and visible node counts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualTreeResponse {
    /// Underlying virtual list response covering emitted rows and viewport bounds.
    pub list: VirtualListResponse,
    /// Unfiltered total flattened row count currently in the active tree.
    pub total_count: usize,
    /// Visible row count after search filtering.
    pub visible_count: usize,
}

/// Fluent declarative builder orchestrating [`UiTree`](irisui_core::UiTree)-retained tree
/// flattening, expansion toggling, multi-selection (`Ctrl`/`Shift`), search filtering, and
/// context menus with zero panel-owned `TreeModel` or `TreeState`.
pub struct RetainedVirtualTreeBuilder<'s, 'a, 'c, S, Ctx, FSel, FCtx>
where
    S: TreeDataSource,
{
    scope: &'s mut UiScope<'a>,
    source: &'s S,
    domain_revision: u64,
    source_len: usize,
    search_query: &'s str,
    config: &'s VirtualTreeConfig<S::Id>,
    ctx: &'c mut Ctx,
    on_selection: FSel,
    on_context_menu: FCtx,
}

impl<'s, 'a, 'c, S, Ctx, FSel, FCtx> RetainedVirtualTreeBuilder<'s, 'a, 'c, S, Ctx, FSel, FCtx>
where
    S: TreeDataSource,
    FSel: FnMut(&mut Ctx, TreeSelectionEvent<S::Id>),
    FCtx: FnMut(&mut UiScope<'_>, &mut Ctx, S::Id),
{
    /// Registers a selection callback invoked whenever primary or secondary clicks alter selection.
    #[inline]
    pub fn on_selection<NewFSel>(
        self,
        on_selection: NewFSel,
    ) -> RetainedVirtualTreeBuilder<'s, 'a, 'c, S, Ctx, NewFSel, FCtx>
    where
        NewFSel: FnMut(&mut Ctx, TreeSelectionEvent<S::Id>),
    {
        RetainedVirtualTreeBuilder {
            scope: self.scope,
            source: self.source,
            domain_revision: self.domain_revision,
            source_len: self.source_len,
            search_query: self.search_query,
            config: self.config,
            ctx: self.ctx,
            on_selection,
            on_context_menu: self.on_context_menu,
        }
    }

    /// Registers a context menu popup builder invoked when a row receives a secondary click.
    #[inline]
    pub fn context_menu<NewFCtx>(
        self,
        on_context_menu: NewFCtx,
    ) -> RetainedVirtualTreeBuilder<'s, 'a, 'c, S, Ctx, FSel, NewFCtx>
    where
        NewFCtx: FnMut(&mut UiScope<'_>, &mut Ctx, S::Id),
    {
        RetainedVirtualTreeBuilder {
            scope: self.scope,
            source: self.source,
            domain_revision: self.domain_revision,
            source_len: self.source_len,
            search_query: self.search_query,
            config: self.config,
            ctx: self.ctx,
            on_selection: self.on_selection,
            on_context_menu,
        }
    }

    /// Synchronizes retained flattening + search filter caches inside [`irisui_core::UiTree`] and renders
    /// visible rows via [`UiScope::virtual_list`].
    pub fn render_row<FRow>(mut self, mut render_content: FRow) -> VirtualTreeResponse
    where
        FRow: FnMut(&mut UiScope<'_>, &mut Ctx, &TreeRowEntry<S::Id>, usize, bool),
    {
        let tree_tag = self.config.viewport_tag.raw();
        let source_salt = std::ptr::from_ref(self.source) as usize as u64;
        let default_expanded = self.config.default_expanded;
        let initial_expanded = self.config.initial_expanded_keys;
        let (total_count, total_visible, has_filter) = {
            let state =
                self.scope
                    .tree
                    .retained_tree_mut(tree_tag, default_expanded, initial_expanded);
            sync_retained_flatten(
                state,
                self.source,
                self.domain_revision,
                self.source_len,
                source_salt,
            );
            sync_retained_filter(state, self.source, source_salt, self.search_query);
            (
                state.total_count(),
                state.visible_count(),
                state.has_active_filter(),
            )
        };

        let stride = self.config.tree_config.row_stride();
        let empty_msg = if has_filter {
            self.config.filtered_empty_notice
        } else {
            self.config.empty_notice
        };
        let config = self.config;
        let source = self.source;
        let ctx = &mut *self.ctx;
        let on_selection = &mut self.on_selection;
        let on_context_menu = &mut self.on_context_menu;
        let external_selected_key = config.external_selected.map(|id| source.node_key(id));

        let list = self
            .scope
            .virtual_list(total_visible, stride)
            .name(config.viewport_name)
            .tag(tree_tag)
            .background(Color::TRANSPARENT)
            .scrollbar_tags(config.scrollbar_tags.0.raw(), config.scrollbar_tags.1.raw())
            .overscan(2)
            .empty_notice(empty_msg)
            .render_item(|row_scope, visible_idx| {
                let (row_meta, tree_selected) = {
                    let Some(state) = row_scope.tree.retained_tree(tree_tag) else {
                        return;
                    };
                    let Some(row_meta) = state.visible_row(visible_idx) else {
                        return;
                    };
                    (row_meta, state.is_selected(row_meta.key))
                };
                let Some(id) = source.id_from_key(row_meta.key) else {
                    return;
                };
                let entry = TreeRowEntry {
                    id,
                    depth: row_meta.depth,
                    has_children: row_meta.has_children,
                    is_expanded: row_meta.is_expanded,
                };
                let is_selected = tree_selected || external_selected_key == Some(row_meta.key);
                let row_props = TreeRowProps::new(
                    entry.depth,
                    entry.has_children,
                    entry.is_expanded,
                    is_selected,
                    config
                        .row_tag_base
                        .indexed(row_meta.master_idx as u64)
                        .raw(),
                    config
                        .foldout_tag_base
                        .indexed(row_meta.master_idx as u64)
                        .raw(),
                )
                .with_row_index(visible_idx);

                let modifiers = row_scope.modifiers();
                let resp = row_scope
                    .tree_row(config.row_name, row_props, &config.tree_config, |row_s| {
                        render_content(row_s, ctx, &entry, row_meta.master_idx, is_selected);
                    })
                    .context_menu(row_scope, |menu| {
                        on_context_menu(menu, ctx, entry.id);
                    });

                if resp.clicked() {
                    let (changed, primary_key) = {
                        let st = row_scope.tree.retained_tree_mut(
                            tree_tag,
                            default_expanded,
                            initial_expanded,
                        );
                        (
                            st.handle_row_click(row_meta.key, modifiers.ctrl(), modifiers.shift()),
                            st.primary_selected(),
                        )
                    };
                    if changed {
                        let primary = primary_key.and_then(|k| source.id_from_key(k));
                        on_selection(ctx, TreeSelectionEvent::Primary(primary));
                    }
                } else if resp.foldout_clicked {
                    let st = row_scope.tree.retained_tree_mut(
                        tree_tag,
                        default_expanded,
                        initial_expanded,
                    );
                    st.toggle_expanded(row_meta.key);
                } else if resp.secondary_clicked() {
                    row_scope
                        .tree
                        .retained_tree_mut(tree_tag, default_expanded, initial_expanded)
                        .select_single(row_meta.key);
                    on_selection(ctx, TreeSelectionEvent::Secondary(entry.id));
                }
            });

        VirtualTreeResponse {
            list,
            total_count,
            visible_count: total_visible,
        }
    }
}

fn sync_retained_flatten<S: TreeDataSource>(
    state: &mut RetainedTreeState,
    source: &S,
    domain_revision: u64,
    source_len: usize,
    source_salt: u64,
) {
    if state.flatten_cache_hits(domain_revision, source_len, source_salt) {
        return;
    }
    state.begin_flatten_rebuild();

    let mut collector = TreeNodeCollector::<S::Id>::default();
    source.collect_nodes(&mut collector);
    if !collector.all_nodes.is_empty() {
        if collector.parent_map.is_empty() {
            collector.all_nodes.sort();
            for (master_idx, &id) in collector.all_nodes.iter().enumerate() {
                state.push_flattened_row(RetainedTreeRow {
                    key: source.node_key(id),
                    depth: 0,
                    has_children: false,
                    is_expanded: true,
                    master_idx,
                });
            }
        } else {
            for children in collector.children_map.values_mut() {
                children.sort();
            }
            for &id in &collector.all_nodes {
                let has_live_parent = collector
                    .parent_map
                    .get(&id)
                    .is_some_and(|parent| collector.all_nodes.contains(parent));
                if !has_live_parent {
                    collector.root_ids.push(id);
                }
            }
            collector.root_ids.sort();
            let root_count = collector.root_ids.len();
            for idx in 0..root_count {
                let root_id = collector.root_ids[idx];
                push_retained_dfs(state, source, &collector, root_id, 0);
            }
        }
    }
    state.finish_flatten_rebuild(domain_revision, source_len, source_salt);
}

fn push_retained_dfs<S: TreeDataSource>(
    state: &mut RetainedTreeState,
    source: &S,
    collector: &TreeNodeCollector<S::Id>,
    id: S::Id,
    depth: u16,
) {
    let key = source.node_key(id);
    let child_count = collector.children_map.get(&id).map_or(0, Vec::len);
    let has_children = child_count > 0;
    let is_expanded = state.is_expanded(key);
    let master_idx = state.total_count();

    state.push_flattened_row(RetainedTreeRow {
        key,
        depth,
        has_children,
        is_expanded,
        master_idx,
    });

    if is_expanded && has_children {
        for idx in 0..child_count {
            if let Some(child_id) = collector.children_map.get(&id).and_then(|v| v.get(idx)) {
                push_retained_dfs(state, source, collector, *child_id, depth + 1);
            }
        }
    }
}

fn sync_retained_filter<S: TreeDataSource>(
    state: &mut RetainedTreeState,
    source: &S,
    source_salt: u64,
    search_query: &str,
) {
    let trimmed = search_query.trim();
    let query_hash = if trimmed.is_empty() {
        0
    } else {
        hash_label(trimmed)
    };
    if state.filter_cache_hits(source_salt, query_hash) {
        return;
    }

    state.begin_filter(!trimmed.is_empty());
    if !trimmed.is_empty() {
        let query_lower = trimmed.to_lowercase();
        let mut scratch = String::new();
        let total = state.rows().len();
        for master_idx in 0..total {
            let key = state.rows()[master_idx].key;
            if let Some(id) = source.id_from_key(key) {
                scratch.clear();
                source.search_text(id, &mut scratch);
                scratch.make_ascii_lowercase();
                if scratch.contains(&query_lower) {
                    state.push_filtered_index(master_idx);
                }
            }
        }
    }
    state.finish_filter(source_salt, query_hash);
}

impl<'a> UiScope<'a> {
    /// Creates a [`RetainedVirtualTreeBuilder`] backed directly by [`UiTree`](irisui_core::UiTree)
    /// expansion, selection, and flattened DFS row caches without consumer-owned `TreeModel` or
    /// `TreeState` fields.
    #[inline]
    pub fn virtual_tree_retained<'s, 'c, S, Ctx>(
        &'s mut self,
        source: &'s S,
        domain_revision: u64,
        source_len: usize,
        search_query: &'s str,
        config: &'s VirtualTreeConfig<S::Id>,
        ctx: &'c mut Ctx,
    ) -> DefaultRetainedVirtualTreeBuilder<'s, 'a, 'c, S, Ctx>
    where
        S: TreeDataSource,
    {
        RetainedVirtualTreeBuilder {
            scope: self,
            source,
            domain_revision,
            source_len,
            search_query,
            config,
            ctx,
            on_selection: noop_selection::<Ctx, S::Id>,
            on_context_menu: noop_context_menu::<Ctx, S::Id>,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::{UiTag, UiTree};

    struct SliceSource<'a>(&'a [(u32, Option<u32>, &'a str)]);

    impl<'a> TreeDataSource for SliceSource<'a> {
        type Id = u32;

        fn node_key(&self, id: Self::Id) -> u64 {
            u64::from(id)
        }

        fn id_from_key(&self, key: u64) -> Option<Self::Id> {
            u32::try_from(key).ok()
        }

        fn collect_nodes(&self, collector: &mut TreeNodeCollector<Self::Id>) {
            for &(id, parent, _) in self.0 {
                collector.push_node(id, parent);
            }
        }

        fn search_text(&self, id: Self::Id, out: &mut String) {
            if let Some(&(_, _, label)) = self.0.iter().find(|&&(nid, _, _)| nid == id) {
                out.push_str(label);
            }
        }
    }

    #[test]
    fn test_virtual_tree_retained_manages_expansion_selection_and_filtering() {
        const VP: UiTag = UiTag::from_str("test.vt_ret.vp");
        const TRACK: UiTag = UiTag::from_str("test.vt_ret.track");
        const THUMB: UiTag = UiTag::from_str("test.vt_ret.thumb");
        const ROW: UiTag = UiTag::from_str("test.vt_ret.row");
        const FOLD: UiTag = UiTag::from_str("test.vt_ret.fold");

        let data = [(10, None, "Root"), (20, Some(10), "Child")];
        let src = SliceSource(&data);
        let cfg = VirtualTreeConfig::new("RetVp", "RetRow", VP, (TRACK, THUMB), ROW, FOLD)
            .with_default_expanded(true);

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        tree.event_state.frame_clicked_tag = Some(ROW.indexed(1).raw());
        let mut emitted = Vec::new();
        let resp = {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .virtual_tree_retained(&src, 1, data.len(), "", &cfg, &mut emitted)
                .on_selection(|out, ev| out.push(ev))
                .render_row(|row_s, _out, entry, _idx, _sel| {
                    row_s.label(if entry.id == 10 { "Root" } else { "Child" });
                })
        };
        assert_eq!(resp.total_count, 2);
        assert_eq!(resp.visible_count, 2);
        assert_eq!(emitted, vec![TreeSelectionEvent::Primary(Some(20))]);
        assert!(
            tree.retained_tree(VP.raw())
                .expect("retained")
                .is_selected(20)
        );

        // Foldout collapse persists directly inside UiTree without consumer TreeState.
        tree.clear();
        let root2 = tree.create_root().expect("root2");
        tree.event_state.frame_clicked_tag = Some(FOLD.indexed(0).raw());
        {
            let mut scope = UiScope::new(&mut tree, root2);
            scope
                .virtual_tree_retained(&src, 1, data.len(), "", &cfg, &mut emitted)
                .render_row(|row_s, _out, entry, _idx, _sel| {
                    row_s.label(if entry.id == 10 { "Root" } else { "Child" });
                });
        }
        tree.clear();
        let root3 = tree.create_root().expect("root3");
        tree.event_state.frame_clicked_tag = None;
        let resp_collapsed = {
            let mut scope = UiScope::new(&mut tree, root3);
            scope
                .virtual_tree_retained(&src, 1, data.len(), "", &cfg, &mut emitted)
                .render_row(|row_s, _out, entry, _idx, _sel| {
                    row_s.label(if entry.id == 10 { "Root" } else { "Child" });
                })
        };
        assert_eq!(resp_collapsed.visible_count, 1);
    }
}