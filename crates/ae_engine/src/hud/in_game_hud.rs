// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular In-Game HUD Subsystem & Gameplay Event Bridge.
//!
//! Maintains HUD entities (Health Bar, Score Display, Reticle) in the ECS World
//! and dynamically reacts to `DamageEvent`, `HealEvent`, and `ScoreEvent`.
//!

use ae_core::events::DynamicEventBus;
use ae_core::ui::{UiAnchor, UiElement, UiProgressBar, UiText};
use hecs::{Entity, World};

/// Tag component for the in-game Player Health Bar HUD element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerHealthBarTag;

/// Tag component for the in-game Score Display HUD element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreDisplayTag;

/// State tracker for active in-game HUD session.
#[derive(Default)]
pub struct InGameHudState {
    pub current_score: i32,
    pub player_health: f32,
    pub player_max_health: f32,
    pub health_bar_entity: Option<Entity>,
    pub score_text_entity: Option<Entity>,
}

impl InGameHudState {
    /// Creates a new HUD state with full health and 0 score.
    pub fn new() -> Self {
        Self {
            current_score: 0,
            player_health: 100.0,
            player_max_health: 100.0,
            health_bar_entity: None,
            score_text_entity: None,
        }
    }

    /// Spawns default in-game HUD entities into the ECS World if they do not exist.
    pub fn ensure_hud_spawned(&mut self, world: &mut World) {
        // 1. Health Bar
        if self.health_bar_entity.is_none() || !world.contains(self.health_bar_entity.unwrap()) {
            let ent = world.spawn((
                PlayerHealthBarTag,
                UiElement {
                    anchor: UiAnchor::TopLeft,
                    offset: [120.0, 36.0],
                    size: [180.0, 16.0],
                    visible: true,
                    z_index: 10,
                },
                UiProgressBar {
                    min: 0.0,
                    max: self.player_max_health,
                    value: self.player_health,
                    fill_color: [0.2, 0.85, 0.35, 1.0], // Neon Green
                    background_color: [0.08, 0.10, 0.14, 0.85],
                    border_color: [0.3, 0.4, 0.5, 0.8],
                },
            ));
            self.health_bar_entity = Some(ent);
        }

        // 2. Score Counter Text
        if self.score_text_entity.is_none() || !world.contains(self.score_text_entity.unwrap()) {
            let ent = world.spawn((
                ScoreDisplayTag,
                UiElement {
                    anchor: UiAnchor::TopRight,
                    offset: [-100.0, 36.0],
                    size: [160.0, 24.0],
                    visible: true,
                    z_index: 10,
                },
                UiText::new(format!("SCORE: {:05}", self.current_score), 16.0)
                    .with_color([1.0, 0.85, 0.2, 1.0]),
            ));
            self.score_text_entity = Some(ent);
        }
    }

    /// Resets health and score counters and clears HUD entities from world.
    pub fn reset(&mut self, world: &mut World) {
        self.current_score = 0;
        self.player_health = self.player_max_health;
        if let Some(ent) = self.health_bar_entity.take() {
            let _ = world.despawn(ent);
        }
        if let Some(ent) = self.score_text_entity.take() {
            let _ = world.despawn(ent);
        }
    }

    /// Processes incoming gameplay events and updates corresponding HUD elements.
    pub fn update_from_events(&mut self, world: &mut World, event_bus: &mut DynamicEventBus) {
        self.ensure_hud_spawned(world);

        // 1. Process Score Events
        if let Some(events) = event_bus.receive::<ae_core::events::ScoreEvent>() {
            for ev in events {
                self.current_score = (self.current_score + ev.delta).max(0);
                if let Some(score_ent) = self.score_text_entity
                    && let Ok(mut text) = world.get::<&mut UiText>(score_ent)
                {
                    text.text = format!("SCORE: {:05}", self.current_score);
                }
            }
        }

        // 2. Process Damage Events - only apply to Player Health Bar when Player is damaged
        if let Some(events) = event_bus.receive::<ae_core::events::DamageEvent>() {
            for ev in events {
                let is_player_target = world.get::<&ae_core::ecs::PlayerTag>(ev.target).is_ok();
                if is_player_target
                    && let Some(bar_ent) = self.health_bar_entity
                    && let Ok(mut bar) = world.get::<&mut UiProgressBar>(bar_ent)
                {
                    self.player_health = (self.player_health - ev.amount).max(0.0);
                    bar.value = self.player_health;
                    // Color transitions: Green -> Yellow -> Red
                    let fraction = bar.fraction();
                    if fraction < 0.3 {
                        bar.fill_color = [0.9, 0.2, 0.2, 1.0]; // Red
                    } else if fraction < 0.6 {
                        bar.fill_color = [0.9, 0.75, 0.2, 1.0]; // Yellow
                    } else {
                        bar.fill_color = [0.2, 0.85, 0.35, 1.0]; // Green
                    }
                }
            }
        }

        // 3. Process Heal Events
        if let Some(events) = event_bus.receive::<ae_core::events::HealEvent>() {
            for ev in events {
                let is_player_target = world.get::<&ae_core::ecs::PlayerTag>(ev.target).is_ok();
                if is_player_target
                    && let Some(bar_ent) = self.health_bar_entity
                    && let Ok(mut bar) = world.get::<&mut UiProgressBar>(bar_ent)
                {
                    self.player_health =
                        (self.player_health + ev.amount).min(self.player_max_health);
                    bar.value = self.player_health;
                    let fraction = bar.fraction();
                    if fraction >= 0.6 {
                        bar.fill_color = [0.2, 0.85, 0.35, 1.0];
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    #[test]
    fn test_in_game_hud_spawning_and_event_reactions() {
        let mut world = World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut hud = InGameHudState::new();

        // 1. Spawning check
        hud.ensure_hud_spawned(&mut world);
        assert!(hud.health_bar_entity.is_some());
        assert!(hud.score_text_entity.is_some());

        // 2. Score Event
        event_bus.send(ae_core::events::ScoreEvent {
            delta: 250,
            new_total: 250,
        });
        hud.update_from_events(&mut world, &mut event_bus);
        assert_eq!(hud.current_score, 250);

        {
            let score_text = world
                .get::<&UiText>(hud.score_text_entity.unwrap())
                .unwrap();
            assert_eq!(score_text.text, "SCORE: 00250");
        }

        // 3. Damage Event on Player
        let player_ent = world.spawn((ae_core::ecs::PlayerTag,));
        event_bus.send(ae_core::events::DamageEvent {
            source: None,
            target: player_ent,
            amount: 50.0,
            hit_point: None,
            hit_normal: None,
        });

        hud.update_from_events(&mut world, &mut event_bus);
        assert_eq!(hud.player_health, 50.0);

        {
            let health_bar = world
                .get::<&UiProgressBar>(hud.health_bar_entity.unwrap())
                .unwrap();
            assert_eq!(health_bar.value, 50.0);
            assert_eq!(health_bar.fraction(), 0.5);
        }
    }
}