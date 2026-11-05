// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Isolated Workbench Panel Interface & Metadata
//!
//! Provides the [`WorkbenchPanel`] trait for autonomous, decoupled desktop panels
//! capable of declaring their metadata, domain binding, close policies, and
//! self-contained declarative UI rendering.

use iris_widgets::UiScope;

/// Common interface implemented by autonomous workbench tool panels.
///
/// Decouples individual tool views (Hierarchy, Inspector, Console, Assets, Viewport)
/// from monolithic parameter bags, providing single-responsibility encapsulation.
pub trait WorkbenchPanel<TabId, AppState>: Send + Sync {
    /// Returns the unique tab identifier of this panel.
    fn id(&self) -> TabId;

    /// Returns the user-facing title of this panel displayed in tab headers.
    fn title(&self) -> &str;

    /// Returns an optional vector icon or icon glyph name for the tab header.
    fn icon(&self) -> Option<&'static str> {
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

    /// Declaratively renders the panel content into the given [`UiScope`].
    ///
    /// # Arguments
    /// * `scope` - Declarative UI scope scoped to the panel's container node.
    /// * `state` - Mutable reference to the application domain state.
    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut AppState);
}

/// Immutable metadata snapshot describing a registered workbench panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelMetadata<TabId> {
    /// Unique panel tab identifier.
    pub id: TabId,
    /// User-facing display title.
    pub title: String,
    /// Optional icon identifier or vector name.
    pub icon: Option<String>,
    /// Whether the panel can be closed by the user.
    pub closeable: bool,
    /// Whether the panel is subscribed to domain model transactions.
    pub is_domain_bound: bool,
}

impl<TabId: Clone> PanelMetadata<TabId> {
    /// Constructs panel metadata from an existing [`WorkbenchPanel`].
    #[must_use]
    pub fn from_panel<AppState>(panel: &dyn WorkbenchPanel<TabId, AppState>) -> Self {
        Self {
            id: panel.id(),
            title: panel.title().to_string(),
            icon: panel.icon().map(str::to_string),
            closeable: panel.closeable(),
            is_domain_bound: panel.is_domain_bound(),
        }
    }
}
