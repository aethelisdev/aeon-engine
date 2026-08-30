// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Shadows Settings Card Builder
//!
//! Renders directional cascaded shadow configuration (resolution, cascades, PCF, bias).

use super::super::super::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesTargets,
    PreferencesToggleId,
};
use super::helpers::{build_checkbox, build_dropdown_row, build_section_header, build_slider_row};
use super::types::{CardLayoutContext, CheckboxParams, DropdownRowParams, SliderRowParams};
use irisui::prelude::*;

/// Builds the Shadows configuration card.
pub fn build_shadows_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: CardLayoutContext,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_collapsed = params.collapsed_sections.contains("graphics_shadows");
    let gs = params.graphics_settings;
    let sh_h = if is_collapsed {
        36.0
    } else if gs.shadow_enabled {
        174.0
    } else {
        64.0
    };
    let sh_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(sh_card_id) {
        node.set_name("ShadowsCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_rect_y + ctx.y_offset,
            ctx.content_w,
            sh_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, sh_card_id);

    build_section_header(
        tree,
        sh_card_id,
        super::types::SectionHeaderParams {
            base_x: ctx.base_x,
            y: ctx.content_rect_y + ctx.y_offset,
            width: ctx.content_w,
            section_id: "graphics_shadows",
            title: "🌓  Shadows",
            is_collapsed,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

    if is_collapsed {
        return sh_h;
    }

    build_checkbox(
        tree,
        sh_card_id,
        CheckboxParams {
            rect: Rect::new(
                ctx.base_x + 14.0,
                ctx.content_rect_y + ctx.y_offset + 36.0,
                ctx.content_w - 28.0,
                20.0,
            ),
            label: "Enable Shadows",
            is_checked: gs.shadow_enabled,
            toggle_id: PreferencesToggleId::ShadowsEnabled,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

    if gs.shadow_enabled {
        let mut row_y = ctx.content_rect_y + ctx.y_offset + 64.0;
        build_dropdown_row(
            tree,
            sh_card_id,
            DropdownRowParams {
                base_x: ctx.base_x + 14.0,
                y: row_y,
                width: ctx.content_w - 28.0,
                label: "Resolution",
                selected_text: gs.shadow_resolution.label(),
                dropdown_id: PreferencesDropdownId::ShadowResolution,
                cursor_pos: params.cursor_pos,
                is_open: params.active_dropdown == Some(PreferencesDropdownId::ShadowResolution),
            },
            targets,
        );
        row_y += 28.0;
        build_dropdown_row(
            tree,
            sh_card_id,
            DropdownRowParams {
                base_x: ctx.base_x + 14.0,
                y: row_y,
                width: ctx.content_w - 28.0,
                label: "Cascade Count",
                selected_text: &format!("{} Cascades", gs.shadow_cascades),
                dropdown_id: PreferencesDropdownId::ShadowCascades,
                cursor_pos: params.cursor_pos,
                is_open: params.active_dropdown == Some(PreferencesDropdownId::ShadowCascades),
            },
            targets,
        );
        row_y += 28.0;
        build_dropdown_row(
            tree,
            sh_card_id,
            DropdownRowParams {
                base_x: ctx.base_x + 14.0,
                y: row_y,
                width: ctx.content_w - 28.0,
                label: "PCF Quality",
                selected_text: gs.shadow_pcf.label(),
                dropdown_id: PreferencesDropdownId::ShadowPcf,
                cursor_pos: params.cursor_pos,
                is_open: params.active_dropdown == Some(PreferencesDropdownId::ShadowPcf),
            },
            targets,
        );
        row_y += 28.0;
        let (is_editing, editing_buf) = match params.active_number_input {
            Some((PreferencesSliderId::ShadowBias, buf)) => (true, buf),
            _ => (false, ""),
        };
        build_slider_row(
            tree,
            sh_card_id,
            SliderRowParams {
                base_x: ctx.base_x + 14.0,
                y: row_y,
                width: ctx.content_w - 28.0,
                label: "Bias",
                val_text: &format!("{:.4}", gs.shadow_bias),
                current_val: gs.shadow_bias,
                min_val: 0.0001,
                max_val: 0.05,
                slider_id: PreferencesSliderId::ShadowBias,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
    }

    sh_h
}