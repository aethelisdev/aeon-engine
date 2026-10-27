// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Shell Workbench Descriptor
//!
//! Provides the primary [`WorkbenchDescriptor`] combining window titles, top menu bars,
//! bottom status bars, and the binary split-tree docking hierarchy into a unified declarative state.

use super::menu::MenuBarDescriptor;
use super::status::StatusBarDescriptor;
use iris_dock::DockState;

/// Declarative configuration descriptor for the entire application workbench shell.
///
/// Encapsulates top-level shell chrome properties:
/// - Application and window title.
/// - Top application menu bar ([`MenuBarDescriptor`]).
/// - Bottom telemetry and status bar ([`StatusBarDescriptor`]).
/// - Multi-pane docking state and detached floating windows ([`DockState<TabId>`]).
#[derive(Debug, Clone)]
pub struct WorkbenchDescriptor<TabId> {
    /// Window or application shell title.
    pub title: String,

    /// Docking state managing binary split-trees, tab bars, and detached floating windows.
    pub dock_state: DockState<TabId>,

    /// Declarative top application menu bar descriptor, if enabled.
    pub menubar: Option<MenuBarDescriptor>,

    /// Declarative bottom status bar and telemetry capsules descriptor, if enabled.
    pub statusbar: Option<StatusBarDescriptor>,
}

impl<TabId> WorkbenchDescriptor<TabId> {
    /// Creates a new workbench shell descriptor with the specified title and docking state.
    ///
    /// Menubar and statusbar are initially `None`.
    ///
    /// # Arguments
    /// * `title` - Application or window title (e.g. `"Aeon Engine Studio"`).
    /// * `dock_state` - Initialized docking tree controller.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, dock_state: DockState<TabId>) -> Self {
        Self {
            title: title.into(),
            dock_state,
            menubar: None,
            statusbar: None,
        }
    }

    /// Attaches a top application menu bar via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_menubar(mut self, menubar: MenuBarDescriptor) -> Self {
        self.menubar = Some(menubar);
        self
    }

    /// Attaches a bottom status bar via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_statusbar(mut self, statusbar: StatusBarDescriptor) -> Self {
        self.statusbar = Some(statusbar);
        self
    }

    /// Returns the window or application title.
    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns an immutable reference to the dock state.
    #[inline]
    #[must_use]
    pub const fn dock_state(&self) -> &DockState<TabId> {
        &self.dock_state
    }

    /// Returns a mutable reference to the dock state.
    #[inline]
    pub fn dock_state_mut(&mut self) -> &mut DockState<TabId> {
        &mut self.dock_state
    }

    /// Returns an immutable reference to the menu bar descriptor, if present.
    #[inline]
    #[must_use]
    pub const fn menubar(&self) -> Option<&MenuBarDescriptor> {
        self.menubar.as_ref()
    }

    /// Returns an immutable reference to the status bar descriptor, if present.
    #[inline]
    #[must_use]
    pub const fn statusbar(&self) -> Option<&StatusBarDescriptor> {
        self.statusbar.as_ref()
    }

    /// Declaratively renders the top application menu bar and baseline divider if configured.
    ///
    /// # Arguments
    /// * `scope` - Active declarative UI scope attached to the root container.
    /// * `screen_width` - Available width in physical pixels.
    ///
    /// Returns the semantic interaction tag of any clicked menu action, or `None`.
    pub fn render_menubar(
        &self,
        scope: &mut iris_widgets::declarative::UiScope<'_>,
        screen_width: f32,
    ) -> Option<u64> {
        self.menubar
            .as_ref()
            .and_then(|m| m.render(scope, screen_width))
    }

    /// Declaratively renders the bottom status bar and top baseline divider if configured.
    ///
    /// # Arguments
    /// * `scope` - Active declarative UI scope attached to the root container.
    /// * `screen_width` - Available width in physical pixels.
    /// * `screen_height` - Available window height in physical pixels.
    pub fn render_statusbar(
        &self,
        scope: &mut iris_widgets::declarative::UiScope<'_>,
        screen_width: f32,
        screen_height: f32,
    ) {
        if let Some(s) = &self.statusbar {
            s.render(scope, screen_width, screen_height);
        }
    }

    /// Declaratively renders the entire application shell chrome (top menubar, bottom statusbar,
    /// and baseline dividers) in a single unified invocation.
    ///
    /// Eliminates manual absolute positioning, ad-hoc divider nodes, and redundant coordinate
    /// calculations in consumer application shell update loops.
    ///
    /// # Arguments
    /// * `scope` - Active declarative UI scope attached to the root container.
    /// * `screen_width` - Available width in physical pixels.
    /// * `screen_height` - Available window height in physical pixels.
    ///
    /// Returns the semantic interaction tag of any clicked menu or chrome action, or `None`.
    pub fn render_chrome(
        &self,
        scope: &mut iris_widgets::declarative::UiScope<'_>,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<u64> {
        let menu_action = self.render_menubar(scope, screen_width);
        self.render_statusbar(scope, screen_width, screen_height);
        menu_action
    }
}