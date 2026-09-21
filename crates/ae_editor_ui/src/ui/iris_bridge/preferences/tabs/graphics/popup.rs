// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Dropdown Popup Renderer
//!
//! Renders top-layer Z-order popup menus for graphics resolution, cascades, PCF, FPS, MSAA, and Sky.

use super::super::super::types::{PreferencesDropdownId, PreferencesTargets};
use super::types::{
    CASCADE_OPTIONS, FPS_OPTIONS, MSAA_OPTIONS, PCF_OPTIONS, SHADOW_RES_OPTIONS, SKY_OPTIONS,
};
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;

/// Helper to render dropdown menu popups in the Graphics tab.
pub fn render_graphics_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    active_dd: PreferencesDropdownId,
    gs: &GraphicsSettings,
    targets: &mut PreferencesTargets,
    cursor_pos: Point,
) {
    let Some(&(_, btn_rect)) = targets.dropdowns.iter().find(|(id, _)| *id == active_dd) else {
        return;
    };

    let (items_count, item_labels, selected_idx): (usize, Vec<String>, Option<usize>) =
        match active_dd {
            PreferencesDropdownId::ShadowResolution => (
                SHADOW_RES_OPTIONS.len(),
                SHADOW_RES_OPTIONS
                    .iter()
                    .map(|s| s.label().to_string())
                    .collect(),
                SHADOW_RES_OPTIONS
                    .iter()
                    .position(|&s| s == gs.shadow_resolution),
            ),
            PreferencesDropdownId::ShadowCascades => (
                CASCADE_OPTIONS.len(),
                CASCADE_OPTIONS.iter().map(|(_, l)| l.to_string()).collect(),
                CASCADE_OPTIONS
                    .iter()
                    .position(|&(c, _)| c == gs.shadow_cascades),
            ),
            PreferencesDropdownId::ShadowPcf => (
                PCF_OPTIONS.len(),
                PCF_OPTIONS.iter().map(|p| p.label().to_string()).collect(),
                PCF_OPTIONS.iter().position(|&p| p == gs.shadow_pcf),
            ),
            PreferencesDropdownId::FpsLimit => (
                FPS_OPTIONS.len(),
                FPS_OPTIONS.iter().map(|f| f.label().to_string()).collect(),
                FPS_OPTIONS.iter().position(|&f| f == gs.fps_limit),
            ),
            PreferencesDropdownId::MsaaSamples => (
                MSAA_OPTIONS.len(),
                MSAA_OPTIONS.iter().map(|(_, l)| l.to_string()).collect(),
                MSAA_OPTIONS.iter().position(|&(m, _)| m == gs.msaa_samples),
            ),
            PreferencesDropdownId::SkyQuality => (
                SKY_OPTIONS.len(),
                SKY_OPTIONS.iter().map(|s| s.label().to_string()).collect(),
                SKY_OPTIONS.iter().position(|&s| s == gs.sky_quality),
            ),
            _ => return,
        };

    let _ = items_count;
    let labels_refs: Vec<&str> = item_labels.iter().map(|s| s.as_str()).collect();
    ComboboxPopupBuilder::new(btn_rect)
        .items(&labels_refs)
        .selected_index(selected_idx)
        .cursor_pos(cursor_pos)
        .name("GraphicsPopup")
        .build(tree, parent_id);
}