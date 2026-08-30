// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Preferences Card
//!
//! Renders physics simulation frequency settings with discrete Hz snapping and numeric pill input.

use super::types::EditorCardContext;
use crate::ui::iris_bridge::preferences::types::{
    PHYSICS_HZ_PRESETS, PreferencesSliderId, PreferencesTargets,
};
use ae_editor::editor_state::EditorConfig;
use irisui::prelude::*;

/// Builds the Physics settings card and registers interactive targets.
pub fn build_physics_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    virtual_y: f32,
    ctx: EditorCardContext<'_>,
    cfg: &EditorConfig,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_phys_collapsed = ctx.collapsed_sections.contains("editor_physics");
    let phys_h = if is_phys_collapsed { 36.0 } else { 88.0 };

    let phys_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(phys_card_id) {
        node.set_name("PhysicsCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_y + virtual_y - ctx.scroll_y,
            ctx.content_w,
            phys_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, phys_card_id);

    let phys_header_rect = Rect::new(
        ctx.base_x + 8.0,
        ctx.content_y + virtual_y - ctx.scroll_y + 6.0,
        ctx.content_w - 16.0,
        24.0,
    );
    targets
        .section_toggles
        .push(("editor_physics", phys_header_rect));
    let is_phys_hdr_hovered = phys_header_rect.contains_point(ctx.cursor_pos);

    let phys_title = tree.create_node();
    if let Some(node) = tree.get_mut(phys_title) {
        node.set_name("PhysTitle");
        let arrow = if is_phys_collapsed { "▸" } else { "▾" };
        node.set_text(format!("{} 🎮  Physics Settings", arrow));
        node.font_size = 13.0;
        node.line_height = 24.0;
        node.text_color = if is_phys_hdr_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.91, 0.96, 1.0)
        };
        node.computed_rect = phys_header_rect;
    }
    let _ = tree.add_child(phys_card_id, phys_title);

    let track_w = ctx.content_w - 28.0 - 170.0 - ctx.val_box_w - 16.0;
    if !is_phys_collapsed {
        let phys_sl_y = ctx.content_y + virtual_y - ctx.scroll_y + 36.0;
        let phys_lbl = tree.create_node();
        if let Some(node) = tree.get_mut(phys_lbl) {
            node.set_name("PhysLabel");
            node.set_text("Fixed Update Frequency:");
            node.font_size = 11.5;
            node.line_height = 20.0;
            node.text_color = Color::rgba(0.75, 0.78, 0.85, 1.0);
            node.computed_rect = Rect::new(ctx.base_x + 14.0, phys_sl_y, 170.0, 20.0);
        }
        let _ = tree.add_child(phys_card_id, phys_lbl);

        let phys_track_rect = Rect::new(ctx.base_x + 14.0 + 170.0, phys_sl_y + 8.0, track_w, 4.0);
        let is_phys_track_hovered =
            Rect::new(phys_track_rect.x, phys_sl_y, track_w, 20.0).contains_point(ctx.cursor_pos);
        let phys_track = tree.create_node();
        if let Some(node) = tree.get_mut(phys_track) {
            node.set_name("PhysTrack");
            node.computed_rect = phys_track_rect;
            node.style = Style::new()
                .background(Color::rgba(0.08, 0.09, 0.12, 0.90))
                .border(1.0, Color::rgba(0.18, 0.21, 0.30, 0.50))
                .border_radius(2.0);
        }
        let _ = tree.add_child(phys_card_id, phys_track);

        let snapped_hz = PHYSICS_HZ_PRESETS
            .iter()
            .copied()
            .min_by(|a, b| {
                (a - cfg.physics_hz)
                    .abs()
                    .total_cmp(&(b - cfg.physics_hz).abs())
            })
            .unwrap_or(cfg.physics_hz);
        let phys_norm = ((snapped_hz - 30.0) / (240.0 - 30.0)).clamp(0.0, 1.0);
        let phys_fill = tree.create_node();
        if let Some(node) = tree.get_mut(phys_fill) {
            node.set_name("PhysFill");
            node.computed_rect = Rect::new(
                phys_track_rect.x,
                phys_track_rect.y,
                (track_w * phys_norm).max(2.0),
                4.0,
            );
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.72, 0.88, 0.95))
                .border_radius(2.0);
        }
        let _ = tree.add_child(phys_track, phys_fill);

        let phys_thumb = tree.create_node();
        if let Some(node) = tree.get_mut(phys_thumb) {
            node.set_name("PhysThumb");
            node.computed_rect = Rect::new(
                phys_track_rect.x + track_w * phys_norm - 4.0,
                phys_sl_y + 3.0,
                8.0,
                14.0,
            );
            node.style = Style::new()
                .background(if is_phys_track_hovered {
                    Color::rgba(0.0, 0.95, 1.0, 1.0)
                } else {
                    Color::rgba(0.88, 0.92, 0.98, 1.0)
                })
                .border_radius(2.0);
        }
        let _ = tree.add_child(phys_card_id, phys_thumb);

        // Modern Sleek Number Input Pill Box
        let (is_editing, editing_buf) = match ctx.active_number_input {
            Some((PreferencesSliderId::PhysicsFrequency, buf)) => (true, buf),
            _ => (false, ""),
        };
        let val_box_rect = Rect::new(
            ctx.base_x + 14.0 + 170.0 + track_w + 8.0,
            phys_sl_y - 1.0,
            ctx.val_box_w,
            ctx.val_box_h,
        );
        let is_box_hovered = val_box_rect.contains_point(ctx.cursor_pos);

        let val_box_id = tree.create_node();
        if let Some(node) = tree.get_mut(val_box_id) {
            node.set_name("PhysValBox");
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
        let _ = tree.add_child(phys_card_id, val_box_id);

        let phys_val = tree.create_node();
        if let Some(node) = tree.get_mut(phys_val) {
            node.set_name("PhysVal");
            if is_editing {
                let cursor_str = if ctx.blink_caret { "|" } else { "" };
                node.set_text(format!("{}{}", editing_buf, cursor_str));
                node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
            } else {
                node.set_text(format!("{:.0} Hz", snapped_hz));
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
        let _ = tree.add_child(val_box_id, phys_val);

        let phys_sub = tree.create_node();
        if let Some(node) = tree.get_mut(phys_sub) {
            node.set_name("PhysSub");
            node.set_text("Physics simulation frequency. Higher values improve simulation accuracy but increase CPU usage.");
            node.font_size = 10.5;
            node.line_height = 14.0;
            node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + 14.0,
                phys_sl_y + 24.0,
                ctx.content_w - 28.0,
                14.0,
            );
        }
        let _ = tree.add_child(phys_card_id, phys_sub);

        targets.sliders.push((
            PreferencesSliderId::PhysicsFrequency,
            Rect::new(phys_track_rect.x, phys_sl_y, track_w, 20.0),
            30.0,
            240.0,
            snapped_hz,
        ));

        targets.number_inputs.push((
            PreferencesSliderId::PhysicsFrequency,
            val_box_rect,
            30.0,
            240.0,
            snapped_hz,
        ));
    }

    phys_h
}