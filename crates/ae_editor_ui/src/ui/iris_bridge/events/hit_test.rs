// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hit-testing and overlay boundary verification subsystem for Iris UI editor components.
//!
//! Powered directly by Iris UI's native [`UiTree`] stacking layers ([`UiLayer`]),
//! providing zero-allocation, generic spatial queries without inspecting subsystem structs.

use crate::ui::iris_bridge::types::IrisEditorOverlay;
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Returns true if the coordinate is over any active popup layer widget (dropdowns, menus, palettes).
    ///
    /// Evaluated via [`UiTree::layer_at`] against [`UiLayer::Popup`] without inspecting panel structs.
    pub fn is_point_over_popup(&self, point: Point) -> bool {
        self.tree.layer_at(point) == Some(UiLayer::Popup)
    }

    /// Returns true if the coordinate is over an active floating modal dialog, Preferences window, or menubar dropdown.
    ///
    /// When true, underlying dock splitters, tabs, and panel controls MUST NOT receive click or drag interactions.
    pub fn is_point_over_modal_or_dropdown(&self, point: Point) -> bool {
        if point.y <= Self::MENUBAR_HEIGHT {
            return true;
        }
        if self.tree.is_modal_active() {
            return true;
        }
        self.tree
            .layer_at(point)
            .is_some_and(|layer| layer >= UiLayer::Modal)
    }

    /// Returns true if the given coordinate is over any interactive Iris UI element.
    ///
    /// Checks top menubar, bottom status bar, and queries the native UI tree via [`UiTree::hit_test_layered`].
    /// In the 3D viewport region without UI quads, returns `false` to allow camera and scene manipulation.
    pub fn is_point_over_overlay(&self, point: Point) -> bool {
        // 1. Top Menubar
        if point.y <= Self::MENUBAR_HEIGHT {
            return true;
        }
        // 2. Bottom Status Bar
        if self.screen_height > Self::STATUS_BAR_HEIGHT
            && point.y >= (self.screen_height - Self::STATUS_BAR_HEIGHT)
        {
            return true;
        }
        // 3. Any active UI node in the retained UiTree
        self.tree.hit_test_layered(point).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_queries_for_modals_and_popups() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        // Modal node
        let modal = tree.create_node();
        if let Some(node) = tree.get_mut(modal) {
            node.set_role(WidgetRole::ModalWindow);
            node.computed_rect = Rect::new(200.0, 200.0, 300.0, 200.0);
        }
        let _ = tree.add_child(root, modal);

        // Popup node
        let popup = tree.create_node();
        if let Some(node) = tree.get_mut(popup) {
            node.set_role(WidgetRole::DropdownPopup);
            node.computed_rect = Rect::new(100.0, 100.0, 150.0, 200.0);
        }
        let _ = tree.add_child(root, popup);

        // Verify layer queries
        assert_eq!(
            tree.layer_at(Point::new(120.0, 120.0)),
            Some(UiLayer::Popup)
        );
        assert_eq!(
            tree.layer_at(Point::new(350.0, 250.0)),
            Some(UiLayer::Modal)
        );
        assert_eq!(tree.layer_at(Point::new(50.0, 50.0)), None);
    }
}