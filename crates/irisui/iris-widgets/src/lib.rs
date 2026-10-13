// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Widgets (`iris-widgets`)
//!
//! Standard widget set and game-engine editor components for Iris UI.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod canvas;
pub mod cascading_menu;
pub mod color_picker;
pub mod context_menu;
pub mod controls;
pub mod declarative;
pub mod grid_view;
pub mod input;
pub mod modal;
pub mod numeric_input;
pub mod panel;
pub mod scroll_area;
pub mod settings;
pub mod tree_view;

pub use canvas::ChartDrawer;

pub use cascading_menu::{
    CascadingMenuBuilder, CascadingMenuFrame, CascadingMenuIcon, CascadingMenuItem,
    CascadingMenuStyle,
};
pub use color_picker::{ColorPickerDragMode, HsvColorPickerState, hsv_to_rgb, rgb_to_hsv};
pub use context_menu::{
    ContextMenuBuilder, ContextMenuHeader, ContextMenuIcon, ContextMenuItem, ContextMenuStyle,
};
pub use controls::{
    FocusEventResponse, FocusModifiers, FocusPolicy, FocusState, FocusTarget, FocusableArea,
    NUMERIC_DRAG_COARSE_MULTIPLIER, NUMERIC_DRAG_FINE_MULTIPLIER, NUMERIC_DRAG_THRESHOLD,
    NumericDragResponse, NumericDragState, handle_focus_event, handle_focus_event_with_modifiers,
    handle_numeric_drag_event,
};
pub use declarative::{
    InputBoxProps, PropertySliderOptions, PropertyTextOptions, PropertyTextResponse,
    PropertyVec3Options, PropertyVec3Response, ScopeId, ScrollAreaConfig, TreeNodeProps,
    TreeRowProps, TreeRowResponse, TreeViewConfig, TreeViewProps, UiScope, WidgetResponse,
    WrappedLabelDescriptor, calculate_visible_slice_and_caret, combine_seeds, hash_label,
    hash_label_with_seed, layout_subtree, measure_content_height, measure_height,
    measure_height_constrained, measure_text_advance, measure_width, split_label_id,
};
pub use grid_view::ResponsiveGrid;
pub use input::{TextInputState, copy_to_clipboard, get_clipboard_text};
pub use modal::{
    MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER, MODAL_TAG_SCRIM,
    ModalDialogAction, ModalDialogStyle, evaluate_modal_tag,
};
pub use numeric_input::{NumericInputEditState, NumericInputStyle};
pub use panel::PanelBuilder;
pub use scroll_area::{
    DEFAULT_SCROLL_STEP, ScrollArea, ScrollAreaBuilder, ScrollAreaFrame, ScrollAreaState,
    ScrollAreaStyle, ScrollBarGeometry, ScrollBarHit, ScrollBarVisibility, ScrollDirection,
    ScrollResponse, ScrollbarDragState, VirtualItemHeight, VirtualList, VirtualScrollConfig,
    VirtualSlice, WheelScrollConfig, WheelScrollResponse, evaluate_wheel_scroll,
    handle_scroll_area_event, handle_scrollbar_event,
};
pub use settings::{
    SettingSectionBuilder, SettingSectionFrame, SettingSectionStyle, TabbedDialogBuilder,
    TabbedDialogFrame, TabbedDialogStyle, TabbedDialogTab,
};
pub use tree_view::{
    DEFAULT_TREE_BASE_PADDING, DEFAULT_TREE_INDENT_STEP, DEFAULT_TREE_LINE_ARM_GAP,
    DEFAULT_TREE_LINE_ARM_LENGTH, DEFAULT_TREE_LINE_THICKNESS, DEFAULT_TREE_ROW_HEIGHT,
    TREE_CHEVRON_ARROW_COLLAPSED, TREE_CHEVRON_ARROW_EXPANDED, TREE_CHEVRON_COLLAPSED,
    TREE_CHEVRON_EXPANDED, TreeClickTarget, TreeConnectorGeometry, TreeIndentConfig, TreeItemInfo,
    TreeLineHorizontal, TreeLineVertical, TreeNavResponse, TreeRowStyle, TreeSelectionModifiers,
    TreeSelectionResponse, TreeState, compute_tree_connector_geometry, compute_tree_scroll_to_view,
    evaluate_tree_click, handle_tree_arrow_navigation, handle_tree_foldout_click,
    handle_tree_navigation_event, handle_tree_row_click, handle_tree_selection_event,
    tree_chevron_arrow_glyph, tree_chevron_glyph,
};

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::Color;

    #[test]
    fn test_text_input_state_utf8_safety() {
        let mut state = TextInputState::new("Ağaç");
        assert_eq!(state.buffer, "Ağaç");
        state.backspace();
        assert_eq!(state.buffer, "Ağa");
    }

    #[test]
    fn test_chart_drawer_polyline() {
        let mut cmd_list = iris_wgpu::DrawCommandList::new();
        let dummy_samples = [8.33f32, 16.67, 12.0, 24.0, 8.0, 16.0];
        ChartDrawer::draw_polyline(
            &mut cmd_list,
            iris_core::Rect::new(0.0, 0.0, 300.0, 100.0),
            |i| dummy_samples.get(i).copied(),
            dummy_samples.len(),
            36.0,
            6,
            |_| Color::GREEN,
        );

        assert_eq!(cmd_list.quads.len(), 5);
    }
}