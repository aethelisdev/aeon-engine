// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Registry & Tab Adapters
//!
//! Provides the generic [`WorkbenchPanelRegistry`] mapping abstract tab identifiers to
//! autonomous [`WorkbenchPanel`] implementations and declarative drawing closures,
//! decoupling workbench chrome from domain state.

use super::panel::{PanelMetadata, WorkbenchPanel};
use crate::error::{WorkbenchError, WorkbenchResult};
use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_dock::TabViewer;
use iris_widgets::UiScope;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// Type alias for a thread-safe panel drawing closure.
///
/// Accepts a mutable [`UiScope`] representing the panel's layout scope and a mutable
/// reference to the caller's custom `Context` containing domain/engine state.
pub type PanelDrawFn<Context> = Box<dyn FnMut(&mut UiScope<'_>, &mut Context) + Send + Sync>;

/// Central panel registry mapping panel identifiers to autonomous panels and declarative UI drawing functions.
///
/// Designed to decouple the workbench application shell from domain-specific editor state.
/// The workbench calls the registered panel or closure when constructing isolated dock leaf panels
/// or detached floating windows without knowing the internal ECS or engine architecture.
pub struct WorkbenchPanelRegistry<TabId, Context> {
    panels: HashMap<TabId, Box<dyn WorkbenchPanel<TabId, Context>>>,
    draw_callbacks: HashMap<TabId, PanelDrawFn<Context>>,
    metadata: HashMap<TabId, PanelMetadata<TabId>>,
    domain_panels: HashSet<TabId>,
}

/// Type alias for [`WorkbenchPanelRegistry`] providing standard workbench ergonomics.
pub type PanelRegistry<TabId, Context> = WorkbenchPanelRegistry<TabId, Context>;

impl<TabId, Context> Default for WorkbenchPanelRegistry<TabId, Context> {
    fn default() -> Self {
        Self {
            panels: HashMap::new(),
            draw_callbacks: HashMap::new(),
            metadata: HashMap::new(),
            domain_panels: HashSet::new(),
        }
    }
}

impl<TabId, Context> WorkbenchPanelRegistry<TabId, Context>
where
    TabId: Eq + Hash + Clone + Debug,
{
    /// Creates an empty panel registry.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an autonomous stateful panel implementing [`WorkbenchPanel`].
    ///
    /// Automatically records panel metadata (title, icon, closeable) and subscribes
    /// the panel to domain notifications if `panel.is_domain_bound()` returns `true`.
    ///
    /// # Arguments
    /// * `panel` - Any concrete struct implementing [`WorkbenchPanel`].
    pub fn register<P>(&mut self, panel: P)
    where
        P: WorkbenchPanel<TabId, Context> + 'static,
    {
        let id = panel.id();
        let meta = PanelMetadata::from_panel(&panel);
        if meta.is_domain_bound {
            self.domain_panels.insert(id.clone());
        }
        self.metadata.insert(id.clone(), meta);
        self.panels.insert(id, Box::new(panel));
    }

    /// Registers standalone panel metadata without requiring an immediate rendering closure.
    ///
    /// Useful for metadata registration in decoupled window/event coordinators.
    pub fn register_metadata(&mut self, metadata: PanelMetadata<TabId>) {
        if metadata.is_domain_bound {
            self.domain_panels.insert(metadata.id.clone());
        }
        self.metadata.insert(metadata.id.clone(), metadata);
    }

    /// Registers a boxed trait object implementing [`WorkbenchPanel`].
    pub fn register_boxed(&mut self, panel: Box<dyn WorkbenchPanel<TabId, Context>>) {
        let id = panel.id();
        let meta = PanelMetadata::from_panel(panel.as_ref());
        if meta.is_domain_bound {
            self.domain_panels.insert(id.clone());
        }
        self.metadata.insert(id.clone(), meta);
        self.panels.insert(id, panel);
    }

    /// Registers a declarative drawing closure for the given panel tab identifier.
    ///
    /// If a panel with the same identifier was previously registered, its callback is replaced.
    ///
    /// # Arguments
    /// * `tab` - The panel tab identifier (e.g. enum variant `PanelId::Hierarchy`).
    /// * `draw_fn` - Thread-safe closure executing declarative UI calls within [`UiScope`].
    pub fn register_panel(
        &mut self,
        tab: TabId,
        draw_fn: impl FnMut(&mut UiScope<'_>, &mut Context) + Send + Sync + 'static,
    ) {
        self.draw_callbacks.insert(tab, Box::new(draw_fn));
    }

    /// Registers a declarative drawing closure for a domain data-bound panel tab identifier.
    ///
    /// Domain-bound panels automatically subscribe to model transaction updates (e.g. Undo/Redo commands
    /// or entity property mutations), guaranteeing that retained inputs and property inspectors refresh
    /// on external state changes.
    ///
    /// # Arguments
    /// * `tab` - The panel tab identifier.
    /// * `draw_fn` - Thread-safe closure executing declarative UI calls within [`UiScope`].
    pub fn register_domain_panel(
        &mut self,
        tab: TabId,
        draw_fn: impl FnMut(&mut UiScope<'_>, &mut Context) + Send + Sync + 'static,
    ) {
        self.domain_panels.insert(tab.clone());
        self.register_panel(tab, draw_fn);
    }

    /// Marks an already-registered panel as a domain data-bound subscriber.
    pub fn mark_domain_panel(&mut self, tab: TabId) {
        self.domain_panels.insert(tab);
    }

    /// Unmarks a panel from domain data subscription without removing its drawing callback.
    ///
    /// Returns `true` if the panel was previously registered as a domain panel.
    pub fn unmark_domain_panel(&mut self, tab: &TabId) -> bool {
        self.domain_panels.remove(tab)
    }

    /// Returns `true` if the specified panel is registered as a domain data-bound subscriber.
    #[inline]
    #[must_use]
    pub fn is_domain_panel(&self, tab: &TabId) -> bool {
        self.domain_panels.contains(tab)
    }

    /// Returns an immutable reference to the set of registered domain data subscriber panels.
    #[inline]
    #[must_use]
    pub fn domain_panels(&self) -> &HashSet<TabId> {
        &self.domain_panels
    }

    /// Synchronizes all registered domain panels into an active [`iris_dock::UiNotifier`].
    pub fn sync_domain_panels_to_notifier(&self, notifier: &mut iris_dock::UiNotifier<TabId>)
    where
        TabId: Copy + Send + Sync + 'static,
    {
        for &tab in &self.domain_panels {
            notifier.register_domain_panel(tab);
        }
    }

    /// Unregisters the panel and drawing closure for the specified panel tab identifier.
    ///
    /// Also unregisters the panel from domain data subscription and metadata if present.
    /// Returns `true` if a panel or callback was removed.
    pub fn unregister_panel(&mut self, tab: &TabId) -> bool {
        self.domain_panels.remove(tab);
        self.metadata.remove(tab);
        let removed_panel = self.panels.remove(tab).is_some();
        let removed_callback = self.draw_callbacks.remove(tab).is_some();
        removed_panel || removed_callback
    }

    /// Returns `true` if a panel or drawing closure is registered for the given panel tab.
    #[inline]
    #[must_use]
    pub fn is_registered(&self, tab: &TabId) -> bool {
        self.panels.contains_key(tab) || self.draw_callbacks.contains_key(tab)
    }

    /// Returns the total number of registered panels and callbacks.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.panels.len() + self.draw_callbacks.len()
    }

    /// Returns `true` if no panels or callbacks have been registered.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty() && self.draw_callbacks.is_empty()
    }

    /// Returns an immutable reference to an autonomous stateful panel if registered.
    #[must_use]
    pub fn get_panel(&self, tab: &TabId) -> Option<&dyn WorkbenchPanel<TabId, Context>> {
        self.panels.get(tab).map(|b| b.as_ref())
    }

    /// Returns a mutable reference to an autonomous stateful panel if registered.
    pub fn get_panel_mut(
        &mut self,
        tab: &TabId,
    ) -> Option<&mut (dyn WorkbenchPanel<TabId, Context> + 'static)> {
        self.panels.get_mut(tab).map(|b| b.as_mut())
    }

    /// Returns metadata describing the specified panel if registered.
    #[must_use]
    pub fn panel_metadata(&self, tab: &TabId) -> Option<&PanelMetadata<TabId>> {
        self.metadata.get(tab)
    }

    /// Invokes the registered panel or drawing closure for `tab` within the provided [`UiScope`].
    ///
    /// # Errors
    /// Returns [`WorkbenchError::PanelNotFound`] if neither a panel nor a drawing callback
    /// is registered for `tab`.
    pub fn draw(
        &mut self,
        tab: &TabId,
        scope: &mut UiScope<'_>,
        context: &mut Context,
    ) -> WorkbenchResult<()> {
        if let Some(panel) = self.panels.get_mut(tab) {
            panel.render(scope, context);
            Ok(())
        } else if let Some(callback) = self.draw_callbacks.get_mut(tab) {
            (callback)(scope, context);
            Ok(())
        } else {
            Err(WorkbenchError::PanelNotFound(format!("{tab:?}")))
        }
    }

    /// Draws the specified panel directly into a dock leaf container node.
    ///
    /// Constructs an isolated [`UiScope`] rooted at `leaf_id`, executes the registered
    /// panel or drawing closure, seals `leaf_id` as an Invalidation Boundary
    /// ([`UiTree::set_boundary`]), and schedules repainting ([`DirtyFlags::PAINT`]).
    ///
    /// This guarantees that dock leaf containers are treated as autonomous invalidation
    /// roots for $O(M)$ Taffy partial subtree rebuilding and GPU batch caching.
    ///
    /// # Arguments
    /// * `tab` - The panel tab identifier.
    /// * `tree` - Generational UI tree hosting the leaf node.
    /// * `leaf_id` - The container widget ID allocated for this panel leaf.
    /// * `context` - Domain/application context.
    ///
    /// # Errors
    /// Returns [`WorkbenchError::PanelNotFound`] if no panel or closure is registered for `tab`.
    pub fn draw_leaf(
        &mut self,
        tab: &TabId,
        tree: &mut UiTree,
        leaf_id: WidgetId,
        context: &mut Context,
    ) -> WorkbenchResult<()> {
        let mut scope = UiScope::new(tree, leaf_id);
        self.draw(tab, &mut scope, context)?;
        tree.set_boundary(leaf_id, true);
        tree.mark_dirty(leaf_id, DirtyFlags::PAINT | DirtyFlags::TEXT);
        Ok(())
    }

    /// Invokes the registered panel renderers for all tabs present in `dirty_tabs`.
    ///
    /// Resolves each dirty tab's container [`WidgetId`] via `leaf_resolver` and delegates
    /// directly to [`Self::draw_leaf`], guaranteeing that every rendered leaf node is sealed
    /// as an Invalidation Boundary ([`UiTree::set_boundary`]) and marked with [`DirtyFlags::PAINT`].
    /// Clean tabs are skipped entirely, guaranteeing $O(M)$ partial leaf invalidation.
    ///
    /// Returns the number of panels actually rendered.
    pub fn render_dirty_tabs<I, R>(
        &mut self,
        dirty_tabs: I,
        tree: &mut UiTree,
        leaf_resolver: R,
        context: &mut Context,
    ) -> usize
    where
        I: IntoIterator,
        I::Item: Borrow<TabId>,
        R: Fn(&TabId) -> Option<WidgetId>,
    {
        let mut rendered = 0;
        for tab in dirty_tabs {
            let tab = tab.borrow();
            if let Some(leaf_id) = leaf_resolver(tab)
                && self.draw_leaf(tab, tree, leaf_id, context).is_ok()
            {
                rendered += 1;
            }
        }
        rendered
    }

    /// Constructs a [`RegistryTabViewer`] adapter implementing [`TabViewer`] for dock area integration.
    #[must_use]
    pub fn tab_viewer(&self) -> RegistryTabViewer<'_, TabId, Context> {
        RegistryTabViewer::new(self)
    }

    /// Constructs an active [`RegistryActiveTabViewer`] adapter that delegates `ui` rendering
    /// directly to the registered panel closures using `context`.
    pub fn active_tab_viewer<'a>(
        &'a mut self,
        context: &'a mut Context,
    ) -> RegistryActiveTabViewer<'a, TabId, Context> {
        RegistryActiveTabViewer::new(self, context)
    }
}

