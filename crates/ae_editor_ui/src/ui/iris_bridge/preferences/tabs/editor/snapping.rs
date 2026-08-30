// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Snapping Preferences Card
//!
//! Renders the snapping mode selection ComboBox, grid size slider with numeric input pill,
//! and the floating popup menu when the dropdown is active.

use super::types::{EditorCardContext, SNAP_MODE_OPTIONS};
use crate::ui::iris_bridge::preferences::types::{
    PreferencesDropdownId, PreferencesSliderId, PreferencesTargets,
};
use ae_editor::snapping::{SnapMode, SnapSettings};
use irisui::prelude::*;

/// Builds the Snapping settings card and registers interactive targets.
pub fn build_snapping_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    virtual_y: f32,
    ctx: EditorCardContext<'_>,
    snapping: &SnapSettings,
    targets: &mut PreferencesTargets,
) -> (f32, Option<Rect>) {
    let is_snap_collapsed = ctx.collapsed_sections.contains("editor_snapping");
    let snap_h = if is_snap_collapsed { 36.0 } else { 100.0 };

    let snap_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(snap_card_id) {
        node.set_name("SnappingCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_y + virtual_y - ctx.scroll_y,
            ctx.content_w,
            snap_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, snap_card_id);

    let snap_header_rect = Rect::new(
        ctx.base_x + 8.0,
        ctx.content_y + virtual_y - ctx.scroll_y + 6.0,
        ctx.content_w - 16.0,
        24.0,
    );
    targets
        .section_toggles
        .push(("editor_snapping", snap_header_rect));
    let is_snap_hdr_hovered = snap_header_rect.contains_point(ctx.cursor_pos);

    let snap_title_id = tree.create_node();
    if let Some(node) = tree.get_mut(snap_title_id) {
        node.set_name("SnapTitle");
        let arrow = if is_snap_collapsed { "▸" } else { "▾" };
        node.set_text(format!("{} 🧲  Snapping", arrow));
        node.font_size = 13.0;
        node.line_height = 24.0;
        node.text_color = if is_snap_hdr_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.91, 0.96, 1.0)
        };
        node.computed_rect = snap_header_rect;
    }
    let _ = tree.add_child(snap_card_id, snap_title_id);

    let mut snap_combo_rect: Option<Rect> = None;
    if !is_snap_collapsed {
        // 1. Snap Mode Dropdown Row
        let snap_mode_label = match snapping.mode {
            SnapMode::Off => "Off",
            SnapMode::Hold => "Hold (Ctrl)",
            SnapMode::Toggle => "Toggle",
        };
        let combo_w = 200.0;
        let combo_rect = Rect::new(
            ctx.base_x + ctx.content_w - combo_w - 14.0,
            ctx.content_y + virtual_y - ctx.scroll_y + 36.0,
            combo_w,
            24.0,
        );
        snap_combo_rect = Some(combo_rect);
        let is_combo_hovered = combo_rect.contains_point(ctx.cursor_pos);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("SnapModeLabel");
            node.set_text("Snap Mode");
            node.font_size = 11.5;
            node.line_height = 24.0;
            node.text_color = Color::rgba(0.75, 0.78, 0.85, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + 14.0,
                ctx.content_y + virtual_y - ctx.scroll_y + 36.0,
                170.0,
                24.0,
            );
        }
        let _ = tree.add_child(snap_card_id, lbl_id);

        let is_combo_open = ctx.active_dropdown == Some(PreferencesDropdownId::SnapMode);
        let combo_id = tree.create_node();
        if let Some(node) = tree.get_mut(combo_id) {
            node.set_name("SnapCombo");
            node.computed_rect = combo_rect;
            let (bg, border_color) = if is_combo_open {
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
        let _ = tree.add_child(snap_card_id, combo_id);

        let combo_txt = tree.create_node();
        if let Some(node) = tree.get_mut(combo_txt) {
            node.set_name("SnapComboText");
            let arrow = if is_combo_open { "▲" } else { "▼" };
            node.set_text(format!("{}  {}", snap_mode_label, arrow));
            node.font_size = 11.5;
            node.line_height = 24.0;
            node.text_color = if is_combo_open || is_combo_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.90, 0.93, 0.98, 1.0)
            };
            node.computed_rect = Rect::new(
                combo_rect.x + 10.0,
                combo_rect.y,
                combo_rect.width - 20.0,
                24.0,
            );
        }
        let _ = tree.add_child(combo_id, combo_txt);
        targets
            .dropdowns
            .push((PreferencesDropdownId::SnapMode, combo_rect));

        // 2. Grid Size Slider
        let track_w = ctx.content_w - 28.0 - 170.0 - ctx.val_box_w - 16.0;
        let slider_y = ctx.content_y + virtual_y - ctx.scroll_y + 70.0;
        let track_rect = Rect::new(ctx.base_x + 14.0 + 170.0, slider_y + 8.0, track_w, 4.0);
        let is_track_hovered =
            Rect::new(track_rect.x, slider_y, track_w, 20.0).contains_point(ctx.cursor_pos);

        let lbl_node = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_node) {
            node.set_name("GridLabel");
            node.set_text("Grid Size");
            node.font_size = 11.5;
            node.line_height = 20.0;
            node.text_color = Color::rgba(0.75, 0.78, 0.85, 1.0);
            node.computed_rect = Rect::new(ctx.base_x + 14.0, slider_y, 170.0, 20.0);
        }
        let _ = tree.add_child(snap_card_id, lbl_node);

        let track_node = tree.create_node();
        if let Some(node) = tree.get_mut(track_node) {
            node.set_name("GridTrack");
            node.computed_rect = track_rect;
            node.style = Style::new()
                .background(Color::rgba(0.08, 0.09, 0.12, 0.90))
                .border(1.0, Color::rgba(0.18, 0.21, 0.30, 0.50))
                .border_radius(2.0);
        }
        let _ = tree.add_child(snap_card_id, track_node);

        let norm = ((snapping.grid_size - 0.1) / (10.0 - 0.1)).clamp(0.0, 1.0);
        let fill_w = (track_w * norm).max(2.0);
        let fill_node = tree.create_node();
        if let Some(node) = tree.get_mut(fill_node) {
            node.set_name("GridFill");
            node.computed_rect = Rect::new(track_rect.x, track_rect.y, fill_w, 4.0);
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.72, 0.88, 0.95))
                .border_radius(2.0);
        }
        let _ = tree.add_child(track_node, fill_node);

        let thumb_x = (track_rect.x + track_w * norm - 4.0)
            .clamp(track_rect.x - 2.0, track_rect.x + track_w - 6.0);
        let thumb_node = tree.create_node();
        if let Some(node) = tree.get_mut(thumb_node) {
            node.set_name("GridThumb");
            node.computed_rect = Rect::new(thumb_x, slider_y + 3.0, 8.0, 14.0);
            node.style = Style::new()
                .background(if is_track_hovered {
                    Color::rgba(0.0, 0.95, 1.0, 1.0)
                } else {
                    Color::rgba(0.88, 0.92, 0.98, 1.0)
                })
                .border_radius(2.0);
        }
        let _ = tree.add_child(snap_card_id, thumb_node);

        // Modern Sleek Number Input Pill Box
        let (is_editing, editing_buf) = match ctx.active_number_input {
            Some((PreferencesSliderId::GridSize, buf)) => (true, buf),
            _ => (false, ""),
        };
        let val_box_rect = Rect::new(
            ctx.base_x + 14.0 + 170.0 + track_w + 8.0,
            slider_y - 1.0,
            ctx.val_box_w,
            ctx.val_box_h,
        );
        let is_box_hovered = val_box_rect.contains_point(ctx.cursor_pos);

        let val_box_id = tree.create_node();
        if let Some(node) = tree.get_mut(val_box_id) {
            node.set_name("GridValBox");
            node.computed_rect = val_box_rect;
            let (bg, border_color) = if is_editing {
                (
                    Color::rgba(0.06, 0.08, 0.12, 1.0),
                    Color::rgba(0.0, 0.85, 1.0, 1.0),
                )
            } else if is_box_hovered {
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
        let _ = tree.add_child(snap_card_id, val_box_id);

        let val_node = tree.create_node();
        if let Some(node) = tree.get_mut(val_node) {
            node.set_name("GridVal");
            if is_editing {
                let cursor_str = if ctx.blink_caret { "|" } else { "" };
                node.set_text(format!("{}{}", editing_buf, cursor_str));
                node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
            } else {
                node.set_text(format!("{:.2}", snapping.grid_size));
                node.text_color = if is_box_hovered {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::rgba(0.90, 0.93, 0.98, 1.0)
                };
            }
            node.font_size = 11.5;
            node.line_height = ctx.val_box_h;
            node.text_align = TextAlign::Center;
            node.computed_rect = val_box_rect;
        }
        let _ = tree.add_child(val_box_id, val_node);

        targets.sliders.push((
            PreferencesSliderId::GridSize,
            Rect::new(track_rect.x, slider_y, track_w, 20.0),
            0.1,
            10.0,
            snapping.grid_size,
        ));

        targets.number_inputs.push((
            PreferencesSliderId::GridSize,
            val_box_rect,
            0.1,
            10.0,
            snapping.grid_size,
        ));
    }

    (snap_h, snap_combo_rect)
}

