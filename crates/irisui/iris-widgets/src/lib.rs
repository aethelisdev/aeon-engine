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
pub use declarative::scope::{TabBarResponse, UiScope};
pub use declarative::subtree::{SubtreeSession, rebuild_leaf_subtree};
pub use declarative::types::{ScopeId, WidgetResponse};
pub use declarative::widget::{
    Button, Checkbox, Chip, ColorPicker, ColorSwatch, ComboBox, DROP_TARGET_SEED, DragVec3,
    DropTargetId, DropTargetResponse, EmptyBox, NumericDrag, TITLE_BAR_CONTROL_SEED, TitleBar,
    TitleBarAction, TitleBarBuilder, TitleBarResponse, Toolbar, ToolbarButton, ToolbarGroup,
    Widget, title_bar_close_tag, title_bar_maximize_tag, title_bar_minimize_tag,
};
pub use floating_panel::{
    DEFAULT_TITLEBAR_HEIGHT, FLOATING_TAG_CLOSE, FLOATING_TAG_TITLEBAR, FLOATING_TAG_WINDOW,
    FloatingPanelAction, FloatingPanelOutcome, FloatingPanelState, calculate_floating_drag_pos,
};
pub use grid_view::ResponsiveGrid;
pub use scroll_area::ScrollArea;

/// Declarative widget prelude re-exporting only the core widgets, scope, and response types.
///
/// Designed to be imported via wildcard `use iris_widgets::prelude::*;` or through the `irisui` facade.
pub mod prelude {
    // 1. Declarative Core & Scope
    pub use crate::declarative::scope::{TabBarResponse, UiScope};
    pub use crate::declarative::subtree::{SubtreeSession, rebuild_leaf_subtree};
    pub use crate::declarative::types::{ScopeId, WidgetResponse};
    pub use crate::declarative::widget::Widget;

    // 2. Primary Showcase Widgets
    pub use crate::declarative::widget::{
        Button, Checkbox, Chip, ColorPicker, ColorSwatch, ComboBox, DROP_TARGET_SEED, DragVec3,
        DropTargetId, DropTargetResponse, EmptyBox, NumericDrag, TITLE_BAR_CONTROL_SEED, TitleBar,
        TitleBarAction, TitleBarBuilder, TitleBarResponse, Toolbar, ToolbarButton, ToolbarGroup,
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
    pub use crate::floating_panel::FloatingPanelState as FloatingState;
    pub use crate::input::TextInputState;
    pub use crate::scroll_area::ScrollAreaState;
    pub use crate::tree_view::TreeState;
}

#[cfg(test)]
mod tests {
    use crate::canvas::ChartDrawer;
    use crate::input::TextInputState;
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