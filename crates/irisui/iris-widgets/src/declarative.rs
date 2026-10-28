// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout Primitives & Context Scope (`iris-widgets::declarative`)
//!
//! Provides a modern, declarative hierarchical UI builder interface resembling
//! modern UI frameworks. Panels use declarative scopes (`row`, `column`, `card`,
//! `button`, `checkbox`, `drag_float`, `drag_vec3`) instead of manually tracking
//! pixel coordinates, parallel target lists, or constructing monolithic boilerplate builders.

pub mod layout;
pub mod scope;
pub mod subtree;
#[cfg(test)]
mod tests;
pub mod types;
pub mod widget;

pub use layout::{
    layout_subtree, measure_content_height, measure_content_height_constrained, measure_height,
    measure_height_constrained, measure_width,
};
pub use scope::{
    CONTEXT_MENU_DEFAULT_WIDTH, CONTEXT_MENU_ITEM_HEIGHT, CollapsingBuilder, CollapsingResponse,
    ContainerResponseExt, ContextMenuExt, InputBoxProps, PopupExt, PopupPlacement,
    PropertySliderOptions, PropertyTextOptions, PropertyTextResponse, PropertyVec3Options,
    PropertyVec3Response, PropertyVec3Target, ScrollAreaConfig, TooltipExt, TreeNodeProps,
    TreeRowProps, TreeRowResponse, TreeViewConfig, TreeViewProps, UiScope, WrappedLabelDescriptor,
    calculate_visible_slice_and_caret, measure_text_advance, property_vec3_axis_tag,
    property_vec3_reset_tag, resolve_property_vec3_for_seed, resolve_property_vec3_for_seeds,
    resolve_property_vec3_tag,
};
pub use subtree::{SubtreeSession, rebuild_leaf_subtree};
pub use types::{
    ScopeId, WidgetResponse, combine_seeds, eval_numeric_expression, format_compact_float,
    hash_label, hash_label_with_seed, split_label_id,
};
pub use widget::{
    Button, Checkbox, Chip, ColorPicker, ColorPickerResponse, ColorSwatch, ComboBox,
    ComboBoxResponse, DragVec3, EmptyBox, NumericDrag, ScalarResponse, ScrollAreaView, TitleBar,
    TitleBarAction, TitleBarBuilder, TitleBarResponse, Toolbar, ToolbarButton, ToolbarGroup,
    ToolbarGroupScope, ToolbarScope, Vec3Response, Widget, title_bar_close_tag,
    title_bar_maximize_tag, title_bar_minimize_tag,
};