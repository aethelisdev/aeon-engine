// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Shadows Settings Card Builder
//!
//! Renders directional cascaded shadow configuration (resolution, cascades, PCF, bias)
//! declaratively using [`UiScope`] and immediate two-way bound property primitives.

use crate::ui::iris_bridge::preferences::components::{pref_dropdown_row, pref_section_card};
use crate::ui::iris_bridge::preferences::types::{PreferencesDropdownId, PreferencesParams};
use irisui::prelude::*;

/// Renders the collapsible Shadows settings section card declaratively using [`UiScope`].
///
/// Binds `gs.shadow_enabled` and `gs.shadow_bias` directly via declarative two-way property primitives,
/// eliminating intermediary action queues and enum dispatch boilerplate.
pub fn build_shadows_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_shadows");
    let active_dropdown = params.active_dropdown;
    let gs = &mut *params.graphics_settings;

    pref_section_card(
        scope,
        "graphics_shadows",
        "🌓  Shadows",
        is_collapsed,
        |body| {
            body.property_checkbox("Enable Directional Shadows", &mut gs.shadow_enabled);

            if gs.shadow_enabled {
                pref_dropdown_row(
                    body,
                    PreferencesDropdownId::ShadowResolution,
                    "Resolution",
                    gs.shadow_resolution.label(),
                    active_dropdown == Some(PreferencesDropdownId::ShadowResolution),
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
                    active_dropdown == Some(PreferencesDropdownId::ShadowCascades),
                );

                pref_dropdown_row(
                    body,
                    PreferencesDropdownId::ShadowPcf,
                    "Filtering (PCF)",
                    gs.shadow_pcf.label(),
                    active_dropdown == Some(PreferencesDropdownId::ShadowPcf),
                );

                body.property_slider("Depth Bias", &mut gs.shadow_bias, 0.0001, 0.05, 0.0005);
            }
        },
    );
}