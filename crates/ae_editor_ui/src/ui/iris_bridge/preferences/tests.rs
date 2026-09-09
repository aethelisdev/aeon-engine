// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Event Routing & Dragging Verification Suite
//!
//! Validates continuous window dragging calculations across panel boundaries and menubar,
//! reliable clamping behavior, and dialog hit-test target isolation.

use super::builder::{
    PREF_CARD_HEIGHT, PREF_CARD_WIDTH, TITLEBAR_HEIGHT, build_preferences_dialog,
};
use super::types::PreferencesParams;
use crate::ui::iris_bridge::events::preferences::calculate_preferences_drag_pos;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use std::collections::HashSet;

#[test]
fn test_preferences_drag_position_clamping_and_smooth_boundary_motion() {
    let screen_width = 1920.0;
    let screen_height = 1080.0;
    let drag_offset = Point::new(50.0, 15.0);

    // 1. Dragging to the center of the screen
    let center_cursor = Point::new(500.0, 300.0);
    let center_pos =
        calculate_preferences_drag_pos(center_cursor, drag_offset, screen_width, screen_height);
    assert_eq!(center_pos.x, 450.0);
    assert_eq!(center_pos.y, 285.0);

    // 2. Dragging cursor into the Hierarchy panel / far left boundary (cursor_pos.x = 20.0)
    let left_cursor = Point::new(20.0, 300.0);
    let left_pos =
        calculate_preferences_drag_pos(left_cursor, drag_offset, screen_width, screen_height);
    assert_eq!(
        left_pos.x, 0.0,
        "Preferences window must clamp smoothly to 0.0 on the left without freezing"
    );

    // 3. Dragging cursor into the top menubar boundary (cursor_pos.y = 10.0 <= MENUBAR_HEIGHT 28.0)
    let top_cursor = Point::new(500.0, 10.0);
    let top_pos =
        calculate_preferences_drag_pos(top_cursor, drag_offset, screen_width, screen_height);
    assert_eq!(
        top_pos.y, 28.0,
        "Preferences window y must clamp safely to 28.0 beneath menubar without freezing"
    );

    // 4. Dragging cursor beyond the right/bottom screen edges
    let bottom_right_cursor = Point::new(2500.0, 1500.0);
    let max_pos = calculate_preferences_drag_pos(
        bottom_right_cursor,
        drag_offset,
        screen_width,
        screen_height,
    );
    let expected_max_x = (screen_width - PREF_CARD_WIDTH).max(0.0);
    let expected_max_y = (screen_height - PREF_CARD_HEIGHT).max(28.0);
    assert_eq!(max_pos.x, expected_max_x);
    assert_eq!(max_pos.y, expected_max_y);
}

#[test]
fn test_preferences_dialog_builder_and_hit_targets() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let graphics_settings = GraphicsSettings::default();
    let snapping_settings = SnapSettings::default();
    let editor_config = EditorConfig::default();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(400.0, 200.0)),
        active_tab: 0,
        scroll_offset_y: 0.0,
        active_dropdown: None,
        collapsed_sections: &collapsed,
        active_number_input: None,
        blink_caret: false,
        cursor_pos: Point::new(420.0, 215.0),
        zoom_factor: 1.0,
        graphics_settings: &graphics_settings,
        snapping_settings: &snapping_settings,
        editor_config: &editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
    };

    let (_widget_id, targets) = build_preferences_dialog(&mut tree, params);

    // Verify card geometry
    assert_eq!(targets.card_rect.x, 400.0);
    assert_eq!(targets.card_rect.y, 200.0);
    assert_eq!(targets.card_rect.width, PREF_CARD_WIDTH);
    assert_eq!(targets.card_rect.height, PREF_CARD_HEIGHT);

    // Verify titlebar geometry
    assert_eq!(targets.title_bar_rect.x, 400.0);
    assert_eq!(targets.title_bar_rect.y, 200.0);
    assert_eq!(targets.title_bar_rect.height, TITLEBAR_HEIGHT);

    // Verify hit testing: Titlebar contains point for dragging
    let title_point = Point::new(450.0, 215.0);
    assert!(
        targets.title_bar_rect.contains_point(title_point),
        "Titlebar must contain point for drag initiation"
    );

    // Verify hit testing: Card rect contains inner point
    let inside_point = Point::new(500.0, 300.0);
    assert!(
        targets.card_rect.contains_point(inside_point),
        "Card rect must contain point to consume clicks and isolate underlying canvas"
    );

    // Verify hit testing: Outside point does not fall within card
    let outside_point = Point::new(100.0, 200.0);
    assert!(
        !targets.card_rect.contains_point(outside_point),
        "Card rect must not contain outside points to permit docked panel clicks"
    );
}