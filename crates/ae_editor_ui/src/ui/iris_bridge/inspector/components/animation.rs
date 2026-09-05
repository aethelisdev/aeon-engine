// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Skeletal Animation Inspector Card
//!
//! Provides inspection and playback feedback for AnimationPlayer components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use super::physics::render_component_header;
use irisui::prelude::*;

/// Inspector handler for AnimationPlayer component.
pub struct AnimationPlayerHandler;

impl ComponentInspectorHandler for AnimationPlayerHandler {
    fn component_name(&self) -> &'static str {
        "AnimationPlayer"
    }

    fn display_title(&self) -> &'static str {
        "Animation Player"
    }

    fn icon(&self) -> &'static str {
        "🎬"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.95, 0.45, 0.70, 1.0) // Vibrant Rose / Magenta
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Animation
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_animation::AnimationPlayer>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (state_text, state_col, clip_title, clip_duration, speed, looping) =
            if let Ok(player) = ctx.world.get::<&ae_animation::AnimationPlayer>(ctx.entity) {
                let (st_txt, st_col) = match player.state {
                    ae_animation::AnimationState::Playing => {
                        ("▶ PLAYING", Color::rgba(0.20, 0.85, 0.40, 1.0))
                    }
                    ae_animation::AnimationState::Paused => {
                        ("⏸ PAUSED", Color::rgba(0.95, 0.75, 0.15, 1.0))
                    }
                    ae_animation::AnimationState::Stopped => {
                        ("⏹ STOPPED", Color::rgba(0.60, 0.62, 0.68, 1.0))
                    }
                };
                let clip_name = player
                    .current_clip
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "No Clip Selected".to_string());
                let duration = player.current_clip.as_ref().map_or(0.0, |c| c.duration);

                (
                    st_txt,
                    st_col,
                    clip_name,
                    duration,
                    player.speed,
                    player.looping,
                )
            } else {
                (
                    "⏹ STOPPED",
                    Color::rgba(0.60, 0.62, 0.68, 1.0),
                    "None".to_string(),
                    0.0,
                    1.0,
                    true,
                )
            };

        let has_skeleton = ctx
            .world
            .get::<&ae_animation::Skeleton>(ctx.entity)
            .map(|s| s.joints.len())
            .ok();

        let padding = 8.0;
        let row_h = 22.0;
        let spacing = 4.0;
        let card_h = 24.0 + 4.0 * (row_h + spacing) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("AnimationPlayerCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let mut cur_y = ctx.base_y + padding + 24.0 + 4.0;

        // Row 1: Status Badge
        let lbl1_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl1_id) {
            node.set_name("AnimStatusLbl");
            node.set_text("Status:");
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(ctx.base_x + padding, cur_y, 60.0, row_h);
        }
        let _ = tree.add_child(card_id, lbl1_id);

        let val1_id = tree.create_node();
        if let Some(node) = tree.get_mut(val1_id) {
            node.set_name("AnimStatusVal");
            node.set_text(state_text);
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = state_col;
            node.computed_rect = Rect::new(
                ctx.base_x + padding + 62.0,
                cur_y,
                ctx.card_w - padding * 2.0 - 62.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, val1_id);
        cur_y += row_h + spacing;

        // Row 2: Active Clip
        let lbl2_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl2_id) {
            node.set_name("AnimClipLbl");
            node.set_text(format!("Clip: {}", clip_title));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl2_id);
        cur_y += row_h + spacing;

        // Row 3: Skeleton Info
        let info_text = if let Some(joints) = has_skeleton {
            format!("🦴 Joints: {} | ⏱ Duration: {:.2}s", joints, clip_duration)
        } else {
            "ℹ Static 3D Mesh (No Armature found)".to_string()
        };
        let lbl3_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl3_id) {
            node.set_name("AnimSkeletonInfo");
            node.set_text(info_text);
            node.font_size = 10.0;
            node.line_height = row_h;
            node.text_color = if has_skeleton.is_some() {
                Color::rgba(0.38, 0.74, 0.97, 1.0)
            } else {
                Color::rgba(0.95, 0.75, 0.15, 0.90)
            };
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl3_id);
        cur_y += row_h + spacing;

        // Row 4: Speed & Looping Indicator
        let lbl4_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl4_id) {
            node.set_name("AnimSpeedLoop");
            let loop_str = if looping { "Loop: Yes" } else { "Loop: No" };
            node.set_text(format!("Speed: {:.2}x  |  {}", speed, loop_str));
            node.font_size = 10.5;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl4_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_animation::AnimationPlayer::default());
    }
}