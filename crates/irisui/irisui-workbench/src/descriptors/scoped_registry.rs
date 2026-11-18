// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Persistent Lifetime-Scoped Panel Registry
//!
//! Stores autonomous [`ScopedWorkbenchPanel`] instances and higher-ranked closures
//! across frames inside long-lived host structs while accepting transient borrowed
//! contexts (`S::Context<'frame>`) on every draw pass with zero per-frame allocations.

use super::panel::{PanelMetadata, ScopedPanelContext, ScopedWorkbenchPanel};
use super::panel_catalog::{
    PanelCatalog, PanelRegistryTarget, RegistryActiveTabViewer, RegistryTabViewer,
    draw_leaf_on_target, render_dirty_tabs_on_target,
};
use crate::error::{WorkbenchError, WorkbenchResult};
use irisui_core::{UiTree, WidgetId};
use irisui_widgets::UiScope;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// Thread-safe higher-ranked panel drawing closure parameterized by [`ScopedPanelContext`].
pub type ScopedPanelDrawFn<S> = Box<
    dyn for<'frame> FnMut(&mut UiScope<'_>, &mut <S as ScopedPanelContext>::Context<'frame>)
        + Send
        + Sync,
>;

/// Persistent registry of panels accepting a frame-borrowed context via [`ScopedPanelContext`].
pub struct ScopedPanelRegistry<TabId, S: ScopedPanelContext> {
    catalog: PanelCatalog<TabId>,
    panels: HashMap<TabId, Box<dyn ScopedWorkbenchPanel<TabId, S>>>,
    draw_callbacks: HashMap<TabId, ScopedPanelDrawFn<S>>,
}

impl<TabId, S: ScopedPanelContext> Default for ScopedPanelRegistry<TabId, S> {
    fn default() -> Self {
        Self {
            catalog: PanelCatalog::default(),
            panels: HashMap::new(),
            draw_callbacks: HashMap::new(),
        }
    }
}

impl<TabId, S> ScopedPanelRegistry<TabId, S>
where
    TabId: Eq + Hash + Clone + Debug,
    S: ScopedPanelContext,
{
    /// Creates an empty persistent scoped panel registry.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an autonomous scoped panel and indexes its metadata and domain subscription.
    pub fn register<P>(&mut self, panel: P)
    where
        P: ScopedWorkbenchPanel<TabId, S> + 'static,
    {
        let id = panel.id();
        let meta = PanelMetadata::from_scoped_panel(&panel);
        self.catalog.record_metadata(meta);
        self.panels.insert(id, Box::new(panel));
    }

    /// Registers standalone panel metadata without requiring an immediate panel drawer.
    pub fn register_metadata(&mut self, metadata: PanelMetadata<TabId>) {
        self.catalog.record_metadata(metadata);
    }

    /// Registers a higher-ranked drawing closure invoked with per-frame borrowed contexts.
    pub fn register_panel(
        &mut self,
        tab: TabId,
        draw_fn: impl for<'frame> FnMut(&mut UiScope<'_>, &mut S::Context<'frame>)
        + Send
        + Sync
        + 'static,
    ) {
        self.draw_callbacks.insert(tab, Box::new(draw_fn));
    }

    /// Registers a higher-ranked drawing closure marked as a domain data subscriber.
    pub fn register_domain_panel(
        &mut self,
        tab: TabId,
        draw_fn: impl for<'frame> FnMut(&mut UiScope<'_>, &mut S::Context<'frame>)
        + Send
        + Sync
        + 'static,
    ) {
        self.catalog.mark_domain_panel(tab.clone());
        self.register_panel(tab, draw_fn);
    }

    /// Returns `true` if `tab` is subscribed to domain invalidation notifications.
    #[inline]
    #[must_use]
    pub fn is_domain_panel(&self, tab: &TabId) -> bool {
        self.catalog.is_domain_panel(tab)
    }

    /// Returns the set of registered domain data-bound panel identifiers.
    #[inline]
    #[must_use]
    pub fn domain_panels(&self) -> &HashSet<TabId> {
        self.catalog.domain_panels()
    }

    /// Synchronizes all registered domain panels into an active [`irisui_dock::UiNotifier`].
    pub fn sync_domain_panels_to_notifier(&self, notifier: &mut irisui_dock::UiNotifier<TabId>)
    where
        TabId: Copy + Send + Sync + 'static,
    {
        self.catalog.sync_domain_panels_to_notifier(notifier);
    }

    /// Unregisters the panel, callback, metadata, and domain subscription for `tab`.
    pub fn unregister_panel(&mut self, tab: &TabId) -> bool {
        self.catalog.remove(tab);
        let removed_panel = self.panels.remove(tab).is_some();
        let removed_callback = self.draw_callbacks.remove(tab).is_some();
        removed_panel || removed_callback
    }

    /// Returns `true` if a panel or drawing callback is registered for `tab`.
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

    /// Returns `true` when no panels or callbacks are registered.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty() && self.draw_callbacks.is_empty()
    }

    /// Returns metadata describing `tab` if registered.
    #[inline]
    #[must_use]
    pub fn panel_metadata(&self, tab: &TabId) -> Option<&PanelMetadata<TabId>> {
        self.catalog.panel_metadata(tab)
    }

    /// Returns an immutable reference to the underlying [`PanelCatalog`].
    #[inline]
    #[must_use]
    pub fn catalog(&self) -> &PanelCatalog<TabId> {
        &self.catalog
    }

    /// Renders `tab` into `scope` using a frame-borrowed context.
    ///
    /// # Errors
    /// Returns [`WorkbenchError::PanelNotFound`] if `tab` is not registered.
    pub fn draw<'frame>(
        &mut self,
        tab: &TabId,
        scope: &mut UiScope<'_>,
        context: &mut S::Context<'frame>,
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

    /// Draws `tab` into a dock leaf container and seals it as an Invalidation Boundary.
    ///
    /// # Errors
    /// Returns [`WorkbenchError::PanelNotFound`] if `tab` is unregistered.
    pub fn draw_leaf<'frame>(
        &mut self,
        tab: &TabId,
        tree: &mut UiTree,
        leaf_id: WidgetId,
        context: &mut S::Context<'frame>,
    ) -> WorkbenchResult<()> {
        draw_leaf_on_target(self, tab, tree, leaf_id, context)
    }

    /// Renders all dirty tabs resolved via `leaf_resolver` and returns the count rendered.
    pub fn render_dirty_tabs<'frame, I, R>(
        &mut self,
        dirty_tabs: I,
        tree: &mut UiTree,
        leaf_resolver: R,
        context: &mut S::Context<'frame>,
    ) -> usize
    where
        I: IntoIterator,
        I::Item: Borrow<TabId>,
        R: Fn(&TabId) -> Option<WidgetId>,
    {
        render_dirty_tabs_on_target(self, dirty_tabs, tree, leaf_resolver, context)
    }

    /// Constructs a read-only [`RegistryTabViewer`] adapter backed by this registry's catalog.
    #[inline]
    #[must_use]
    pub fn tab_viewer(&self) -> RegistryTabViewer<'_, TabId> {
        self.catalog.tab_viewer()
    }

    /// Constructs an active [`RegistryActiveTabViewer`] adapter binding this registry and `context`.
    pub fn active_tab_viewer<'a, 'frame>(
        &'a mut self,
        context: &'a mut S::Context<'frame>,
    ) -> RegistryActiveTabViewer<'a, TabId, Self, S::Context<'frame>> {
        RegistryActiveTabViewer::new(self, context)
    }
}

