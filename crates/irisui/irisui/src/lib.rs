// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI (`irisui`)
//!
//! GPU-accelerated Retained-Mode user interface framework for games, engines,
//! and interactive desktop tools.
//!
//! ## Architecture Overview
//! - [`core`]: Generational arena-based UI tree, geometry primitives, dirty flags, and styling.
//! - [`dock`]: Generational binary split-tree panel docking, tab state, and layout persistence.
//! - [`layout`]: Flexbox and Grid layout resolution powered by Taffy.
//! - [`text`]: Sub-pixel glyph layout, font shaping, and text caching via `cosmic-text` and `glyphon`.
//! - [`wgpu_backend`]: Instanced GPU SDF fragment shader pipeline for rounded rects, borders, and shadows.
//! - [`widgets`]: Standard UI widget builders and game engine editor controls.
//! - [`prelude`]: Commonly used types and builder helpers.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Core arena data structures, geometric primitives, dirty-state flags, and base styling types.
pub use iris_core as core;
/// Hierarchical binary split-tree docking system, tab management, and dock layout persistence.
pub use iris_dock as dock;
/// Flexbox and CSS Grid layout computation adapter powered by Taffy.
pub use iris_layout as layout;
/// Hardware-accelerated typography, font shaping, and glyph caching engine.
pub use iris_text as text;
/// GPU SDF rendering pipeline and shader backend for 2D UI elements.
pub use iris_wgpu as wgpu_backend;
/// Reusable UI widget primitives, control builders, and property inspectors.
pub use iris_widgets as widgets;

/// Convenient common imports for Iris UI applications.
pub mod prelude {
    pub use iris_core::color::Color;
    pub use iris_core::dirty::DirtyFlags;
    pub use iris_core::event::{
        EventDispatcher, FocusManager, HitTestResult, ImeEvent, InteractionEvent, KeyCode,
        MouseButton, UiEvent, WidgetState,
    };
    pub use iris_core::geometry::{Border, BoxShadow, CornerRadii, Insets, Point, Rect, Size};
    pub use iris_core::id::WidgetId;
    pub use iris_core::node::WidgetNode;
    pub use iris_core::style::{AlignItems, FlexDirection, JustifyContent, Style, TextAlign};
    pub use iris_core::tree::UiTree;

    pub use iris_dock::{
        ActiveSplitterDrag, ComputedDockLayout, DockDragState, DockError, DockNode, DockNodeId,
        DockState, DockTree, DropZone, FloatingDockWindow, FloatingWindowId, LeafLayoutInfo,
        MultiViewportManager, SplitDirection, SplitterLayoutInfo, compute_dock_layout,
    };
    pub use iris_layout::{LayoutEngine, LayoutError};
    pub use iris_text::{TextRenderer, TextSection, TextSystem};
    pub use iris_wgpu::{
        DrawCommand, DrawCommandList, IrisRenderer, QuadInstance, TextureQuadInstance,
        TextureQuadPipeline,
    };
    pub use iris_widgets::{
        AssetCardBuilder, ButtonBuilder, CanvasBuilder, ChartDrawer, ChartStyle, ChartThreshold,
        CheckboxBuilder, ColorPickerBuilder, DragValueBuilder, DropdownBuilder,
        DropdownMenuBuilder, HsvColorPickerBuilder, HsvColorPickerState, HsvColorPickerTargets,
        LabelBuilder, MenuBarBuilder, PanelBuilder, PropertyRowBuilder, SectionHeaderBuilder,
        SliderBuilder, StatusBarBuilder, TabBuilder, TextInputBuilder, TextInputState,
        TreeItemBuilder, hsv_to_rgb, rgb_to_hsv,
    };
}