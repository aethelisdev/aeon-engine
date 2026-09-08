// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI and HUD Component Inspector Cards
//!
//! Provides modular inspection cards for 2D UI elements, interactive controls, and HUD tags.

pub mod controls;
pub mod primitives;
pub mod tags;

pub use controls::{UiCheckboxHandler, UiLayoutGroupHandler, UiSliderHandler, UiTextInputHandler};
pub use primitives::{
    UiButtonHandler, UiElementHandler, UiImageHandler, UiPanelHandler, UiProgressBarHandler,
    UiTextHandler,
};
pub use tags::{PlayerHealthBarTagHandler, ReticleTagHandler, ScoreDisplayTagHandler};