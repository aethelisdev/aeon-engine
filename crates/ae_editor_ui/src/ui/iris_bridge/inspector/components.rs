// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Inspector Cards
//!
//! Re-exports all standard pure declarative ECS component cards.

pub mod animation;
pub mod audio;
pub mod character;
pub mod gameplay;
pub mod hierarchy;
pub mod lod;
pub mod physics;
pub mod rendering;
pub mod ui_canvas;

pub use animation::draw_animation_player_card;
pub use audio::{draw_audio_listener_card, draw_audio_source_card};
pub use character::{draw_character_controller_card, draw_player_tag_card};
pub use gameplay::{
    draw_character_action_card, draw_destructible_target_card, draw_moving_platform_card,
    draw_rotator_card, draw_trigger_zone_card, draw_velocity_card,
};
pub use hierarchy::{draw_parent_card, draw_parent_card_if_present};
pub use lod::draw_lod_group_card;
pub use physics::{draw_collider_card, draw_physics_material_card, draw_rigidbody_card};
pub use rendering::{draw_light_card, draw_model_mesh_card, draw_shape_card};
pub use ui_canvas::{
    draw_player_health_bar_tag_card, draw_reticle_tag_card, draw_score_display_tag_card,
    draw_ui_button_card, draw_ui_checkbox_card, draw_ui_element_card, draw_ui_image_card,
    draw_ui_layout_group_card, draw_ui_panel_card, draw_ui_progress_bar_card, draw_ui_slider_card,
    draw_ui_text_card, draw_ui_text_input_card,
};