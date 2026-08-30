// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # General Preferences Tab
//!
//! Renders global engine settings, language configuration, and display / UI scale settings.

use super::super::types::{PreferencesDropdownId, PreferencesParams, PreferencesTargets};
use irisui::prelude::*;

/// Scales table supported in the UI scale ComboBox.
pub const UI_SCALES: [(f32, &str); 7] = [
    (0.75, "75%"),
    (0.80, "80%"),
    (0.90, "90%"),
    (1.00, "100% (Default)"),
    (1.10, "110%"),
    (1.25, "125%"),
    (1.50, "150%"),
];

/// Builds the General & Interface preferences tab content.
pub fn build_general_tab(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let mut virtual_y = 16.0;
    let scroll_y = params.scroll_offset_y;
    let content_w = content_rect.width - 32.0;
    let base_x = content_rect.x + 16.0;

    // 1. Heading
    let heading_id = tree.create_node();
    if let Some(node) = tree.get_mut(heading_id) {
        node.set_name("GeneralHeading");
        node.set_text("General & Interface");
        node.font_size = 17.0;
        node.line_height = 22.0;
        node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            22.0,
        );
    }
    let _ = tree.add_child(parent_id, heading_id);
    virtual_y += 26.0;

    // 2. Subtitle
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("GeneralSubtitle");
        node.set_text("Global engine settings, language, and display scaling.");
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.65, 0.68, 0.76, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, sub_id);
    virtual_y += 24.0;

    // 3. Separator line
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("GeneralSep");
        node.style = Style::new().background(Color::rgba(0.20, 0.22, 0.30, 0.70));
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            1.0,
        );
    }
    let _ = tree.add_child(parent_id, sep_id);
    virtual_y += 16.0;

    // 4. Display & UI Scale Group Card
    let is_collapsed = params.collapsed_sections.contains("general_scale");
    let card_h = if is_collapsed { 36.0 } else { 110.0 };
    let card_rect = Rect::new(
        base_x,
        content_rect.y + virtual_y - scroll_y,
        content_w,
        card_h,
    );
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("ScaleGroupCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card_id);

    let header_rect = Rect::new(
        base_x + 8.0,
        content_rect.y + virtual_y - scroll_y + 6.0,
        content_w - 16.0,
        24.0,
    );
    targets.section_toggles.push(("general_scale", header_rect));
    let is_hdr_hovered = header_rect.contains_point(params.cursor_pos);

    // Group Title
    let grp_title_id = tree.create_node();
    if let Some(node) = tree.get_mut(grp_title_id) {
        node.set_name("ScaleGroupTitle");
        let arrow = if is_collapsed { "▸" } else { "▾" };
        node.set_text(format!("{} 🔍  Display & UI Scale", arrow));
        node.font_size = 13.0;
        node.line_height = 24.0;
        node.text_color = if is_hdr_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.91, 0.96, 1.0)
        };
        node.computed_rect = header_rect;
    }
    let _ = tree.add_child(card_id, grp_title_id);

    if !is_collapsed {
        // Group Description
        let grp_desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(grp_desc_id) {
            node.set_name("ScaleGroupDesc");
            node.set_text("Adjust interface scale for different monitor resolutions (or use Ctrl + / Ctrl - shortcuts):");
            node.font_size = 11.0;
            node.line_height = 15.0;
            node.text_color = Color::rgba(0.60, 0.63, 0.72, 1.0);
            node.computed_rect = Rect::new(
                base_x + 14.0,
                content_rect.y + virtual_y - scroll_y + 34.0,
                content_w - 28.0,
                15.0,
            );
        }
        let _ = tree.add_child(card_id, grp_desc_id);

        // ComboBox Dropdown Button
        let current_zoom = params.zoom_factor;
        let selected_label = UI_SCALES
            .iter()
            .find(|(val, _)| (current_zoom - *val).abs() < 0.01)
            .map(|(_, l)| *l)
            .unwrap_or_else(|| {
                if (current_zoom - 1.0).abs() < 0.01 {
                    "100% (Default)"
                } else {
                    ""
                }
            });
        let display_text = if selected_label.is_empty() {
            format!("{:.0}%", (current_zoom * 100.0).round())
        } else {
            selected_label.to_string()
        };

        let is_open = params.active_dropdown == Some(PreferencesDropdownId::UiScale);
        let combo_rect = Rect::new(
            base_x + 14.0,
            content_rect.y + virtual_y - scroll_y + 58.0,
            220.0,
            28.0,
        );
        let is_combo_hovered = combo_rect.contains_point(params.cursor_pos);
        let combo_id = tree.create_node();
        if let Some(node) = tree.get_mut(combo_id) {
            node.set_name("ScaleComboBox");
            node.computed_rect = combo_rect;
            let (bg, border_color) = if is_open {
                (
                    Color::rgba(0.06, 0.08, 0.12, 1.0),
                    Color::rgba(0.0, 0.85, 1.0, 1.0),
                )
            } else if is_combo_hovered {
                (
                    Color::rgba(0.24, 0.27, 0.37, 1.0),
                    Color::rgba(0.38, 0.46, 0.62, 1.0),
                )
            } else {
                (
                    Color::rgba(0.18, 0.20, 0.27, 0.95),
                    Color::rgba(0.26, 0.30, 0.42, 0.80),
                )
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, border_color)
                .border_radius(5.0);
        }
        let _ = tree.add_child(card_id, combo_id);

        let val_node = tree.create_node();
        if let Some(node) = tree.get_mut(val_node) {
            node.set_name("UiScaleVal");
            node.set_text(&display_text);
            node.font_size = 11.5;
            node.line_height = 28.0;
            node.text_color = if is_combo_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.90, 0.93, 0.98, 1.0)
            };
            node.computed_rect = Rect::new(
                combo_rect.x + 10.0,
                combo_rect.y,
                combo_rect.width - 24.0,
                28.0,
            );
        }
        let _ = tree.add_child(combo_id, val_node);

        let arr_node = tree.create_node();
        if let Some(node) = tree.get_mut(arr_node) {
            node.set_name("UiScaleArr");
            node.set_text(if is_open { "▲" } else { "▼" });
            node.font_size = 10.0;
            node.line_height = 28.0;
            node.text_align = TextAlign::Right;
            node.text_color = Color::rgba(0.65, 0.68, 0.76, 1.0);
            node.computed_rect = Rect::new(
                combo_rect.x + combo_rect.width - 22.0,
                combo_rect.y,
                14.0,
                28.0,
            );
        }
        let _ = tree.add_child(combo_id, arr_node);

        targets
            .dropdowns
            .push((PreferencesDropdownId::UiScale, combo_rect));

        // Floating Popup list if open
        if is_open {
            let popup_h = UI_SCALES.len() as f32 * 26.0 + 8.0;
            let popup_rect =
                Rect::new(combo_rect.x, combo_rect.y + 32.0, combo_rect.width, popup_h);
            targets.active_dropdown_popup_rect = Some(popup_rect);

            let popup_id = tree.create_node();
            if let Some(node) = tree.get_mut(popup_id) {
                node.set_name("UiScalePopup");
                node.computed_rect = popup_rect;
                node.style = Style::new()
                    .background(Color::rgba(0.08, 0.09, 0.13, 0.98))
                    .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.80))
                    .border_radius(4.0)
                    .box_shadow(0.0, 6.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.80));
            }
            let _ = tree.add_child(card_id, popup_id);

            for (idx, &(val, label)) in UI_SCALES.iter().enumerate() {
                let item_y = popup_rect.y + 4.0 + (idx as f32 * 26.0);
                let item_rect = Rect::new(popup_rect.x + 4.0, item_y, popup_rect.width - 8.0, 24.0);
                let is_item_hovered = item_rect.contains_point(params.cursor_pos);
                let is_item_selected = (current_zoom - val).abs() < 0.01;

                let item_id = tree.create_node();
                if let Some(node) = tree.get_mut(item_id) {
                    node.set_name("ScalePopupItem");
                    node.computed_rect = item_rect;
                    let item_bg = if is_item_selected {
                        Color::rgba(0.0, 0.35, 0.45, 0.80)
                    } else if is_item_hovered {
                        Color::rgba(0.18, 0.20, 0.28, 0.90)
                    } else {
                        Color::rgba(0.0, 0.0, 0.0, 0.0)
                    };
                    node.style = Style::new().background(item_bg).border_radius(3.0);
                }
                let _ = tree.add_child(popup_id, item_id);

                let item_lbl_id = tree.create_node();
                if let Some(node) = tree.get_mut(item_lbl_id) {
                    node.set_name("ScaleItemText");
                    node.set_text(label);
                    node.font_size = 11.5;
                    node.line_height = 24.0;
                    let text_color = if is_item_selected {
                        Color::rgba(0.0, 0.90, 1.0, 1.0)
                    } else if is_item_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 1.0)
                    } else {
                        Color::rgba(0.75, 0.78, 0.85, 1.0)
                    };
                    node.text_color = text_color;
                    node.computed_rect =
                        Rect::new(item_rect.x + 8.0, item_rect.y, item_rect.width - 16.0, 24.0);
                }
                let _ = tree.add_child(item_id, item_lbl_id);

                targets
                    .active_dropdown_items
                    .push((idx, item_rect, label.to_string()));
            }
        }
    }

    virtual_y += card_h + 20.0;
    virtual_y
}