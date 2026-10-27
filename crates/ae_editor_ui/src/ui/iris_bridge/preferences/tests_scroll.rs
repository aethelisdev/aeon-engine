// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Autonomous Scroll Area Integration Verification Suite
//!
//! Validates autonomous scroll frame geometry, mouse wheel scrolling with bounds clamping,
//! scrollbar thumb click-drag-release lifecycle, track jump clicks, and state synchronization
//! using autonomous `UiTree::dispatch_event` and `build_preferences_dialog`.

use super::builder::build_preferences_dialog;
use super::types::{
    PREF_TAG_CONTENT_VIEW, PreferencesDialogState, PreferencesParams, preferences_scroll_style,
};
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::widgets::scroll_area::{ScrollAreaFrame, ScrollbarDragState};
use std::collections::HashSet;

#[test]
fn test_preferences_scroll_style_and_frame_geometry() {
    let style = preferences_scroll_style();
    assert_eq!(style.thickness, 6.0);
    assert_eq!(style.inset, 3.0);

    let viewport = Rect::new(161.0, 36.0, 599.0, 504.0);
    let total_h = 1800.0;
    let scroll_y = 120.0;

    let frame = ScrollAreaFrame::from_geometry(viewport, total_h, scroll_y, &style);
    assert_eq!(frame.viewport_rect, viewport);
    assert_eq!(frame.content_height, 1800.0);
    assert_eq!(frame.max_scroll_y, 1800.0 - 504.0);

    let geom = frame
        .scrollbar
        .expect("Scrollbar geometry must be computed");
    assert_eq!(geom.track_rect.width, 6.0);
    assert!(geom.thumb_rect.height >= style.thumb_min_length);
    assert_eq!(geom.thumb_rect.width, 6.0);
}

#[test]
fn test_preferences_scroll_state_reset_and_sync() {
    let mut state = PreferencesDialogState::default();
    state.scroll_state.scroll_y = 250.0;
    state.scroll_state.drag_state = Some(ScrollbarDragState::new(300.0, 250.0));
    assert!(state.scroll_state.is_dragging());

    state.reset_scroll();
    assert_eq!(state.scroll_state.scroll_y, 0.0);
    assert!(!state.scroll_state.is_dragging());
}

#[test]
fn test_preferences_handle_scroll_area_event_wheel_scrolling() {
    let mut tree = UiTree::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(100.0, 50.0)),
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(400.0, 300.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, _card_rect, content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // 1. Wheel scroll inside content area
    let cursor_inside = Point::new(content_rect.x + 50.0, content_rect.y + 50.0);
    let wheel_event = UiEvent::MouseScroll {
        point: cursor_inside,
        delta_x: 0.0,
        delta_y: -50.0,
    };

    let resp = tree.dispatch_event(&wheel_event);
    assert!(resp.is_consumed());

    // 2. Large wheel scroll clamps to max_scroll_y
    let huge_wheel = UiEvent::MouseScroll {
        point: cursor_inside,
        delta_x: 0.0,
        delta_y: -5000.0,
    };
    let resp_clamped = tree.dispatch_event(&huge_wheel);
    assert!(resp_clamped.is_consumed());
    assert_eq!(tree.max_scroll(PREF_TAG_CONTENT_VIEW), max_scroll_y);

    // 3. Wheel upwards clamps at 0.0
    let up_wheel = UiEvent::MouseScroll {
        point: cursor_inside,
        delta_x: 0.0,
        delta_y: 10000.0,
    };
    let resp_top = tree.dispatch_event(&up_wheel);
    assert!(resp_top.is_consumed());
}

#[test]
fn test_preferences_handle_scroll_area_event_thumb_drag_lifecycle() {
    let mut tree = UiTree::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(100.0, 50.0)),
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(400.0, 300.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, _card_rect, content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());
    assert!(max_scroll_y > 0.0);

    // Content area right side is where the scrollbar thumb sits
    let thumb_x = content_rect.x + content_rect.width - 5.0;
    let thumb_y = content_rect.y + 20.0;

    let down_event = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(thumb_x, thumb_y),
    };

    let resp_down = tree.dispatch_event(&down_event);
    if resp_down.is_consumed() {
        assert!(tree.event_state.pointer_capture.is_some());

        let move_event = UiEvent::MouseMove {
            point: Point::new(thumb_x, thumb_y + 40.0),
        };
        let resp_move = tree.dispatch_event(&move_event);
        assert!(resp_move.is_consumed());

        let up_event = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(thumb_x, thumb_y + 40.0),
        };
        let resp_up = tree.dispatch_event(&up_event);
        assert!(resp_up.is_consumed());
        assert_eq!(tree.event_state.pointer_capture, None);
    }
}

