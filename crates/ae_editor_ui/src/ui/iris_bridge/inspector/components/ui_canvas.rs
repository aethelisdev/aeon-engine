// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI and HUD Component Inspector Cards
//!
//! Provides modular inspection cards for 2D UI elements, interactive controls, and HUD tags.

pub mod controls;
pub mod primitives;
pub mod tags;

pub use controls::{
    draw_ui_checkbox_card, draw_ui_layout_group_card, draw_ui_slider_card, draw_ui_text_input_card,
};
pub use primitives::{
    draw_ui_button_card, draw_ui_element_card, draw_ui_image_card, draw_ui_panel_card,
    draw_ui_progress_bar_card, draw_ui_text_card,
};
pub use tags::{
    draw_player_health_bar_tag_card, draw_reticle_tag_card, draw_score_display_tag_card,
};