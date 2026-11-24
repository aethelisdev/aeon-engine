// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod action_map;
pub mod behavior;
pub mod events;
pub mod input;
pub mod math;
pub mod modules;
pub mod spatial;
pub mod state;
pub mod telemetry;
pub mod time;
pub mod transform;
pub mod ui;

pub use ae_plugin_api::Resources;
pub use behavior::{Behavior, BehaviorContext, NativeBehavior};
pub use cgmath;
pub use state::{
    DefaultPausedState, DefaultPlayingState, GameState, StateContext, StateManager, StateTransition,
};
pub use ui::{
    PauseMenuUiTag, UiAnchor, UiButton, UiDrawCommand, UiElement, UiImage, UiLayoutResolver,
    UiProgressBar, UiRect, UiText, UiTextAlignment,
};