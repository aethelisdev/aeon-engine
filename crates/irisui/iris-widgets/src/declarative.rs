// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout Primitives & Context Scope (`iris-widgets::declarative`)
//!
//! Provides a modern, declarative hierarchical UI builder interface resembling
//! modern UI frameworks. Panels use declarative scopes (`row`, `column`, `card`,
//! `button`, `checkbox`, `drag_float`, `drag_vec3`) instead of manually tracking
//! pixel coordinates, parallel target lists, or constructing monolithic boilerplate builders.
//!

pub mod layout;
pub mod scope;
#[cfg(test)]
mod tests;
pub mod types;

pub use layout::{
    layout_subtree, measure_content_height, measure_content_height_constrained, measure_height,
    measure_height_constrained, measure_width,
};
pub use scope::{
    InputBoxProps, PropertySliderOptions, PropertyTextOptions, PropertyTextResponse,
    PropertyVec3Options, PropertyVec3Response, UiScope, WrappedLabelDescriptor,
};
pub use types::{
    ScopeId, WidgetResponse, combine_seeds, hash_label, hash_label_with_seed, split_label_id,
};