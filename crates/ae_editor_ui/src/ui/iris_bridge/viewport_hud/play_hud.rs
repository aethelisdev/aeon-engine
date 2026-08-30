// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Play Mode Viewport HUD Overlay Builder
//!
//! Renders the centered aiming crosshair reticle and bottom-left quick controls reminder badge
//! when the engine is actively executing in Play Mode.

use super::types::ViewportHudParams;
use irisui::prelude::*;

/// Builds the Play Mode HUD overlay (reticle and quick control badge).
pub fn build_play_hud(tree: &mut UiTree, parent_id: WidgetId, params: &ViewportHudParams<'_>) {
    let center_x = params.viewport_rect.x + params.viewport_rect.width * 0.5;
    let center_y = params.viewport_rect.y + params.viewport_rect.height * 0.5;

    // Check whether the active player character has `CharacterAction` (aiming/shooting ability)
    let has_action = if let Some(sel) = params.selected_entity
        && (params.world.get::<&ae_core::ecs::PlayerTag>(sel).is_ok()
            || params
                .world
                .get::<&ae_core::ecs::CharacterController>(sel)
                .is_ok())
    {
        params
            .world
            .get::<&ae_core::ecs::CharacterAction>(sel)
            .is_ok()
    } else {
        params
            .world
            .query::<(&ae_core::ecs::PlayerTag, &ae_core::ecs::CharacterAction)>()
            .iter()
            .next()
            .is_some()
            || params
                .world
                .query::<(
                    &ae_core::ecs::CharacterController,
                    &ae_core::ecs::CharacterAction,
                )>()
                .iter()
                .next()
                .is_some()
    };

    // 1. Center Crosshair Reticle (Only rendered when character has CharacterAction capability)
    if has_action {
        let dot_size = 4.0;
        let dot_rect = Rect::new(
            center_x - dot_size * 0.5,
            center_y - dot_size * 0.5,
            dot_size,
            dot_size,
        );
        let dot_id = tree.create_node();
        if let Some(node) = tree.get_mut(dot_id) {
            node.set_name("PlayReticleCenter");
            node.computed_rect = dot_rect;
            node.style = Style::new()
                .background(Color::rgba(1.0, 1.0, 1.0, 0.90))
                .border(1.0, Color::rgba(0.0, 0.0, 0.0, 0.60))
                .border_radius(dot_size * 0.5);
        }
        let _ = tree.add_child(parent_id, dot_id);

        // Reticle Crosshairs (North, South, East, West)
        let len = 8.0;
        let gap = 4.0;
        let thick = 2.0;

        let crosshair_lines = [
            // North
            Rect::new(center_x - thick * 0.5, center_y - gap - len, thick, len),
            // South
            Rect::new(center_x - thick * 0.5, center_y + gap, thick, len),
            // West
            Rect::new(center_x - gap - len, center_y - thick * 0.5, len, thick),
            // East
            Rect::new(center_x + gap, center_y - thick * 0.5, len, thick),
        ];

        for line_rect in crosshair_lines {
            let line_id = tree.create_node();
            if let Some(node) = tree.get_mut(line_id) {
                node.set_name("PlayReticleLine");
                node.computed_rect = line_rect;
                node.style = Style::new()
                    .background(Color::rgba(1.0, 1.0, 1.0, 0.85))
                    .border(0.5, Color::rgba(0.0, 0.0, 0.0, 0.50))
                    .border_radius(1.0);
            }
            let _ = tree.add_child(parent_id, line_id);
        }
    }

    // 2. Bottom-left quick controls badge
    let guide_text = if has_action {
        "🔫 Left Click: Shoot   |   🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    } else {
        "🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    };

    let badge_w = if has_action { 420.0 } else { 290.0 };
    let badge_h = 24.0;
    let badge_x = params.viewport_rect.x + 16.0;
    let badge_y = params.viewport_rect.y + params.viewport_rect.height - badge_h - 16.0;
    let badge_rect = Rect::new(badge_x, badge_y, badge_w, badge_h);

    let badge_id = tree.create_node();
    if let Some(node) = tree.get_mut(badge_id) {
        node.set_name("PlayGuideBadge");
        node.computed_rect = badge_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.10, 0.85))
            .border(1.0, Color::rgba(0.25, 0.30, 0.42, 0.60))
            .border_radius(5.0)
            .box_shadow(0.0, 4.0, 12.0, Color::rgba(0.0, 0.0, 0.0, 0.60));
    }
    let _ = tree.add_child(parent_id, badge_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name("PlayGuideText");
        node.set_text(guide_text);
        node.font_size = 11.0;
        node.line_height = badge_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.90, 0.93, 0.98, 1.0);
        node.computed_rect = badge_rect;
    }
    let _ = tree.add_child(badge_id, txt_id);
}