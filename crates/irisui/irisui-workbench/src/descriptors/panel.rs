// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Isolated Workbench Panel Interface & Metadata
//!
//! Provides the [`WorkbenchPanel`] trait for autonomous, decoupled desktop panels
//! capable of declaring their metadata, domain binding, close policies, and
//! self-contained declarative UI rendering.

use irisui_dock::TabPlacement;
use irisui_widgets::UiScope;

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

    /// Returns an optional icon glyph or name for the tab header.
    fn icon(&self) -> Option<&'static str> {
        None
    }

    /// Returns an optional texture UV rectangle for an icon in an atlas `[u0, v0, u1, v1]`.
    fn atlas_icon(&self) -> Option<[f32; 4]> {
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

    /// Returns an optional vector icon or icon glyph name for the tab header.
    fn icon(&self) -> Option<&'static str> {
        None
    }

    /// Returns an optional texture UV rectangle for an icon in an atlas `[u0, v0, u1, v1]`.
    fn atlas_icon(&self) -> Option<[f32; 4]> {
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
    /// Optional icon identifier or vector name.
    pub icon: Option<String>,
    /// Optional texture UV rectangle for an icon in an atlas `[u0, v0, u1, v1]`.
    pub atlas_icon: Option<[f32; 4]>,
    /// Whether the panel can be closed by the user.
    pub closeable: bool,
    /// Whether the background behind this tab's content should remain transparent.
    pub clear_background: bool,
    /// Whether the panel is subscribed to domain model transactions.
    pub is_domain_bound: bool,
    /// Optional declarative docking/floating placement rule used when opening or toggling this tab.
    pub placement: Option<TabPlacement<TabId>>,
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
            atlas_icon: None,
            closeable: true,
            clear_background: false,
            is_domain_bound: false,
            placement: None,
        }
    }

    /// Sets the optional unicode icon glyph or string icon identifier via fluent builder.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the optional texture atlas UV coordinates `[u0, v0, u1, v1]` via fluent builder.
    #[inline]
    #[must_use]
    pub fn with_atlas_icon(mut self, atlas_icon: [f32; 4]) -> Self {
        self.atlas_icon = Some(atlas_icon);
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
            .with_optional_atlas_icon(panel.atlas_icon())
            .with_closeable(panel.closeable())
            .with_clear_background(panel.clear_background())
            .with_domain_bound(panel.is_domain_bound())
    }

    /// Constructs panel metadata from an existing [`ScopedWorkbenchPanel`].
    #[must_use]
    pub fn from_scoped_panel<S: ScopedPanelContext>(
        panel: &dyn ScopedWorkbenchPanel<TabId, S>,
    ) -> Self {
        Self::new(panel.id(), panel.title())
            .with_optional_icon(panel.icon())
            .with_optional_atlas_icon(panel.atlas_icon())
            .with_closeable(panel.closeable())
            .with_clear_background(panel.clear_background())
            .with_domain_bound(panel.is_domain_bound())
            .with_optional_placement(panel.placement())
    }

    #[inline]
    fn with_optional_icon(mut self, icon: Option<&str>) -> Self {
        self.icon = icon.map(str::to_string);
        self
    }

    #[inline]
    fn with_optional_atlas_icon(mut self, atlas_icon: Option<[f32; 4]>) -> Self {
        self.atlas_icon = atlas_icon;
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