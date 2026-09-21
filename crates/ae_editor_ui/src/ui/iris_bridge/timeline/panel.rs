// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Root Panel Builder
//!
//! Orchestrates panel container instantiation, hardware-accelerated clipping,
//! empty-state placeholder cards, and child transport and ruler subsystems.
//!

use super::ruler::build_ruler_and_scrubber;
use super::transport::{TRANSPORT_TOOLBAR_HEIGHT, build_transport_toolbar};
use super::types::{TimelinePanelParams, TimelinePanelTargets};
use irisui::prelude::*;

/// Builds the complete Animation Timeline Studio docked panel.
pub fn build_timeline_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    targets: &mut TimelinePanelTargets,
) {
    targets.panel_rect = params.panel_rect;
    targets.play_pause_btn = None;
    targets.stop_btn = None;
    targets.step_back_btn = None;
    targets.step_fwd_btn = None;
    targets.loop_toggle = None;
    targets.speed_buttons.clear();
    targets.scrubber_track_rect = None;
    targets.playhead_needle_rect = None;
    targets.add_player_btn = None;
    targets.clip_duration = 0.0;

    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("AnimationTimelinePanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.09, 0.98))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.60))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, root_id);

    // ── Case A: No Entity Selected ──
    let Some(entity) = params.entity else {
        render_empty_placeholder(
            tree,
            root_id,
            params.panel_rect,
            "🎬",
            "Animation Timeline Studio",
            "No entity selected. Select an animated 3D model in the viewport or hierarchy.",
        );
        return;
    };

    // ── Case B: Selected Entity Missing AnimationPlayer Component ──
    let Some(player) = params.animation_player else {
        render_missing_player_card(
            tree,
            root_id,
            params.panel_rect,
            params.cursor_pos,
            targets,
            entity,
        );
        return;
    };

    // ── Case C: Active AnimationPlayer Present ──
    let duration = player
        .current_clip
        .as_ref()
        .map_or(1.0, |c| c.duration.max(0.1));

    build_transport_toolbar(tree, root_id, params, targets, duration);

    build_ruler_and_scrubber(
        tree,
        root_id,
        params,
        targets,
        params.panel_rect.y + TRANSPORT_TOOLBAR_HEIGHT,
        duration,
    );
}

/// Renders a centered empty-state information card when no animated entity is selected.
fn render_empty_placeholder(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    icon: &str,
    title: &str,
    subtitle: &str,
) {
    let card_w = 460.0_f32.min(panel_rect.width - 40.0).max(280.0);
    let card_h = 70.0;
    let card_x = panel_rect.x + (panel_rect.width - card_w) * 0.5;
    let card_y = panel_rect.y + (panel_rect.height - card_h) * 0.5;
    let card_rect = Rect::new(card_x, card_y, card_w, card_h);

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("TimelineEmptyCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.11, 0.15, 0.85))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.20, 0.24, 0.32, 0.50));
    }
    let _ = tree.add_child(parent_id, card_id);

    // Title line with icon
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("TimelineEmptyTitle");
        node.set_text(format!("{}  {}", icon, title));
        node.font_size = 13.0;
        node.line_height = 20.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.70, 0.75, 0.85, 1.0);
        node.computed_rect = Rect::new(card_x + 10.0, card_y + 12.0, card_w - 20.0, 20.0);
    }
    let _ = tree.add_child(card_id, title_id);

    // Subtitle description line
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("TimelineEmptySubtitle");
        node.set_text(subtitle);
        node.font_size = 10.5;
        node.line_height = 16.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.48, 0.52, 0.62, 1.0);
        node.computed_rect = Rect::new(card_x + 10.0, card_y + 36.0, card_w - 20.0, 18.0);
    }
    let _ = tree.add_child(card_id, sub_id);
}

/// Renders an informational card with an action button when an entity is selected without AnimationPlayer.
fn render_missing_player_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    cursor_pos: Point,
    targets: &mut TimelinePanelTargets,
    _entity: hecs::Entity,
) {
    let card_w = 480.0_f32.min(panel_rect.width - 40.0).max(280.0);
    let card_h = 80.0;
    let card_x = panel_rect.x + (panel_rect.width - card_w) * 0.5;
    let card_y = panel_rect.y + (panel_rect.height - card_h) * 0.5;
    let card_rect = Rect::new(card_x, card_y, card_w, card_h);

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("TimelineMissingPlayerCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.11, 0.15, 0.85))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.25, 0.28, 0.38, 0.60));
    }
    let _ = tree.add_child(parent_id, card_id);

    // Title label
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("TimelineMissingPlayerTitle");
        node.set_text("Selected entity does not have an AnimationPlayer component.");
        node.font_size = 11.5;
        node.line_height = 18.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.72, 0.76, 0.86, 1.0);
        node.computed_rect = Rect::new(card_x + 10.0, card_y + 12.0, card_w - 20.0, 18.0);
    }
    let _ = tree.add_child(card_id, title_id);

    // Add AnimationPlayer action button
    let btn_w = 170.0;
    let btn_h = 24.0;
    let btn_x = card_x + (card_w - btn_w) * 0.5;
    let btn_y = card_y + 40.0;
    let btn_rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
    let is_btn_hovered = btn_rect.contains_point(cursor_pos);
    targets.add_player_btn = Some(btn_rect);

    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("TimelineAddPlayerBtn");
        node.set_text("➕ Add AnimationPlayer");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_btn_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.0, 0.88, 1.0, 1.0)
        };
        node.computed_rect = btn_rect;
        node.style = Style::new()
            .background(if is_btn_hovered {
                Color::rgba(0.0, 0.45, 0.60, 0.80)
            } else {
                Color::rgba(0.0, 0.30, 0.42, 0.60)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_btn_hovered {
                    Color::rgba(0.0, 0.90, 1.0, 0.90)
                } else {
                    Color::rgba(0.0, 0.75, 0.95, 0.60)
                },
            );
    }
    let _ = tree.add_child(card_id, btn_id);
}