// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Handlers Registration
//!
//! Registers all standard ECS component cards into the `InspectorRegistry`.

pub mod animation;
pub mod audio;
pub mod character;
pub mod gameplay;
pub mod hierarchy;
pub mod lod;
pub mod physics;
pub mod registration;
pub mod rendering;
pub mod ui_canvas;

pub use hierarchy::{draw_parent_card, draw_parent_card_if_present};
pub use lod::draw_lod_group_card;
pub use registration::register_all_components;
pub use ui_canvas::{
    draw_player_health_bar_tag_card, draw_reticle_tag_card, draw_score_display_tag_card,
    draw_ui_button_card, draw_ui_checkbox_card, draw_ui_element_card, draw_ui_image_card,
    draw_ui_layout_group_card, draw_ui_panel_card, draw_ui_progress_bar_card, draw_ui_slider_card,
    draw_ui_text_card, draw_ui_text_input_card,
};