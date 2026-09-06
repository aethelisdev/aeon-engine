// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Event Handler & Hit-Testing
//!
//! Evaluates mouse clicks, canvas dragging, viewport panning, and zoom scrolling
//! against the UI Designer targets.
//!

use super::types::{UiDesignerAction, UiDesignerPanelTargets, UiDragState};
use irisui::prelude::Point;

/// Result returned from evaluating a mouse click on the UI Designer panel.
#[derive(Debug, Clone, Default)]
pub struct UiDesignerClickResult {
    /// Action emitted by the click interaction, if any.
    pub action: Option<UiDesignerAction>,
    /// Active drag state initiated if an element was grabbed.
    pub start_element_drag: Option<UiDragState>,
    /// Whether viewport canvas panning was initiated by clicking empty space.
    pub start_canvas_pan: bool,
}

/// Evaluates a mouse click against active UI Designer targets.
pub fn handle_ui_designer_click(
    click_pos: Point,
    targets: &UiDesignerPanelTargets,
    is_aspect_dropdown_open: bool,
    is_add_menu_open: bool,
) -> UiDesignerClickResult {
    let mut result = UiDesignerClickResult::default();

    // ── 1. Aspect Ratio Popup ─────────────────────────────────────────────────
    if is_aspect_dropdown_open {
        if let Some(popup_rect) = targets.aspect_popup_rect
            && popup_rect.contains_point(click_pos)
        {
            for (preset, rect) in &targets.aspect_dropdown_options {
                if rect.contains_point(click_pos) {
                    result.action = Some(UiDesignerAction::SetAspectRatio(*preset));
                    return result;
                }
            }
            return result;
        }
        // Clicked outside open popup: dismiss it
        result.action = Some(UiDesignerAction::ClosePopups);
        return result;
    }

    // ── 2. Add Element Palette Popup ──────────────────────────────────────────
    if is_add_menu_open {
        if let Some(popup_rect) = targets.add_popup_rect
            && popup_rect.contains_point(click_pos)
        {
            for (elem_type, rect) in &targets.add_menu_options {
                if rect.contains_point(click_pos) {
                    result.action = Some(UiDesignerAction::SpawnElement(*elem_type));
                    return result;
                }
            }
            return result;
        }
        // Clicked outside open popup: dismiss it
        result.action = Some(UiDesignerAction::ClosePopups);
        return result;
    }

    // ── 3. Toolbar Buttons ────────────────────────────────────────────────────
    if let Some(rect) = targets.btn_aspect
        && rect.contains_point(click_pos)
    {
        result.action = Some(UiDesignerAction::ToggleAspectDropdown);
        return result;
    }
    if let Some(rect) = targets.btn_zoom_out
        && rect.contains_point(click_pos)
    {
        let new_zoom = (targets.current_zoom - 0.1).max(0.25);
        result.action = Some(UiDesignerAction::SetZoom(new_zoom));
        return result;
    }
    if let Some(rect) = targets.btn_zoom_reset
        && rect.contains_point(click_pos)
    {
        result.action = Some(UiDesignerAction::ResetView);
        return result;
    }
    if let Some(rect) = targets.btn_zoom_in
        && rect.contains_point(click_pos)
    {
        let new_zoom = (targets.current_zoom + 0.1).min(3.0);
        result.action = Some(UiDesignerAction::SetZoom(new_zoom));
        return result;
    }
    if let Some(rect) = targets.btn_snap
        && rect.contains_point(click_pos)
    {
        result.action = Some(UiDesignerAction::CycleGridSnap);
        return result;
    }
    if let Some(rect) = targets.btn_anchors
        && rect.contains_point(click_pos)
    {
        result.action = Some(UiDesignerAction::ToggleAnchorGuides);
        return result;
    }
    if let Some(rect) = targets.btn_grid
        && rect.contains_point(click_pos)
    {
        result.action = Some(UiDesignerAction::ToggleGrid);
        return result;
    }
    if let Some(rect) = targets.btn_add_element
        && rect.contains_point(click_pos)
    {
        result.action = Some(UiDesignerAction::ToggleAddMenu);
        return result;
    }

    // ── 4. Virtual Canvas Elements Hit-Testing (Reverse z-order) ──────────────
    if targets.canvas_rect.width > 0.0 && targets.canvas_rect.height > 0.0 {
        for target in targets.element_targets.iter().rev() {
            if target.rect.contains_point(click_pos) {
                result.action = Some(UiDesignerAction::SelectEntity(Some(target.entity)));

                let screen_w = targets.resolution[0];
                let screen_h = targets.resolution[1];

                let rel_x = (click_pos.x - targets.canvas_rect.x) / targets.canvas_rect.width;
                let rel_y = (click_pos.y - targets.canvas_rect.y) / targets.canvas_rect.height;
                let mouse_canvas = [rel_x * screen_w, rel_y * screen_h];

                result.start_element_drag = Some(UiDragState {
                    entity: target.entity,
                    anchor_origin: target.anchor_origin,
                    drag_start_mouse_canvas: mouse_canvas,
                    initial_offset: target.initial_offset,
                });
                return result;
            }
        }
    }

    // ── 5. Empty Canvas Click (Deselect or Pan) ────────────────────────────────
    if targets.panel_rect.contains_point(click_pos) {
        result.action = Some(UiDesignerAction::SelectEntity(None));
        result.start_canvas_pan = true;
    }

    result
}

/// Evaluates mouse dragging movement for an element or canvas pan.
pub fn handle_ui_designer_drag(
    cursor_pos: Point,
    delta: [f32; 2],
    drag_state: Option<&UiDragState>,
    is_panning: bool,
    targets: &UiDesignerPanelTargets,
) -> Option<UiDesignerAction> {
    if let Some(drag) = drag_state
        && targets.canvas_rect.width > 0.0
        && targets.canvas_rect.height > 0.0
    {
        let screen_w = targets.resolution[0];
        let screen_h = targets.resolution[1];
        let rel_x = (cursor_pos.x - targets.canvas_rect.x) / targets.canvas_rect.width;
        let rel_y = (cursor_pos.y - targets.canvas_rect.y) / targets.canvas_rect.height;
        let current_mouse_canvas = [rel_x * screen_w, rel_y * screen_h];

        let delta_x = current_mouse_canvas[0] - drag.drag_start_mouse_canvas[0];
        let delta_y = current_mouse_canvas[1] - drag.drag_start_mouse_canvas[1];

        let mut new_offset_x = drag.initial_offset[0] + delta_x;
        let mut new_offset_y = drag.initial_offset[1] + delta_y;

        if let Some(snap) = targets.snap_grid
            && snap > 0.0
        {
            new_offset_x = (new_offset_x / snap).round() * snap;
            new_offset_y = (new_offset_y / snap).round() * snap;
        }

        return Some(UiDesignerAction::UpdateElementOffset {
            entity: drag.entity,
            offset: [new_offset_x, new_offset_y],
        });
    }

    if is_panning {
        return Some(UiDesignerAction::PanCanvas(delta));
    }

    None
}

/// Evaluates mouse wheel scroll over the canvas and adjusts zoom.
pub fn handle_ui_designer_scroll(
    cursor_pos: Point,
    scroll_delta_y: f32,
    targets: &UiDesignerPanelTargets,
) -> Option<UiDesignerAction> {
    if targets.panel_rect.contains_point(cursor_pos) && scroll_delta_y.abs() > 0.001 {
        let zoom_change = scroll_delta_y * 0.05;
        let new_zoom = (targets.current_zoom + zoom_change).clamp(0.25, 3.0);
        return Some(UiDesignerAction::SetZoom(new_zoom));
    }
    None
}