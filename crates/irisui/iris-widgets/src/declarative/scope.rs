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
pub mod context_menu;
pub mod core;
pub mod display;
pub mod input;
pub mod input_box;
pub mod menu;
pub mod modal;
pub mod progress;
pub mod properties;
pub mod scroll_area;
pub mod spacing;
pub mod tree_view;

pub use scroll_area::ScrollAreaConfig;
pub use tree_view::{TreeNodeProps, TreeRowProps, TreeRowResponse, TreeViewConfig, TreeViewProps};

pub use super::types::{
    ScopeId, WidgetResponse, combine_seeds, hash_label, hash_label_with_seed, split_label_id,
};
pub use containers::ContainerResponseExt;
pub use context_menu::{CONTEXT_MENU_DEFAULT_WIDTH, CONTEXT_MENU_ITEM_HEIGHT, ContextMenuExt};
pub use core::UiScope;
pub use display::WrappedLabelDescriptor;
pub use input_box::{InputBoxProps, calculate_visible_slice_and_caret, measure_text_advance};
pub use properties::{
    PropertySliderOptions, PropertyTextOptions, PropertyTextResponse, PropertyVec3Options,
    PropertyVec3Response,
};