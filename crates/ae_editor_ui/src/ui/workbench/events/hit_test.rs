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
    /// Returns whether the UI currently captures the pointer or if the pointer is over any active UI element.
    ///
    /// Evaluates active panels, floating windows, modal dialogs, and top menubars.
    /// Returns `false` if the pointer is positioned directly over the interactive 3D viewport canvas.
    #[must_use]
    pub fn wants_pointer_input(&self) -> bool {
        let point = self.iris_overlay.cursor_pos();
        self.wants_pointer_at([point.x, point.y])
    }

    /// Returns true if the point is over any UI panel, floating modal dialog, or outside the 3D viewport.
    ///
    /// # Arguments
    /// * `pos` - `[x, y]` screen-space coordinates in logical points.
    #[must_use]
    pub fn wants_pointer_at(&self, pos: [f32; 2]) -> bool {
        let point = Point::new(pos[0], pos[1]);
        let in_rect = |r: Rect| {
            point.x >= r.x && point.x <= r.right() && point.y >= r.y && point.y <= r.bottom()
        };

        // 1. Top menubar & active modal dialogs / preferences / popups (always highest z-order)
        if pos[1] <= IrisEditorOverlay::MENUBAR_HEIGHT
            || self.iris_overlay.modals.is_any_active()
            || self.iris_overlay.assets.preview_modal.is_some()
            || self.ui_rects.iter().any(|rect| in_rect(*rect))
        {
            return true;
        }

        if let Some(card_rect) = self.iris_overlay.preferences.card_rect
            && in_rect(card_rect)
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
            if self.iris_overlay.is_point_over_overlay(point) {
                return true;
            }
            if self.iris_overlay.tree.wants_pointer_at(point) {
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

    /// Compatibility alias forwarding to [`Self::wants_pointer_at`].
    #[must_use]
    #[inline]
    pub fn is_point_over_ui_rects(&self, pos: [f32; 2]) -> bool {
        self.wants_pointer_at(pos)
    }
}

/// Evaluates whether a cursor point falls within the outer window resize boundary margins.
///
/// Returns the matched [`winit::window::ResizeDirection`] and corresponding [`winit::window::CursorIcon`]
/// for OS-level window resizing on frameless windows.
///
/// # Arguments
/// * `point` - Cursor position in logical window coordinates.
/// * `width` - Window width in logical points.
/// * `height` - Window height in logical points.
#[must_use]
pub fn evaluate_window_resize_border(
    point: Point,
    width: f32,
    height: f32,
) -> Option<(winit::window::ResizeDirection, winit::window::CursorIcon)> {
    const BORDER: f32 = 6.0;
    const CORNER: f32 = 12.0;

    if width < CORNER * 2.0 || height < CORNER * 2.0 {
        return None;
    }

    use winit::window::{CursorIcon, ResizeDirection};

    // Four corners (evaluated first with larger hit area for comfortable diagonal grabbing)
    if point.x <= CORNER && point.y <= CORNER {
        Some((ResizeDirection::NorthWest, CursorIcon::NwResize))
    } else if point.x >= width - CORNER && point.y <= CORNER {
        Some((ResizeDirection::NorthEast, CursorIcon::NeResize))
    } else if point.x <= CORNER && point.y >= height - CORNER {
        Some((ResizeDirection::SouthWest, CursorIcon::SwResize))
    } else if point.x >= width - CORNER && point.y >= height - CORNER {
        Some((ResizeDirection::SouthEast, CursorIcon::SeResize))
    }
    // Four outer edges
    else if point.x <= BORDER {
        Some((ResizeDirection::West, CursorIcon::WResize))
    } else if point.x >= width - BORDER {
        Some((ResizeDirection::East, CursorIcon::EResize))
    } else if point.y <= BORDER {
        Some((ResizeDirection::North, CursorIcon::NResize))
    } else if point.y >= height - BORDER {
        Some((ResizeDirection::South, CursorIcon::SResize))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::window::{CursorIcon, ResizeDirection};

    #[test]
    fn test_evaluate_window_resize_border_corners_and_edges() {
        let (w, h) = (1000.0, 700.0);

        // Center should not be resize border
        assert_eq!(
            evaluate_window_resize_border(Point::new(500.0, 350.0), w, h),
            None
        );

        // Four corners
        assert_eq!(
            evaluate_window_resize_border(Point::new(2.0, 2.0), w, h),
            Some((ResizeDirection::NorthWest, CursorIcon::NwResize))
        );
        assert_eq!(
            evaluate_window_resize_border(Point::new(995.0, 2.0), w, h),
            Some((ResizeDirection::NorthEast, CursorIcon::NeResize))
        );
        assert_eq!(
            evaluate_window_resize_border(Point::new(2.0, 695.0), w, h),
            Some((ResizeDirection::SouthWest, CursorIcon::SwResize))
        );
        assert_eq!(
            evaluate_window_resize_border(Point::new(995.0, 695.0), w, h),
            Some((ResizeDirection::SouthEast, CursorIcon::SeResize))
        );

        // Four edges (away from corners)
        assert_eq!(
            evaluate_window_resize_border(Point::new(2.0, 300.0), w, h),
            Some((ResizeDirection::West, CursorIcon::WResize))
        );
        assert_eq!(
            evaluate_window_resize_border(Point::new(998.0, 300.0), w, h),
            Some((ResizeDirection::East, CursorIcon::EResize))
        );
        assert_eq!(
            evaluate_window_resize_border(Point::new(400.0, 2.0), w, h),
            Some((ResizeDirection::North, CursorIcon::NResize))
        );
        assert_eq!(
            evaluate_window_resize_border(Point::new(400.0, 698.0), w, h),
            Some((ResizeDirection::South, CursorIcon::SResize))
        );

        // Edge case: tiny window smaller than twice the corner margin
        assert_eq!(
            evaluate_window_resize_border(Point::new(2.0, 2.0), 10.0, 10.0),
            None
        );
    }
}