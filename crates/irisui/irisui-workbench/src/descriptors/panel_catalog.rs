// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Shared Panel Metadata Catalog & Tab Viewer Adapters
//!
//! Centralizes panel metadata lookup, domain-panel subscription tracking, [`TabViewer`]
//! presentation helpers, and leaf drawing dispatch for both owned and scoped registries.

use super::panel::{LiveRefreshInterval, PanelMetadata};
use crate::error::WorkbenchResult;
use irisui_core::{DirtyFlags, UiTree, WidgetId};
use irisui_dock::TabViewer;
use irisui_widgets::UiScope;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// Shared catalog storing [`PanelMetadata`] descriptors and domain-bound subscriptions.
#[derive(Debug, Clone)]
pub struct PanelCatalog<TabId> {
    metadata: HashMap<TabId, PanelMetadata<TabId>>,
    domain_panels: HashSet<TabId>,
    live_panels: Vec<(TabId, LiveRefreshInterval)>,
}

impl<TabId> Default for PanelCatalog<TabId> {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            domain_panels: HashSet::new(),
            live_panels: Vec::new(),
        }
    }
}

impl<TabId> PanelCatalog<TabId> {
    /// Returns registered panels declaring periodic live refresh behavior without allocating.
    #[inline]
    #[must_use]
    pub fn live_panels(&self) -> &[(TabId, LiveRefreshInterval)] {
        &self.live_panels
    }
}

impl<TabId> PanelCatalog<TabId>
where
    TabId: Eq + Hash + Clone + Debug,
{
    /// Creates an empty panel metadata catalog.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces metadata for a panel and records its domain subscription flag.
    pub fn record_metadata(&mut self, meta: PanelMetadata<TabId>) {
        if meta.is_domain_bound {
            self.domain_panels.insert(meta.id.clone());
        } else {
            self.domain_panels.remove(&meta.id);
        }
        self.live_panels.retain(|(id, _)| id != &meta.id);
        if meta.live_refresh != LiveRefreshInterval::OnChange {
            self.live_panels.push((meta.id.clone(), meta.live_refresh));
        }
        self.metadata.insert(meta.id.clone(), meta);
    }

    /// Copies all metadata entries and domain-panel subscriptions from `other` into `self`.
    pub fn import_metadata_from(&mut self, other: &Self) {
        for meta in other.metadata.values() {
            self.record_metadata(meta.clone());
        }
        for tab in &other.domain_panels {
            self.domain_panels.insert(tab.clone());
        }
    }

    /// Marks `tab` as a subscriber to domain-level invalidation notifications.
    pub fn mark_domain_panel(&mut self, tab: TabId) {
        self.domain_panels.insert(tab);
    }

    /// Removes `tab` from domain-level invalidation subscriptions, returning whether it existed.
    pub fn unmark_domain_panel(&mut self, tab: &TabId) -> bool {
        self.domain_panels.remove(tab)
    }

    /// Returns `true` if `tab` is registered as a domain data-bound subscriber.
    #[inline]
    #[must_use]
    pub fn is_domain_panel(&self, tab: &TabId) -> bool {
        self.domain_panels.contains(tab)
    }

    /// Returns the set of registered domain data-bound panel identifiers.
    #[inline]
    #[must_use]
    pub fn domain_panels(&self) -> &HashSet<TabId> {
        &self.domain_panels
    }

    /// Synchronizes all registered domain panels into an active [`irisui_dock::UiNotifier`].
    pub fn sync_domain_panels_to_notifier(&self, notifier: &mut irisui_dock::UiNotifier<TabId>)
    where
        TabId: Copy + Send + Sync + 'static,
    {
        for &tab in &self.domain_panels {
            notifier.register_domain_panel(tab);
        }
    }

    /// Removes metadata and domain subscription records for `tab`.
    pub fn remove(&mut self, tab: &TabId) {
        self.domain_panels.remove(tab);
        self.live_panels.retain(|(id, _)| id != tab);
        self.metadata.remove(tab);
    }

    /// Returns metadata describing `tab` if registered.
    #[inline]
    #[must_use]
    pub fn panel_metadata(&self, tab: &TabId) -> Option<&PanelMetadata<TabId>> {
        self.metadata.get(tab)
    }

    /// Constructs a read-only [`RegistryTabViewer`] backed by this catalog.
    #[inline]
    #[must_use]
    pub fn tab_viewer(&self) -> RegistryTabViewer<'_, TabId> {
        RegistryTabViewer::new(self)
    }

    /// Formats the user-visible tab title for `tab` (including optional unicode icon prefix).
    #[must_use]
    pub fn tab_title(&self, tab: &TabId) -> String {
        match self.metadata.get(tab) {
            Some(meta) => match meta.icon {
                Some(irisui_core::UiIcon::Glyph(glyph)) => format!("{glyph} {}", meta.title),
                Some(irisui_core::UiIcon::Atlas(_)) | None => meta.title.clone(),
            },
            None => format!("{tab:?}"),
        }
    }

    /// Returns the raw panel title without emoji icon prefixes for `tab`.
    #[must_use]
    pub fn tab_raw_title(&self, tab: &TabId) -> String {
        if let Some(meta) = self.metadata.get(tab) {
            meta.title.clone()
        } else {
            format!("{tab:?}")
        }
    }

    /// Returns the unified optional [`irisui_core::UiIcon`] registered for `tab`.
    #[inline]
    #[must_use]
    pub fn tab_icon(&self, tab: &TabId) -> Option<irisui_core::UiIcon> {
        self.metadata.get(tab).and_then(|m| m.icon)
    }

    /// Returns whether `tab` permits user closing in the dock tab bar.
    #[inline]
    #[must_use]
    pub fn tab_closeable(&self, tab: &TabId) -> bool {
        self.metadata.get(tab).is_none_or(|m| m.closeable)
    }

    /// Returns whether the background behind `tab` content stays transparent.
    #[inline]
    #[must_use]
    pub fn tab_clear_background(&self, tab: &TabId) -> bool {
        self.metadata.get(tab).is_some_and(|m| m.clear_background)
    }
}

