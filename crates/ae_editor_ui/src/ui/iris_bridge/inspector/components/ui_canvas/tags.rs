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
use super::super::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};

use irisui::prelude::*;

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

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("PlayerHealthBarTagCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            card.label_styled_passive(
                "HealthBarTagDesc",
                "Links this UI progress bar to active player health events.",
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("ScoreDisplayTagCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            card.label_styled_passive(
                "ScoreDisplayTagDesc",
                "Links this UI text to active player score events.",
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("ReticleTagCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            card.label_styled_passive(
                "ReticleTagDesc",
                "Marks this UI element as the primary crosshair reticle.",
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::ReticleTag);
    }
}