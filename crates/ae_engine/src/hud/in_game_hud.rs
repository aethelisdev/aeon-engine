// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular In-Game HUD Subsystem & Gameplay Event Bridge.
//!
//! Bridges gameplay events (`DamageEvent`, `HealEvent`, `ScoreEvent`) to user-defined
//! ECS UI entities tagged with `PlayerHealthBarTag` and `ScoreDisplayTag`.
//!

use ae_core::ecs::{PlayerHealthBarTag, ScoreDisplayTag, UiProgressBar, UiText};
use ae_core::events::DynamicEventBus;
use hecs::World;

/// State tracker for active in-game HUD session.
#[derive(Default)]
pub struct InGameHudState {
    pub current_score: i32,
    pub player_health: f32,
    pub player_max_health: f32,
}

impl InGameHudState {
    /// Creates a new HUD state with full health and 0 score.
    pub fn new() -> Self {
        Self {
            current_score: 0,
            player_health: 100.0,
            player_max_health: 100.0,
        }
    }

    /// Resets health and score counters.
    pub fn reset(&mut self, world: &mut World) {
        self.current_score = 0;
        self.player_health = self.player_max_health;

        // Reset any health bars in the scene back to full
        for (_, bar) in world.query_mut::<(&PlayerHealthBarTag, &mut UiProgressBar)>() {
            bar.value = bar.max;
        }
        // Reset any score text in the scene back to 0
        for (_, text) in world.query_mut::<(&ScoreDisplayTag, &mut UiText)>() {
            text.text = format!("SCORE: {:05}", self.current_score);
        }
    }

    /// Processes incoming gameplay events and updates corresponding user-created HUD elements.
    pub fn update_from_events(&mut self, world: &mut World, event_bus: &mut DynamicEventBus) {
        // 1. Process Score Events
        if let Some(events) = event_bus.receive::<ae_core::events::ScoreEvent>() {
            for ev in events {
                self.current_score = (self.current_score + ev.delta).max(0);
                for (_, text) in world.query_mut::<(&ScoreDisplayTag, &mut UiText)>() {
                    text.text = format!("SCORE: {:05}", self.current_score);
                }
            }
        }

        // 2. Process Damage Events - applies to Player Health Bar when Player is damaged
        if let Some(events) = event_bus.receive::<ae_core::events::DamageEvent>() {
            for ev in events {
                let is_player_target = world.get::<&ae_core::ecs::PlayerTag>(ev.target).is_ok();
                if is_player_target {
                    self.player_health = (self.player_health - ev.amount).max(0.0);
                    for (_, bar) in world.query_mut::<(&PlayerHealthBarTag, &mut UiProgressBar)>() {
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
        }

        // 3. Process Heal Events
        if let Some(events) = event_bus.receive::<ae_core::events::HealEvent>() {
            for ev in events {
                let is_player_target = world.get::<&ae_core::ecs::PlayerTag>(ev.target).is_ok();
                if is_player_target {
                    self.player_health =
                        (self.player_health + ev.amount).min(self.player_max_health);
                    for (_, bar) in world.query_mut::<(&PlayerHealthBarTag, &mut UiProgressBar)>() {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ui::{UiAnchor, UiElement};
    use hecs::World;

    #[test]
    fn test_in_game_hud_event_reactions() {
        let mut world = World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut hud = InGameHudState::new();

        // Spawn a user-created health bar and score display
        let player = world.spawn((ae_core::ecs::PlayerTag,));

        let health_bar_ent = world.spawn((
            PlayerHealthBarTag,
            UiElement::new(UiAnchor::TopLeft, [100.0, 30.0], [200.0, 20.0]),
            UiProgressBar {
                min: 0.0,
                max: 100.0,
                value: 100.0,
                ..Default::default()
            },
        ));

        let score_ent = world.spawn((
            ScoreDisplayTag,
            UiElement::new(UiAnchor::TopRight, [-100.0, 30.0], [150.0, 24.0]),
            UiText::new("SCORE: 00000", 16.0),
        ));

        // Emit damage event to player
        event_bus.send(ae_core::events::DamageEvent {
            source: None,
            target: player,
            amount: 40.0,
            hit_point: None,
            hit_normal: None,
        });

        // Emit score event
        event_bus.send(ae_core::events::ScoreEvent {
            delta: 250,
            new_total: 250,
        });

        hud.update_from_events(&mut world, &mut event_bus);

        assert_eq!(hud.player_health, 60.0);
        assert_eq!(hud.current_score, 250);

        let bar = world.get::<&UiProgressBar>(health_bar_ent).unwrap();
        assert_eq!(bar.value, 60.0);

        let text = world.get::<&UiText>(score_ent).unwrap();
        assert_eq!(text.text, "SCORE: 00250");
    }
}