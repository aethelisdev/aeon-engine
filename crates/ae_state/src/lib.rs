// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Stack-based pushdown automaton game state machine and lifecycle hooks.

pub mod defaults;
pub mod machine;
pub mod types;

pub use defaults::{DefaultPausedState, DefaultPlayingState};
pub use machine::StateManager;
pub use types::{GameState, StateContext, StateTransition};