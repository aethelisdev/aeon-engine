// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Widgets (`iris-widgets`)
//!
//! Standard widget set and game-engine editor components for Iris UI.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod asset_card;
pub mod canvas;
pub mod card;
pub mod cascading_menu;
pub mod color_picker;
pub mod console;
pub mod context_menu;
pub mod declarative;
pub mod dropdown;
pub mod grid_view;
pub mod input;
pub mod modal;
pub mod numeric_input;
pub mod panel;
pub mod scroll_area;
pub mod settings;
pub mod timeline;
pub mod tree_view;

pub use asset_card::{
    AssetCardBadge, AssetCardBuilder, AssetCardFrame, AssetCardPreview, AssetCardStyle,
};
pub use canvas::ChartDrawer;
pub use card::{CardBuilder, CardFrame, CardIcon, CardStyle};
pub use cascading_menu::{
    CascadingMenuBuilder, CascadingMenuFrame, CascadingMenuIcon, CascadingMenuItem,
    CascadingMenuStyle,
};
pub use color_picker::{
    ColorPickerClickAction, ColorPickerCursor, ColorPickerDragMode, HsvColorPickerBuilder,
    HsvColorPickerState, HsvColorPickerTargets, evaluate_color_picker_click,
    evaluate_color_picker_cursor, evaluate_color_picker_drag, hsv_to_rgb, rgb_to_hsv,
};
pub use console::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_PANEL_ROOT, CONSOLE_TAG_ROW, CONSOLE_TAG_SCROLLBAR_THUMB,
    CONSOLE_TAG_SCROLLBAR_TRACK, CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT,
    CONSOLE_TAG_TOOLBAR, CONSOLE_TAG_VIEWPORT, ConsoleFilterLevel, ConsoleLogCounts,
    ConsoleLogLevel, ConsoleToolbarAction, ConsoleToolbarCursor, evaluate_console_toolbar_click,
    evaluate_console_toolbar_cursor, is_console_tag,
};

pub use context_menu::{
    ContextMenuBuilder, ContextMenuHeader, ContextMenuIcon, ContextMenuItem, ContextMenuStyle,
};
pub use declarative::{
    InputBoxProps, UiScope, WidgetResponse, hash_label, layout_subtree, measure_content_height,
    measure_height,
};
pub use dropdown::{
    ComboboxButtonBuilder, ComboboxButtonFrame, ComboboxButtonStyle, ComboboxPopupBuilder,
    ComboboxPopupFrame, ComboboxPopupStyle, ComboboxRowBuilder, ComboboxRowFrame, ComboboxRowStyle,
};
pub use grid_view::ResponsiveGrid;
pub use input::TextInputState;
pub use modal::{
    MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER, MODAL_TAG_SCRIM,
    ModalDialogAction, ModalDialogStyle, evaluate_modal_tag,
};
pub use numeric_input::{NumericInputEditState, NumericInputPillBuilder, NumericInputStyle};
pub use panel::PanelBuilder;
pub use scroll_area::{
    ScrollAreaBuilder, ScrollAreaFrame, ScrollAreaStyle, ScrollBarGeometry, ScrollBarHit,
    ScrollBarVisibility, ScrollDirection, VirtualItemHeight, VirtualList, VirtualScrollConfig,
    VirtualSlice,
};
pub use settings::{
    SettingSectionBuilder, SettingSectionFrame, SettingSectionStyle, TabbedDialogBuilder,
    TabbedDialogFrame, TabbedDialogStyle, TabbedDialogTab,
};
pub use timeline::{
    DEFAULT_RULER_HEIGHT, DEFAULT_SCRUBBER_HEIGHT, DEFAULT_SPEED_PRESETS, MediaTransportAction,
    MediaTransportStyle, TIMELINE_TAG_LOOP, TIMELINE_TAG_PANEL_ROOT, TIMELINE_TAG_PLAY_PAUSE,
    TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_SCRUBBER_TRACK, TIMELINE_TAG_SPEED_BASE,
    TIMELINE_TAG_STEP_BACK, TIMELINE_TAG_STEP_FWD, TIMELINE_TAG_STOP, TimelineKeyframeMarker,
    TimelineRulerStyle, evaluate_timeline_transport_tag, is_timeline_tag,
};
pub use tree_view::{TreeRowBuilder, TreeRowFrame, TreeRowIcon, TreeRowStyle};

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Color, UiTree};

    #[test]
    fn test_text_input_state_utf8_safety() {
        let mut state = TextInputState::new("Ağaç");
        assert_eq!(state.buffer, "Ağaç");
        state.backspace();
        assert_eq!(state.buffer, "Ağa");
    }

    #[test]
    fn test_card_builder_and_style() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let frame = CardBuilder::new(&mut tree, root)
            .name("TestCard")
            .rect(iris_core::Rect::new(0.0, 0.0, 200.0, 100.0))
            .title("Node Inspector")
            .icon_text("⚙")
            .build();

        assert!(tree.get(frame.card_id).is_some());
        let node = tree.get(frame.card_id).unwrap();
        assert_eq!(node.name.as_deref(), Some("TestCard"));
    }

    #[test]
    fn test_game_engine_builders() {
        let mut tree = UiTree::new();
        let root_id = tree.create_node();

        let row_frame = TreeRowBuilder::new(iris_core::Rect::new(0.0, 0.0, 200.0, 24.0))
            .label("Player Character")
            .is_selected(true)
            .build(&mut tree, root_id);
        assert!(tree.get(row_frame.row_id).is_some());

        let asset_frame = AssetCardBuilder::new(iris_core::Rect::new(0.0, 0.0, 115.0, 125.0))
            .title("shader.wgsl")
            .badge(Some(AssetCardBadge::new("WGSL", Color::YELLOW)))
            .build(&mut tree, root_id);
        assert!(tree.get(asset_frame.card_id).is_some());
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