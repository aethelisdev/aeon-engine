// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Isolated Workbench Panel Interface & Metadata
//!
//! Provides the [`WorkbenchPanel`] trait for autonomous, decoupled desktop panels
//! capable of declaring their metadata, domain binding, close policies, and
//! self-contained declarative UI rendering.

use irisui_dock::TabPlacement;
use irisui_widgets::UiScope;

/// Scheduling policy governing when a panel's [`ScopedWorkbenchPanel::refresh_live`] hook runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LiveRefreshInterval {
    /// Refreshes only when invalidated or rebuilt (no periodic background tick).
    #[default]
    OnChange,
    /// Invokes `refresh_live` on every `InPlaceRefresh` frame while visible.
    EveryFrame,
    /// Invokes `refresh_live` at most once per specified millisecond period while visible.
    ThrottledMs(u32),
}

/// Common interface implemented by autonomous workbench tool panels.
///
/// Decouples individual tool views (Hierarchy, Inspector, Console, Assets, Viewport)
/// from monolithic parameter bags, providing single-responsibility encapsulation.
/// Defines a `'static` context family mapping to a per-frame borrowed panel context.
///
/// Allows long-lived hosts (`Workbench`, `EngineUi`) to own persistent panel registries
/// once at initialization while supplying transient borrowed world, renderer, or input
/// references (`Context<'frame>`) on every frame without heap allocations.
pub trait ScopedPanelContext: 'static {
    /// Concrete borrowed context type supplied to [`ScopedWorkbenchPanel::render`] each frame.
    type Context<'frame>;
}

/// Autonomous workbench panel trait accepting a per-frame borrowed [`ScopedPanelContext`].
///
/// Mirrors [`WorkbenchPanel`] metadata contracts while permitting lifetime-parameterized
/// rendering contexts inside persistent registries without per-frame boxing or clones.
pub trait ScopedWorkbenchPanel<TabId, S: ScopedPanelContext>: Send + Sync {
    /// Returns the unique tab identifier of this panel.
    fn id(&self) -> TabId;

    /// Returns the user-facing title of this panel displayed in tab headers.
    fn title(&self) -> &str;

    /// Returns an optional unified icon ([`irisui_core::UiIcon::Glyph`] or [`irisui_core::UiIcon::Atlas`]).
    fn icon(&self) -> Option<irisui_core::UiIcon> {
        None
    }

    /// Whether this panel is bound to domain data mutations (e.g. Undo/Redo, model changes).
    fn is_domain_bound(&self) -> bool {
        false
    }

    /// Whether this panel can be closed by the user in the dock tab bar.
    fn closeable(&self) -> bool {
        true
    }

    /// Determines whether the background behind this panel's content should remain transparent.
    fn clear_background(&self) -> bool {
        false
    }

    /// Returns an optional declarative docking/floating placement rule used when toggling this tab.
    fn placement(&self) -> Option<TabPlacement<TabId>> {
        None
    }

    /// Declares the periodic [`LiveRefreshInterval`] governing [`Self::refresh_live`] execution.
    fn live_refresh_interval(&self) -> LiveRefreshInterval {
        LiveRefreshInterval::OnChange
    }

    /// Performs lightweight in-place live updates (canvas primitives, telemetry text) during `InPlaceRefresh`.
    fn refresh_live<'frame>(&mut self, _scope: &mut UiScope<'_>, _state: &mut S::Context<'frame>) {}

    /// Declaratively renders the panel content into `scope` using the frame-borrowed `state`.
    fn render<'frame>(&mut self, scope: &mut UiScope<'_>, state: &mut S::Context<'frame>);
}

/// Common interface implemented by autonomous workbench tool panels.
///
/// Decouples individual tool views from monolithic parameter bags, providing
/// single-responsibility encapsulation.
pub trait WorkbenchPanel<TabId, AppState>: Send + Sync {
    /// Returns the unique tab identifier of this panel.
    fn id(&self) -> TabId;

    /// Returns the user-facing title of this panel displayed in tab headers.
    fn title(&self) -> &str;

    /// Returns an optional unified icon ([`irisui_core::UiIcon::Glyph`] or [`irisui_core::UiIcon::Atlas`]).
    fn icon(&self) -> Option<irisui_core::UiIcon> {
        None
    }

    /// Whether this panel is bound to domain data mutations (e.g. Undo/Redo, ECS changes).
    ///
    /// If `true`, the panel is automatically invalidated when domain revision increments.
    fn is_domain_bound(&self) -> bool {
        false
    }

    /// Whether this panel can be closed by the user in the dock tab bar.
    ///
    /// Returning `false` hides the close button and prevents the panel from being dismissed.
    fn closeable(&self) -> bool {
        true
    }

