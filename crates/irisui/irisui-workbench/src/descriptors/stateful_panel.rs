// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Generic Stateful Scoped Panel Container
//!
//! Eliminates per-panel boilerplate trait implementations in host applications by
//! pairing declarative [`PanelMetadata`], owned `State`, and higher-ranked closures
//! for [`ScopedWorkbenchPanel::render`] and optional [`ScopedWorkbenchPanel::refresh_live`].

use std::fmt::Debug;
use std::hash::Hash;

use irisui_dock::TabPlacement;
use irisui_widgets::UiScope;

use super::panel::{LiveRefreshInterval, PanelMetadata, ScopedPanelContext, ScopedWorkbenchPanel};

/// Boxed higher-ranked callback invoked once per rebuild to render a [`StatefulPanel`].
///
/// Allocated strictly once at panel registration; zero heap allocations occur per frame.
pub type StatefulPanelDrawFn<State, S> = Box<
    dyn for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut <S as ScopedPanelContext>::Context<'frame>,
        ) + Send
        + Sync,
>;

/// Boxed higher-ranked callback invoked during `InPlaceRefresh` to update a [`StatefulPanel`].
///
/// Allocated strictly once at panel registration; zero heap allocations occur per frame.
pub type StatefulPanelLiveFn<State, S> = Box<
    dyn for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut <S as ScopedPanelContext>::Context<'frame>,
        ) + Send
        + Sync,
>;

/// Generic library-owned [`ScopedWorkbenchPanel`] hosting arbitrary persistent `State`
/// alongside declarative [`PanelMetadata`] and stateless draw/live-refresh callbacks.
///
/// Replaces handwritten single-panel wrapper structs in host applications: panels keep
/// their persistent UI state strictly inside [`crate::descriptors::ScopedPanelRegistry`]
/// and retrieve it safely via [`crate::descriptors::ScopedPanelRegistry::panel_state`].
pub struct StatefulPanel<TabId, State, S: ScopedPanelContext> {
    metadata: PanelMetadata<TabId>,
    state: State,
    draw_fn: StatefulPanelDrawFn<State, S>,
    refresh_live_fn: Option<StatefulPanelLiveFn<State, S>>,
}

impl<TabId, State, S> StatefulPanel<TabId, State, S>
where
    TabId: Copy + Eq + Hash + Debug + Send + Sync + 'static,
    State: Send + Sync + 'static,
    S: ScopedPanelContext,
{
    /// Constructs a [`StatefulPanel`] initialized with `State::default()`.
    #[inline]
    #[must_use]
    pub fn new(
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
        State: Default,
    {
        Self {
            metadata,
            state: State::default(),
            draw_fn: Box::new(draw_fn),
            refresh_live_fn: None,
        }
    }

    /// Constructs a [`StatefulPanel`] initialized with an explicit starting `state`.
    #[inline]
    #[must_use]
    pub fn with_initial_state(
        metadata: PanelMetadata<TabId>,
        state: State,
        draw_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            metadata,
            state,
            draw_fn: Box::new(draw_fn),
            refresh_live_fn: None,
        }
    }

    /// Attaches an in-place live-refresh callback executed on [`ScopedWorkbenchPanel::refresh_live`].
    #[inline]
    #[must_use]
    pub fn with_live_refresh(
        mut self,
        refresh_live_fn: impl for<'scope, 'frame> FnMut(
            &mut State,
            &mut UiScope<'scope>,
            &mut S::Context<'frame>,
        ) + Send
        + Sync
        + 'static,
    ) -> Self {
        self.refresh_live_fn = Some(Box::new(refresh_live_fn));
        self
    }

    /// Returns an immutable reference to the panel's owned persistent state.
    #[inline]
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// Returns a mutable reference to the panel's owned persistent state.
    #[inline]
    #[must_use]
    pub const fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl<TabId, State, S> ScopedWorkbenchPanel<TabId, S> for StatefulPanel<TabId, State, S>
where
    TabId: Copy + Eq + Hash + Debug + Send + Sync + 'static,
    State: Send + Sync + 'static,
    S: ScopedPanelContext,
{
    fn id(&self) -> TabId {
        self.metadata.id
    }

    fn title(&self) -> &str {
        &self.metadata.title
    }

    fn icon(&self) -> Option<irisui_core::UiIcon> {
        self.metadata.icon
    }

    fn is_domain_bound(&self) -> bool {
        self.metadata.is_domain_bound
    }

    fn closeable(&self) -> bool {
        self.metadata.closeable
    }

    fn clear_background(&self) -> bool {
        self.metadata.clear_background
    }

    fn placement(&self) -> Option<TabPlacement<TabId>> {
        self.metadata.placement
    }

    fn live_refresh_interval(&self) -> LiveRefreshInterval {
        self.metadata.live_refresh
    }

    fn skip_when_collapsed(&self) -> bool {
        self.metadata.skip_when_collapsed
    }

    fn surface_style(&self) -> Option<super::panel_surface::PanelSurfaceStyle> {
        self.metadata.surface
    }

    fn refresh_live<'frame>(&mut self, scope: &mut UiScope<'_>, state: &mut S::Context<'frame>) {
        if let Some(refresh_fn) = self.refresh_live_fn.as_mut() {
            refresh_fn(&mut self.state, scope, state);
        }
    }

    fn render<'frame>(&mut self, scope: &mut UiScope<'_>, state: &mut S::Context<'frame>) {
        (self.draw_fn)(&mut self.state, scope, state);
    }
}