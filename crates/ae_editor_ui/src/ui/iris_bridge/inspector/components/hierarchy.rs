// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchy & Parenting Inspector Card
//!
//! Provides inspection and management for entity parent-child relationships.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use super::physics::render_component_header;
use irisui::prelude::*;

/// Inspector handler for entity hierarchy and parenting relationships.
pub struct ParentHandler;

impl ComponentInspectorHandler for ParentHandler {
    fn component_name(&self) -> &'static str {
        "Parenting"
    }

    fn display_title(&self) -> &'static str {
        "Parent / Hierarchy"
    }

    fn icon(&self) -> &'static str {
        "🔗"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.98, 0.80, 0.08, 1.0) // Gold / Warm Amber (#facc15)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Hierarchy
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Parent>(entity).is_ok()
            || world.get::<&ae_core::ecs::Children>(entity).is_ok()
    }

    fn can_remove(&self) -> bool {
        false
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let parent_entity = ctx
            .world
            .get::<&ae_core::ecs::Parent>(ctx.entity)
            .ok()
            .map(|p| p.0);

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + row_h + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ParentingCard");
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

        let cur_y = ctx.base_y + padding + 24.0 + 4.0;

        if let Some(parent) = parent_entity {
            let parent_name = ctx
                .world
                .get::<&ae_core::ecs::Name>(parent)
                .map(|n| n.0.clone())
                .unwrap_or_else(|_| format!("Entity {:?}", parent));

            let btn_w = 80.0;
            let lbl_w = ctx.card_w - padding * 2.0 - btn_w - 6.0;

            let lbl_id = tree.create_node();
            if let Some(node) = tree.get_mut(lbl_id) {
                node.set_name("ParentLbl");
                node.set_text(format!("Parent: {}", parent_name));
                node.font_size = 11.0;
                node.line_height = row_h;
                node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
                node.computed_rect = Rect::new(ctx.base_x + padding, cur_y, lbl_w, row_h);
            }
            let _ = tree.add_child(card_id, lbl_id);

            // Unparent Button
            let btn_rect = Rect::new(
                ctx.base_x + ctx.card_w - padding - btn_w,
                cur_y,
                btn_w,
                row_h,
            );
            let is_btn_hovered = btn_rect.contains_point(ctx.params.cursor_pos);
            let btn_id = tree.create_node();
            if let Some(node) = tree.get_mut(btn_id) {
                node.set_name("UnparentBtn");
                node.computed_rect = btn_rect;
                let (bg, border, txt_col) = if is_btn_hovered {
                    (
                        Color::rgba(0.35, 0.10, 0.10, 0.95),
                        Color::rgba(0.70, 0.18, 0.18, 0.85),
                        Color::rgba(1.0, 0.40, 0.40, 1.0),
                    )
                } else {
                    (
                        Color::rgba(0.157, 0.165, 0.188, 0.98),
                        Color::rgba(0.212, 0.220, 0.259, 0.85),
                        Color::rgba(0.82, 0.84, 0.88, 1.0),
                    )
                };
                node.style = Style::new()
                    .background(bg)
                    .border(1.0, border)
                    .border_radius(4.0);
                node.set_text("❌ Unparent");
                node.font_size = 10.0;
                node.line_height = row_h;
                node.text_align = TextAlign::Center;
                node.text_color = txt_col;
            }
            let _ = tree.add_child(card_id, btn_id);
            ctx.targets.unparent_btn_rect = Some(btn_rect);
        } else {
            let lbl_id = tree.create_node();
            if let Some(node) = tree.get_mut(lbl_id) {
                node.set_name("ParentRootLbl");
                node.set_text("Parent: None (Root Entity)");
                node.font_size = 11.0;
                node.line_height = row_h;
                node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
                node.computed_rect = Rect::new(
                    ctx.base_x + padding,
                    cur_y,
                    ctx.card_w - padding * 2.0,
                    row_h,
                );
            }
            let _ = tree.add_child(card_id, lbl_id);
        }

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::Children::default());
    }
}