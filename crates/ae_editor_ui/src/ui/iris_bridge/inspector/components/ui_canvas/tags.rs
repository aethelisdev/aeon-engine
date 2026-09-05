// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI HUD Marker Tags Inspector Cards
//!
//! Provides handlers for HUD marker components linking UI elements to gameplay events:
//! - `PlayerHealthBarTag`
//! - `ScoreDisplayTag`
//! - `ReticleTag`

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::ComponentCategory;
use super::super::physics::render_component_header;
use irisui::prelude::*;

/// Inspector handler for PlayerHealthBarTag HUD marker component.
pub struct PlayerHealthBarTagHandler;

impl ComponentInspectorHandler for PlayerHealthBarTagHandler {
    fn component_name(&self) -> &'static str {
        "PlayerHealthBarTag"
    }

    fn display_title(&self) -> &'static str {
        "Health Bar Tag"
    }

    fn icon(&self) -> &'static str {
        "❤️"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.95, 0.25, 0.35, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world
            .get::<&ae_core::ecs::PlayerHealthBarTag>(entity)
            .is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let card_h = 24.0 + 20.0 + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("PlayerHealthBarTagCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
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

        let desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(desc_id) {
            node.set_name("HealthBarTagDesc");
            node.set_text("Links this UI progress bar to active player health events.");
            node.font_size = 10.5;
            node.line_height = 18.0;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                ctx.base_y + padding + 24.0 + 4.0,
                ctx.card_w - padding * 2.0,
                20.0,
            );
        }
        let _ = tree.add_child(card_id, desc_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::PlayerHealthBarTag);
    }
}

/// Inspector handler for `🏆 ScoreDisplayTag` HUD marker component.
pub struct ScoreDisplayTagHandler;

impl ComponentInspectorHandler for ScoreDisplayTagHandler {
    fn component_name(&self) -> &'static str {
        "ScoreDisplayTag"
    }

    fn display_title(&self) -> &'static str {
        "Score Display Tag"
    }

    fn icon(&self) -> &'static str {
        "🏆"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.98, 0.80, 0.15, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::ScoreDisplayTag>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let card_h = 24.0 + 20.0 + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ScoreDisplayTagCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
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

        let desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(desc_id) {
            node.set_name("ScoreDisplayTagDesc");
            node.set_text("Links this UI text to active player score events.");
            node.font_size = 10.5;
            node.line_height = 18.0;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                ctx.base_y + padding + 24.0 + 4.0,
                ctx.card_w - padding * 2.0,
                20.0,
            );
        }
        let _ = tree.add_child(card_id, desc_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::ScoreDisplayTag);
    }
}

/// Inspector handler for `🎯 ReticleTag` HUD marker component.
pub struct ReticleTagHandler;

impl ComponentInspectorHandler for ReticleTagHandler {
    fn component_name(&self) -> &'static str {
        "ReticleTag"
    }

    fn display_title(&self) -> &'static str {
        "Reticle Tag"
    }

    fn icon(&self) -> &'static str {
        "🎯"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::ReticleTag>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let card_h = 24.0 + 20.0 + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ReticleTagCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
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

        let desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(desc_id) {
            node.set_name("ReticleTagDesc");
            node.set_text("Marks this UI element as the primary crosshair reticle.");
            node.font_size = 10.5;
            node.line_height = 18.0;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                ctx.base_y + padding + 24.0 + 4.0,
                ctx.card_w - padding * 2.0,
                20.0,
            );
        }
        let _ = tree.add_child(card_id, desc_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::ReticleTag);
    }
}