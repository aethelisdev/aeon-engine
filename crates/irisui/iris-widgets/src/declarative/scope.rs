// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout Scope & Widget Primitives (`iris-widgets::declarative::scope`)
//!
//! Provides a modern hierarchical builder pattern resembling declarative UI systems.
//! Supports immediate interaction responses ([`WidgetResponse`]), automatic flow layout,
//! and strongly typed property inspection primitives partitioned across domain submodules.
//!

pub mod buttons;
pub mod compass;
pub mod containers;
pub mod core;
pub mod display;
pub mod input;
pub mod menu;
pub mod modal;
pub mod progress;
pub mod properties;

pub use super::types::{
    ScopeId, WidgetResponse, combine_seeds, hash_label, hash_label_with_seed, split_label_id,
};
pub use core::UiScope;
pub use display::WrappedLabelDescriptor;
pub use input::InputBoxProps;
pub use properties::{
    PropertySliderOptions, PropertyTextOptions, PropertyTextResponse, PropertyVec3Options,
    PropertyVec3Response,
};