// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Mouse cursor icon calculation and hover target resolution for Iris UI editor overlays.
//!
//! Evaluates the active mouse coordinates against top menubar elements, docked splitters,
//! dock tabs, close buttons, floating window borders, and interactive panel widgets
//! to determine the appropriate system cursor icon (`Pointer`, `Resize`, `Crosshair`, `Text`, `Default`).

use crate::ui::iris_bridge::types::{InspectorColorDragMode, IrisEditorOverlay};
use irisui::dock::{DEFAULT_RESIZE_MARGIN, FloatingWindowCursor, evaluate_floating_resize_cursor};
use irisui::prelude::*;
use winit::window::CursorIcon;

/// Converts an engine-agnostic [`WidgetCursor`] enum from Iris UI into a platform-native [`CursorIcon`].
///
/// Handles all standard cursor shapes including pointers, directional resize handles,
/// text carats, grabs, and crosshairs.
#[inline]
pub fn map_widget_cursor_to_winit(cur: WidgetCursor) -> CursorIcon {
    match cur {
        WidgetCursor::Default => CursorIcon::Default,
        WidgetCursor::Pointer => CursorIcon::Pointer,
        WidgetCursor::Text => CursorIcon::Text,
        WidgetCursor::Crosshair => CursorIcon::Crosshair,
        WidgetCursor::Grab => CursorIcon::Grab,
        WidgetCursor::Grabbing => CursorIcon::Grabbing,
        WidgetCursor::ColResize => CursorIcon::ColResize,
        WidgetCursor::RowResize => CursorIcon::RowResize,
        WidgetCursor::EwResize => CursorIcon::EwResize,
        WidgetCursor::NsResize => CursorIcon::NsResize,
        WidgetCursor::NeswResize => CursorIcon::NeswResize,
        WidgetCursor::NwseResize => CursorIcon::NwseResize,
        WidgetCursor::NotAllowed => CursorIcon::NotAllowed,
    }
}

impl IrisEditorOverlay {
    /// Determines the appropriate mouse cursor icon based on current interactive hover targets.
    ///
    /// Evaluates cursor position against menubar buttons, window resize borders, docked splitters,
    /// dock tabs, tab close targets, and all panel-specific interactive elements (buttons, inputs,
    /// sliders, chips, and color pickers).
    pub fn requested_cursor_icon(&self) -> CursorIcon {
        let p = self.cursor_pos();
        if self.inspector.drag_number.is_some() {
            return CursorIcon::EwResize;
        }

        // 1. Floating window resize edges
        if let Some(edge_cursor) = evaluate_floating_resize_cursor(
            &self.chrome.floating_window_rects,
            p,
            DEFAULT_RESIZE_MARGIN,
        ) {
            return match edge_cursor {
                FloatingWindowCursor::NwseResize => CursorIcon::NwseResize,
                FloatingWindowCursor::NeswResize => CursorIcon::NeswResize,
                FloatingWindowCursor::ColResize => CursorIcon::ColResize,
                FloatingWindowCursor::RowResize => CursorIcon::RowResize,
            };
        }

        // 2. Native Tree-Driven Cursor Resolution (Primary Iris UI Authority)
        // Resolves menubar items, buttons, dock tabs, close buttons, splitters,
        // numeric input pills, tree rows, and asset cards with full layer/modal occlusion.
        let tree_cursor = self.tree.cursor_at(p);
        if tree_cursor != WidgetCursor::Default {
            return map_widget_cursor_to_winit(tree_cursor);
        }

        if let Some(mode) = self.inspector.color_drag_mode {
            return match mode {
                InspectorColorDragMode::SaturationValue => CursorIcon::Crosshair,
                InspectorColorDragMode::Hue => CursorIcon::NsResize,
            };
        }
        if let Some(ref targets) = self.inspector.targets {
            if let Some(ref picker) = targets.color_picker
                && let Some(cur) = irisui::prelude::evaluate_color_picker_cursor(picker, p, None)
            {
                return match cur {
                    irisui::prelude::ColorPickerCursor::Crosshair => CursorIcon::Crosshair,
                    irisui::prelude::ColorPickerCursor::NsResize => CursorIcon::NsResize,
                    irisui::prelude::ColorPickerCursor::Pointer => CursorIcon::Pointer,
                };
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
        if let Some(hit) = self.tree.hit_test_target(p)
            && hit.cursor == Some(WidgetCursor::Pointer)
        {
            return CursorIcon::Pointer;
        }

        // 5. Assets panel interactive items
        if let Some(ref targets) = self.assets.targets {
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

        // 6. 2D Visual UI Designer interactive items
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
            tab: PanelId::Hierarchy,
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

    #[test]
    fn test_floating_window_resize_cursor_mapping() {
        use irisui::dock::{
            DEFAULT_RESIZE_MARGIN, FloatingWindowCursor, evaluate_floating_resize_cursor,
        };
        use winit::window::CursorIcon;

        let rects = vec![Rect::new(100.0, 100.0, 200.0, 200.0)];

        // Top-left resize edge
        let hit = evaluate_floating_resize_cursor(
            &rects,
            Point::new(102.0, 102.0),
            DEFAULT_RESIZE_MARGIN,
        );
        assert_eq!(hit, Some(FloatingWindowCursor::NwseResize));
        let mapped: CursorIcon = match hit.unwrap() {
            FloatingWindowCursor::NwseResize => CursorIcon::NwseResize,
            FloatingWindowCursor::NeswResize => CursorIcon::NeswResize,
            FloatingWindowCursor::ColResize => CursorIcon::ColResize,
            FloatingWindowCursor::RowResize => CursorIcon::RowResize,
        };
        assert_eq!(mapped, CursorIcon::NwseResize);

        // Center should return None
        let hit_center = evaluate_floating_resize_cursor(
            &rects,
            Point::new(200.0, 200.0),
            DEFAULT_RESIZE_MARGIN,
        );
        assert_eq!(hit_center, None);
    }

    #[test]
    fn test_map_widget_cursor_to_winit_all_variants() {
        use super::map_widget_cursor_to_winit;
        use irisui::prelude::WidgetCursor;
        use winit::window::CursorIcon;

        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Default),
            CursorIcon::Default
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Pointer),
            CursorIcon::Pointer
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Text),
            CursorIcon::Text
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Crosshair),
            CursorIcon::Crosshair
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Grab),
            CursorIcon::Grab
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Grabbing),
            CursorIcon::Grabbing
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::ColResize),
            CursorIcon::ColResize
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::RowResize),
            CursorIcon::RowResize
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::EwResize),
            CursorIcon::EwResize
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::NsResize),
            CursorIcon::NsResize
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::NeswResize),
            CursorIcon::NeswResize
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::NwseResize),
            CursorIcon::NwseResize
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::NotAllowed),
            CursorIcon::NotAllowed
        );
    }
}