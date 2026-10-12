// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI (`irisui`)
//!
//! Umbrella facade crate uniting the modular Iris UI framework crates:
//! - `iris-core`: Generational arena, nodes, styles, dirty tracking, and geometry primitives.
//! - `iris-layout`: Pure, high-performance flexbox and absolute layout solver.
//! - `iris-text`: Glyphon-based GPU subpixel font rendering and text measurement engine.
//! - `iris-wgpu`: High-performance batched instanced quad renderer and custom external texture pipelines.
//! - `iris-widgets`: Industrial, hardware-accelerated game engine UI widget kit.
//! - `iris-dock`: Generational binary split-tree docking and floating window multi-viewport engine.
//! - `iris-winit`: Winit platform adapter, event translation, HighDPI scaling, and cursor mapping.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use iris_core as core;
pub use iris_dock as dock;
pub use iris_layout as layout;
pub use iris_text as text;
pub use iris_wgpu as wgpu_backend;
pub use iris_widgets as widgets;
pub use iris_winit as winit;

/// Fluent, comprehensive umbrella prelude re-exporting all standard Iris UI primitives.
///
/// Designed to be imported as `use irisui::prelude::*;` for , idiomatic engine UI development.
pub mod prelude {
    pub use iris_core::{
        AlignItems, Border, BoxShadow, Color, CornerRadii, DirtyFlags, DispatchResult,
        EventDispatcher, EventState, ExternalTextureId, FlexDirection, FlexWrap, FocusManager,
        HitTargetInfo, HitTestResult, ImeEvent, Insets, InteractionEvent, IrisCoreError,
        JustifyContent, KeyCode, MouseButton, Point, Position, Rect, ResponseFlags, Size, Style,
        TextAlign, TextWrap, UiEvent, UiLayer, UiTree, WidgetCursor, WidgetId, WidgetNode,
        WidgetRole, WidgetState,
    };
    pub use iris_dock::{
        ActiveSplitterDrag, ComputedDockLayout, ComputedFloatingLayout, DEFAULT_FLOATING_PANE_SIZE,
        DEFAULT_RESIZE_MARGIN, DockArea, DockChevronTarget, DockChromeFrame, DockChromeParams,
        DockChromeStyle, DockCloseTarget, DockDragState, DockError, DockInteractionConfig,
        DockLayoutOptions, DockNavigatorGeometry, DockNavigatorStyle, DockNode, DockNodeId,
        DockOverflowClickAction, DockOverflowItemTarget, DockOverflowMenuFrame,
        DockOverflowMenuParams, DockOverflowMenuStyle, DockPanel, DockResponse, DockSplitterTarget,
        DockState, DockStyle, DockTabTarget, DockTree, DropZone, FloatingDockWindow,
        FloatingDragState, FloatingInteractionResponse, FloatingResizeEdge, FloatingTabBadgeParams,
        FloatingWindow, FloatingWindowClampBounds, FloatingWindowClickAction, FloatingWindowCursor,
        FloatingWindowId, FloatingWindowStyle, LeafLayoutInfo, MultiViewportManager, PanelRegistry,
        PendingTabDrag, SimpleTabViewer, SplitDirection, SplitterInteractionResponse,
        SplitterLayoutInfo, TAB_DRAG_MOTION_THRESHOLD_SQ, TabBarLayoutInfo, TabContextMenuAction,
        TabContextMenuState, TabDragInteractionResponse, TabLayoutInfo,
        TabStripInteractionResponse, TabViewer, UiNotifier, build_dock_chrome,
        build_dock_navigator_nodes, build_dock_overflow_menu, build_drop_preview_node,
        build_floating_tab_badge, build_floating_windows_layer, calculate_drop_preview_rect,
        calculate_drop_zone, calculate_leaf_half_drop_zone, calculate_screen_drop_zone,
        calculate_tab_reorder_index, clamp_floating_windows, compute_dock_layout,
        compute_dock_layout_advanced, compute_dock_layout_with_options,
        compute_dock_layout_with_viewer, compute_floating_layouts, compute_tab_bar_layout,
        detect_resize_edge, evaluate_dock_overflow_click, evaluate_floating_resize_cursor,
        evaluate_floating_window_click, find_active_tab_content_rect, handle_floating_window_event,
        handle_splitter_event, handle_tab_drag_event, handle_tab_strip_event, hit_test_navigator,
    };
    pub use iris_layout::{LayoutEngine, LayoutError};
    pub use iris_text::{
        TextCollectionOptions, TextRenderer, TextSection, TextSystem, collect_text_sections,
        collect_text_sections_with_options, measure_text_advance, set_default_text_family,
    };
    pub use iris_wgpu::{
        CustomDrawCallback, DrawCommand, DrawCommandList, ExternalTexturePipeline,
        ExternalTextureQuadInstance, ExternalTextures, IrisRenderer, QuadInstance,
        TextureQuadInstance, TextureQuadPipeline, TreeCompilerOptions, compile_tree_draw_commands,
        compile_tree_draw_commands_into,
    };
    pub use iris_widgets::{
        CascadingMenuBuilder, CascadingMenuFrame, CascadingMenuIcon, CascadingMenuItem,
        CascadingMenuStyle, ChartDrawer, ColorPickerDragMode, ContextMenuBuilder,
        ContextMenuHeader, ContextMenuIcon, ContextMenuItem, ContextMenuStyle, DEFAULT_SCROLL_STEP,
        FocusEventResponse, FocusModifiers, FocusPolicy, FocusState, FocusTarget, FocusableArea,
        HsvColorPickerState, InputBoxProps, MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM,
        MODAL_TAG_DANGER, MODAL_TAG_SCRIM, ModalDialogAction, ModalDialogStyle,
        NUMERIC_DRAG_COARSE_MULTIPLIER, NUMERIC_DRAG_FINE_MULTIPLIER, NUMERIC_DRAG_THRESHOLD,
        NumericDragResponse, NumericDragState, NumericInputEditState, NumericInputStyle,
        PanelBuilder, PropertySliderOptions, PropertyTextOptions, PropertyTextResponse,
        PropertyVec3Options, PropertyVec3Response, ResponsiveGrid, ScrollArea, ScrollAreaBuilder,
        ScrollAreaConfig, ScrollAreaFrame, ScrollAreaState, ScrollAreaStyle, ScrollBarGeometry,
        ScrollBarHit, ScrollBarVisibility, ScrollDirection, ScrollResponse, ScrollbarDragState,
        SettingSectionBuilder, SettingSectionFrame, SettingSectionStyle, TabbedDialogBuilder,
        TabbedDialogFrame, TabbedDialogStyle, TabbedDialogTab, TextInputState, TreeClickTarget,
        TreeConnectorGeometry, TreeIndentConfig, TreeItemInfo, TreeLineHorizontal,
        TreeLineVertical, TreeNavResponse, TreeRowProps, TreeRowStyle, TreeSelectionModifiers,
        TreeSelectionResponse, TreeState, TreeViewConfig, TreeViewProps, UiScope,
        VirtualItemHeight, VirtualList, VirtualScrollConfig, VirtualSlice, WheelScrollConfig,
        WheelScrollResponse, WidgetResponse, WrappedLabelDescriptor,
        calculate_visible_slice_and_caret, compute_tree_connector_geometry,
        compute_tree_scroll_to_view, copy_to_clipboard, evaluate_modal_tag, evaluate_tree_click,
        evaluate_wheel_scroll, get_clipboard_text, handle_focus_event,
        handle_focus_event_with_modifiers, handle_numeric_drag_event, handle_scroll_area_event,
        handle_scrollbar_event, handle_tree_arrow_navigation, handle_tree_foldout_click,
        handle_tree_navigation_event, handle_tree_row_click, handle_tree_selection_event,
        hash_label, hash_label_with_seed, hsv_to_rgb, layout_subtree, measure_content_height,
        measure_height, measure_height_constrained, measure_width, rgb_to_hsv,
        tree_chevron_arrow_glyph, tree_chevron_glyph,
    };
}