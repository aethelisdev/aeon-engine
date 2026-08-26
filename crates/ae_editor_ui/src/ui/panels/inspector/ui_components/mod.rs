// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Specialized Inspector UI Handlers for In-Game UI, Canvas Layout, and HUD Component Primitives.
//!
//! Modularized submodules adhering to Single Responsibility Principle:
//! - [`transform`]: 2D screen anchoring and transform editing (`UiElement`).
//! - [`containers`]: Styling boxes and layout flow groups (`UiPanel`, `UiLayoutGroup`).
//! - [`text`]: Typography labels and input fields (`UiText`, `UiTextInput`).
//! - [`controls`]: Interactive buttons, sliders, and checkboxes (`UiButton`, `UiSlider`, `UiCheckbox`).
//! - [`gauges`]: Progress meters and health bars (`UiProgressBar`).
//! - [`media`]: Custom sprite images and 9-slice frames (`UiImage`).
//! - [`tags`]: Gameplay HUD event binding tags (`PlayerHealthBarTag`, `ScoreDisplayTag`, `ReticleTag`).
//!

pub mod containers;
pub mod controls;
pub mod gauges;
pub mod media;
pub mod tags;
pub mod text;
pub mod transform;

pub use containers::{UiLayoutGroupUiHandler, UiPanelUiHandler};
pub use controls::{UiButtonUiHandler, UiCheckboxUiHandler, UiSliderUiHandler};
pub use gauges::UiProgressBarUiHandler;
pub use media::UiImageUiHandler;
pub use tags::{PlayerHealthBarTagUiHandler, ReticleTagUiHandler, ScoreDisplayTagUiHandler};
pub use text::{UiTextInputUiHandler, UiTextUiHandler};
pub use transform::UiElementUiHandler;