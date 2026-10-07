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

/// Fluent, comprehensive umbrella prelude re-exporting all standard Iris UI primitives.
///
/// Designed to be imported as `use irisui::prelude::*;` for , idiomatic engine UI development.
pub mod prelude {
    pub use iris_core::{
        AlignItems, Border, BoxShadow, Color, CornerRadii, DirtyFlags, EventDispatcher,
        ExternalTextureId, FlexDirection, FlexWrap, FocusManager, HitTargetInfo, HitTestResult,
        ImeEvent, Insets, InteractionEvent, IrisCoreError, JustifyContent, KeyCode, MouseButton,
        Point, Position, Rect, Size, Style, TextAlign, TextWrap, UiEvent, UiLayer, UiTree,
        WidgetCursor, WidgetId, WidgetNode, WidgetRole, WidgetState,
    };
    pub use iris_dock::{
        ActiveSplitterDrag, ComputedDockLayout, ComputedFloatingLayout, DEFAULT_RESIZE_MARGIN,
        DockChevronTarget, DockChromeFrame, DockChromeParams, DockChromeStyle, DockCloseTarget,
        DockDragState, DockError, DockLayoutOptions, DockNavigatorGeometry, DockNavigatorStyle,
        DockNode, DockNodeId, DockOverflowClickAction, DockOverflowItemTarget,
        DockOverflowMenuFrame, DockOverflowMenuParams, DockOverflowMenuStyle, DockPanel,
        DockSplitterTarget, DockState, DockStyle, DockTabTarget, DockTree, DropZone,
        FloatingDockWindow, FloatingDragState, FloatingResizeEdge, FloatingTabBadgeParams,
        FloatingWindow, FloatingWindowClampBounds, FloatingWindowClickAction, FloatingWindowCursor,
        FloatingWindowId, FloatingWindowStyle, LeafLayoutInfo, MultiViewportManager, PanelRegistry,
        SimpleTabViewer, SplitDirection, SplitterLayoutInfo, TabBarLayoutInfo,
        TabContextMenuAction, TabContextMenuState, TabLayoutInfo, TabViewer, UiNotifier,
        build_dock_chrome, build_dock_navigator_nodes, build_dock_overflow_menu,
        build_drop_preview_node, build_floating_tab_badge, build_floating_windows_layer,
        calculate_drop_preview_rect, calculate_drop_zone, calculate_leaf_half_drop_zone,
        calculate_screen_drop_zone, calculate_tab_reorder_index, clamp_floating_windows,
        compute_dock_layout, compute_dock_layout_advanced, compute_dock_layout_with_options,
        compute_dock_layout_with_viewer, compute_floating_layouts, compute_tab_bar_layout,
        detect_resize_edge, evaluate_dock_overflow_click, evaluate_floating_resize_cursor,
        evaluate_floating_window_click, find_active_tab_content_rect, hit_test_navigator,
    };
    pub use iris_layout::{LayoutEngine, LayoutError};
    pub use iris_text::{
        TextCollectionOptions, TextRenderer, TextSection, TextSystem, collect_text_sections,
        collect_text_sections_with_options,
    };
    pub use iris_wgpu::{
        CustomDrawCallback, DrawCommand, DrawCommandList, ExternalTexturePipeline,
        ExternalTextureQuadInstance, ExternalTextures, IrisRenderer, QuadInstance,
        TextureQuadInstance, TextureQuadPipeline, TreeCompilerOptions, compile_tree_draw_commands,
        compile_tree_draw_commands_into,
    };
    pub use iris_widgets::{
        CardBuilder, CardFrame, CardIcon, CardStyle, CascadingMenuBuilder, CascadingMenuFrame,
        CascadingMenuIcon, CascadingMenuItem, CascadingMenuStyle, ChartDrawer,
        ColorPickerClickAction, ColorPickerCursor, ColorPickerDragMode, ComboboxButtonBuilder,
        ComboboxButtonFrame, ComboboxButtonStyle, ComboboxPopupBuilder, ComboboxPopupFrame,
        ComboboxPopupStyle, ComboboxRowBuilder, ComboboxRowFrame, ComboboxRowStyle,
        ContextMenuBuilder, ContextMenuHeader, ContextMenuIcon, ContextMenuItem, ContextMenuStyle,
        HsvColorPickerBuilder, HsvColorPickerState, HsvColorPickerTargets, InputBoxProps,
        MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER, MODAL_TAG_SCRIM,
        ModalDialogAction, ModalDialogStyle, NumericInputEditState, NumericInputPillBuilder,
        NumericInputStyle, PanelBuilder, PropertySliderOptions, PropertyTextOptions,
        PropertyTextResponse, PropertyVec3Options, PropertyVec3Response, ResponsiveGrid,
        ScrollArea, ScrollAreaBuilder, ScrollAreaFrame, ScrollAreaStyle, ScrollBarGeometry,
        ScrollBarHit, ScrollBarVisibility, ScrollDirection, SettingSectionBuilder,
        SettingSectionFrame, SettingSectionStyle, TabbedDialogBuilder, TabbedDialogFrame,
        TabbedDialogStyle, TabbedDialogTab, TextInputState, TreeRowBuilder, TreeRowFrame,
        TreeRowIcon, TreeRowStyle, UiScope, VirtualItemHeight, VirtualList, VirtualScrollConfig,
        VirtualSlice, WidgetResponse, WrappedLabelDescriptor, evaluate_color_picker_click,
        evaluate_color_picker_cursor, evaluate_color_picker_drag, evaluate_modal_tag, hash_label,
        hash_label_with_seed, hsv_to_rgb, layout_subtree, measure_content_height, measure_height,
        measure_height_constrained, measure_width, rgb_to_hsv,
    };
}