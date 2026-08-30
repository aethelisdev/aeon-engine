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

    let popup_h = (items_count as f32) * 24.0 + 4.0;
    let popup_rect = Rect::new(
        btn_rect.x,
        btn_rect.y + btn_rect.height + 2.0,
        btn_rect.width,
        popup_h,
    );
    targets.active_dropdown_popup_rect = Some(popup_rect);

    let popup_id = tree.create_node();
    if let Some(node) = tree.get_mut(popup_id) {
        node.set_name("GraphicsPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.13, 0.98))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
            .border_radius(6.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.85));
    }
    let _ = tree.add_child(parent_id, popup_id);

    for (idx, label) in item_labels.into_iter().enumerate() {
        let item_y = popup_rect.y + 2.0 + (idx as f32) * 24.0;
        let item_rect = Rect::new(popup_rect.x + 2.0, item_y, popup_rect.width - 4.0, 22.0);
        let is_hovered = item_rect.contains_point(cursor_pos);
        let is_selected = selected_idx == Some(idx);

        let item_id = tree.create_node();
        if let Some(node) = tree.get_mut(item_id) {
            node.set_name("GraphicsPopupItem");
            node.computed_rect = item_rect;
            let bg = if is_selected {
                Color::rgba(0.0, 0.35, 0.45, 0.80)
            } else if is_hovered {
                Color::rgba(0.24, 0.27, 0.37, 0.95)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.0)
            };
            node.style = Style::new().background(bg).border_radius(4.0);
        }
        let _ = tree.add_child(popup_id, item_id);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("GraphicsItemText");
            node.set_text(&label);
            node.font_size = 11.5;
            node.line_height = 22.0;
            node.text_color = if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else if is_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.85, 0.88, 0.95, 1.0)
            };
            node.computed_rect =
                Rect::new(item_rect.x + 8.0, item_rect.y, item_rect.width - 16.0, 22.0);
        }
        let _ = tree.add_child(item_id, lbl_id);

        targets.active_dropdown_items.push((idx, item_rect, label));
    }
}