impl<TabId, S> PanelRegistryTarget<TabId, S::Context<'_>> for ScopedPanelRegistry<TabId, S>
where
    TabId: Eq + Hash + Clone + Debug,
    S: ScopedPanelContext,
{
    #[inline]
    fn catalog(&self) -> &PanelCatalog<TabId> {
        &self.catalog
    }

    #[inline]
    fn draw(
        &mut self,
        tab: &TabId,
        scope: &mut UiScope<'_>,
        context: &mut S::Context<'_>,
    ) -> WorkbenchResult<()> {
        Self::draw(self, tab, scope, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::Style;
    use irisui_dock::{TabPlacement, TabViewer};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum ScopedTab {
        Hierarchy,
        Stats,
    }

    struct BorrowedView<'a> {
        label: &'a str,
        draws: &'a mut usize,
    }

    struct TestScope;

    impl ScopedPanelContext for TestScope {
        type Context<'frame> = BorrowedView<'frame>;
    }

    struct HierarchyScopedPanel;

    impl ScopedWorkbenchPanel<ScopedTab, TestScope> for HierarchyScopedPanel {
        fn id(&self) -> ScopedTab {
            ScopedTab::Hierarchy
        }

        fn title(&self) -> &str {
            "Hierarchy"
        }

        fn is_domain_bound(&self) -> bool {
            true
        }

        fn placement(&self) -> Option<TabPlacement<ScopedTab>> {
            Some(TabPlacement::FloatingCentered {
                size: irisui_core::Size::new(300.0, 200.0),
            })
        }

        fn render<'frame>(&mut self, scope: &mut UiScope<'_>, state: &mut BorrowedView<'frame>) {
            *state.draws += 1;
            scope.label(state.label);
        }
    }

    #[test]
    fn persistent_scoped_registry_renders_multiple_frame_lifetimes_without_rebuild() {
        let mut registry = ScopedPanelRegistry::<ScopedTab, TestScope>::new();
        registry.register(HierarchyScopedPanel);
        registry.register_panel(ScopedTab::Stats, |scope, state| {
            *state.draws += 10;
            scope.container(Style::new().height(20.0), |c| {
                c.label(state.label);
            });
        });

        assert!(registry.is_registered(&ScopedTab::Hierarchy));
        assert!(registry.is_registered(&ScopedTab::Stats));
        assert!(registry.is_domain_panel(&ScopedTab::Hierarchy));
        assert!(!registry.is_domain_panel(&ScopedTab::Stats));
        let meta = registry
            .panel_metadata(&ScopedTab::Hierarchy)
            .expect("metadata");
        assert!(meta.placement.is_some());
        assert_eq!(
            registry.tab_viewer().title(&ScopedTab::Hierarchy),
            "Hierarchy"
        );

        let mut tree = UiTree::new();
        let leaf = tree.create_root().expect("root");
        let mut draws = 0usize;

        {
            let frame1_text = String::from("Frame1");
            let mut ctx1 = BorrowedView {
                label: &frame1_text,
                draws: &mut draws,
            };
            registry
                .draw_leaf(&ScopedTab::Hierarchy, &mut tree, leaf, &mut ctx1)
                .expect("draw frame1");
        }
        assert_eq!(draws, 1);

        {
            let frame2_text = String::from("Frame2");
            let mut ctx2 = BorrowedView {
                label: &frame2_text,
                draws: &mut draws,
            };
            let rebuilt = registry.render_dirty_tabs(
                [ScopedTab::Stats],
                &mut tree,
                |_| Some(leaf),
                &mut ctx2,
            );
            assert_eq!(rebuilt, 1);
        }
        assert_eq!(draws, 11);
    }
}