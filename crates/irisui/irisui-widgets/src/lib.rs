// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Widgets (`irisui-widgets`)
//!
//! Standard widget set and game-engine editor components for Iris UI.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod canvas;
pub mod cascading_menu;
pub mod clipboard;
pub mod color_picker;
pub mod context_menu;
pub mod controls;
pub mod declarative;
pub mod floating_panel;
pub mod grid_view;
pub mod input;
pub mod modal;
pub mod numeric_input;
pub mod panel;
pub mod scroll_area;
pub mod settings;
pub mod tree_view;

// Primary declarative showcase elements re-exported at crate root
pub use clipboard::SystemClipboard;
pub use controls::{GutterMetrics, GutterStyle, MultilineNavOutcome, MultilineTextState};
pub use declarative::caret_sync::{
    build_caret_aware_compile_options, build_caret_aware_text_options, sync_caret_blink_for_tree,
    sync_caret_blink_with_delta_for_tree,
};
pub use declarative::scope::{
    ActionBarButton, AnchoredPopupConfig, AnchoredSelectOption, AnchoredSelectPillSpec,
    AssetSlotConfig, AssetSlotResponse, BadgePillSpec, CascadingMenuEntry, ColorPickerPopupConfig,
    ColorPickerPopupResponse, ColorPickerPopupTags, ComboRowLayout, DataTableBuilder,
    DataTableConfig, DropdownCardConfig, DualActionBarResponse, ImmediateCanvasSpec, InputBoxStyle,
    LabeledStatusDescriptor, LiveTextProps, LogRowDescriptor, ModalDescriptor,
    MultilineEditorProps, NumberLabelDescriptor, PopupCardStyle, PropertyActionRow,
    PropertyColorResponse, PropertyPaletteConfig, PropertyPaletteResponse, ScrubberMarker,
    ScrubberResponse, ScrubberTrackSpec, SelectPillTriggerStyle, TabBarResponse, TableColumnSpec,
    TableColumnWidth, TimeRulerSpec, TimeRulerStyle, TrackLaneGroupSpec, TrackLaneSpec,
    TreeSelectionEvent, UiScope, VirtualGridBuilder, VirtualGridConfig, VirtualTreeBuilder,
    VirtualTreeConfig, WrappedLabelDescriptor,
};
pub use declarative::subtree::{SubtreeSession, rebuild_leaf_subtree};
pub use declarative::types::{ScopeId, WidgetResponse};
pub use declarative::widget::{
    Button, Checkbox, Chip, ColorPicker, ColorSwatch, ComboBox, ContextMenuHost,
    DEFAULT_DRAG_PREVIEW_OFFSET, DEFAULT_OVERSCAN, DROP_TARGET_SEED, DragCurve, DragVec3,
    DropTargetId, DropTargetResponse, DropdownHost, DropdownItem, EmptyBox, FloatingDragPreview,
    FloatingDragPreviewStyle, NumericDrag, POPUP_HOST_ITEM_SEED, TITLE_BAR_CONTROL_SEED, TitleBar,
    TitleBarAction, TitleBarBuilder, TitleBarResponse, Toolbar, ToolbarButton, ToolbarGroup,
    ViewportDropOverlay, VirtualList, VirtualListBuilder, VirtualListResponse, VirtualSlice,
    VirtualSliceParams, Widget, compute_slice, compute_virtual_slice, title_bar_close_tag,
    title_bar_maximize_tag, title_bar_minimize_tag,
};
pub use floating_panel::{
    DEFAULT_TITLEBAR_HEIGHT, FLOATING_TAG_CLOSE, FLOATING_TAG_TITLEBAR, FLOATING_TAG_WINDOW,
    FloatingPanelAction, FloatingPanelOutcome, FloatingPanelState, calculate_floating_card_rect,
    calculate_floating_drag_pos,
};
pub use grid_view::ResponsiveGrid;
pub use scroll_area::ScrollArea;

