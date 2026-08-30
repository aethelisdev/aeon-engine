// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Runtime Preferences Card
//!
//! Renders engine hot-reload and live editor runtime execution settings.

use super::types::EditorCardContext;
use crate::ui::iris_bridge::preferences::types::{PreferencesTargets, PreferencesToggleId};
use irisui::prelude::*;

/// Builds the Runtime settings card and registers interactive targets.
pub fn build_runtime_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    virtual_y: f32,
    ctx: EditorCardContext<'_>,
    enable_live_updates: bool,
    targets: &mut PreferencesTargets,
) -> f32 {
    let is_rt_collapsed = ctx.collapsed_sections.contains("editor_runtime");
    let rt_h = if is_rt_collapsed { 36.0 } else { 74.0 };

    let rt_card_id = tree.create_node();
    if let Some(node) = tree.get_mut(rt_card_id) {
        node.set_name("RuntimeCard");
        node.computed_rect = Rect::new(
            ctx.base_x,
            ctx.content_y + virtual_y - ctx.scroll_y,
            ctx.content_w,
            rt_h,
        );
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, rt_card_id);

    let rt_header_rect = Rect::new(
        ctx.base_x + 8.0,
        ctx.content_y + virtual_y - ctx.scroll_y + 6.0,
        ctx.content_w - 16.0,
        24.0,
    );
    targets
        .section_toggles
        .push(("editor_runtime", rt_header_rect));
    let is_rt_hdr_hovered = rt_header_rect.contains_point(ctx.cursor_pos);

    let rt_title = tree.create_node();
    if let Some(node) = tree.get_mut(rt_title) {
        node.set_name("RtTitle");
        let arrow = if is_rt_collapsed { "▸" } else { "▾" };
        node.set_text(format!("{} ⚙  Runtime Settings", arrow));
        node.font_size = 13.0;
        node.line_height = 24.0;
        node.text_color = if is_rt_hdr_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.88, 0.91, 0.96, 1.0)
        };
        node.computed_rect = rt_header_rect;
    }
    let _ = tree.add_child(rt_card_id, rt_title);

    if !is_rt_collapsed {
        let rt_cb_rect = Rect::new(
            ctx.base_x + 14.0,
            ctx.content_y + virtual_y - ctx.scroll_y + 34.0,
            ctx.content_w - 28.0,
            18.0,
        );
        let is_rt_checked = enable_live_updates;
        let is_rt_hovered = rt_cb_rect.contains_point(ctx.cursor_pos);

        let rt_box = tree.create_node();
        if let Some(node) = tree.get_mut(rt_box) {
            node.set_name("RtBox");
            node.computed_rect = Rect::new(rt_cb_rect.x, rt_cb_rect.y, 16.0, 16.0);
            let bg = if is_rt_checked {
                Color::rgba(0.0, 0.70, 0.85, 1.0)
            } else if is_rt_hovered {
                Color::rgba(0.18, 0.20, 0.28, 1.0)
            } else {
                Color::rgba(0.11, 0.12, 0.16, 1.0)
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, Color::rgba(0.25, 0.30, 0.42, 1.0))
                .border_radius(3.0);
        }
        let _ = tree.add_child(rt_card_id, rt_box);

        if is_rt_checked {
            let chk = tree.create_node();
            if let Some(node) = tree.get_mut(chk) {
                node.set_name("RtCheck");
                node.set_text("✓");
                node.font_size = 11.0;
                node.line_height = 16.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.05, 0.06, 0.08, 1.0);
                node.computed_rect = Rect::new(rt_cb_rect.x, rt_cb_rect.y, 16.0, 16.0);
            }
            let _ = tree.add_child(rt_box, chk);
        }

        let rt_lbl = tree.create_node();
        if let Some(node) = tree.get_mut(rt_lbl) {
            node.set_name("RtLabel");
            node.set_text("Enable Live Editor Updates (Hot Reload)");
            node.font_size = 12.0;
            node.line_height = 16.0;
            node.text_color = if is_rt_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.80, 0.83, 0.90, 1.0)
            };
            node.computed_rect = Rect::new(
                rt_cb_rect.x + 24.0,
                rt_cb_rect.y,
                ctx.content_w - 52.0,
                16.0,
            );
        }
        let _ = tree.add_child(rt_card_id, rt_lbl);

        let rt_sub = tree.create_node();
        if let Some(node) = tree.get_mut(rt_sub) {
            node.set_name("RtSub");
            node.set_text("Disables hot reload. Useful when debugging core engine systems.");
            node.font_size = 10.5;
            node.line_height = 14.0;
            node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
            node.computed_rect = Rect::new(
                rt_cb_rect.x + 24.0,
                rt_cb_rect.y + 18.0,
                ctx.content_w - 52.0,
                14.0,
            );
        }
        let _ = tree.add_child(rt_card_id, rt_sub);

        targets
            .toggles
            .push((PreferencesToggleId::LiveUpdatesEnabled, rt_cb_rect));
    }

    rt_h
}