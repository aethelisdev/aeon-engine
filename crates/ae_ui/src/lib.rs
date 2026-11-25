// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! In-game ECS UI layout resolution, gameplay HUD event synchronization, pause state overlay,
//! declarative Iris HUD builder, and standalone runtime HUD renderer.

pub mod in_game_hud;
pub mod iris_builder;
pub mod iris_pass;
pub mod layout;
pub mod pause_menu;
pub mod runtime_overlay;

pub use ae_plugin_api::{
    PlayerHealthBarTag, ReticleTag, ScoreDisplayTag, UiAnchor, UiButton, UiCheckbox, UiElement,
    UiImage, UiLayoutGroup, UiLayoutType, UiPanel, UiProgressBar, UiRect, UiSliceMode, UiSlider,
    UiText, UiTextAlignment, UiTextInput,
};
pub use in_game_hud::InGameHudState;
pub use iris_builder::{
    TAG_PLAY_EXIT, TAG_PLAY_RESUME, build_in_game_pause_menu, build_ingame_ecs_ui,
};
pub use iris_pass::{IrisRenderPassParams, iris_render_pass};
pub use layout::{PauseMenuUiTag, UiDrawCommand, UiLayoutResolver};
pub use pause_menu::InGamePauseState;
pub use runtime_overlay::{RuntimeHudOverlay, RuntimePauseAction};