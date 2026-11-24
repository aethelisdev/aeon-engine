// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Built-in baseline gameplay and pause states.

use crate::types::GameState;

/// Default in-game simulation state representing active gameplay.
#[derive(Default)]
pub struct DefaultPlayingState;

impl GameState for DefaultPlayingState {
    fn name(&self) -> &'static str {
        "Playing"
    }
}

/// Default pause state representing a paused overlay.
#[derive(Default)]
pub struct DefaultPausedState;

impl GameState for DefaultPausedState {
    fn name(&self) -> &'static str {
        "Paused"
    }

    fn is_pause_state(&self) -> bool {
        true
    }
}