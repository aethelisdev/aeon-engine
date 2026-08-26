// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod behavior;
pub mod camera;
pub mod commands;
/// AE Core - Core engine runtime and orchestration infrastructure.
/// Provides a dynamic downcasting-based `Resources` store to prevent
/// borrow checker conflicts between modular subsystems.
pub mod ecs;
pub mod events;
pub mod math;
pub mod modules;
pub mod registry;
pub mod spatial;
pub mod state;
pub mod telemetry;
pub mod time;
pub mod ui;

pub use ae_plugin_api::Resources;
pub use behavior::{Behavior, BehaviorContext, NativeBehavior};
pub use cgmath;
pub use commands::{CommandFn, EntityCommandBuffer};
pub use state::{
    DefaultPausedState, DefaultPlayingState, GameState, StateContext, StateManager, StateTransition,
};
pub use ui::{
    PauseMenuUiTag, UiAnchor, UiButton, UiDrawCommand, UiElement, UiImage, UiLayoutResolver,
    UiProgressBar, UiRect, UiText, UiTextAlignment,
};