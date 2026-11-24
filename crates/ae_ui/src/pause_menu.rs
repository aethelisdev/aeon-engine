// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! In-Game Pause Menu Overlay & State Machine Integration.
//!
//! Spawns centered pause UI entities tagged with [`PauseMenuUiTag`] on entry and
//! despawns them cleanly upon resuming gameplay.

use crate::layout::PauseMenuUiTag;
use ae_plugin_api::{UiAnchor, UiButton, UiElement, UiText, UiTextAlignment};
use ae_state::{GameState, StateContext, StateTransition};

/// Dedicated `GameState` for the In-Game Pause overlay.
#[derive(Default)]
pub struct InGamePauseState;

impl InGamePauseState {
    /// Creates a new `InGamePauseState` instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl GameState for InGamePauseState {
    fn name(&self) -> &'static str {
        "InGamePauseMenu"
    }

    fn is_pause_state(&self) -> bool {
        true
    }

    fn on_enter(&mut self, ctx: &mut StateContext<'_>) {
        log::info!("[PauseState] Game Paused! Spawning Pause Menu UI...");

        ctx.commands.spawn_with(|w| {
            w.spawn((
                PauseMenuUiTag,
                UiElement {
                    anchor: UiAnchor::Center,
                    offset: [0.0, -80.0],
                    size: [240.0, 40.0],
                    pivot: [0.5, 0.5],
                    z_index: 50,
                    alpha: 1.0,
                    visible: true,
                },
                UiText::new("GAME PAUSED", 26.0)
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_alignment(UiTextAlignment::Center),
            ))
        });

        ctx.commands.spawn_with(|w| {
            w.spawn((
                PauseMenuUiTag,
                UiElement {
                    anchor: UiAnchor::Center,
                    offset: [0.0, -15.0],
                    size: [180.0, 36.0],
                    pivot: [0.5, 0.5],
                    z_index: 50,
                    alpha: 1.0,
                    visible: true,
                },
                UiButton::new("Resume"),
            ))
        });

        ctx.commands.spawn_with(|w| {
            w.spawn((
                PauseMenuUiTag,
                UiElement {
                    anchor: UiAnchor::Center,
                    offset: [0.0, 35.0],
                    size: [180.0, 36.0],
                    pivot: [0.5, 0.5],
                    z_index: 50,
                    alpha: 1.0,
                    visible: true,
                },
                UiButton::new("Exit to Editor"),
            ))
        });
    }

    fn on_update(&mut self, _ctx: &mut StateContext<'_>, _dt: f32) -> StateTransition {
        StateTransition::None
    }

    fn on_exit(&mut self, ctx: &mut StateContext<'_>) {
        log::info!("[PauseState] Resuming gameplay...");
        let mut to_despawn = Vec::new();
        for (ent, _tag) in ctx
            .world
            .query::<(ae_ecs::Entity, &PauseMenuUiTag)>()
            .iter()
        {
            to_despawn.push(ent);
        }
        for ent in to_despawn {
            ctx.commands.despawn(ent);
        }
    }
}