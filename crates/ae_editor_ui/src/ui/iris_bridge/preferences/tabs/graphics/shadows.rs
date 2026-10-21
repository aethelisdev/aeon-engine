// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Shadows Settings Card Builder
//!
//! Renders directional cascaded shadow configuration (resolution, cascades, PCF, bias)
//! declaratively using [`UiScope`] and immediate two-way bound property primitives.

use super::types::{CASCADE_OPTIONS, PCF_OPTIONS, SHADOW_RES_OPTIONS};
use crate::ui::iris_bridge::preferences::components::{pref_combobox, pref_section_card};
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Renders the collapsible Shadows settings section card declaratively using [`UiScope`].
///
/// Binds `gs.shadow_enabled` and `gs.shadow_bias` directly via declarative two-way property primitives,
/// eliminating intermediary action queues and enum dispatch boilerplate.
pub fn build_shadows_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_shadows");
    let gs = &mut *params.graphics_settings;

    pref_section_card(
        scope,
        "graphics_shadows",
        "🌓  Shadows",
        is_collapsed,
        |body| {
            body.property_checkbox("Enable Directional Shadows", &mut gs.shadow_enabled);

            if gs.shadow_enabled {
                let resp = pref_combobox(
                    body,
                    "Resolution",
                    gs.shadow_resolution.label(),
                    &["Low (512)", "Medium (1024)", "High (2048)", "Ultra (4096)"],
                );
                if resp.changed()
                    && let Some(idx) = resp.selected_index()
                    && let Some(&res) = SHADOW_RES_OPTIONS.get(idx)
                {
                    gs.shadow_resolution = res;
                }

                let cascade_str = match gs.shadow_cascades {
                    3 => "3 Cascades (Default)",
                    _ => "4 Cascades (High Fidelity)",
                };
                let resp = pref_combobox(
                    body,
                    "Cascade Count",
                    cascade_str,
                    &["3 Cascades (Default)", "4 Cascades (High Fidelity)"],
                );
                if resp.changed()
                    && let Some(idx) = resp.selected_index()
                    && let Some(&(cascades, _)) = CASCADE_OPTIONS.get(idx)
                {
                    gs.shadow_cascades = cascades;
                }

                let resp = pref_combobox(
                    body,
                    "Filtering (PCF)",
                    gs.shadow_pcf.label(),
                    &["Off (Sharp)", "3x3 Soft", "5x5 Ultra Soft"],
                );
                if resp.changed()
                    && let Some(idx) = resp.selected_index()
                    && let Some(&pcf) = PCF_OPTIONS.get(idx)
                {
                    gs.shadow_pcf = pcf;
                }

                body.property_slider("Depth Bias", &mut gs.shadow_bias, 0.0001, 0.05, 0.0005);
            }
        },
    );
}