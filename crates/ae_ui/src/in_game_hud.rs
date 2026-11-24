// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular In-Game HUD Subsystem & Gameplay Event Bridge.
//!
//! Bridges gameplay events (`DamageEvent`, `HealEvent`, `ScoreEvent`) to user-defined
//! ECS UI entities tagged with `PlayerHealthBarTag` and `ScoreDisplayTag`.

use ae_core::events::{DamageEvent, DynamicEventBus, HealEvent, ScoreEvent};
use ae_ecs::World;
use ae_plugin_api::{PlayerHealthBarTag, PlayerTag, ScoreDisplayTag, UiProgressBar, UiText};

/// State tracker for active in-game HUD session.
#[derive(Default)]
pub struct InGameHudState {
    /// Current accumulated player score.
    pub current_score: i32,
    /// Current player health value synchronized with HUD bars.
    pub player_health: f32,
    /// Maximum player health upper bound.
    pub player_max_health: f32,
}

impl InGameHudState {
    /// Creates a new HUD state with full health and 0 score.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_score: 0,
            player_health: 100.0,
            player_max_health: 100.0,
        }
    }

    /// Resets health and score counters and synchronizes existing HUD widgets in `world`.
    pub fn reset(&mut self, world: &mut World) {
        self.current_score = 0;
        self.player_health = self.player_max_health;

        for (_, bar) in world.query_mut::<(&PlayerHealthBarTag, &mut UiProgressBar)>() {
            bar.value = bar.max;
        }
        for (_, text) in world.query_mut::<(&ScoreDisplayTag, &mut UiText)>() {
            text.text = format!("SCORE: {:05}", self.current_score);
        }
    }

    /// Processes incoming gameplay events and updates corresponding HUD entities.
    pub fn update_from_events(&mut self, world: &mut World, event_bus: &mut DynamicEventBus) {
        if let Some(events) = event_bus.receive::<ScoreEvent>() {
            for ev in events {
                self.current_score = (self.current_score + ev.delta).max(0);
                for (_, text) in world.query_mut::<(&ScoreDisplayTag, &mut UiText)>() {
                    text.text = format!("SCORE: {:05}", self.current_score);
                }
            }
        }

        if let Some(events) = event_bus.receive::<DamageEvent>() {
            for ev in events {
                if !world.has::<PlayerTag>(ev.target) {
                    continue;
                }
                self.player_health = (self.player_health - ev.amount).max(0.0);
                for (_, bar) in world.query_mut::<(&PlayerHealthBarTag, &mut UiProgressBar)>() {
                    bar.value = self.player_health;
                    let fraction = bar.fraction();
                    bar.fill_color = if fraction < 0.3 {
                        [0.9, 0.2, 0.2, 1.0]
                    } else if fraction < 0.6 {
                        [0.9, 0.75, 0.2, 1.0]
                    } else {
                        [0.2, 0.85, 0.35, 1.0]
                    };
                }
            }
        }

        if let Some(events) = event_bus.receive::<HealEvent>() {
            for ev in events {
                if !world.has::<PlayerTag>(ev.target) {
                    continue;
                }
                self.player_health = (self.player_health + ev.amount).min(self.player_max_health);
                for (_, bar) in world.query_mut::<(&PlayerHealthBarTag, &mut UiProgressBar)>() {
                    bar.value = self.player_health;
                    if bar.fraction() >= 0.6 {
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
    use ae_ecs::World;
    use ae_plugin_api::{UiAnchor, UiElement};

    #[test]
    fn test_in_game_hud_event_reactions_and_clamping() {
        let mut world = World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut hud = InGameHudState::new();

        let player = world.spawn((PlayerTag,));
        let enemy = world.spawn(());

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

        // Non-player damage must not affect player health bar
        event_bus.send(DamageEvent {
            source: None,
            target: enemy,
            amount: 50.0,
            hit_point: None,
            hit_normal: None,
        });
        event_bus.send(DamageEvent {
            source: None,
            target: player,
            amount: 40.0,
            hit_point: None,
            hit_normal: None,
        });
        event_bus.send(ScoreEvent {
            delta: -10,
            new_total: 0,
        });
        event_bus.send(ScoreEvent {
            delta: 250,
            new_total: 250,
        });

        hud.update_from_events(&mut world, &mut event_bus);

        assert_eq!(hud.player_health, 60.0);
        assert_eq!(hud.current_score, 250);

        let bar_val = world
            .inspect::<UiProgressBar, _>(health_bar_ent, |b| b.value)
            .expect("bar value");
        assert_eq!(bar_val, 60.0);

        let score_text = world
            .inspect::<UiText, _>(score_ent, |t| t.text.clone())
            .expect("score text");
        assert_eq!(score_text, "SCORE: 00250");
    }
}