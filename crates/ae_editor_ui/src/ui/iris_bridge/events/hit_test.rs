// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hit-testing and overlay boundary verification subsystem for Iris UI editor components.
//!
//! Powered directly by Iris UI's native [`UiTree`] stacking layers ([`UiLayer`]),
//! providing zero-allocation, generic spatial queries without inspecting subsystem structs.

use crate::ui::iris_bridge::overlay_tree::ActiveOverlay;
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
        if self.overlay_tree.is_open() {
            if matches!(
                self.overlay_tree.active_overlay(),
                Some(ActiveOverlay::Modal(
                    crate::ui::iris_bridge::ModalKind::Preferences,
                ))
            ) {
                if self.overlay_tree.contains_point(point) {
                    return true;
                }
            } else if matches!(
                self.overlay_tree.active_overlay(),
                Some(ActiveOverlay::Modal(_))
            ) || self.overlay_tree.contains_point(point)
            {
                return true;
            }
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
        let root = tree.create_root().expect("root node");

        let mut scope = UiScope::new(&mut tree, root);

        // Modal node
        scope.container(
            Style::new()
                .position_absolute()
                .left(200.0)
                .top(200.0)
                .width(300.0)
                .height(200.0),
            |modal_parent| {
                modal_parent.modal_card(300.0, 200.0, |modal_scope| {
                    modal_scope.label_styled_passive(
                        "TestModalContent",
                        "Modal Content",
                        12.0,
                        Color::WHITE,
                        TextAlign::Left,
                        Style::new().width(300.0).height(200.0),
                    );
                });
            },
        );

        // Popup node
        scope.dropdown_menu_card_named("TestPopup", 100.0, 100.0, 150.0, |popup_scope| {
            popup_scope.label_styled_passive(
                "TestPopupContent",
                "Popup Content",
                12.0,
                Color::WHITE,
                TextAlign::Left,
                Style::new().width(150.0).height(200.0),
            );
        });

        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));

        // Verify layer queries
        assert_eq!(
            tree.layer_at(Point::new(120.0, 120.0)),
            Some(UiLayer::Popup)
        );
        assert_eq!(
            tree.layer_at(Point::new(350.0, 250.0)),
            Some(UiLayer::Modal)
        );
        assert_eq!(
            tree.layer_at(Point::new(50.0, 50.0)),
            Some(UiLayer::Content)
        );
        assert_eq!(tree.layer_at(Point::new(2000.0, 2000.0)), None);
    }

    #[test]
    fn test_preferences_modal_hit_test_bounds() {
        let mut overlay_tree = crate::ui::iris_bridge::OverlayTree::new();
        let pref_bounds = Rect::new(400.0, 200.0, 760.0, 540.0);
        overlay_tree.open(
            ActiveOverlay::Modal(crate::ui::iris_bridge::ModalKind::Preferences),
            crate::ui::iris_bridge::OverlayDismissPolicy::ExplicitOnly,
        );
        overlay_tree.set_overlay_bounds(pref_bounds);

        // Inside Preferences dialog
        assert!(overlay_tree.contains_point(Point::new(500.0, 300.0)));
        assert!(overlay_tree.contains_point(Point::new(400.0, 200.0)));

        // Outside Preferences dialog (in background workspace, y > MENUBAR_HEIGHT)
        assert!(!overlay_tree.contains_point(Point::new(100.0, 100.0)));
        assert!(!overlay_tree.contains_point(Point::new(1500.0, 500.0)));
    }
}