// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Context menu model and actions for tab headers in Iris UI docking system.
//!
//! Provides standard actions (close, close others, close to right, float, split)
//! and tracks open context menu state.

use crate::tree::DockNodeId;
use iris_core::Point;
use serde::{Deserialize, Serialize};

/// Standard and user-defined contextual actions available when right-clicking a tab.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TabContextMenuAction {
    /// Closes the selected tab.
    Close,
    /// Closes all other tabs in the same leaf pane, preserving only the selected tab.
    CloseOthers,
    /// Closes all tabs located to the right of the selected tab in the same leaf pane.
    CloseToRight,
    /// Tears off the selected tab into an independent floating window.
    FloatWindow,
    /// Splits the current leaf horizontally, moving the tab to the right pane.
    SplitRight,
    /// Splits the current leaf vertically, moving the tab to the bottom pane.
    SplitDown,
    /// Custom user-defined or plugin action with a designated command identifier.
    Custom(String),
}

impl TabContextMenuAction {
    /// Returns the user-facing display label for standard menu actions.
    pub fn display_label(&self) -> &str {
        match self {
            Self::Close => "Close",
            Self::CloseOthers => "Close Others",
            Self::CloseToRight => "Close Tabs to the Right",
            Self::FloatWindow => "Float Window",
            Self::SplitRight => "Split Right",
            Self::SplitDown => "Split Down",
            Self::Custom(label) => label.as_str(),
        }
    }
}

/// Active state tracking an open tab context menu popup.
#[derive(Debug, Clone, PartialEq)]
pub struct TabContextMenuState {
    /// Generational identifier of the leaf containing the target tab.
    pub leaf_id: DockNodeId,
    /// Zero-based index of the target tab within the leaf.
    pub tab_index: usize,
    /// Screen cursor coordinate where the right-click occurred.
    pub position: Point,
    /// List of available context menu actions presented to the user.
    pub actions: Vec<TabContextMenuAction>,
}

impl TabContextMenuState {
    /// Creates a new active context menu state at the specified cursor position.
    pub fn new(
        leaf_id: DockNodeId,
        tab_index: usize,
        position: Point,
        actions: Vec<TabContextMenuAction>,
    ) -> Self {
        Self {
            leaf_id,
            tab_index,
            position,
            actions,
        }
    }

    /// Returns the default set of contextual actions for standard closeable tabs.
    pub fn default_actions(
        is_closeable: bool,
        tab_count: usize,
        tab_index: usize,
    ) -> Vec<TabContextMenuAction> {
        let mut actions = Vec::new();

        if is_closeable {
            actions.push(TabContextMenuAction::Close);
        }

        if tab_count > 1 {
            actions.push(TabContextMenuAction::CloseOthers);
        }

        if tab_index + 1 < tab_count {
            actions.push(TabContextMenuAction::CloseToRight);
        }

        actions.push(TabContextMenuAction::FloatWindow);
        actions.push(TabContextMenuAction::SplitRight);
        actions.push(TabContextMenuAction::SplitDown);

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_context_menu_default_actions() {
        let actions = TabContextMenuState::default_actions(true, 3, 0);
        assert!(actions.contains(&TabContextMenuAction::Close));
        assert!(actions.contains(&TabContextMenuAction::CloseOthers));
        assert!(actions.contains(&TabContextMenuAction::CloseToRight));
        assert!(actions.contains(&TabContextMenuAction::FloatWindow));
        assert!(actions.contains(&TabContextMenuAction::SplitRight));
        assert!(actions.contains(&TabContextMenuAction::SplitDown));

        // For the last tab in a leaf, CloseToRight should not be present
        let last_tab_actions = TabContextMenuState::default_actions(true, 3, 2);
        assert!(!last_tab_actions.contains(&TabContextMenuAction::CloseToRight));

        // For a single non-closeable tab
        let single_pinned_actions = TabContextMenuState::default_actions(false, 1, 0);
        assert!(!single_pinned_actions.contains(&TabContextMenuAction::Close));
        assert!(!single_pinned_actions.contains(&TabContextMenuAction::CloseOthers));
        assert!(!single_pinned_actions.contains(&TabContextMenuAction::CloseToRight));
        assert!(single_pinned_actions.contains(&TabContextMenuAction::FloatWindow));
    }

    #[test]
    fn test_display_labels() {
        assert_eq!(TabContextMenuAction::Close.display_label(), "Close");
        assert_eq!(
            TabContextMenuAction::CloseOthers.display_label(),
            "Close Others"
        );
        assert_eq!(
            TabContextMenuAction::FloatWindow.display_label(),
            "Float Window"
        );
        let custom = TabContextMenuAction::Custom("Duplicate".into());
        assert_eq!(custom.display_label(), "Duplicate");
    }
}