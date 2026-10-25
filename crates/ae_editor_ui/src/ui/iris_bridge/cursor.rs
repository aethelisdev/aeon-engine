// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interactive cursor evaluation and resolution across composite UI layers.
//!
//! Provides cursor hit testing and shape determination for the retained Iris UI overlay,
//! handling modal dialogs, popups, floating window resize margins, and docked panel widgets.
//!

use crate::ui::iris_bridge::types::IrisEditorOverlay;
use irisui::core::WidgetCursor;

impl IrisEditorOverlay {
    /// Evaluates the active UI cursor across all composite layers (Shell elevated layers, Floating Windows, Dock Panels, Shell base).
    ///
    /// Resolves interactive cursors in strict back-to-front Z-order using pure, hardware-abstracted
    /// [`WidgetCursor`] tokens without platform window dependencies.
    #[must_use]
    pub fn active_cursor(&self) -> WidgetCursor {
        let p = self.cursor_pos();
        let is_overlay_hit = self.is_point_over_overlay(p);

        // 1. Topmost Overlay or elevated modals (checked directly from unified tree layered hit-test)
        if is_overlay_hit {
            let tree_cursor = self.tree.cursor_at(p);
            if tree_cursor != WidgetCursor::Default {
                return tree_cursor;
            }
        }

        // 2. Floating window 8-way resize boundary evaluation (Higher Z-layer than docked panels)
        if p.y > Self::MENUBAR_HEIGHT
            && !is_overlay_hit
            && let Some(cur) = irisui::dock::evaluate_floating_resize_cursor(
                &self.chrome.floating_window_rects,
                p,
                irisui::dock::DEFAULT_RESIZE_MARGIN,
            )
        {
            return cur.to_widget_cursor();
        }

        // 3. Unified Tree Cursor (Menubar, StatusBar, Viewport HUD, Docked Panels)
        let tree_cursor = self.tree.cursor_at(p);
        if tree_cursor != WidgetCursor::Default {
            return tree_cursor;
        }

        WidgetCursor::Default
    }
}