/// Unified dispatch interface implemented by both owned and lifetime-scoped panel registries.
///
/// Enables `DualLayoutEliminator` and `Workbench` shell orchestration to render panels
/// cleanly regardless of whether the registry context borrows transient frame references.
pub trait PanelRegistryTarget<TabId, Ctx> {
    /// Returns the shared panel metadata catalog for tab presentation queries.
    fn catalog(&self) -> &PanelCatalog<TabId>;

    /// Invokes the registered panel or callback for `tab` inside `scope`.
    ///
    /// # Errors
    /// Returns [`crate::error::WorkbenchError::PanelNotFound`] when `tab` is unregistered.
    fn draw(
        &mut self,
        tab: &TabId,
        scope: &mut UiScope<'_>,
        context: &mut Ctx,
    ) -> WorkbenchResult<()>;

    /// Executes [`ScopedWorkbenchPanel::refresh_live`] for `tab` during `InPlaceRefresh`.
    fn refresh_live(&mut self, tab: &TabId, scope: &mut UiScope<'_>, context: &mut Ctx);
}

/// Draws `tab` into `leaf_id`, seals `leaf_id` as an Invalidation Boundary, and marks it dirty.
///
/// # Errors
/// Propagates [`crate::error::WorkbenchError::PanelNotFound`] if `tab` is missing.
pub fn draw_leaf_on_target<TabId, Ctx, R>(
    target: &mut R,
    tab: &TabId,
    tree: &mut UiTree,
    leaf_id: WidgetId,
    context: &mut Ctx,
) -> WorkbenchResult<()>
where
    R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
{
    let mut scope = UiScope::new(tree, leaf_id);
    target.draw(tab, &mut scope, context)?;
    tree.set_boundary(leaf_id, true);
    tree.mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
    Ok(())
}

/// Renders all dirty tabs resolved via `leaf_resolver` into `target` and returns rendered count.
pub fn render_dirty_tabs_on_target<TabId, Ctx, R, I, F>(
    target: &mut R,
    dirty_tabs: I,
    tree: &mut UiTree,
    leaf_resolver: F,
    context: &mut Ctx,
) -> usize
where
    R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
    I: IntoIterator,
    I::Item: Borrow<TabId>,
    F: Fn(&TabId) -> Option<WidgetId>,
{
    let mut rendered = 0;
    for tab in dirty_tabs {
        let tab = tab.borrow();
        if let Some(leaf_id) = leaf_resolver(tab)
            && draw_leaf_on_target(target, tab, tree, leaf_id, context).is_ok()
        {
            rendered += 1;
        }
    }
    rendered
}

/// Read-only [`TabViewer`] adapter delegating tab titles, icons, and policies to [`PanelCatalog`].
pub struct RegistryTabViewer<'a, TabId> {
    catalog: &'a PanelCatalog<TabId>,
}