/// Renders the floating popup menu for SnapMode when open.
pub fn render_snap_mode_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    combo_rect: Rect,
    snapping: &SnapSettings,
    cursor_pos: Point,
    targets: &mut PreferencesTargets,
) {
    let popup_h = (SNAP_MODE_OPTIONS.len() as f32) * 24.0 + 4.0;
    let popup_rect = Rect::new(
        combo_rect.x,
        combo_rect.y + combo_rect.height + 2.0,
        combo_rect.width,
        popup_h,
    );
    targets.active_dropdown_popup_rect = Some(popup_rect);

    let popup_id = tree.create_node();
    if let Some(node) = tree.get_mut(popup_id) {
        node.set_name("SnapPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.13, 0.98))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
            .border_radius(6.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.85));
    }
    let _ = tree.add_child(parent_id, popup_id);

    for (idx, &(mode, label)) in SNAP_MODE_OPTIONS.iter().enumerate() {
        let item_y = popup_rect.y + 2.0 + (idx as f32) * 24.0;
        let item_rect = Rect::new(popup_rect.x + 2.0, item_y, popup_rect.width - 4.0, 22.0);
        let is_hovered = item_rect.contains_point(cursor_pos);
        let is_selected = snapping.mode == mode;

        let item_id = tree.create_node();
        if let Some(node) = tree.get_mut(item_id) {
            node.set_name("SnapPopupItem");
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

        let txt = tree.create_node();
        if let Some(node) = tree.get_mut(txt) {
            node.set_name("SnapItemText");
            node.set_text(label);
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
        let _ = tree.add_child(item_id, txt);

        targets
            .active_dropdown_items
            .push((idx, item_rect, label.to_string()));
    }
}