// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Virtualized TreeView Builder Bound to [`TreeModel`]
//!
//! Connects [`TreeModel`] and [`TreeState`] directly to [`VirtualList`] frustum culling,
//! deterministic [`UiTag`] row/foldout tagging, multi-selection (`Ctrl` / `Shift`),
//! foldout chevron toggling, and context menus — eliminating per-panel `VirtualList`
//! and `handle_tree_row_click` boilerplate in consumer applications.

use super::context_menu::ContextMenuExt;
use super::core::UiScope;
use super::tree_view::{TreeRowProps, TreeViewConfig};
use crate::declarative::widget::virtual_list::VirtualListResponse;
use crate::tree_view::data_source::{TreeModel, TreeRowEntry};
use crate::tree_view::model::TreeState;
use crate::tree_view::selection::{TreeSelectionModifiers, handle_tree_row_click};
use irisui_core::{Color, UiTag};
use std::hash::Hash;

/// Configuration descriptor for rendering a virtualized interactive [`TreeModel`] on [`UiScope`].
#[derive(Debug, Clone)]
pub struct VirtualTreeConfig<Id> {
    /// Debug name assigned to the scrollable virtual list viewport node.
    pub viewport_name: &'static str,
    /// Debug name assigned to each visible tree row node.
    pub row_name: &'static str,
    /// Semantic [`UiTag`] bound to the scrollable viewport.
    pub viewport_tag: UiTag,
    /// Semantic [`UiTag`] pair `(track, thumb)` bound to the overlay vertical scrollbar.
    pub scrollbar_tags: (UiTag, UiTag),
    /// Base [`UiTag`] indexed via [`UiTag::indexed`] for individual visible rows.
    pub row_tag_base: UiTag,
    /// Base [`UiTag`] indexed via [`UiTag::indexed`] for foldout chevrons.
    pub foldout_tag_base: UiTag,
    /// Message displayed when the underlying tree model has zero total items.
    pub empty_notice: &'static str,
    /// Message displayed when a search filter matches zero items.
    pub filtered_empty_notice: &'static str,
    /// Optional external primary selected identifier highlighted in addition to `TreeState`.
    pub external_selected: Option<Id>,
    /// Default expansion behavior applied by [`UiScope::virtual_tree_retained`].
    pub default_expanded: bool,
    /// Optional initial 64-bit node keys seeded expanded on first initialization.
    pub initial_expanded_keys: &'static [u64],
    /// Row styling, indentation, height, and branch line visual configuration.
    pub tree_config: TreeViewConfig,
}

impl<Id> VirtualTreeConfig<Id> {
    /// Creates a [`VirtualTreeConfig`] with semantic tags and sensible defaults.
    #[inline]
    #[must_use]
    pub fn new(
        viewport_name: &'static str,
        row_name: &'static str,
        viewport_tag: UiTag,
        scrollbar_tags: (UiTag, UiTag),
        row_tag_base: UiTag,
        foldout_tag_base: UiTag,
    ) -> Self {
        Self {
            viewport_name,
            row_name,
            viewport_tag,
            scrollbar_tags,
            row_tag_base,
            foldout_tag_base,
            empty_notice: "Tree is empty",
            filtered_empty_notice: "No matching items found",
            external_selected: None,
            default_expanded: false,
            initial_expanded_keys: &[],
            tree_config: TreeViewConfig::new(),
        }
    }

    /// Sets the fallback notice shown when the unfiltered tree is empty.
    #[inline]
    #[must_use]
    pub const fn with_empty_notice(mut self, empty_notice: &'static str) -> Self {
        self.empty_notice = empty_notice;
        self
    }

    /// Sets the notice shown when search filtering hides all nodes.
    #[inline]
    #[must_use]
    pub const fn with_filtered_empty_notice(mut self, filtered_empty_notice: &'static str) -> Self {
        self.filtered_empty_notice = filtered_empty_notice;
        self
    }

    /// Synchronizes external selection highlighting for rows selected outside [`TreeState`].
    #[inline]
    #[must_use]
    pub fn with_external_selected(mut self, external_selected: Option<Id>) -> Self {
        self.external_selected = external_selected;
        self
    }

    /// Configures whether nodes default to expanded or collapsed in [`UiScope::virtual_tree_retained`].
    #[inline]
    #[must_use]
    pub const fn with_default_expanded(mut self, default_expanded: bool) -> Self {
        self.default_expanded = default_expanded;
        self
    }

    /// Seeds initial expanded 64-bit node keys on first [`UiScope::virtual_tree_retained`] render.
    #[inline]
    #[must_use]
    pub const fn with_initial_expanded_keys(mut self, keys: &'static [u64]) -> Self {
        self.initial_expanded_keys = keys;
        self
    }