impl<'a, TabId> RegistryTabViewer<'a, TabId> {
    /// Constructs a read-only tab viewer adapter referencing `catalog`.
    #[inline]
    #[must_use]
    pub fn new(catalog: &'a PanelCatalog<TabId>) -> Self {
        Self { catalog }
    }
}

impl<TabId> TabViewer<TabId> for RegistryTabViewer<'_, TabId>
where
    TabId: Eq + Hash + Clone + Debug,
{
    fn title(&self, tab: &TabId) -> String {
        self.catalog.tab_title(tab)
    }

    fn raw_title(&self, tab: &TabId) -> String {
        self.catalog.tab_raw_title(tab)
    }

    fn icon(&self, tab: &TabId) -> Option<irisui_core::UiIcon> {
        self.catalog.tab_icon(tab)
    }

    fn closeable(&self, tab: &TabId) -> bool {
        self.catalog.tab_closeable(tab)
    }

    fn clear_background(&self, tab: &TabId) -> bool {
        self.catalog.tab_clear_background(tab)
    }

    fn ui(&mut self, _scope: &mut UiScope<'_>, _tab: &TabId) {}
}

/// Active [`TabViewer`] adapter delegating metadata to [`PanelCatalog`] and content drawing to a [`PanelRegistryTarget`].
pub struct RegistryActiveTabViewer<'a, TabId, R: ?Sized, Context> {
    registry: &'a mut R,
    context: &'a mut Context,
    _tab: std::marker::PhantomData<fn(&TabId)>,
}

impl<'a, TabId, R: ?Sized, Context> RegistryActiveTabViewer<'a, TabId, R, Context> {
    /// Constructs an active tab viewer binding `registry` and per-frame `context`.
    #[inline]
    #[must_use]
    pub fn new(registry: &'a mut R, context: &'a mut Context) -> Self {
        Self {
            registry,
            context,
            _tab: std::marker::PhantomData,
        }
    }
}

impl<TabId, R, Context> TabViewer<TabId> for RegistryActiveTabViewer<'_, TabId, R, Context>
where
    TabId: Eq + Hash + Clone + Debug,
    R: PanelRegistryTarget<TabId, Context> + ?Sized,
{
    fn title(&self, tab: &TabId) -> String {
        self.registry.catalog().tab_title(tab)
    }

    fn raw_title(&self, tab: &TabId) -> String {
        self.registry.catalog().tab_raw_title(tab)
    }

    fn icon(&self, tab: &TabId) -> Option<irisui_core::UiIcon> {
        self.registry.catalog().tab_icon(tab)
    }

    fn closeable(&self, tab: &TabId) -> bool {
        self.registry.catalog().tab_closeable(tab)
    }

    fn clear_background(&self, tab: &TabId) -> bool {
        self.registry.catalog().tab_clear_background(tab)
    }

    fn ui(&mut self, scope: &mut UiScope<'_>, tab: &TabId) {
        let leaf_id = scope.parent();
        let _ = self.registry.draw(tab, scope, self.context);
        scope.tree_mut().set_boundary(leaf_id, true);
        scope
            .tree_mut()
            .mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
    }
}