/// Declarative widget prelude re-exporting only the core widgets, scope, and response types.
///
/// Designed to be imported via wildcard `use irisui_widgets::prelude::*;` or through the `irisui` facade.
pub mod prelude {
    // 1. Declarative Core & Scope
    pub use crate::declarative::caret_sync::{
        build_caret_aware_compile_options, build_caret_aware_text_options,
        sync_caret_blink_for_tree, sync_caret_blink_with_delta_for_tree,
    };
    pub use crate::declarative::scope::{
        ActionBarButton, ActionCardButton, ActionCardConfig, AnchoredPopupConfig,
        AnchoredSelectOption, AnchoredSelectPillSpec, AssetSlotConfig, AssetSlotResponse,
        BadgePillSpec, CardBuilder, CardHeaderDescriptor, CardHeaderResponse, CardOptions,
        CardResponse, CascadingMenuEntry, CascadingPopupConfig, ColorPaletteCardConfig,
        ColorPickerPopupConfig, ColorPickerPopupResponse, ColorPickerPopupTags,
        ComboBoxActionResponse, ComboBoxMapConfig, ComboRowLayout, DataTableBuilder,
        DataTableConfig, DropdownCardConfig, DualActionBarResponse, ImmediateCanvasSpec,
        InputBoxStyle, LabeledInputButton, LabeledInputConfig, LabeledStatusDescriptor,
        LiveTextProps, LogRowBadge, LogRowDescriptor, MultilineEditorProps, NavItemConfig,
        NumberLabelDescriptor, PopupCardStyle, PopupPlacement, PrimaryButtonConfig,
        PropertyActionRow, PropertyColorResponse, PropertyColorSwatchRowConfig,
        PropertyColorSwatchRowResponse, PropertyPaletteConfig, PropertyPaletteResponse,
        PropertyTextOptions, ScrubberMarker, ScrubberResponse, ScrubberTrackSpec,
        SelectPillTriggerStyle, SelectionCardConfig, TabBarResponse, TableColumnSpec,
        TableColumnWidth, TimeRulerSpec, TimeRulerStyle, TrackLaneGroupSpec, TrackLaneSpec,
        TreeSelectionEvent, TreeViewConfig, UiScope, VerticalScrollbarSpec, VirtualGridBuilder,
        VirtualGridConfig, VirtualTreeBuilder, VirtualTreeConfig, WrappedLabelDescriptor,
        euler_deg_to_quat_xyzw, quat_xyzw_to_euler_deg, resolve_euler_deg_from_quat_xyzw,
    };
    pub use crate::declarative::subtree::{SubtreeSession, rebuild_leaf_subtree};
    pub use crate::declarative::types::{ScopeId, WidgetResponse};
    pub use crate::declarative::widget::Widget;

    // 2. Primary Showcase Widgets
    pub use crate::declarative::widget::{
        Button, Checkbox, Chip, ColorPicker, ColorSwatch, ComboBox, ContextMenuHost,
        DEFAULT_DRAG_PREVIEW_OFFSET, DEFAULT_OVERSCAN, DEFAULT_PROPERTY_LABEL_WIDTH,
        DROP_TARGET_SEED, DragCurve, DragVec2, DragVec3, DropTargetId, DropTargetResponse,
        DropdownHost, DropdownItem, EmptyBox, FloatingDragPreview, FloatingDragPreviewStyle,
        NumericDrag, POPUP_HOST_ITEM_SEED, TITLE_BAR_CONTROL_SEED, TitleBar, TitleBarAction,
        TitleBarBuilder, TitleBarResponse, Toolbar, ToolbarButton, ToolbarGroup, Vec2Response,
        Vec3Response, ViewportDropOverlay, VirtualList, VirtualListBuilder, VirtualListResponse,
        VirtualSlice, VirtualSliceParams, compute_slice, compute_virtual_slice,
        title_bar_close_tag, title_bar_maximize_tag, title_bar_minimize_tag,
    };
    pub use crate::floating_panel::{
        FloatingPanelAction, FloatingPanelOutcome, FloatingPanelState,
    };
    pub use crate::grid_view::ResponsiveGrid;
    pub use crate::scroll_area::ScrollArea;

    // 3. Extension Traits
    pub use crate::declarative::scope::{
        ContainerResponseExt, ContextMenuExt, PopupExt, TooltipExt,
    };

    // 4. Primary Persistent State Structures
    pub use crate::color_picker::HsvColorPickerState;
    pub use crate::controls::{
        GutterMetrics, GutterStyle, MultilineNavOutcome, MultilineTextState,
    };
    pub use crate::floating_panel::{
        FloatingPanelState as FloatingState, calculate_floating_card_rect,
    };
    pub use crate::input::TextInputState;
    pub use crate::scroll_area::ScrollAreaState;
    pub use crate::tree_view::{
        TreeDataSource, TreeIndentConfig, TreeModel, TreeNodeCollector, TreeRowEntry, TreeRowStyle,
        TreeState,
    };
}

#[cfg(test)]
mod tests {
    use crate::canvas::ChartDrawer;
    use crate::input::TextInputState;
    use irisui_core::Color;

    #[test]
    fn test_text_input_state_utf8_safety() {
        let mut state = TextInputState::new("Ağaç");
        assert_eq!(state.buffer, "Ağaç");
        state.backspace();
        assert_eq!(state.buffer, "Ağa");
    }

    #[test]
    fn test_chart_drawer_polyline() {
        let mut cmd_list = irisui_wgpu::DrawCommandList::new();
        let dummy_samples = [8.33f32, 16.67, 12.0, 24.0, 8.0, 16.0];
        ChartDrawer::draw_polyline(
            &mut cmd_list,
            irisui_core::Rect::new(0.0, 0.0, 300.0, 100.0),
            |i| dummy_samples.get(i).copied(),
            dummy_samples.len(),
            36.0,
            6,
            |_| Color::GREEN,
        );

        assert_eq!(cmd_list.quads.len(), 5);
    }
}