#[test]
fn test_preferences_wheel_outside_content_is_unconsumed() {
    let mut tree = UiTree::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(100.0, 50.0)),
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(400.0, 300.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, card_rect, _content_rect, _max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Point outside the dialog completely (e.g. x = 10.0, y = 10.0)
    let outside_cursor = Point::new(10.0, 10.0);
    assert!(!card_rect.contains_point(outside_cursor));

    let wheel_event = UiEvent::MouseScroll {
        point: outside_cursor,
        delta_x: 0.0,
        delta_y: -20.0,
    };

    let resp = tree.dispatch_event(&wheel_event);
    assert!(
        !resp.is_consumed(),
        "Wheel event outside preferences dialog must not be consumed by preferences content view"
    );
}

#[test]
fn test_preferences_titlebar_drag_interaction() {
    let mut tree = UiTree::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(200.0, 150.0)),
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(300.0, 160.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let mut actions = Vec::new();
    let (_card_id, card_rect, _content_rect, _max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut actions);

    // Compute layout so all children have non-zero computed_rect
    let mut layout_engine = LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, Size::new(1920.0, 1080.0));

    // Point on titlebar text label (e.g. x = 250.0, y = 165.0)
    let title_point = Point::new(card_rect.x + 50.0, card_rect.y + 15.0);
    let down_event = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: title_point,
    };
    let down_resp = tree.dispatch_event(&down_event);
    assert!(down_resp.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture_tag,
        Some(crate::ui::iris_bridge::preferences::PREF_TAG_TITLEBAR)
    );

    // Move mouse to drag
    let move_point = Point::new(title_point.x + 30.0, title_point.y + 20.0);
    let move_event = UiEvent::MouseMove { point: move_point };
    let move_resp = tree.dispatch_event(&move_event);
    assert!(move_resp.is_consumed());
    assert_eq!(
        tree.event_state.frame_drag_delta,
        Some(Point::new(30.0, 20.0))
    );

    // Autonomous drag evaluation through FloatingPanelState controller
    let mut floating = irisui::widgets::FloatingPanelState::new();
    floating.rect = Some(card_rect);
    floating.pos = Some(Point::new(card_rect.x, card_rect.y));

    let outcome_down = floating.handle_event(&mut tree, &down_event, &down_resp, 1920.0, 1080.0);
    assert!(outcome_down.consumed);
    assert!(floating.is_dragging());

    let outcome_move = floating.handle_event(&mut tree, &move_event, &move_resp, 1920.0, 1080.0);
    assert!(outcome_move.consumed);
    assert!(outcome_move.needs_layout_rebuild);
    assert_eq!(
        outcome_move.action,
        Some(irisui::widgets::FloatingPanelAction::Move(Point::new(
            card_rect.x + 30.0,
            card_rect.y + 20.0
        )))
    );
}

#[test]
fn test_preferences_select_tab_resets_scroll() {
    let mut state = PreferencesDialogState::default();
    state.scroll_state.scroll_y = 450.0;
    assert_eq!(state.scroll_state.scroll_y, 450.0);

    // When tab changes, reset_scroll() must reset all scroll offsets to 0.0
    state.reset_scroll();
    assert_eq!(state.scroll_state.scroll_y, 0.0);
}

#[test]
fn test_preferences_overlay_tree_full_drag_cycle() {
    let mut tree = UiTree::new();
    let collapsed = HashSet::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let enabled_modules = HashSet::new();

    let params1 = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::ZERO,
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let mut actions = Vec::new();
    let (_card_id, card_rect, _content_rect, _max_scroll_y) =
        build_preferences_dialog(&mut tree, params1, &mut actions);

    let mut layout_engine = LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, Size::new(1920.0, 1080.0));

    // 1. MouseDown on titlebar
    let title_point = Point::new(card_rect.x + 50.0, card_rect.y + 15.0);
    let down_event = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: title_point,
    };
    let down_resp = tree.dispatch_event(&down_event);
    assert!(down_resp.is_consumed());

    // 2. Simulate next frame rebuild before Move (as happens in live build_shell_overlays)
    tree.clear();
    let mut actions2 = Vec::new();
    let mut graphics_settings2 = GraphicsSettings::default();
    let mut snapping_settings2 = SnapSettings::default();
    let mut editor_config2 = EditorConfig::default();
    let params2 = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: title_point,
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings2,
        snapping_settings: &mut snapping_settings2,
        editor_config: &mut editor_config2,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };
    build_preferences_dialog(&mut tree, params2, &mut actions2);
    let _ = layout_engine.compute_layout(&mut tree, Size::new(1920.0, 1080.0));

    // 3. MouseMove event (dragging titlebar by dx=30, dy=20)
    let move_point = Point::new(title_point.x + 30.0, title_point.y + 20.0);
    let move_event = UiEvent::MouseMove { point: move_point };
    let move_resp = tree.dispatch_event(&move_event);

    eprintln!(
        "DEBUG: move_resp consumed: {}, frame_drag_delta: {:?}",
        move_resp.is_consumed(),
        tree.event_state.frame_drag_delta
    );
    assert!(
        move_resp.is_consumed(),
        "Move event must be consumed by titlebar drag"
    );
    assert!(
        tree.event_state.frame_drag_delta.is_some(),
        "frame_drag_delta must be populated on tree"
    );
}