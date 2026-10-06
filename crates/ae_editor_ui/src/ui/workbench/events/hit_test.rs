// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor UI Hit Testing & Occlusion Verification
//!
//! Determines whether screen-space coordinates land over active UI panels,
//! floating modal windows, top menu bars, or within the interactive 3D viewport canvas.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::EngineUi;
use irisui::prelude::{Point, Rect};

impl EngineUi {
    /// Returns true if the point is over any UI panel, floating modal dialog, or outside the 3D viewport.
    ///
    /// # Arguments
    /// * `pos` - `[x, y]` screen-space coordinates in logical points.
    pub fn is_point_over_ui_rects(&self, pos: [f32; 2]) -> bool {
        let point = Point::new(pos[0], pos[1]);
        let in_rect = |r: Rect| {
            point.x >= r.x && point.x <= r.right() && point.y >= r.y && point.y <= r.bottom()
        };

        // 1. Top menubar & active modal dialogs / preferences / popups (always highest z-order)
        if pos[1] <= IrisEditorOverlay::MENUBAR_HEIGHT
            || self.iris_overlay.modals.is_about_active
            || self.iris_overlay.modals.is_delete_active
            || self.iris_overlay.modals.is_new_folder_active
            || self.iris_overlay.modals.is_rename_active
            || self.iris_overlay.modals.is_loading_active
            || self.iris_overlay.assets.preview_modal.is_some()
            || self.ui_rects.iter().any(|rect| in_rect(*rect))
        {
            return true;
        }

        if let Some(card_rect) = self.iris_overlay.preferences.card_rect
            && (in_rect(card_rect) || self.iris_overlay.preferences.dropdown.is_some())
        {
            return true;
        }

        if let Some(dd_rect) = self.iris_overlay.menubar.dropdown_rect
            && in_rect(dd_rect)
        {
            return true;
        }

        // 2. If the point is inside the active 3D viewport canvas (docked or floating)
        if in_rect(self.last_viewport_rect) {
            if self.iris_overlay.is_point_over_popup(point) {
                return true;
            }
            if let Some(hit) = self.iris_overlay.tree.hit_test_target(point)
                && (hit.role == irisui::prelude::WidgetRole::Button || hit.tag > 0)
            {
                return true;
            }

            // Check if another detached floating window occludes this viewport point
            let is_occluded_by_other_floating = self
                .layout_state
                .dock_state
                .floating_windows
                .iter()
                .any(|w| {
                    let contains_viewport = w
                        .tree
                        .all_tabs()
                        .contains(&crate::ui::panel_layout::PanelId::Viewport);
                    if !contains_viewport {
                        w.contains_point(point)
                    } else {
                        false
                    }
                });

            if is_occluded_by_other_floating {
                return true;
            }

            // Directly over the 3D viewport canvas: mouse input belongs 100% to 3D scene
            return false;
        }

        // 3. Point is outside 3D viewport canvas -> belongs to UI panels
        true
    }
}