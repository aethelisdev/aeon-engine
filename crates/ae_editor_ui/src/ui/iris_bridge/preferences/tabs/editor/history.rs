// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # History Preferences Card
//!
//! Renders undo history limit configuration with an interactive slider and numeric pill input.

use super::types::EditorCardContext;
use crate::ui::iris_bridge::preferences::types::{PreferencesSliderId, PreferencesTargets};
use ae_editor::editor_state::EditorConfig;
use irisui::prelude::*;

/// Builds the History settings card and registers interactive targets.
pub fn build_history_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    virtual_y: f32,
    ctx: EditorCardContext<'_>,
    cfg: &EditorConfig,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_hist_collapsed = ctx.collapsed_sections.contains("editor_history");
    let hist_h = if is_hist_collapsed { 36.0 } else { 88.0 };

    let hist_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(hist_card_id) {
        node.set_name("HistoryCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_y + virtual_y - ctx.scroll_y,
            ctx.content_w,
            hist_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, hist_card_id);

    let hist_header_rect = Rect::new(
        ctx.base_x + 8.0,
        ctx.content_y + virtual_y - ctx.scroll_y + 6.0,
        ctx.content_w - 16.0,
        24.0,
    );
    targets
        .section_toggles
        .push(("editor_history", hist_header_rect));
    let is_hist_hdr_hovered = hist_header_rect.contains_point(ctx.cursor_pos);

    let hist_title = tree.create_node();
    if let Some(node) = tree.get_mut(hist_title) {
        node.set_name("HistTitle");
        let arrow = if is_hist_collapsed { "▸" } else { "▾" };
        node.set_text(format!("{} 📝  History Settings", arrow));
        node.font_size = 13.0;
        node.line_height = 24.0;
        node.text_color = if is_hist_hdr_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.91, 0.96, 1.0)
        };
        node.computed_rect = hist_header_rect;
    }
    let _ = tree.add_child(hist_card_id, hist_title);

    let track_w = ctx.content_w - 28.0 - 170.0 - ctx.val_box_w - 16.0;
    if !is_hist_collapsed {
        let hist_sl_y = ctx.content_y + virtual_y - ctx.scroll_y + 36.0;
        let hist_lbl = tree.create_node();
        if let Some(node) = tree.get_mut(hist_lbl) {
            node.set_name("HistLabel");
            node.set_text("Undo History Limit:");
            node.font_size = 11.5;
            node.line_height = 20.0;
            node.text_color = Color::rgba(0.75, 0.78, 0.85, 1.0);
            node.computed_rect = Rect::new(ctx.base_x + 14.0, hist_sl_y, 170.0, 20.0);
        }
        let _ = tree.add_child(hist_card_id, hist_lbl);

        let hist_track_rect = Rect::new(ctx.base_x + 14.0 + 170.0, hist_sl_y + 8.0, track_w, 4.0);
        let is_hist_track_hovered =
            Rect::new(hist_track_rect.x, hist_sl_y, track_w, 20.0).contains_point(ctx.cursor_pos);
        let hist_track = tree.create_node();
        if let Some(node) = tree.get_mut(hist_track) {
            node.set_name("HistTrack");
            node.computed_rect = hist_track_rect;
            node.style = Style::new()
                .background(Color::rgba(0.08, 0.09, 0.12, 0.90))
                .border(1.0, Color::rgba(0.18, 0.21, 0.30, 0.50))
                .border_radius(2.0);
        }
        let _ = tree.add_child(hist_card_id, hist_track);

        let hist_norm = ((cfg.max_undo_history as f32 - 10.0) / (5000.0 - 10.0)).clamp(0.0, 1.0);
        let hist_fill = tree.create_node();
        if let Some(node) = tree.get_mut(hist_fill) {
            node.set_name("HistFill");
            node.computed_rect = Rect::new(
                hist_track_rect.x,
                hist_track_rect.y,
                (track_w * hist_norm).max(2.0),
                4.0,
            );
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.72, 0.88, 0.95))
                .border_radius(2.0);
        }
        let _ = tree.add_child(hist_track, hist_fill);

        let hist_thumb = tree.create_node();
        if let Some(node) = tree.get_mut(hist_thumb) {
            node.set_name("HistThumb");
            node.computed_rect = Rect::new(
                hist_track_rect.x + track_w * hist_norm - 4.0,
                hist_sl_y + 3.0,
                8.0,
                14.0,
            );
            node.style = Style::new()
                .background(if is_hist_track_hovered {
                    Color::rgba(0.0, 0.95, 1.0, 1.0)
                } else {
                    Color::rgba(0.88, 0.92, 0.98, 1.0)
                })
                .border_radius(2.0);
        }
        let _ = tree.add_child(hist_card_id, hist_thumb);

        // Modern Sleek Number Input Pill Box
        let (is_editing, editing_buf) = match ctx.active_number_input {
            Some((PreferencesSliderId::UndoHistoryLimit, buf)) => (true, buf),
            _ => (false, ""),
        };
        let val_box_rect = Rect::new(
            ctx.base_x + 14.0 + 170.0 + track_w + 8.0,
            hist_sl_y - 1.0,
            ctx.val_box_w,
            ctx.val_box_h,
        );
        let is_box_hovered = val_box_rect.contains_point(ctx.cursor_pos);

        let val_box_id = tree.create_node();
        if let Some(node) = tree.get_mut(val_box_id) {
            node.set_name("HistValBox");
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
        let _ = tree.add_child(hist_card_id, val_box_id);

        let hist_val = tree.create_node();
        if let Some(node) = tree.get_mut(hist_val) {
            node.set_name("HistVal");
            if is_editing {
                let cursor_str = if ctx.blink_caret { "|" } else { "" };
                node.set_text(format!("{}{}", editing_buf, cursor_str));
                node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
            } else {
                node.set_text(format!("{}", cfg.max_undo_history));
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
        let _ = tree.add_child(val_box_id, hist_val);

        let hist_sub = tree.create_node();
        if let Some(node) = tree.get_mut(hist_sub) {
            node.set_name("HistSub");
            node.set_text("Maximum number of actions stored in RAM. Lower values prevent memory bloat during extremely long sessions.");
            node.font_size = 10.5;
            node.line_height = 14.0;
            node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + 14.0,
                hist_sl_y + 24.0,
                ctx.content_w - 28.0,
                14.0,
            );
        }
        let _ = tree.add_child(hist_card_id, hist_sub);

        targets.sliders.push((
            PreferencesSliderId::UndoHistoryLimit,
            Rect::new(hist_track_rect.x, hist_sl_y, track_w, 20.0),
            10.0,
            5000.0,
            cfg.max_undo_history as f32,
        ));

        targets.number_inputs.push((
            PreferencesSliderId::UndoHistoryLimit,
            val_box_rect,
            10.0,
            5000.0,
            cfg.max_undo_history as f32,
        ));
    }

    hist_h
}