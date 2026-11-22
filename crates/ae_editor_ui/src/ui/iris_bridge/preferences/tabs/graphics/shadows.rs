// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Shadows Settings Card Builder
//!
//! Renders directional cascaded shadow configuration (resolution, cascades, PCF, bias)
//! declaratively using [`UiScope`] and immediate two-way bound property primitives.

use super::types::{PCF_OPTIONS, SHADOW_RES_OPTIONS};
use crate::ui::iris_bridge::preferences::tabs::PREF_COMBO_LAYOUT;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;

const SHADOW_RES_LABELS: [&str; 4] = ["Low (512)", "Medium (1024)", "High (2048)", "Ultra (4096)"];
const CASCADE_LABELS: [&str; 2] = ["3 Cascades (Default)", "4 Cascades (High Fidelity)"];
const PCF_LABELS: [&str; 3] = ["Off (Sharp)", "3x3 Soft", "5x5 Ultra Soft"];

/// Renders the collapsible Shadows settings section card declaratively using [`UiScope`].
pub fn build_shadows_card(scope: &mut UiScope<'_>, graphics: &mut GraphicsSettings) {
    scope.collapsing_section("graphics_shadows", "🌓  Shadows", true, |body| {
        body.property_checkbox("Enable Directional Shadows", &mut graphics.shadow_enabled);

        body.when(graphics.shadow_enabled, |s| {
            s.prop_combobox_mapped_styled(
                "Resolution",
                SHADOW_RES_OPTIONS
                    .iter()
                    .position(|&r| r == graphics.shadow_resolution)
                    .unwrap_or(2),
                &SHADOW_RES_LABELS,
                PREF_COMBO_LAYOUT,
                &mut graphics.shadow_resolution,
                |idx| {
                    SHADOW_RES_OPTIONS
                        .get(idx)
                        .copied()
                        .unwrap_or(SHADOW_RES_OPTIONS[2])
                },
            );

            s.prop_combobox_mapped_styled(
                "Cascade Count",
                usize::from(graphics.shadow_cascades != 3),
                &CASCADE_LABELS,
                PREF_COMBO_LAYOUT,
                &mut graphics.shadow_cascades,
                |idx| if idx == 0 { 3 } else { 4 },
            );

            s.prop_combobox_mapped_styled(
                "Filtering (PCF)",
                PCF_OPTIONS
                    .iter()
                    .position(|&p| p == graphics.shadow_pcf)
                    .unwrap_or(1),
                &PCF_LABELS,
                PREF_COMBO_LAYOUT,
                &mut graphics.shadow_pcf,
                |idx| PCF_OPTIONS.get(idx).copied().unwrap_or(PCF_OPTIONS[1]),
            );

            s.property_slider(
                "Depth Bias",
                &mut graphics.shadow_bias,
                0.0001,
                0.05,
                0.0005,
            );
        });
    });
}