/// Read-only TabViewer implementation delegating tab title and closeable metadata to [`WorkbenchPanelRegistry`].
pub struct RegistryTabViewer<'a, TabId, Context> {
    registry: &'a WorkbenchPanelRegistry<TabId, Context>,
}

impl<'a, TabId, Context> RegistryTabViewer<'a, TabId, Context> {
    /// Constructs a tab viewer adapter referencing the given panel registry.
    #[inline]
    #[must_use]
    pub fn new(registry: &'a WorkbenchPanelRegistry<TabId, Context>) -> Self {
        Self { registry }
    }
}

impl<'a, TabId, Context> TabViewer<TabId> for RegistryTabViewer<'a, TabId, Context>
where
    TabId: Eq + Hash + Clone + Debug,
{
    fn title(&self, tab: &TabId) -> String {
        if let Some(meta) = self.registry.panel_metadata(tab) {
            if meta.atlas_icon.is_some() {
                meta.title.clone()
            } else if let Some(icon) = &meta.icon {
                format!("{icon} {}", meta.title)
            } else {
                meta.title.clone()
            }
        } else {
            format!("{tab:?}")
        }
    }

    fn raw_title(&self, tab: &TabId) -> String {
        if let Some(meta) = self.registry.panel_metadata(tab) {
            meta.title.clone()
        } else {
            format!("{tab:?}")
        }
    }

    fn atlas_icon(&self, tab: &TabId) -> Option<[f32; 4]> {
        self.registry.panel_metadata(tab).and_then(|m| m.atlas_icon)
    }

    fn closeable(&self, tab: &TabId) -> bool {
        self.registry
            .panel_metadata(tab)
            .is_none_or(|meta| meta.closeable)
    }

    fn clear_background(&self, tab: &TabId) -> bool {
        self.registry
            .panel_metadata(tab)
            .is_some_and(|meta| meta.clear_background)
    }

    fn ui(&mut self, _scope: &mut UiScope<'_>, _tab: &TabId) {}
}

/// Active TabViewer implementation delegating tab presentation and content rendering to [`WorkbenchPanelRegistry`].
pub struct RegistryActiveTabViewer<'a, TabId, Context> {
    registry: &'a mut WorkbenchPanelRegistry<TabId, Context>,
    context: &'a mut Context,
}

