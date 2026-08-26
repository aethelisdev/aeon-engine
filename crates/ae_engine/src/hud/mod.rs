// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular In-Game HUD, Health Bar, Score, and Pause Menu Systems.
//!

pub mod in_game_hud;
pub mod pause_menu;

pub use ae_core::ui::PauseMenuUiTag;
pub use in_game_hud::{InGameHudState, PlayerHealthBarTag, ScoreDisplayTag};
pub use pause_menu::InGamePauseState;