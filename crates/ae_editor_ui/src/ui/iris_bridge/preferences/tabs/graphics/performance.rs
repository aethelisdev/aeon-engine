// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance & Post-Processing Settings Cards Builder
//!
//! Renders framerate limiting, hardware MSAA samples, and HDR bloom cards.

use super::super::super::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesTargets,
    PreferencesToggleId,
};
use super::helpers::{build_checkbox, build_dropdown_row, build_slider_row};
use super::types::{CardLayoutContext, CheckboxParams, DropdownRowParams, SliderRowParams};
use irisui::prelude::*;

/// Builds the Performance & Framerate card.
pub fn build_perf_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: CardLayoutContext,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_collapsed = params.collapsed_sections.contains("graphics_perf");
    let perf_h = if is_collapsed { 36.0 } else { 72.0 };
    let card_rect = Rect::new(
        ctx.base_x,
        ctx.content_rect_y + ctx.y_offset,
        ctx.content_w,
        perf_h,
    );
    let section = SettingSectionBuilder::new(card_rect, "⚡  Performance & Framerate")
        .collapsed(is_collapsed)
        .cursor_pos(Some(params.cursor_pos))
        .build(tree, parent_id);
    let perf_card_id = section.card_id;
    targets
        .section_toggles
        .push(("graphics_perf", section.header_rect));

    if is_collapsed {
        return perf_h;
    }

    build_dropdown_row(
        tree,
        perf_card_id,
        DropdownRowParams {
            base_x: ctx.base_x + 14.0,
            y: ctx.content_rect_y + ctx.y_offset + 36.0,
            width: ctx.content_w - 28.0,
            label: "Framerate Limit",
            selected_text: params.graphics_settings.fps_limit.label(),
            dropdown_id: PreferencesDropdownId::FpsLimit,
            cursor_pos: params.cursor_pos,
            is_open: params.active_dropdown == Some(PreferencesDropdownId::FpsLimit),
        },
        targets,
    );

    perf_h
}

/// Builds the Anti-Aliasing (MSAA) card.
pub fn build_aa_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: CardLayoutContext,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_collapsed = params.collapsed_sections.contains("graphics_aa");
    let aa_h = if is_collapsed { 36.0 } else { 72.0 };
    let card_rect = Rect::new(
        ctx.base_x,
        ctx.content_rect_y + ctx.y_offset,
        ctx.content_w,
        aa_h,
    );
    let section = SettingSectionBuilder::new(card_rect, "🔍  Anti-Aliasing (MSAA)")
        .collapsed(is_collapsed)
        .cursor_pos(Some(params.cursor_pos))
        .build(tree, parent_id);
    let aa_card_id = section.card_id;
    targets
        .section_toggles
        .push(("graphics_aa", section.header_rect));

    if is_collapsed {
        return aa_h;
    }

    let msaa_label = match params.graphics_settings.msaa_samples {
        1 => "Off (1x)",
        2 => "2x",
        _ => "4x (Default)",
    };
    build_dropdown_row(
        tree,
        aa_card_id,
        DropdownRowParams {
            base_x: ctx.base_x + 14.0,
            y: ctx.content_rect_y + ctx.y_offset + 36.0,
            width: ctx.content_w - 28.0,
            label: "MSAA Samples",
            selected_text: msaa_label,
            dropdown_id: PreferencesDropdownId::MsaaSamples,
            cursor_pos: params.cursor_pos,
            is_open: params.active_dropdown == Some(PreferencesDropdownId::MsaaSamples),
        },
        targets,
    );

    aa_h
}

/// Builds the HDR Bloom Post-Processing card.
pub fn build_post_processing_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: CardLayoutContext,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_collapsed = params.collapsed_sections.contains("graphics_pp");
    let gs = params.graphics_settings;
    let pp_h = if is_collapsed {
        36.0
    } else if gs.bloom_enabled {
        96.0
    } else {
        64.0
    };
    let card_rect = Rect::new(
        ctx.base_x,
        ctx.content_rect_y + ctx.y_offset,
        ctx.content_w,
        pp_h,
    );
    let section = SettingSectionBuilder::new(card_rect, "✨  Post-Processing (Bloom)")
        .collapsed(is_collapsed)
        .cursor_pos(Some(params.cursor_pos))
        .build(tree, parent_id);
    let pp_card_id = section.card_id;
    targets
        .section_toggles
        .push(("graphics_pp", section.header_rect));

    if is_collapsed {
        return pp_h;
    }

    build_checkbox(
        tree,
        pp_card_id,
        CheckboxParams {
            rect: Rect::new(
                ctx.base_x + 14.0,
                ctx.content_rect_y + ctx.y_offset + 36.0,
                ctx.content_w - 28.0,
                20.0,
            ),
            label: "Enable Bloom",
            is_checked: gs.bloom_enabled,
            toggle_id: PreferencesToggleId::BloomEnabled,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

    if gs.bloom_enabled {
        let (is_editing, editing_buf) = match params.active_number_input {
            Some((PreferencesSliderId::BloomIntensity, buf)) => (true, buf),
            _ => (false, ""),
        };
        build_slider_row(
            tree,
            pp_card_id,
            SliderRowParams {
                base_x: ctx.base_x + 14.0,
                y: ctx.content_rect_y + ctx.y_offset + 64.0,
                width: ctx.content_w - 28.0,
                label: "Bloom Intensity",
                val_text: &format!("{:.2}", gs.bloom_intensity),
                current_val: gs.bloom_intensity,
                min_val: 0.0,
                max_val: 3.0,
                slider_id: PreferencesSliderId::BloomIntensity,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
    }

    pp_h
}