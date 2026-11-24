// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod behavior;
pub mod events;
pub mod modules;
pub mod state;
pub mod time;

pub use ae_plugin_api::Resources;
pub use behavior::{Behavior, BehaviorContext, NativeBehavior};
pub use state::{
    DefaultPausedState, DefaultPlayingState, GameState, StateContext, StateManager, StateTransition,
};