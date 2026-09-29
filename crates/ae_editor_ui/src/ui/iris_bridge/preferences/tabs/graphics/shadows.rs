// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Shadows Settings Card Builder
//!
//! Renders directional cascaded shadow configuration (resolution, cascades, PCF, bias)
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{
    pref_dropdown_row, pref_section_card, pref_slider_row, pref_toggle_row,
};
use crate::ui::iris_bridge::preferences::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesToggleId,
};
use irisui::prelude::*;

/// Renders the collapsible Shadows settings section card declaratively using [`UiScope`].
pub fn build_shadows_card(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_shadows");
    let gs = params.graphics_settings;

    pref_section_card(
        scope,
        "graphics_shadows",
        "🌓  Shadows",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_toggle_row(
                body,
                PreferencesToggleId::ShadowsEnabled,
                "Enable Directional Shadows",
                gs.shadow_enabled,
                params.hovered_tag,
            );

            if gs.shadow_enabled {
                pref_dropdown_row(
                    body,
                    PreferencesDropdownId::ShadowResolution,
                    "Resolution",
                    gs.shadow_resolution.label(),
                    params.active_dropdown == Some(PreferencesDropdownId::ShadowResolution),
                    params.hovered_tag,
                );

                let cascade_str = match gs.shadow_cascades {
                    3 => "3 Cascades (Default)",
                    _ => "4 Cascades (High Fidelity)",
                };
                pref_dropdown_row(
                    body,
                    PreferencesDropdownId::ShadowCascades,
                    "Cascade Count",
                    cascade_str,
                    params.active_dropdown == Some(PreferencesDropdownId::ShadowCascades),
                    params.hovered_tag,
                );

                pref_dropdown_row(
                    body,
                    PreferencesDropdownId::ShadowPcf,
                    "Filtering (PCF)",
                    gs.shadow_pcf.label(),
                    params.active_dropdown == Some(PreferencesDropdownId::ShadowPcf),
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::ShadowBias,
                    "Depth Bias",
                    gs.shadow_bias,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );
            }
        },
    );
}