// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Event Handler & Hit-Testing
//!
//! Evaluates mouse clicks, canvas dragging, viewport panning, and zoom scrolling
//! purely via 64-bit hardware semantic tags and [`UiDesignerCanvasMetrics`].
//!

use super::popups::{ASPECT_RATIO_PRESETS, UI_ELEMENT_TYPES};
use super::types::{
    UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN, UI_DESIGNER_TAG_ASPECT_BTN,
    UI_DESIGNER_TAG_GRID_BTN, UI_DESIGNER_TAG_SNAP_BTN, UI_DESIGNER_TAG_ZOOM_IN,
    UI_DESIGNER_TAG_ZOOM_OUT, UI_DESIGNER_TAG_ZOOM_RESET, UiDesignerAction,
    UiDesignerCanvasMetrics, UiDragState, UiElementDragContext, parse_add_item_tag,
    parse_aspect_item_tag, parse_element_tag,
};
use irisui::prelude::*;

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

/// Evaluates a mouse click against active UI Designer tags and metrics.
pub fn handle_ui_designer_click(
    click_pos: Point,
    hit_target: Option<&HitTargetInfo>,
    metrics: &UiDesignerCanvasMetrics,
    drag_contexts: &[UiElementDragContext],
    is_aspect_dropdown_open: bool,
    is_add_menu_open: bool,
) -> UiDesignerClickResult {
    let mut result = UiDesignerClickResult::default();

    // ── 1. Aspect Ratio Popup ─────────────────────────────────────────────────
    if is_aspect_dropdown_open {
        if let Some(hit) = hit_target
            && hit.layer == UiLayer::Popup
            && hit.role == WidgetRole::DropdownItem
            && let Some(idx) = parse_aspect_item_tag(hit.tag)
            && let Some(&preset) = ASPECT_RATIO_PRESETS.get(idx)
        {
            result.action = Some(UiDesignerAction::SetAspectRatio(preset));
            return result;
        }
        // Clicked outside open popup: dismiss it
        result.action = Some(UiDesignerAction::ClosePopups);
        return result;
    }

    // ── 2. Add Element Palette Popup ──────────────────────────────────────────
    if is_add_menu_open {
        if let Some(hit) = hit_target
            && hit.layer == UiLayer::Popup
            && hit.role == WidgetRole::DropdownItem
            && let Some(idx) = parse_add_item_tag(hit.tag)
            && let Some(&elem_type) = UI_ELEMENT_TYPES.get(idx)
        {
            result.action = Some(UiDesignerAction::SpawnElement(elem_type));
            return result;
        }
        // Clicked outside open popup: dismiss it
        result.action = Some(UiDesignerAction::ClosePopups);
        return result;
    }

    // ── 3. Tag-Based Hardware Hit-Testing ─────────────────────────────────────
    if let Some(hit) = hit_target {
        // Priority 1: Check if an on-canvas UI element was clicked
        if let Some(elem_idx) = parse_element_tag(hit.tag)
            && let Some(drag_ctx) = drag_contexts.get(elem_idx)
        {
            result.action = Some(UiDesignerAction::SelectEntity(Some(drag_ctx.entity)));
            let mouse_canvas = metrics.screen_to_canvas(click_pos);
            result.start_element_drag = Some(UiDragState {
                entity: drag_ctx.entity,
                anchor_origin: drag_ctx.anchor_origin,
                drag_start_mouse_canvas: mouse_canvas,
                initial_offset: drag_ctx.initial_offset,
            });
            return result;
        }

        match hit.tag {
            UI_DESIGNER_TAG_ASPECT_BTN => {
                result.action = Some(UiDesignerAction::ToggleAspectDropdown);
                return result;
            }
            UI_DESIGNER_TAG_ZOOM_OUT => {
                let new_zoom = (metrics.current_zoom - 0.1).max(0.25);
                result.action = Some(UiDesignerAction::SetZoom(new_zoom));
                return result;
            }
            UI_DESIGNER_TAG_ZOOM_RESET => {
                result.action = Some(UiDesignerAction::ResetView);
                return result;
            }
            UI_DESIGNER_TAG_ZOOM_IN => {
                let new_zoom = (metrics.current_zoom + 0.1).min(3.0);
                result.action = Some(UiDesignerAction::SetZoom(new_zoom));
                return result;
            }
            UI_DESIGNER_TAG_SNAP_BTN => {
                result.action = Some(UiDesignerAction::CycleGridSnap);
                return result;
            }
            UI_DESIGNER_TAG_ANCHORS_BTN => {
                result.action = Some(UiDesignerAction::ToggleAnchorGuides);
                return result;
            }
            UI_DESIGNER_TAG_GRID_BTN => {
                result.action = Some(UiDesignerAction::ToggleGrid);
                return result;
            }
            UI_DESIGNER_TAG_ADD_ELEMENT_BTN => {
                result.action = Some(UiDesignerAction::ToggleAddMenu);
                return result;
            }
            _ => {}
        }
    }

    // ── 4. Empty Canvas Click (Deselect or Pan) ────────────────────────────────
    if metrics.panel_rect.contains_point(click_pos) {
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
    metrics: &UiDesignerCanvasMetrics,
) -> Option<UiDesignerAction> {
    if let Some(drag) = drag_state
        && metrics.canvas_rect.width > 0.0
        && metrics.canvas_rect.height > 0.0
    {
        let current_mouse_canvas = metrics.screen_to_canvas(cursor_pos);

        let delta_x = current_mouse_canvas[0] - drag.drag_start_mouse_canvas[0];
        let delta_y = current_mouse_canvas[1] - drag.drag_start_mouse_canvas[1];

        let mut new_offset_x = drag.initial_offset[0] + delta_x;
        let mut new_offset_y = drag.initial_offset[1] + delta_y;

        if let Some(snap) = metrics.snap_grid
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
    metrics: &UiDesignerCanvasMetrics,
) -> Option<UiDesignerAction> {
    if metrics.panel_rect.contains_point(cursor_pos) && scroll_delta_y.abs() > 0.001 {
        let zoom_change = scroll_delta_y * 0.05;
        let new_zoom = (metrics.current_zoom + zoom_change).clamp(0.25, 3.0);
        return Some(UiDesignerAction::SetZoom(new_zoom));
    }
    None
}