// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI HUD Marker Tags Inspector Cards
//!
//! Provides declarative inspector cards and handlers for HUD marker components linking UI elements to gameplay events:
//! - `PlayerHealthBarTag`: Binds a UI progress bar to the player's current health pool
//! - `ScoreDisplayTag`: Binds a UI text element to the current game score readout
//! - `ReticleTag`: Marks a UI element as the primary HUD crosshair reticle

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::ComponentCategory;
use irisui::prelude::*;

/// Renders the declarative PlayerHealthBarTag HUD marker component card into [`UiScope`] if present on [`hecs::Entity`].
pub fn draw_player_health_bar_tag_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "PlayerHealthBarTagCard",
        UiScope::comp_card_desc(
            "PlayerHealthBarTag",
            "Health Bar Tag",
            "❤️",
            Color::rgba(0.95, 0.25, 0.35, 1.0),
        ),
        world.get::<&ae_core::ecs::PlayerHealthBarTag>(entity),
        |card, _tag| {
            card.label_styled_passive(
                "HealthBarTagDesc",
                "Links this UI progress bar to active player health events.",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        },
    );
}

/// Inspector handler for `PlayerHealthBarTag` HUD marker component.
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        draw_player_health_bar_tag_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::PlayerHealthBarTag));
    }
}

/// Renders the declarative ScoreDisplayTag HUD marker component card into [`UiScope`] if present on [`hecs::Entity`].
pub fn draw_score_display_tag_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "ScoreDisplayTagCard",
        UiScope::comp_card_desc(
            "ScoreDisplayTag",
            "Score Display Tag",
            "🏆",
            Color::rgba(0.98, 0.80, 0.15, 1.0),
        ),
        world.get::<&ae_core::ecs::ScoreDisplayTag>(entity),
        |card, _tag| {
            card.label_styled_passive(
                "ScoreDisplayTagDesc",
                "Links this UI text to active player score events.",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        },
    );
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        draw_score_display_tag_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::ScoreDisplayTag));
    }
}

/// Renders the declarative ReticleTag HUD marker component card into [`UiScope`] if present on [`hecs::Entity`].
pub fn draw_reticle_tag_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "ReticleTagCard",
        UiScope::comp_card_desc(
            "ReticleTag",
            "Reticle Tag",
            "🎯",
            Color::rgba(0.20, 0.85, 1.0, 1.0),
        ),
        world.get::<&ae_core::ecs::ReticleTag>(entity),
        |card, _tag| {
            card.label_styled_passive(
                "ReticleTagDesc",
                "Marks this UI element as the primary crosshair reticle.",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        },
    );
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        draw_reticle_tag_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::ReticleTag));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_player_health_bar_tag_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PlayerHealthBarTag,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_player_health_bar_tag_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("PlayerHealthBarTagCard")),
            "PlayerHealthBarTagCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("HealthBarTagDesc")),
            "HealthBarTagDesc label must be rendered"
        );
    }

    #[test]
    fn test_draw_score_display_tag_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::ScoreDisplayTag,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_score_display_tag_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ScoreDisplayTagCard")),
            "ScoreDisplayTagCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ScoreDisplayTagDesc")),
            "ScoreDisplayTagDesc label must be rendered"
        );
    }

    #[test]
    fn test_draw_reticle_tag_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::ReticleTag,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_reticle_tag_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ReticleTagCard")),
            "ReticleTagCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ReticleTagDesc")),
            "ReticleTagDesc label must be rendered"
        );
    }
}