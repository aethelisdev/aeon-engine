// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Environment & Sky Settings Card Builder
//!
//! Renders physical Rayleigh/Mie sky scattering, ozone absorption, sun parameters, procedural clouds, and fog.

use super::super::super::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesTargets,
    PreferencesToggleId,
};
use super::helpers::{build_checkbox, build_dropdown_row, build_section_header, build_slider_row};
use super::types::{CardLayoutContext, CheckboxParams, DropdownRowParams, SliderRowParams};
use ae_renderer::graphics_settings::SkyQuality;
use irisui::prelude::*;

/// Builds the Environment & Sky configuration card.
pub fn build_environment_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: CardLayoutContext,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_collapsed = params.collapsed_sections.contains("graphics_env");
    let gs = params.graphics_settings;
    let is_advanced_sky = gs.sky_quality != SkyQuality::Low;
    let env_h = if is_collapsed {
        36.0
    } else if is_advanced_sky {
        440.0
    } else {
        180.0
    };
    let base_x = ctx.base_x;
    let content_w = ctx.content_w;

    let env_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(env_card_id) {
        node.set_name("EnvCard");
        node.computed_rect = Rect::new(base_x, ctx.content_rect_y + ctx.y_offset, content_w, env_h);
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, env_card_id);

    build_section_header(
        tree,
        env_card_id,
        super::types::SectionHeaderParams {
            base_x,
            y: ctx.content_rect_y + ctx.y_offset,
            width: content_w,
            section_id: "graphics_env",
            title: "⛅  Environment & Sky",
            is_collapsed,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );

    if is_collapsed {
        return env_h;
    }

    let mut env_row_y = ctx.content_rect_y + ctx.y_offset + 36.0;

    build_dropdown_row(
        tree,
        env_card_id,
        DropdownRowParams {
            base_x: base_x + 14.0,
            y: env_row_y,
            width: content_w - 28.0,
            label: "Sky Quality",
            selected_text: gs.sky_quality.label(),
            dropdown_id: PreferencesDropdownId::SkyQuality,
            cursor_pos: params.cursor_pos,
            is_open: params.active_dropdown == Some(PreferencesDropdownId::SkyQuality),
        },
        targets,
    );
    env_row_y += 28.0;

    let get_editing_state = |sid: PreferencesSliderId| -> (bool, &str) {
        match params.active_number_input {
            Some((id, buf)) if id == sid => (true, buf),
            _ => (false, ""),
        }
    };

    let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::SunPitch);
    build_slider_row(
        tree,
        env_card_id,
        SliderRowParams {
            base_x: base_x + 14.0,
            y: env_row_y,
            width: content_w - 28.0,
            label: "Sun Pitch",
            val_text: &format!("{:.2} rad", gs.sun_pitch),
            current_val: gs.sun_pitch,
            min_val: -std::f32::consts::PI,
            max_val: std::f32::consts::PI,
            slider_id: PreferencesSliderId::SunPitch,
            cursor_pos: params.cursor_pos,
            is_editing,
            editing_buffer: editing_buf,
            blink_caret: params.blink_caret,
        },
        targets,
    );
    env_row_y += 26.0;

    let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::SunYaw);
    build_slider_row(
        tree,
        env_card_id,
        SliderRowParams {
            base_x: base_x + 14.0,
            y: env_row_y,
            width: content_w - 28.0,
            label: "Sun Yaw",
            val_text: &format!("{:.2} rad", gs.sun_yaw),
            current_val: gs.sun_yaw,
            min_val: -std::f32::consts::PI,
            max_val: std::f32::consts::PI,
            slider_id: PreferencesSliderId::SunYaw,
            cursor_pos: params.cursor_pos,
            is_editing,
            editing_buffer: editing_buf,
            blink_caret: params.blink_caret,
        },
        targets,
    );
    env_row_y += 28.0;

    if is_advanced_sky {
        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::AtmosphereDensity);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Atmosphere Density",
                val_text: &format!("{:.2}", gs.atmosphere_density),
                current_val: gs.atmosphere_density,
                min_val: 0.0,
                max_val: 5.0,
                slider_id: PreferencesSliderId::AtmosphereDensity,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 26.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::OzoneDensity);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Ozone Absorption (Chappuis)",
                val_text: &format!("{:.2}", gs.ozone_density),
                current_val: gs.ozone_density,
                min_val: 0.0,
                max_val: 3.0,
                slider_id: PreferencesSliderId::OzoneDensity,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 26.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::SunDiscSize);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Sun Disc Size",
                val_text: &format!("{:.2}", gs.sun_disc_size),
                current_val: gs.sun_disc_size,
                min_val: 0.1,
                max_val: 5.0,
                slider_id: PreferencesSliderId::SunDiscSize,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 26.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::SunGlowStrength);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Sun Glow Strength",
                val_text: &format!("{:.2}", gs.sun_glow_strength),
                current_val: gs.sun_glow_strength,
                min_val: 0.0,
                max_val: 5.0,
                slider_id: PreferencesSliderId::SunGlowStrength,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 28.0;

        // Procedural Clouds Sub-Header
        let cld_title = tree.create_node();
        if let Some(node) = tree.get_mut(cld_title) {
            node.set_name("CloudsTitle");
            node.set_text("☁  Procedural Clouds");
            node.font_size = 12.0;
            node.line_height = 16.0;
            node.text_color = Color::rgba(0.0, 0.90, 1.0, 1.0);
            node.computed_rect = Rect::new(base_x + 14.0, env_row_y, content_w - 28.0, 16.0);
        }
        let _ = tree.add_child(env_card_id, cld_title);
        env_row_y += 20.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::CloudCoverage);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Cloud Coverage",
                val_text: &format!("{:.2}", gs.cloud_coverage),
                current_val: gs.cloud_coverage,
                min_val: 0.0,
                max_val: 1.0,
                slider_id: PreferencesSliderId::CloudCoverage,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 24.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::CloudDensity);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Cloud Density",
                val_text: &format!("{:.2}", gs.cloud_density),
                current_val: gs.cloud_density,
                min_val: 0.1,
                max_val: 3.0,
                slider_id: PreferencesSliderId::CloudDensity,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 24.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::CloudSpeed);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Wind Speed (Drift)",
                val_text: &format!("{:.2}", gs.cloud_speed),
                current_val: gs.cloud_speed,
                min_val: 0.0,
                max_val: 5.0,
                slider_id: PreferencesSliderId::CloudSpeed,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 24.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::CloudEvolution);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Turbulence / Evolution",
                val_text: &format!("{:.2}", gs.cloud_evolution),
                current_val: gs.cloud_evolution,
                min_val: 0.0,
                max_val: 3.0,
                slider_id: PreferencesSliderId::CloudEvolution,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 24.0;

        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::CloudAltitude);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Cloud Base Altitude",
                val_text: &format!("{:.0} m", gs.cloud_altitude),
                current_val: gs.cloud_altitude,
                min_val: 500.0,
                max_val: 5000.0,
                slider_id: PreferencesSliderId::CloudAltitude,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
        env_row_y += 26.0;
    }

    // Depth Fog Sub-Section
    build_checkbox(
        tree,
        env_card_id,
        CheckboxParams {
            rect: Rect::new(base_x + 14.0, env_row_y, content_w - 28.0, 20.0),
            label: "Enable Atmospheric Depth Fog",
            is_checked: gs.fog_enabled,
            toggle_id: PreferencesToggleId::FogEnabled,
            cursor_pos: params.cursor_pos,
        },
        targets,
    );
    env_row_y += 24.0;

    if gs.fog_enabled {
        let (is_editing, editing_buf) = get_editing_state(PreferencesSliderId::FogDistance);
        build_slider_row(
            tree,
            env_card_id,
            SliderRowParams {
                base_x: base_x + 14.0,
                y: env_row_y,
                width: content_w - 28.0,
                label: "Fog Distance",
                val_text: &format!("{:.0} m", gs.fog_distance),
                current_val: gs.fog_distance,
                min_val: 100.0,
                max_val: 2000.0,
                slider_id: PreferencesSliderId::FogDistance,
                cursor_pos: params.cursor_pos,
                is_editing,
                editing_buffer: editing_buf,
                blink_caret: params.blink_caret,
            },
            targets,
        );
    }

    env_h
}