    /// Applies visual row styling, indentation, height, and branch line geometry.
    #[inline]
    #[must_use]
    pub fn with_tree_config(mut self, tree_config: TreeViewConfig) -> Self {
        self.tree_config = tree_config;
        self
    }
}

/// Interaction events emitted by [`VirtualTreeBuilder::render_row`] when selection changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSelectionEvent<Id> {
    /// Primary click updated active selection to `Option<Id>`.
    Primary(Option<Id>),
    /// Secondary (right) click focused `Id` before opening its row context menu.
    Secondary(Id),
}

/// Default selection handler function pointer used before `.on_selection(...)` is chained.
pub type DefaultSelectionFn<Ctx, Id> = fn(&mut Ctx, TreeSelectionEvent<Id>);

/// Default context menu handler function pointer used before `.context_menu(...)` is chained.
pub type DefaultContextMenuFn<Ctx, Id> = fn(&mut UiScope<'_>, &mut Ctx, Id);

/// Default [`VirtualTreeBuilder`] type alias returned by [`UiScope::virtual_tree`].
pub type DefaultVirtualTreeBuilder<'s, 'a, 'm, 't, 'c, Id, Ctx> = VirtualTreeBuilder<
    's,
    'a,
    'm,
    't,
    'c,
    Id,
    Ctx,
    DefaultSelectionFn<Ctx, Id>,
    DefaultContextMenuFn<Ctx, Id>,
>;

fn noop_selection<Ctx, Id>(_ctx: &mut Ctx, _event: TreeSelectionEvent<Id>) {}
fn noop_context_menu<Ctx, Id>(_menu: &mut UiScope<'_>, _ctx: &mut Ctx, _id: Id) {}

/// Fluent declarative builder orchestrating [`TreeModel`] virtualization, selection,
/// foldout toggling, and per-row context menus without intermediate consumer `let` bindings.
pub struct VirtualTreeBuilder<'s, 'a, 'm, 't, 'c, Id, Ctx, FSel, FCtx>
where
    Id: Copy + Eq + Hash + Ord,
{
    scope: &'s mut UiScope<'a>,
    model: &'m TreeModel<Id>,
    tree_state: &'t mut TreeState<Id>,
    config: &'m VirtualTreeConfig<Id>,
    ctx: &'c mut Ctx,
    on_selection: FSel,
    on_context_menu: FCtx,
}

impl<'s, 'a, 'm, 't, 'c, Id, Ctx, FSel, FCtx>
    VirtualTreeBuilder<'s, 'a, 'm, 't, 'c, Id, Ctx, FSel, FCtx>
where
    Id: Copy + Eq + Hash + Ord,
    FSel: FnMut(&mut Ctx, TreeSelectionEvent<Id>),
    FCtx: FnMut(&mut UiScope<'_>, &mut Ctx, Id),
{
    /// Registers a selection callback invoked whenever primary or secondary clicks alter selection.
    #[inline]
    pub fn on_selection<NewFSel>(
        self,
        on_selection: NewFSel,
    ) -> VirtualTreeBuilder<'s, 'a, 'm, 't, 'c, Id, Ctx, NewFSel, FCtx>
    where
        NewFSel: FnMut(&mut Ctx, TreeSelectionEvent<Id>),
    {
        VirtualTreeBuilder {
            scope: self.scope,
            model: self.model,
            tree_state: self.tree_state,
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
    ) -> VirtualTreeBuilder<'s, 'a, 'm, 't, 'c, Id, Ctx, FSel, NewFCtx>
    where
        NewFCtx: FnMut(&mut UiScope<'_>, &mut Ctx, Id),
    {
        VirtualTreeBuilder {
            scope: self.scope,
            model: self.model,
            tree_state: self.tree_state,
            config: self.config,
            ctx: self.ctx,
            on_selection: self.on_selection,
            on_context_menu,
        }
    }

    /// Executes the virtualized tree pass, emitting visible rows via `render_content`.
    ///
    /// Handles empty/filtered placeholders, overscan culling, `UiTag`-indexed row & foldout
    /// tagging, `Ctrl`/`Shift` multi-selection, foldout chevron toggling, and context menus.
    pub fn render_row<FRow>(mut self, mut render_content: FRow) -> VirtualListResponse
    where
        FRow: FnMut(&mut UiScope<'_>, &mut Ctx, &TreeRowEntry<Id>, usize, bool),
    {
        let total_visible = self.model.visible_count();
        let stride = self.config.tree_config.row_stride();
        let empty_msg = if self.model.has_active_filter() {
            self.config.filtered_empty_notice
        } else {
            self.config.empty_notice
        };
        let tree_state = &mut *self.tree_state;
        let model = self.model;
        let config = self.config;
        let ctx = &mut *self.ctx;
        let on_selection = &mut self.on_selection;
        let on_context_menu = &mut self.on_context_menu;

        self.scope
            .virtual_list(total_visible, stride)
            .name(config.viewport_name)
            .tag(config.viewport_tag.raw())
            .background(Color::TRANSPARENT)
            .scrollbar_tags(config.scrollbar_tags.0.raw(), config.scrollbar_tags.1.raw())
            .overscan(2)
            .empty_notice(empty_msg)
            .render_item(|row_scope, visible_idx| {
                let Some((master_idx, entry)) = model.visible_row(visible_idx) else {
                    return;
                };
                let is_selected =
                    tree_state.is_selected(&entry.id) || config.external_selected == Some(entry.id);
                let row_props = TreeRowProps::new(
                    entry.depth,
                    entry.has_children,
                    entry.is_expanded,
                    is_selected,
                    config.row_tag_base.indexed(master_idx as u64).raw(),
                    config.foldout_tag_base.indexed(master_idx as u64).raw(),
                )
                .with_row_index(visible_idx);

                let modifiers = row_scope.modifiers();
                let resp = row_scope
                    .tree_row(config.row_name, row_props, &config.tree_config, |row_s| {
                        render_content(row_s, ctx, entry, master_idx, is_selected);
                    })
                    .context_menu(row_scope, |menu| {
                        on_context_menu(menu, ctx, entry.id);
                    });

                if resp.clicked() {
                    let click_res = handle_tree_row_click(
                        tree_state,
                        entry.id,
                        TreeSelectionModifiers::from_ctrl_shift(
                            modifiers.ctrl(),
                            modifiers.shift(),
                        ),
                        modifiers.shift().then(|| model.visible_order()),
                    );
                    if click_res.selection_changed {
                        on_selection(
                            ctx,
                            TreeSelectionEvent::Primary(tree_state.primary_selected().copied()),
                        );
                    }
                } else if resp.foldout_clicked {
                    tree_state.toggle_expanded(&entry.id);
                } else if resp.secondary_clicked() {
                    tree_state.select_single(entry.id);
                    on_selection(ctx, TreeSelectionEvent::Secondary(entry.id));
                }
            })
    }
}

impl<'a> UiScope<'a> {
    /// Creates a fluent [`VirtualTreeBuilder`] rendering an interactive cached [`TreeModel`]
    /// backed by any [`TreeDataSource`], threading mutable caller context `ctx` cleanly
    /// across row rendering, selection callbacks, and context menus.
    #[inline]
    pub fn virtual_tree<'s, 'm, 't, 'c, Id, Ctx>(
        &'s mut self,
        model: &'m TreeModel<Id>,
        tree_state: &'t mut TreeState<Id>,
        config: &'m VirtualTreeConfig<Id>,
        ctx: &'c mut Ctx,
    ) -> DefaultVirtualTreeBuilder<'s, 'a, 'm, 't, 'c, Id, Ctx>
    where
        Id: Copy + Eq + Hash + Ord,
    {
        VirtualTreeBuilder {
            scope: self,
            model,
            tree_state,
            config,
            ctx,
            on_selection: noop_selection::<Ctx, Id>,
            on_context_menu: noop_context_menu::<Ctx, Id>,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_view::data_source::{TreeDataSource, TreeNodeCollector};
    use irisui_core::UiTree;

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
    fn test_virtual_tree_builder_renders_rows_handles_selection_and_empty_state() {
        const VP: UiTag = UiTag::from_str("test.vt.vp");
        const TRACK: UiTag = UiTag::from_str("test.vt.track");
        const THUMB: UiTag = UiTag::from_str("test.vt.thumb");
        const ROW: UiTag = UiTag::from_str("test.vt.row");
        const FOLD: UiTag = UiTag::from_str("test.vt.fold");

        let data = [(10, None, "Root"), (20, Some(10), "Child")];
        let src = SliceSource(&data);
        let mut model = TreeModel::new();
        let mut state = TreeState::expanded_by_default();
        model.sync(&src, &state, 1, data.len(), "");

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        tree.event_state.frame_clicked_tag = Some(ROW.indexed(1).raw());

        let mut emitted = Vec::new();
        let cfg =
            VirtualTreeConfig::new("TestTreeVp", "TestTreeRow", VP, (TRACK, THUMB), ROW, FOLD);
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .virtual_tree(&model, &mut state, &cfg, &mut emitted)
                .on_selection(|out, ev| out.push(ev))
                .render_row(|row_s, _out, entry, _idx, _sel| {
                    row_s.label(if entry.id == 10 { "Root" } else { "Child" });
                });
        }

        assert_eq!(emitted, vec![TreeSelectionEvent::Primary(Some(20))]);
        assert!(state.is_selected(&20));
    }
}