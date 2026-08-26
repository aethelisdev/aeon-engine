// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handlers for In-Game HUD Gameplay Tags (`PlayerHealthBarTag`, `ScoreDisplayTag`, `ReticleTag`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use ae_core::ecs::{PlayerHealthBarTag, ReticleTag, ScoreDisplayTag};

/// UI Handler for `PlayerHealthBarTag` HUD binding tag.
pub struct PlayerHealthBarTagUiHandler;

impl ComponentUiHandler for PlayerHealthBarTagUiHandler {
    fn component_name(&self) -> &'static str {
        "PlayerHealthBarTag"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Player Health Bar Tag",
            "❤️",
            egui::Color32::from_rgb(220, 60, 80),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Player Health Bar Tag")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&PlayerHealthBarTag>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, _ctx: &mut InspectorContext) {
        ui.label("Binds this progress bar to receive Player Damage and Heal events automatically.");
    }
}

/// UI Handler for `ScoreDisplayTag` HUD binding tag.
pub struct ScoreDisplayTagUiHandler;

impl ComponentUiHandler for ScoreDisplayTagUiHandler {
    fn component_name(&self) -> &'static str {
        "ScoreDisplayTag"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Score Display Tag",
            "⭐",
            egui::Color32::from_rgb(240, 190, 40),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Score Display Tag")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ScoreDisplayTag>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, _ctx: &mut InspectorContext) {
        ui.label("Binds this text label to receive gameplay ScoreEvent score increments.");
    }
}

/// UI Handler for `ReticleTag` HUD binding tag.
pub struct ReticleTagUiHandler;

impl ComponentUiHandler for ReticleTagUiHandler {
    fn component_name(&self) -> &'static str {
        "ReticleTag"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Crosshair Reticle Tag",
            "🎯",
            egui::Color32::from_rgb(80, 200, 240),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Crosshair Reticle Tag")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ReticleTag>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, _ctx: &mut InspectorContext) {
        ui.label("Binds this element as the center-screen crosshair reticle.");
    }
}