    /// Determines whether the background behind this panel's content should remain transparent.
    ///
    /// Used for embedded 3D viewports where the underlying render pass should not be obscured.
    fn clear_background(&self) -> bool {
        false
    }

    /// Declares the periodic [`LiveRefreshInterval`] governing [`Self::refresh_live`] execution.
    fn live_refresh_interval(&self) -> LiveRefreshInterval {
        LiveRefreshInterval::OnChange
    }

    /// Performs lightweight in-place live updates (canvas primitives, telemetry text) during `InPlaceRefresh`.
    fn refresh_live(&mut self, _scope: &mut UiScope<'_>, _state: &mut AppState) {}

    /// Declaratively renders the panel content into the given [`UiScope`].
    ///
    /// # Arguments
    /// * `scope` - Declarative UI scope scoped to the panel's container node.
    /// * `state` - Mutable reference to the application domain state.
    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut AppState);
}

/// Immutable metadata snapshot describing a registered workbench panel.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelMetadata<TabId> {
    /// Unique panel tab identifier.
    pub id: TabId,
    /// User-facing display title.
    pub title: String,
    /// Optional unified icon ([`irisui_core::UiIcon::Glyph`] or [`irisui_core::UiIcon::Atlas`]).
    pub icon: Option<irisui_core::UiIcon>,
    /// Whether the panel can be closed by the user.
    pub closeable: bool,
    /// Whether the background behind this tab's content should remain transparent.
    pub clear_background: bool,
    /// Whether the panel is subscribed to domain model transactions.
    pub is_domain_bound: bool,
    /// Optional declarative docking/floating placement rule used when opening or toggling this tab.
    pub placement: Option<TabPlacement<TabId>>,
    /// Scheduling policy for in-place [`refresh_live`](ScopedWorkbenchPanel::refresh_live) ticks.
    pub live_refresh: LiveRefreshInterval,
}

impl<TabId: Clone> PanelMetadata<TabId> {
    /// Creates a new panel metadata snapshot with the specified tab identifier and display title.
    #[inline]
    #[must_use]
    pub fn new(id: TabId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            icon: None,
            closeable: true,
            clear_background: false,
            is_domain_bound: false,
            placement: None,
            live_refresh: LiveRefreshInterval::OnChange,
        }
    }

    /// Sets the optional unified icon via fluent builder.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<irisui_core::UiIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets whether the panel can be closed by the user via fluent builder.
    #[inline]
    #[must_use]
    pub fn with_closeable(mut self, closeable: bool) -> Self {
        self.closeable = closeable;
        self
    }

    /// Configures the declarative placement rule used when opening or toggling this panel.
    #[inline]
    #[must_use]
    pub fn with_placement(mut self, placement: TabPlacement<TabId>) -> Self {
        self.placement = Some(placement);
        self
    }

    /// Constructs panel metadata from an existing [`WorkbenchPanel`].
    #[must_use]
    pub fn from_panel<AppState>(panel: &dyn WorkbenchPanel<TabId, AppState>) -> Self {
        Self::new(panel.id(), panel.title())
            .with_optional_icon(panel.icon())
            .with_closeable(panel.closeable())
            .with_clear_background(panel.clear_background())
            .with_domain_bound(panel.is_domain_bound())
            .with_live_refresh(panel.live_refresh_interval())
    }

    /// Constructs panel metadata from an existing [`ScopedWorkbenchPanel`].
    #[must_use]
    pub fn from_scoped_panel<S: ScopedPanelContext>(
        panel: &dyn ScopedWorkbenchPanel<TabId, S>,
    ) -> Self {
        Self::new(panel.id(), panel.title())
            .with_optional_icon(panel.icon())
            .with_closeable(panel.closeable())
            .with_clear_background(panel.clear_background())
            .with_domain_bound(panel.is_domain_bound())
            .with_optional_placement(panel.placement())
            .with_live_refresh(panel.live_refresh_interval())
    }

    /// Sets the periodic live-refresh scheduling policy via fluent builder.
    #[inline]
    #[must_use]
    pub const fn with_live_refresh(mut self, interval: LiveRefreshInterval) -> Self {
        self.live_refresh = interval;
        self
    }

    #[inline]
    fn with_optional_icon(mut self, icon: Option<irisui_core::UiIcon>) -> Self {
        self.icon = icon;
        self
    }

    #[inline]
    fn with_clear_background(mut self, clear_background: bool) -> Self {
        self.clear_background = clear_background;
        self
    }

    #[inline]
    fn with_domain_bound(mut self, is_domain_bound: bool) -> Self {
        self.is_domain_bound = is_domain_bound;
        self
    }

    #[inline]
    fn with_optional_placement(mut self, placement: Option<TabPlacement<TabId>>) -> Self {
        self.placement = placement;
        self
    }
}