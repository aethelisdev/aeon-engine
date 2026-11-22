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
use super::stateful_panel::StatefulPanel;
use crate::error::{WorkbenchError, WorkbenchResult};
use irisui_core::{UiTree, WidgetId};
use irisui_widgets::UiScope;
use std::any::Any;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

trait StoredScopedPanel<TabId, S: ScopedPanelContext>: ScopedWorkbenchPanel<TabId, S> {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<P, TabId, S> StoredScopedPanel<TabId, S> for P
where
    P: ScopedWorkbenchPanel<TabId, S> + 'static,
    S: ScopedPanelContext,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Thread-safe higher-ranked panel drawing closure parameterized by [`ScopedPanelContext`].
pub type ScopedPanelDrawFn<S> = Box<
    dyn for<'frame> FnMut(&mut UiScope<'_>, &mut <S as ScopedPanelContext>::Context<'frame>)
        + Send
        + Sync,
>;

/// Persistent registry of panels accepting a frame-borrowed context via [`ScopedPanelContext`].
pub struct ScopedPanelRegistry<TabId, S: ScopedPanelContext> {
    catalog: PanelCatalog<TabId>,
    panels: HashMap<TabId, Box<dyn StoredScopedPanel<TabId, S>>>,
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

    /// Registers an autonomous scoped panel via fluent builder chaining.
    #[must_use]
    pub fn with_panel<P>(mut self, panel: P) -> Self
    where
        P: ScopedWorkbenchPanel<TabId, S> + 'static,
    {
        self.register(panel);
        self
    }

    /// Registers a library-managed [`StatefulPanel`] initialized with `State::default()`.
    pub fn register_stateful_panel<State>(
        &mut self,
        metadata: PanelMetadata<TabId>,
        draw_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
    ) where
        TabId: Copy + Send + Sync + 'static,
        State: Default + Send + Sync + 'static,
    {
        self.register(StatefulPanel::new(metadata, draw_fn));
    }

    /// Registers a library-managed [`StatefulPanel`] with an attached `refresh_live` hook.
    pub fn register_live_stateful_panel<State>(
        &mut self,
        metadata: PanelMetadata<TabId>,
        draw_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
        refresh_live_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
    ) where
        TabId: Copy + Send + Sync + 'static,
        State: Default + Send + Sync + 'static,
    {
        self.register(StatefulPanel::new(metadata, draw_fn).with_live_refresh(refresh_live_fn));
    }

    /// Registers a library-managed [`StatefulPanel`] initialized with `State::default()`
    /// via fluent builder chaining.
    #[must_use]
    pub fn with_stateful_panel<State>(
        mut self,
        metadata: PanelMetadata<TabId>,
        draw_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
    ) -> Self
    where
        TabId: Copy + Send + Sync + 'static,
        State: Default + Send + Sync + 'static,
    {
        self.register_stateful_panel(metadata, draw_fn);
        self
    }

