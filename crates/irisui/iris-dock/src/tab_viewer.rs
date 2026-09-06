// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Lifecycle management trait and callback hooks for tabbed panels (`TabViewer`).
//!
//! Provides customizable delegates for tab titles, close button visibility,
//! close veto interception, tab addition triggers, and multi-window policies.

use crate::context_menu::TabContextMenuAction;
use crate::tree::DockNodeId;
use std::fmt::Display;

/// Lifecycle management delegate governing tab presentation and interaction policies.
/// Implementors customize tab headers, control whether tabs can be closed or torn off,
/// and intercept close events to prompt for unsaved changes or perform cleanup.
pub trait TabViewer<Tab> {
    /// Returns the user-facing display title for the specified tab.
    fn title(&self, tab: &Tab) -> String;

    /// Determines whether a close button (`x`) should be rendered for this tab.
    /// Returning `false` prevents the user from closing the tab (e.g., for persistent viewport panes).
    /// Defaults to `true`.
    fn closeable(&self, _tab: &Tab) -> bool {
        true
    }

    /// Callback invoked when the user requests closing this tab.
    /// Return `true` to approve closing and detaching the tab from its host leaf.
    /// Return `false` to veto the close request (e.g., when a dialog warns of unsaved scene modifications).
    /// Defaults to `true`.
    fn on_close(&mut self, _tab: &mut Tab) -> bool {
        true
    }

    /// Callback invoked when the user clicks the add tab button (`+`) on a leaf's tab bar.
    /// Implementations may display a popup menu or spawn a default panel into the specified leaf.
    fn on_add(&mut self, _leaf_id: DockNodeId) {}

    /// Determines whether the tab can be dragged out of its leaf for docking or floating.
    /// Returning `false` pins the tab in place within its current leaf.
    /// Defaults to `true`.
    fn is_draggable(&self, _tab: &Tab) -> bool {
        true
    }

    /// Determines whether this tab may be detached into an independent OS floating window.
    /// Defaults to `true`.
    fn allowed_in_windows(&self, _tab: &Tab) -> bool {
        true
    }

    /// Determines whether the tab has unsaved modifications and displays a dirty indicator dot (`•`).
    /// Defaults to `false`.
    fn is_modified(&self, _tab: &Tab) -> bool {
        false
    }

    /// Returns an optional tooltip string displayed when hovering over this tab's header button.
    /// Defaults to `None`.
    fn tooltip(&self, _tab: &Tab) -> Option<String> {
        None
    }

    /// Returns the list of contextual actions available when right-clicking this tab.
    fn context_menu(&self, tab: &Tab) -> Vec<TabContextMenuAction> {
        let is_closeable = self.closeable(tab);
        let mut actions = Vec::new();
        if is_closeable {
            actions.push(TabContextMenuAction::Close);
        }
        actions.push(TabContextMenuAction::CloseOthers);
        actions.push(TabContextMenuAction::CloseToRight);
        if self.allowed_in_windows(tab) {
            actions.push(TabContextMenuAction::FloatWindow);
        }
        actions.push(TabContextMenuAction::SplitRight);
        actions.push(TabContextMenuAction::SplitDown);
        actions
    }

    /// Determines whether the background behind this tab's content should remain transparent.
    /// Useful for embedded 3D viewports where the underlying render pass should not be obscured.
    /// Defaults to `false`.
    fn clear_background(&self, _tab: &Tab) -> bool {
        false
    }
}

/// Simple default [`TabViewer`] implementation using [`Display`] for tab titles.
/// Useful for quick prototyping or uniform tab types that implement `Display`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleTabViewer;

impl<Tab: Display> TabViewer<Tab> for SimpleTabViewer {
    fn title(&self, tab: &Tab) -> String {
        tab.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockTab {
        name: String,
        is_dirty: bool,
        is_pinned: bool,
    }

    struct MockViewer {
        add_triggered: Option<DockNodeId>,
    }

    impl TabViewer<MockTab> for MockViewer {
        fn title(&self, tab: &MockTab) -> String {
            if tab.is_dirty {
                format!("* {}", tab.name)
            } else {
                tab.name.clone()
            }
        }

        fn closeable(&self, tab: &MockTab) -> bool {
            !tab.is_pinned
        }

        fn on_close(&mut self, tab: &mut MockTab) -> bool {
            // Veto close if dirty
            !tab.is_dirty
        }

        fn on_add(&mut self, leaf_id: DockNodeId) {
            self.add_triggered = Some(leaf_id);
        }

        fn is_draggable(&self, tab: &MockTab) -> bool {
            !tab.is_pinned
        }
    }

    #[test]
    fn test_tab_viewer_lifecycle_and_veto() {
        let mut viewer = MockViewer {
            add_triggered: None,
        };

        let mut clean_tab = MockTab {
            name: "Scene".into(),
            is_dirty: false,
            is_pinned: false,
        };
        let mut dirty_tab = MockTab {
            name: "Script".into(),
            is_dirty: true,
            is_pinned: false,
        };
        let pinned_tab = MockTab {
            name: "Viewport".into(),
            is_dirty: false,
            is_pinned: true,
        };

        assert_eq!(viewer.title(&clean_tab), "Scene");
        assert_eq!(viewer.title(&dirty_tab), "* Script");

        assert!(viewer.closeable(&clean_tab));
        assert!(!viewer.closeable(&pinned_tab));

        assert!(viewer.is_draggable(&clean_tab));
        assert!(!viewer.is_draggable(&pinned_tab));

        // Close veto
        assert!(viewer.on_close(&mut clean_tab));
        assert!(!viewer.on_close(&mut dirty_tab));

        // Add hook
        let dummy_leaf = DockNodeId::default();
        viewer.on_add(dummy_leaf);
        assert_eq!(viewer.add_triggered, Some(dummy_leaf));
    }

    #[test]
    fn test_simple_tab_viewer() {
        let viewer = SimpleTabViewer;
        assert_eq!(viewer.title(&"Hierarchy"), "Hierarchy");
        assert!(viewer.closeable(&"Hierarchy"));
        assert!(viewer.is_draggable(&"Hierarchy"));
        assert!(!viewer.is_modified(&"Hierarchy"));
        assert_eq!(viewer.tooltip(&"Hierarchy"), None);
        let menu = viewer.context_menu(&"Hierarchy");
        assert!(menu.contains(&TabContextMenuAction::Close));
        assert!(menu.contains(&TabContextMenuAction::FloatWindow));
    }
}