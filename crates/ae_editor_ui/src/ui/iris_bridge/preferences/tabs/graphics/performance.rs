// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance & Post-Processing Settings Cards Builder
//!
//! Renders framerate limiting, hardware MSAA samples, and HDR bloom cards.

use super::super::super::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesTargets,
    PreferencesToggleId,
};
use super::helpers::{build_checkbox, build_dropdown_row, build_section_header, build_slider_row};
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
    let perf_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(perf_card_id) {
        node.set_name("PerfCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_rect_y + ctx.y_offset,
            ctx.content_w,
            perf_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, perf_card_id);

    build_section_header(
        tree,
        perf_card_id,
        super::types::SectionHeaderParams {
            base_x: ctx.base_x,
            y: ctx.content_rect_y + ctx.y_offset,
            width: ctx.content_w,
            section_id: "graphics_perf",
            title: "⚡  Performance & Framerate",
            is_collapsed,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

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
    let aa_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(aa_card_id) {
        node.set_name("AaCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_rect_y + ctx.y_offset,
            ctx.content_w,
            aa_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, aa_card_id);

    build_section_header(
        tree,
        aa_card_id,
        super::types::SectionHeaderParams {
            base_x: ctx.base_x,
            y: ctx.content_rect_y + ctx.y_offset,
            width: ctx.content_w,
            section_id: "graphics_aa",
            title: "🔍  Anti-Aliasing (MSAA)",
            is_collapsed,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

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
    let pp_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(pp_card_id) {
        node.set_name("PpCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_rect_y + ctx.y_offset,
            ctx.content_w,
            pp_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, pp_card_id);

    build_section_header(
        tree,
        pp_card_id,
        super::types::SectionHeaderParams {
            base_x: ctx.base_x,
            y: ctx.content_rect_y + ctx.y_offset,
            width: ctx.content_w,
            section_id: "graphics_pp",
            title: "✨  Post-Processing (Bloom)",
            is_collapsed,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

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