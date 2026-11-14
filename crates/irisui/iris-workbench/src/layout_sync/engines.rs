// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Framework Subtree Engines Bundle
//!
//! Provides the [`SubtreeEngines`] descriptor bundling mutable references to [`UiTree`],
//! [`LayoutEngine`], and [`TextSystem`] into an ergonomic container for layout passes and
//! granular $O(M)$ subtree reconstruction cycles.

use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::layout_sync::dual_layout::DualLayoutEliminator;
use iris_core::{UiTree, WidgetId};
use iris_dock::UiNotifier;
use iris_layout::{LayoutEngine, LayoutError};
use iris_text::TextSystem;
use iris_widgets::declarative::UiScope;
use std::fmt::Debug;
use std::hash::Hash;

/// Mutable references to core framework rendering and layout engines.
///
/// Bundles [`UiTree`], [`LayoutEngine`], and [`TextSystem`] into a single ergonomic descriptor
/// to ensure clean architecture and prevent argument proliferation across layout sync passes.
pub struct SubtreeEngines<'a> {
    /// Generational UI widget tree.
    pub tree: &'a mut UiTree,
    /// Flexbox boundary solver layout engine.
    pub layout_engine: &'a mut LayoutEngine,
    /// Text measurement and font layout system.
    pub text_system: &'a mut TextSystem,
}

impl<'a> SubtreeEngines<'a> {
    /// Constructs a new bundle of framework layout engines.
    #[inline]
    pub fn new(
        tree: &'a mut UiTree,
        layout_engine: &'a mut LayoutEngine,
        text_system: &'a mut TextSystem,
    ) -> Self {
        Self {
            tree,
            layout_engine,
            text_system,
        }
    }

    /// Atomically rebuilds an individual leaf panel via declarative closure.
    #[inline]
    pub fn rebuild_leaf<F>(&mut self, leaf_id: WidgetId, render_fn: F) -> Result<(), LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        DualLayoutEliminator::rebuild_leaf_panel(
            self.tree,
            self.layout_engine,
            self.text_system,
            leaf_id,
            render_fn,
        )
    }

    /// Atomically rebuilds an individual leaf panel in-place via declarative closure.
    ///
    /// Validates that the container node exists and has non-zero computed dimensions.
    /// Untracks previous child layout nodes from Taffy, purges previous descendant widgets,
    /// synthesizes fresh widgets via `render_fn`, shapes typography, and resolves boundary
    /// Taffy flexbox models in $O(M)$ time without full tree clears or generational resets.
    /// Re-seals the leaf as an Invalidation Boundary and flags it for repainting and text preparation.
    ///
    /// Returns `Ok(true)` if reconstructed, or `Ok(false)` if the leaf was missing or collapsed (zero-area).
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    #[inline]
    pub fn rebuild_single_panel_leaf<F>(
        &mut self,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        DualLayoutEliminator::rebuild_single_panel_leaf(
            self.tree,
            self.layout_engine,
            self.text_system,
            leaf_id,
            render_fn,
        )
    }

    /// Atomically rebuilds an individual leaf panel with a fallible render closure.
    ///
    /// Encapsulates [`SubtreeSession`](iris_widgets::declarative::SubtreeSession) execution,
    /// guaranteeing that any error returned by `render_fn` safely aborts layout computation
    /// and is propagated as a [`LayoutError::Taffy`].
    ///
    /// Returns `Ok(true)` if reconstructed, or `Ok(false)` if missing or zero-sized.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if the node is missing, geometry is invalid, or `render_fn` fails.
    #[inline]
    pub fn rebuild_single_panel_leaf_fallible<E, F>(
        &mut self,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>) -> Result<(), E>,
        E: std::fmt::Display,
    {
        DualLayoutEliminator::rebuild_single_panel_leaf_fallible(
            self.tree,
            self.layout_engine,
            self.text_system,
            leaf_id,
            render_fn,
        )
    }

    /// Atomically rebuilds an individual leaf panel with an eliminator instance, recording text prepare.
    #[inline]
    pub fn rebuild_single_panel_leaf_with_eliminator<F>(
        &mut self,
        eliminator: &mut DualLayoutEliminator,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        eliminator.rebuild_single_panel_leaf_instance(
            self.tree,
            self.layout_engine,
            self.text_system,
            leaf_id,
            render_fn,
        )
    }

    /// Atomically rebuilds an individual registered dock panel.
    #[inline]
    pub fn rebuild_panel<TabId, Context>(
        &mut self,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        panel_id: TabId,
        leaf_id: WidgetId,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
    {
        DualLayoutEliminator::rebuild_panel(self, registry, context, panel_id, leaf_id)
    }

    /// Atomically rebuilds an individual registered dock panel and tracks text preparation on `eliminator`.
    #[inline]
    pub fn rebuild_panel_with_eliminator<TabId, Context>(
        &mut self,
        eliminator: &mut DualLayoutEliminator,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        panel_id: TabId,
        leaf_id: WidgetId,
    ) -> Result<bool, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
    {
        let rebuilt =
            DualLayoutEliminator::rebuild_panel(self, registry, context, panel_id, leaf_id)?;
        if rebuilt {
            eliminator.mark_needs_text_prepare();
        }
        Ok(rebuilt)
    }

    /// Rebuilds all dirty dock panels flagged in `notifier` while mapping physical dirty boundaries.
    #[inline]
    pub fn rebuild_dirty_panels_with_boundaries<TabId, Context, F, B>(
        &mut self,
        eliminator: &mut DualLayoutEliminator,
        notifier: &mut UiNotifier<TabId>,
        registry: &mut WorkbenchPanelRegistry<TabId, Context>,
        context: &mut Context,
        leaf_resolver: F,
        boundary_resolver: B,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
        B: FnMut(WidgetId) -> Option<TabId>,
    {
        eliminator.rebuild_dirty_panels_with_boundaries(
            self,
            notifier,
            registry,
            context,
            leaf_resolver,
            boundary_resolver,
        )
    }
}