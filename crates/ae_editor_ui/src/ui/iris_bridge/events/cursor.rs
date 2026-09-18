// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Mouse cursor icon calculation and hover target resolution for Iris UI editor overlays.
//!
//! Evaluates the active mouse coordinates against top menubar elements, docked splitters,
//! dock tabs, close buttons, floating window borders, and interactive panel widgets
//! to determine the appropriate system cursor icon (`Pointer`, `Resize`, `Crosshair`, `Text`, `Default`).

use crate::ui::iris_bridge::types::{InspectorColorDragMode, IrisEditorOverlay};
use irisui::dock::SplitDirection;
use irisui::prelude::*;
use winit::window::CursorIcon;

impl IrisEditorOverlay {
    /// Determines the appropriate mouse cursor icon based on current interactive hover targets.
    /// Evaluates cursor position against menubar buttons, window resize borders, docked splitters,
    /// dock tabs, tab close targets, and all panel-specific interactive elements (buttons, inputs,
    /// sliders, chips, and color pickers).
    pub fn requested_cursor_icon(&self) -> CursorIcon {
        let p = self.cursor_pos();
        if self.inspector.drag_number.is_some() {
            return CursorIcon::EwResize;
        }

        // 0. Top Menubar buttons & Foreground popup items
        if let Some(hit) = self.tree.hit_test_target(p)
            && (hit.role == WidgetRole::MenuBarItem
                || (hit.role == WidgetRole::Button
                    && hit.tag == super::super::menubar::TAG_ACTION_PLAY_PAUSE)
                || (hit.layer == UiLayer::Popup && hit.role == WidgetRole::DropdownItem))
        {
            return CursorIcon::Pointer;
        }

        // 1. Floating window resize edges
        for rect in &self.chrome.floating_window_rects {
            if rect.contains_point(p) {
                const MARGIN: f32 = 6.0;
                let on_left = p.x <= rect.x + MARGIN;
                let on_right = p.x >= rect.right() - MARGIN;
                let on_top = p.y <= rect.y + MARGIN;
                let on_bottom = p.y >= rect.bottom() - MARGIN;

                if (on_top && on_left) || (on_bottom && on_right) {
                    return CursorIcon::NwseResize;
                } else if (on_top && on_right) || (on_bottom && on_left) {
                    return CursorIcon::NeswResize;
                } else if on_left || on_right {
                    return CursorIcon::ColResize;
                } else if on_top || on_bottom {
                    return CursorIcon::RowResize;
                }
            }
        }

        // 2. Occlusion check: underlying docked splitters and tabs must not change cursor if occluded
        let is_occluded = self.is_point_over_modal_or_dropdown(p)
            || self
                .chrome
                .floating_window_rects
                .iter()
                .any(|r| r.contains_point(p));

        if !is_occluded && let Some(ref frame) = self.chrome.native_dock_frame {
            if frame
                .active_overflow_rect
                .is_some_and(|r| r.contains_point(p))
            {
                for item in &frame.overflow_item_targets {
                    if item.rect.contains_point(p) {
                        return CursorIcon::Pointer;
                    }
                }
                return CursorIcon::Default;
            }
            for tab in &frame.tab_targets {
                if tab.rect.contains_point(p) {
                    return CursorIcon::Pointer;
                }
            }
            for chevron in &frame.chevron_targets {
                if chevron.rect.contains_point(p) {
                    return CursorIcon::Pointer;
                }
            }
            for close in &frame.close_targets {
                if close.rect.contains_point(p) {
                    return CursorIcon::Pointer;
                }
            }
            for splitter in &frame.splitter_targets {
                if splitter.rect.contains_point(p) {
                    return match splitter.direction {
                        SplitDirection::Horizontal => CursorIcon::ColResize,
                        SplitDirection::Vertical => CursorIcon::RowResize,
                    };
                }
            }
        }
        if let Some(mode) = self.inspector.color_drag_mode {
            return match mode {
                InspectorColorDragMode::SaturationValue => CursorIcon::Crosshair,
                InspectorColorDragMode::Hue => CursorIcon::NsResize,
            };
        }
        if let Some(ref targets) = self.inspector.targets {
            if targets
                .color_picker_sv_box_rect
                .is_some_and(|r| r.contains_point(p))
            {
                return CursorIcon::Crosshair;
            }
            if targets
                .color_picker_hue_bar_rect
                .is_some_and(|r| r.contains_point(p))
            {
                return CursorIcon::NsResize;
            }
            if targets
                .number_inputs
                .iter()
                .any(|(_, r, _, _, _)| r.contains_point(p))
            {
                return CursorIcon::EwResize;
            }
            if targets
                .dropdowns
                .iter()
                .any(|(_, r, _)| r.contains_point(p))
                || targets
                    .component_delete_btns
                    .iter()
                    .any(|(_, r)| r.contains_point(p))
                || targets.add_component_btn_rect.contains_point(p)
                || targets.save_prefab_btn_rect.contains_point(p)
                || targets
                    .color_swatch_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets.hex_input_rect.is_some_and(|r| r.contains_point(p))
                || targets
                    .add_palette_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets
                    .clear_palette_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets.preset_btn_rect.is_some_and(|r| r.contains_point(p))
                || targets
                    .checkboxes
                    .iter()
                    .any(|(_, r, _)| r.contains_point(p))
                || targets
                    .palette_swatches
                    .iter()
                    .any(|(_, r, _)| r.contains_point(p))
                || targets
                    .color_picker_close_btn_rect
                    .is_some_and(|r| r.contains_point(p))
            {
                return CursorIcon::Pointer;
            }
            if let Some(picker_r) = targets.color_picker_popup_rect
                && picker_r.contains_point(p)
            {
                return CursorIcon::Pointer;
            }
        }

        // 3. Hierarchy interactive items
        if let Some(ref targets) = self.hierarchy.targets
            && (targets.add_btn_rect.contains_point(p)
                || targets.delete_btn_rect.is_some_and(|r| r.contains_point(p))
                || targets
                    .search_clear_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets
                    .entity_rows
                    .iter()
                    .any(|(_, row_r, _, _)| row_r.contains_point(p)))
        {
            return CursorIcon::Pointer;
        }

        // 4. Viewport HUD interactive items
        if let Some(ref hud) = self.viewport_hud.targets
            && (hud.buttons.iter().any(|(_, r)| r.contains_point(p))
                || hud
                    .dropdown_triggers
                    .iter()
                    .any(|(_, r)| r.contains_point(p)))
        {
            return CursorIcon::Pointer;
        }

        // 5. Assets panel interactive items
        if let Some(ref targets) = self.assets.targets {
            if let Some(ref pm) = targets.preview_modal {
                if pm.close_btn_rect.contains_point(p)
                    || pm.reveal_btn_rect.contains_point(p)
                    || pm.action_btn_rect.is_some_and(|r| r.contains_point(p))
                {
                    return CursorIcon::Pointer;
                }
                if pm.orbit_canvas_rect.is_some_and(|r| r.contains_point(p)) {
                    return CursorIcon::Grab;
                }
            }
            if let Some(ref cm) = targets.context_menu
                && cm.card_rect.contains_point(p)
            {
                return CursorIcon::Pointer;
            }
            if targets.import_btn_rect.contains_point(p)
                || targets.reveal_btn_rect.contains_point(p)
                || targets.clean_btn_rect.contains_point(p)
                || targets.grid_toggle_rect.contains_point(p)
                || targets.list_toggle_rect.contains_point(p)
                || targets.sidebar_toggle_btn_rect.contains_point(p)
                || targets
                    .search_clear_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets
                    .new_subfolder_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets.breadcrumbs.iter().any(|b| b.rect.contains_point(p))
                || targets
                    .category_chips
                    .iter()
                    .any(|(_, r)| r.contains_point(p))
                || targets
                    .folder_nodes
                    .iter()
                    .any(|n| n.row_rect.contains_point(p))
                || targets.grid_cards.iter().any(|c| c.rect.contains_point(p))
                || targets.list_rows.iter().any(|r| r.rect.contains_point(p))
            {
                return CursorIcon::Pointer;
            }
            if targets.search_input_rect.contains_point(p) {
                return CursorIcon::Text;
            }
        }

        // 6. Material & Surface Studio interactive items
        if let Some(ref targets) = self.material.targets
            && (targets
                .btn_change_texture
                .is_some_and(|r| r.contains_point(p))
                || targets
                    .btn_remove_texture
                    .is_some_and(|r| r.contains_point(p))
                || targets.btn_add_texture.is_some_and(|r| r.contains_point(p))
                || targets.btn_add_color.is_some_and(|r| r.contains_point(p))
                || targets
                    .submesh_alpha_buttons
                    .iter()
                    .any(|(_, _, _, r)| r.contains_point(p))
                || targets
                    .submesh_texture_buttons
                    .iter()
                    .any(|(_, _, r)| r.contains_point(p)))
        {
            return CursorIcon::Pointer;
        }

        // 7. 2D Visual UI Designer interactive items
        if let Some(ref targets) = self.ui_designer.targets
            && (targets.btn_aspect.is_some_and(|r| r.contains_point(p))
                || targets.btn_zoom_out.is_some_and(|r| r.contains_point(p))
                || targets.btn_zoom_reset.is_some_and(|r| r.contains_point(p))
                || targets.btn_zoom_in.is_some_and(|r| r.contains_point(p))
                || targets.btn_snap.is_some_and(|r| r.contains_point(p))
                || targets.btn_anchors.is_some_and(|r| r.contains_point(p))
                || targets.btn_grid.is_some_and(|r| r.contains_point(p))
                || targets.btn_add_element.is_some_and(|r| r.contains_point(p))
                || targets
                    .element_rects
                    .iter()
                    .any(|(_, r)| r.contains_point(p)))
        {
            return CursorIcon::Pointer;
        }

        CursorIcon::Default
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::iris_bridge::native_dock::{NativeDockCloseTarget, NativeDockTabTarget};
    use crate::ui::panel_layout::PanelId;
    use irisui::dock::DockNodeId;
    use irisui::prelude::{Point, Rect};

    #[test]
    fn test_dock_tab_target_contains_point() {
        let tab = NativeDockTabTarget {
            leaf: DockNodeId::default(),
            tab_index: 0,
            panel: PanelId::Hierarchy,
            rect: Rect::new(10.0, 10.0, 80.0, 24.0),
            leaf_rect: Rect::new(10.0, 10.0, 200.0, 400.0),
        };
        assert!(tab.rect.contains_point(Point::new(20.0, 15.0)));
        assert!(!tab.rect.contains_point(Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_dock_close_target_contains_point() {
        let close = NativeDockCloseTarget {
            leaf: DockNodeId::default(),
            tab_index: 0,
            rect: Rect::new(75.0, 14.0, 14.0, 16.0),
        };
        assert!(close.rect.contains_point(Point::new(80.0, 20.0)));
        assert!(!close.rect.contains_point(Point::new(50.0, 20.0)));
    }
}