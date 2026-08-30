// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating 2D HSV Color Picker Popup Builder
//!
//! Renders hardware-accelerated GPU SDF floating popup with a 2D Saturation-Value box,
//! vertical Rainbow Hue spectrum bar, live color preview, and close button.

use super::types::{InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;

/// Builds the floating 2D HSV Color Picker popup for the currently active Inspector entity.
pub fn build_color_picker_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
) {
    targets.color_picker_popup_rect = None;
    targets.color_picker_close_btn_rect = None;
    targets.color_picker_sv_box_rect = None;
    targets.color_picker_hue_bar_rect = None;

    if !params.is_color_picker_open {
        return;
    }

    // Anchor to the Object Color swatch rect if available
    let Some(anchor_rect) = targets.color_swatch_rect else {
        return;
    };

    let state = HsvColorPickerState {
        hue: params.inspector_hsv[0],
        saturation: params.inspector_hsv[1],
        value: params.inspector_hsv[2],
        alpha: 1.0,
    };

    let (_widget_id, picker_targets) = HsvColorPickerBuilder::new(
        tree,
        parent_id,
        anchor_rect,
        state,
        params.cursor_pos,
        params.panel_rect,
    )
    .build();

    targets.color_picker_popup_rect = Some(picker_targets.card_rect);
    targets.color_picker_close_btn_rect = picker_targets.close_btn_rect;
    targets.color_picker_sv_box_rect = Some(picker_targets.sv_box_rect);
    targets.color_picker_hue_bar_rect = Some(picker_targets.hue_bar_rect);
}