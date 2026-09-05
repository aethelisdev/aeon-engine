// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Viewport Billboard Icons Builder
//!
//! Projects 3D world-space entity locations (Light, Audio, Camera) onto 2D viewport coordinates
//! and renders interactive billboard icon badges with selection highlights.

use super::types::{ViewportHudParams, ViewportHudTargets};
use crate::ui::iris_bridge::icons::{ICON_AUDIO, ICON_CAMERA, ICON_LIGHT};
use hecs::Entity;
use irisui::prelude::*;

/// Builds 3D projected billboard icon badges across the viewport.
pub fn build_billboard_icons(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let vp_matrix = params.camera.build_view_projection_matrix();
    let vp_w = params.viewport_rect.width;
    let vp_h = params.viewport_rect.height;

    for (ent, pos) in params
        .world
        .query::<(Entity, &ae_core::ecs::Position)>()
        .iter()
    {
        // Hide billboard icons for hidden entities unless actively selected in the editor
        if params.world.get::<&ae_core::ecs::Hidden>(ent).is_ok()
            && params.selected_entity != Some(ent)
        {
            continue;
        }

        let is_light = params.world.get::<&ae_core::ecs::Light>(ent).is_ok();
        let is_audio_source = params.world.get::<&ae_audio::AudioSource>(ent).is_ok();
        let is_audio_listener = params.world.get::<&ae_audio::AudioListener>(ent).is_ok();

        if !is_light && !is_audio_source && !is_audio_listener {
            continue;
        }

        let icon_uv = if is_light {
            ICON_LIGHT
        } else if is_audio_source {
            ICON_AUDIO
        } else {
            ICON_CAMERA
        };

        // Project 3D position to Clip Space
        let pos_v4 = cgmath::Vector4::new(pos.x, pos.y, pos.z, 1.0);
        let clip_v4 = vp_matrix * pos_v4;

        if clip_v4.w <= 0.001 {
            continue; // Behind near plane
        }

        let ndc_x = clip_v4.x / clip_v4.w;
        let ndc_y = clip_v4.y / clip_v4.w;

        if !(-1.2..=1.2).contains(&ndc_x) || !(-1.2..=1.2).contains(&ndc_y) {
            continue; // Outside viewport bounds
        }

        let screen_x = params.viewport_rect.x + (ndc_x + 1.0) * 0.5 * vp_w;
        let screen_y = params.viewport_rect.y + (1.0 - ndc_y) * 0.5 * vp_h;

        let icon_size = 24.0;
        let icon_rect = Rect::new(
            screen_x - icon_size * 0.5,
            screen_y - icon_size * 0.5,
            icon_size,
            icon_size,
        );

        let is_selected = params.selected_entity == Some(ent);
        let is_hover = icon_rect.contains_point(params.cursor_pos);

        let icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(icon_id) {
            node.set_name("BillboardIcon");
            node.computed_rect = icon_rect;
            let (bg, border) = if is_selected {
                (
                    Color::rgba(0.70, 0.45, 0.08, 0.90),
                    Color::rgba(1.0, 0.85, 0.20, 1.0),
                )
            } else if is_hover {
                (
                    Color::rgba(0.25, 0.28, 0.38, 0.90),
                    Color::rgba(0.0, 0.85, 1.0, 0.90),
                )
            } else {
                (
                    Color::rgba(0.08, 0.09, 0.13, 0.80),
                    Color::rgba(0.30, 0.35, 0.48, 0.60),
                )
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, border)
                .border_radius(icon_size * 0.5)
                .box_shadow(0.0, 2.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 0.60));
        }
        let _ = tree.add_child(parent_id, icon_id);

        let inner_size = 15.0;
        let inner_x = icon_rect.x + (icon_size - inner_size) * 0.5;
        let inner_y = icon_rect.y + (icon_size - inner_size) * 0.5;

        let badge_ic_id = tree.create_node();
        if let Some(node) = tree.get_mut(badge_ic_id) {
            node.set_name("BillboardBadgeIcon");
            node.computed_rect = Rect::new(inner_x, inner_y, inner_size, inner_size);
            node.set_texture_uv(icon_uv);
            let tint = if is_selected {
                Color::WHITE
            } else if is_hover {
                Color::rgba(0.0, 0.95, 1.0, 1.0)
            } else if is_light {
                Color::rgba(1.0, 0.88, 0.35, 1.0)
            } else if is_audio_source {
                Color::rgba(0.40, 0.75, 1.0, 1.0)
            } else {
                Color::rgba(0.85, 0.88, 0.95, 1.0)
            };
            node.set_texture_tint(tint);
        }
        let _ = tree.add_child(icon_id, badge_ic_id);

        targets.billboard_icons.push((ent, icon_rect));
    }
}