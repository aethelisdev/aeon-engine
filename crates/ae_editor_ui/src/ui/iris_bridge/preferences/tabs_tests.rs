// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences All Tabs & Two-Way Data Binding Verification Suite
//!
//! Validates that all ten preference tabs (0..=9) render robustly without errors,
//! and that Navigation (Tab 3) and Input (Tab 7) property sliders mutate [`EditorConfig`]
//! directly in real-time via two-way declarative bindings.

use super::builder::build_preferences_dialog;
use super::types::PreferencesParams;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::widgets::hash_label;
use std::collections::HashSet;

#[test]
fn test_preferences_all_ten_tabs_render_cleanly() {
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();

    for tab_idx in 0..=9 {
        let mut tree = UiTree::new();
        let mut gs = GraphicsSettings::default();
        let mut snapping_settings = SnapSettings::default();
        let mut editor_config = EditorConfig::default();

        let params = PreferencesParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            window_pos: None,
            active_tab: tab_idx,
            scroll_offset_y: 0.0,
            is_scrollbar_dragging: false,
            active_dropdown: None,
            dropdown_trigger_rect: None,
            collapsed_sections: &collapsed,
            blink_caret: false,
            active_number_input: None,
            cursor_pos: Point::new(0.0, 0.0),
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snapping_settings,
            editor_config: &mut editor_config,
            enable_live_updates: false,
            enabled_modules: &enabled_modules,
            events: &[],
        };

        let (_card_id, card_rect, content_rect, max_scroll_y) =
            build_preferences_dialog(&mut tree, params);

        assert!(
            card_rect.width > 500.0 && card_rect.height > 400.0,
            "Tab {} card rect must be valid",
            tab_idx
        );
        assert!(
            content_rect.width > 300.0 && content_rect.height > 200.0,
            "Tab {} content rect must be valid",
            tab_idx
        );
        assert!(
            max_scroll_y >= 0.0,
            "Tab {} max_scroll_y must be non-negative",
            tab_idx
        );
        assert!(
            tree.len() >= 10,
            "Tab {} must construct at least 10 UI nodes, got {}",
            tab_idx,
            tree.len()
        );
    }
}

#[test]
fn test_preferences_two_way_data_binding_navigation_and_input_tabs() {
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();

    // 1. Navigation Tab (Tab 3): Test Base Fly Speed, Shift Boost, and Scroll Zoom Speed drag
    let mut tree_nav = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig {
        camera_base_speed: 5.0,
        camera_shift_multiplier: 3.0,
        camera_scroll_speed: 1.5,
        mouse_sensitivity: 0.005,
        max_undo_history: 100,
        physics_hz: 60.0,
    };

    let tag_fly_speed = hash_label("Base Fly Speed");
    let tag_boost = hash_label("Shift Boost Multiplier");
    let tag_zoom = hash_label("Scroll Zoom Speed");

    let nav_events = [
        (
            tag_fly_speed,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            tag_boost,
            InteractionEvent::Drag {
                delta: Point::new(20.0, 0.0),
            },
        ),
        (
            tag_zoom,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
    ];

    let params_nav = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: 3, // Navigation tab
        scroll_offset_y: 0.0,
        is_scrollbar_dragging: false,
        active_dropdown: None,
        dropdown_trigger_rect: None,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &nav_events,
    };

    build_preferences_dialog(&mut tree_nav, params_nav);

    // Verify mutations in editor_config:
    // camera_base_speed: 5.0 + 10.0 * 0.2 = 7.0
    assert!(
        (editor_config.camera_base_speed - 7.0).abs() < 1e-4,
        "Base fly speed must mutate from 5.0 to 7.0: got {}",
        editor_config.camera_base_speed
    );
    // camera_shift_multiplier: 3.0 + 20.0 * 0.1 = 5.0
    assert!(
        (editor_config.camera_shift_multiplier - 5.0).abs() < 1e-4,
        "Shift boost multiplier must mutate from 3.0 to 5.0: got {}",
        editor_config.camera_shift_multiplier
    );
    // camera_scroll_speed: 1.5 + 10.0 * 0.05 = 2.0
    assert!(
        (editor_config.camera_scroll_speed - 2.0).abs() < 1e-4,
        "Scroll zoom speed must mutate from 1.5 to 2.0: got {}",
        editor_config.camera_scroll_speed
    );

    // 2. Input Tab (Tab 7): Test Look Sensitivity drag
    let mut tree_input = UiTree::new();
    let tag_look_sens = hash_label("Look Sensitivity");

    let input_events = [(
        tag_look_sens,
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];

    let params_input = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: 7, // Input tab
        scroll_offset_y: 0.0,
        is_scrollbar_dragging: false,
        active_dropdown: None,
        dropdown_trigger_rect: None,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &input_events,
    };

    build_preferences_dialog(&mut tree_input, params_input);

    // Look Sensitivity: 0.005 + 10.0 * 0.0002 = 0.007
    assert!(
        (editor_config.mouse_sensitivity - 0.007).abs() < 1e-5,
        "Mouse sensitivity must mutate from 0.005 to 0.007: got {}",
        editor_config.mouse_sensitivity
    );
}

#[test]
fn test_preferences_keymap_scroll_covers_full_content() {
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut tree = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: 4, // Keymap tab
        scroll_offset_y: 0.0,
        is_scrollbar_dragging: false,
        active_dropdown: None,
        dropdown_trigger_rect: None,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
    };

    let (_card_id, _card_rect, content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params);

    // Dynamic measurement must provide sufficient scroll extent so bottom buttons are never cut off
    assert!(
        max_scroll_y >= 500.0,
        "Keymap tab max_scroll_y must be at least 500px to cover all shortcut categories without truncation, got {max_scroll_y}"
    );
    assert_eq!(content_rect.height, 504.0);
}