impl<'a, TabId, Context> RegistryActiveTabViewer<'a, TabId, Context> {
    /// Constructs an active tab viewer adapter binding the registry and its active context.
    #[inline]
    #[must_use]
    pub fn new(
        registry: &'a mut WorkbenchPanelRegistry<TabId, Context>,
        context: &'a mut Context,
    ) -> Self {
        Self { registry, context }
    }
}

impl<'a, TabId, Context> TabViewer<TabId> for RegistryActiveTabViewer<'a, TabId, Context>
where
    TabId: Eq + Hash + Clone + Debug,
{
    fn title(&self, tab: &TabId) -> String {
        if let Some(meta) = self.registry.panel_metadata(tab) {
            if meta.atlas_icon.is_some() {
                meta.title.clone()
            } else if let Some(icon) = &meta.icon {
                format!("{icon} {}", meta.title)
            } else {
                meta.title.clone()
            }
        } else {
            format!("{tab:?}")
        }
    }

    fn raw_title(&self, tab: &TabId) -> String {
        if let Some(meta) = self.registry.panel_metadata(tab) {
            meta.title.clone()
        } else {
            format!("{tab:?}")
        }
    }

    fn atlas_icon(&self, tab: &TabId) -> Option<[f32; 4]> {
        self.registry.panel_metadata(tab).and_then(|m| m.atlas_icon)
    }

    fn closeable(&self, tab: &TabId) -> bool {
        self.registry
            .panel_metadata(tab)
            .is_none_or(|meta| meta.closeable)
    }

    fn clear_background(&self, tab: &TabId) -> bool {
        self.registry
            .panel_metadata(tab)
            .is_some_and(|meta| meta.clear_background)
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