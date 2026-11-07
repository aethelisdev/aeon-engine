// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Property Rows & Two-Way Bound Form Primitives
//!
//! Provides strongly typed, two-way bound property row components on [`UiScope`]:
//! - Structural rows, checkboxes, dropdowns, and section dividers in [`rows`].
//! - Continuous sliders, scalar inputs, and number boxes in [`sliders`].
//! - Color-coded 3D/2D vector and RGB channel adjusters in [`vectors`].
//!
//! All primitives support the `##` label/tag separator and hierarchical [`UiScope::with_id`]
//! seed isolation without per-frame heap allocations.

pub mod color;
pub mod numeric_input;
pub mod rows;
pub mod sliders;
pub mod text;
pub mod vec3_bind;
pub mod vectors;

pub use crate::declarative::scope::input::ComboRowLayout;
pub use crate::declarative::widget::ComboBoxResponse;
pub use color::PropertyColorResponse;
pub use numeric_input::{NumericEditConfig, NumericEditResult, handle_numeric_edit_interaction};
pub use sliders::PropertySliderOptions;
pub use text::{PropertyTextOptions, PropertyTextResponse};
pub use vec3_bind::{
    euler_deg_to_quat_xyzw, quat_xyzw_to_euler_deg, resolve_euler_deg_from_quat_xyzw,
};
pub use vectors::{
    PropertyVec2Options, PropertyVec2Response, PropertyVec3Options, PropertyVec3Response,
    PropertyVec3Target, property_vec2_axis_tag, property_vec3_axis_tag, property_vec3_reset_tag,
    resolve_property_vec3_for_seed, resolve_property_vec3_for_seeds, resolve_property_vec3_tag,
};