    /// Registers a library-managed [`StatefulPanel`] with a `refresh_live` hook
    /// via fluent builder chaining.
    #[must_use]
    pub fn with_live_stateful_panel<State>(
        mut self,
        metadata: PanelMetadata<TabId>,
        draw_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
        refresh_live_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
    ) -> Self
    where
        TabId: Copy + Send + Sync + 'static,
        State: Default + Send + Sync + 'static,
    {
        self.register_live_stateful_panel(metadata, draw_fn, refresh_live_fn);
        self
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

    /// Returns an immutable typed reference to the registered panel owning `tab`, if stored
    /// as a [`ScopedWorkbenchPanel`] object of concrete type `P`.
    #[must_use]
    pub fn panel<P>(&self, tab: &TabId) -> Option<&P>
    where
        P: ScopedWorkbenchPanel<TabId, S> + 'static,
    {
        self.panels.get(tab)?.as_any().downcast_ref::<P>()
    }

    /// Returns a mutable typed reference to the registered panel owning `tab`, enabling
    /// persistent panel state updates without external state bags.
    #[must_use]
    pub fn panel_mut<P>(&mut self, tab: &TabId) -> Option<&mut P>
    where
        P: ScopedWorkbenchPanel<TabId, S> + 'static,
    {
        self.panels.get_mut(tab)?.as_any_mut().downcast_mut::<P>()
    }

    /// Returns an immutable reference to the state owned by a [`StatefulPanel`] at `tab`.
    #[must_use]
    pub fn panel_state<State>(&self, tab: &TabId) -> Option<&State>
    where
        TabId: Copy + Send + Sync + 'static,
        State: Send + Sync + 'static,
    {
        self.panel::<StatefulPanel<TabId, State, S>>(tab)
            .map(StatefulPanel::state)
    }

    /// Returns a mutable reference to the state owned by a [`StatefulPanel`] at `tab`.
    #[must_use]
    pub fn panel_state_mut<State>(&mut self, tab: &TabId) -> Option<&mut State>
    where
        TabId: Copy + Send + Sync + 'static,
        State: Send + Sync + 'static,
    {
        self.panel_mut::<StatefulPanel<TabId, State, S>>(tab)
            .map(StatefulPanel::state_mut)
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
        if self.catalog.tab_skip_when_collapsed(tab) && scope.is_collapsed() {
            return Ok(());
        }
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

    #[inline]
    fn refresh_live(&mut self, tab: &TabId, scope: &mut UiScope<'_>, context: &mut S::Context<'_>) {
        if self.catalog.tab_skip_when_collapsed(tab) && scope.is_collapsed() {
            return;
        }
        if let Some(panel) = self.panels.get_mut(tab) {
            panel.refresh_live(scope, context);
        }
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

    struct HierarchyScopedPanel {
        draws: usize,
    }

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
            self.draws += 1;
            *state.draws += 1;
            scope.label(state.label);
        }
    }

    #[test]
    fn persistent_scoped_registry_renders_multiple_frame_lifetimes_without_rebuild() {
        let mut registry = ScopedPanelRegistry::<ScopedTab, TestScope>::new();
        registry.register(HierarchyScopedPanel { draws: 0 });
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

        let panel_ref = registry
            .panel::<HierarchyScopedPanel>(&ScopedTab::Hierarchy)
            .expect("typed panel ref");
        assert_eq!(panel_ref.draws, 1);

        let panel_mut = registry
            .panel_mut::<HierarchyScopedPanel>(&ScopedTab::Hierarchy)
            .expect("typed panel mut");
        panel_mut.draws = 42;
        assert_eq!(
            registry
                .panel::<HierarchyScopedPanel>(&ScopedTab::Hierarchy)
                .expect("typed panel ref after mut")
                .draws,
            42
        );
        assert!(
            registry
                .panel::<HierarchyScopedPanel>(&ScopedTab::Stats)
                .is_none()
        );
    }

    #[derive(Default)]
    struct CounterPanelState {
        renders: usize,
        refreshes: usize,
    }

    fn draw_counter_panel(
        state: &mut CounterPanelState,
        scope: &mut UiScope<'_>,
        ctx: &mut BorrowedView<'_>,
    ) {
        state.renders += 1;
        *ctx.draws += 1;
        scope.label(ctx.label);
    }

    fn refresh_counter_panel(
        state: &mut CounterPanelState,
        _scope: &mut UiScope<'_>,
        ctx: &mut BorrowedView<'_>,
    ) {
        state.refreshes += 1;
        *ctx.draws += 100;
    }

    #[test]
    fn stateful_panel_fluent_registration_and_state_access() {
        let mut registry = ScopedPanelRegistry::<ScopedTab, TestScope>::new()
            .with_live_stateful_panel(
                PanelMetadata::new(ScopedTab::Stats, "Stats"),
                draw_counter_panel,
                refresh_counter_panel,
            );

        let mut tree = UiTree::new();
        let leaf = tree.create_root().expect("root");
        let mut draws = 0usize;
        let label = String::from("Live");
        let mut ctx = BorrowedView {
            label: &label,
            draws: &mut draws,
        };

        registry
            .draw_leaf(&ScopedTab::Stats, &mut tree, leaf, &mut ctx)
            .expect("draw stateful panel");
        {
            let mut scope = UiScope::new(&mut tree, leaf);
            PanelRegistryTarget::refresh_live(
                &mut registry,
                &ScopedTab::Stats,
                &mut scope,
                &mut ctx,
            );
        }

        assert_eq!(draws, 101);
        let state = registry
            .panel_state::<CounterPanelState>(&ScopedTab::Stats)
            .expect("stateful panel lookup");
        assert_eq!(state.renders, 1);
        assert_eq!(state.refreshes, 1);

        registry
            .panel_state_mut::<CounterPanelState>(&ScopedTab::Stats)
            .expect("mutable state lookup")
            .renders = 9;
        assert_eq!(
            registry
                .panel_state::<CounterPanelState>(&ScopedTab::Stats)
                .expect("stateful panel lookup after mut")
                .renders,
            9
        );
    }

    #[test]
    fn skip_when_collapsed_guards_draw_and_refresh_live() {
        let mut registry = ScopedPanelRegistry::<ScopedTab, TestScope>::new()
            .with_live_stateful_panel(
                PanelMetadata::new(ScopedTab::Stats, "Stats").with_skip_collapsed(true),
                draw_counter_panel,
                refresh_counter_panel,
            );

        let mut tree = UiTree::new();
        let collapsed_leaf = tree.create_root().expect("root");
        let mut draws = 0usize;
        let label = String::from("Hidden");
        let mut ctx = BorrowedView {
            label: &label,
            draws: &mut draws,
        };

        registry
            .draw_leaf(&ScopedTab::Stats, &mut tree, collapsed_leaf, &mut ctx)
            .expect("draw skipped when collapsed");
        {
            let mut scope = UiScope::new(&mut tree, collapsed_leaf);
            PanelRegistryTarget::refresh_live(
                &mut registry,
                &ScopedTab::Stats,
                &mut scope,
                &mut ctx,
            );
        }

        assert_eq!(draws, 0);
        let state = registry
            .panel_state::<CounterPanelState>(&ScopedTab::Stats)
            .expect("state");
        assert_eq!(state.renders, 0);
        assert_eq!(state.refreshes, 0);
    }
}