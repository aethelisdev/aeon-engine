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
//!

pub mod rows;
pub mod sliders;
pub mod text;
pub mod vectors;

pub use sliders::PropertySliderOptions;
pub use text::{PropertyTextOptions, PropertyTextResponse};
pub use vectors::{PropertyVec3Options, PropertyVec3Response};