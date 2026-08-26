// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! In-Game Pause Menu Overlay & State Machine Integration.
//!
//! Provides a centered, clean pause overlay with Resume and Exit buttons.
//! Automatically cleans up transient UI entities upon exit.
//!

use ae_core::state::{GameState, StateContext, StateTransition};
use ae_core::ui::{PauseMenuUiTag, UiAnchor, UiButton, UiElement, UiText, UiTextAlignment};

/// Dedicated GameState for the In-Game Pause overlay.
#[derive(Default)]
pub struct InGamePauseState;

impl InGamePauseState {
    /// Creates a new InGamePauseState instance.
    pub fn new() -> Self {
        Self
    }
}

impl GameState for InGamePauseState {
    fn name(&self) -> &'static str {
        "InGamePauseMenu"
    }

    fn on_enter(&mut self, ctx: &mut StateContext<'_>) {
        log::info!("⏸️ [PauseState] Game Paused! Spawning Pause Menu UI...");

        // 1. Spawns Pause Title text
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

        // 2. Resume Button
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

        // 3. Exit to Editor Button
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
        log::info!("▶️ [PauseState] Resuming gameplay...");
        let mut to_despawn = Vec::new();
        for (ent, _tag) in ctx.world.query::<(hecs::Entity, &PauseMenuUiTag)>().iter() {
            to_despawn.push(ent);
        }
        for ent in to_despawn {
            ctx.commands.despawn(ent);
        }
    }
}