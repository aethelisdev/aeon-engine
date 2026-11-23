// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular In-Game HUD, Health Bar, Score, and Pause Menu Systems.

pub mod in_game_hud;
pub mod overlay;
pub mod pause_menu;

pub use ae_plugin_api::{PlayerHealthBarTag, ScoreDisplayTag};
pub use in_game_hud::InGameHudState;
pub use overlay::{
    RuntimeHudOverlay, RuntimePauseAction, TAG_PLAY_EXIT, TAG_PLAY_RESUME,
    build_in_game_pause_menu, build_ingame_ecs_ui,
};
pub use pause_menu::InGamePauseState;