// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout Scope & Widget Primitives (`iris-widgets::declarative::scope`)
//!
//! Provides a modern hierarchical builder pattern resembling declarative UI systems.
//! Supports immediate interaction responses ([`WidgetResponse`]), automatic flow layout,
//! and strongly typed property inspection primitives partitioned across domain submodules.

pub mod action_card;
pub mod asset_slot;
pub mod buttons;
pub mod collapsing;
pub mod compass;
pub mod conditional;
pub mod containers;
pub mod context_menu;
pub mod core;
pub mod display;
pub mod hover;
pub mod input;
pub mod input_box;
pub mod labeled_input;
pub mod menu;
pub mod menu_bar;
pub mod modal;
pub mod multiline;
pub mod nav_item;
pub mod panel;
pub mod primary_button;
pub mod progress;
pub mod properties;
pub mod root;
pub mod scroll_area;
pub mod selection_card;
pub mod spacing;
pub mod tabs;
pub mod tag_events;
pub mod title_bar;
pub mod tooltip;
pub mod tree_view;

pub use action_card::{ActionCardButton, ActionCardConfig};
pub use collapsing::{CollapsingBuilder, CollapsingResponse};
pub use labeled_input::{LabeledInputButton, LabeledInputConfig};
pub use menu_bar::{
    MENUBAR_DEFAULT_HEIGHT, MENUBAR_DROPDOWN_DEFAULT_WIDTH, MenuBarBuilder, MenuBarScope, MenuScope,
};
pub use multiline::MultilineEditorProps;
pub use nav_item::NavItemConfig;
pub use panel::PanelBuilder;
pub use primary_button::PrimaryButtonConfig;
pub use scroll_area::ScrollAreaConfig;
pub use selection_card::SelectionCardConfig;
pub use tabs::TabBarResponse;
pub use title_bar::TitleBarBuilder;
pub use tooltip::TooltipExt;
pub use tree_view::{TreeNodeProps, TreeRowProps, TreeRowResponse, TreeViewConfig, TreeViewProps};

pub use super::types::{
    ScopeId, WidgetResponse, combine_seeds, hash_label, hash_label_with_seed, split_label_id,
};
pub use asset_slot::{AssetSlotConfig, AssetSlotResponse};
pub use buttons::{ActionBarButton, DualActionBarResponse};
pub use containers::{
    CardBuilder, CardHeaderDescriptor, CardHeaderResponse, CardOptions, CardResponse,
    ColumnBuilder, ContainerResponseExt, InnerResponse, RowBuilder,
};
pub use context_menu::{CONTEXT_MENU_DEFAULT_WIDTH, CONTEXT_MENU_ITEM_HEIGHT, ContextMenuExt};
pub use core::UiScope;
pub use display::WrappedLabelDescriptor;
pub use input::ComboRowLayout;
pub use input_box::{InputBoxProps, calculate_visible_slice_and_caret, measure_text_advance};
pub use menu::{CascadingPopupConfig, PopupExt, PopupPlacement};
pub use properties::{
    ColorPaletteCardConfig, ColorPickerPopupConfig, ColorPickerPopupResponse, ColorPickerPopupTags,
    ComboBoxActionResponse, PropertyActionRow, PropertyColorResponse, PropertyColorSwatchRowConfig,
    PropertyColorSwatchRowResponse, PropertyPaletteConfig, PropertyPaletteResponse,
    PropertySliderOptions, PropertyTextOptions, PropertyTextResponse, PropertyVec3Options,
    PropertyVec3Response, PropertyVec3Target, property_vec3_axis_tag, property_vec3_reset_tag,
    resolve_hsv_from_color, resolve_property_vec3_for_seed, resolve_property_vec3_for_seeds,
    resolve_property_vec3_tag,
};