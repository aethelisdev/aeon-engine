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

/// Dispatches interactive 2D HSV color picker actions: start, live preview, and atomic commit.
pub fn handle_color_edit_action(
    color_edit_start: &mut Option<(hecs::Entity, ae_core::ecs::Color)>,
    inspector_hsv: &mut [f32; 3],
    inspector_color_hex: &mut String,
    world: &hecs::World,
    ui_actions: &mut Vec<crate::ui::types::EngineUiAction>,
    action: super::types::InspectorAction,
) {
    let fallback = ae_core::ecs::Color {
        r: 0.60,
        g: 0.75,
        b: 0.95,
        a: 1.0,
    };
    match action {
        super::types::InspectorAction::StartColorEdit(entity) => {
            if color_edit_start.is_none() {
                let cur = world
                    .get::<&ae_core::ecs::Color>(entity)
                    .map(|c| *c)
                    .unwrap_or(fallback);
                *color_edit_start = Some((entity, cur));
            }
        }
        super::types::InspectorAction::LiveSetObjectColor(entity, col) => {
            if color_edit_start.is_none() {
                let cur = world
                    .get::<&ae_core::ecs::Color>(entity)
                    .map(|c| *c)
                    .unwrap_or(fallback);
                *color_edit_start = Some((entity, cur));
            }
            let new_col = ae_core::ecs::Color {
                r: col.r,
                g: col.g,
                b: col.b,
                a: col.a,
            };
            if let Ok(mut existing) = world.get::<&mut ae_core::ecs::Color>(entity) {
                *existing = new_col;
            }
            let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
            *inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
            let (h, s, v) = irisui::prelude::rgb_to_hsv(col.r, col.g, col.b);
            *inspector_hsv = [h, s, v];
        }
        super::types::InspectorAction::CommitColorEdit(entity) => {
            if let Some((snap_entity, start_col)) = color_edit_start.take()
                && snap_entity == entity
            {
                let final_col = world
                    .get::<&ae_core::ecs::Color>(entity)
                    .map(|c| *c)
                    .unwrap_or(start_col);
                if start_col != final_col {
                    ui_actions.push(crate::ui::types::EngineUiAction::ModifyColor(
                        entity, start_col, final_col,
                    ));
                }
            }
        }
        _ => {}
    }
}