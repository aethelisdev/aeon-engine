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
use irisui::widgets::declarative::hash_label;
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

        let mut cur_tab = tab_idx;
        let params = PreferencesParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            window_pos: None,
            active_tab: &mut cur_tab,
            collapsed_sections: &collapsed,
            blink_caret: false,
            active_number_input: None,
            cursor_pos: Point::new(0.0, 0.0),
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snapping_settings,
            editor_config: &mut editor_config,
            enable_live_updates: &mut false,
            enabled_modules: &enabled_modules,
            events: &[],
            current_theme: Default::default(),
        };

        let (_card_id, card_rect, content_rect, max_scroll_y) =
            build_preferences_dialog(&mut tree, params, &mut Vec::new());

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
        autosave_enabled: true,
        autosave_interval_secs: 300.0,
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
        active_tab: &mut 3, // Navigation tab
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: &mut false,
        enabled_modules: &enabled_modules,
        events: &nav_events,
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree_nav, params_nav, &mut Vec::new());

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
        active_tab: &mut 7, // Input tab
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: &mut false,
        enabled_modules: &enabled_modules,
        events: &input_events,
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree_input, params_input, &mut Vec::new());

    // Look Sensitivity: 0.005 + 10.0 * 0.0002 = 0.007
    assert!(
        (editor_config.mouse_sensitivity - 0.007).abs() < 1e-5,
        "Mouse sensitivity must mutate from 0.005 to 0.007: got {}",
        editor_config.mouse_sensitivity
    );

    // 3. Editor Tab (Tab 2): Test Auto-Save Interval drag while enabled, then toggle checkbox
    let mut tree_editor = UiTree::new();
    let tag_autosave_interval = hash_label("Auto-Save Interval");
    let tag_autosave_toggle = hash_label("Enable Periodic Auto-Save");
    let drag_events = [(
        tag_autosave_interval,
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];

    let params_editor = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 2, // Editor tab
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: &mut false,
        enabled_modules: &enabled_modules,
        events: &drag_events,
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree_editor, params_editor, &mut Vec::new());

    // Auto-Save Interval: 300.0 + 10.0 * 5.0 = 350.0
    assert!(
        (editor_config.autosave_interval_secs - 350.0).abs() < 1e-3,
        "Auto-Save interval must mutate from 300.0 to 350.0: got {}",
        editor_config.autosave_interval_secs
    );

    let click_events = [(
        tag_autosave_toggle,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let params_toggle = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 2,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: &mut false,
        enabled_modules: &enabled_modules,
        events: &click_events,
        current_theme: Default::default(),
    };
    build_preferences_dialog(&mut tree_editor, params_toggle, &mut Vec::new());
    assert!(
        !editor_config.autosave_enabled,
        "Enable Periodic Auto-Save checkbox click must toggle autosave_enabled to false"
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
        active_tab: &mut 4, // Keymap tab
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: &mut false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, _card_rect, content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Dynamic measurement must provide sufficient scroll extent so bottom buttons are never cut off
    assert!(
        max_scroll_y >= 500.0,
        "Keymap tab max_scroll_y must be at least 500px to cover all shortcut categories without truncation, got {max_scroll_y}"
    );
    assert_eq!(content_rect.height, 504.0);
}

#[test]
fn test_about_dialog_occludes_underlying_preferences_text() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    let screen_width = 1920.0;
    let screen_height = 1080.0;

    // 1. Build Preferences dialog in center
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    let pref_left = ((screen_width - super::builder::PREF_CARD_WIDTH) * 0.5).round();
    let pref_top = ((screen_height - super::builder::PREF_CARD_HEIGHT) * 0.5).round();

    let pref_params = PreferencesParams {
        screen_width,
        screen_height,
        window_pos: Some(Point::new(pref_left, pref_top)),
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: &mut false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };
    let (_pref_id, pref_card_rect, _, _) =
        build_preferences_dialog(&mut tree, pref_params, &mut Vec::new());

    // 2. Build InfoCard dialog centered on top of Preferences via DialogHost
    let mut dialog_host = irisui::workbench::DialogHost::<crate::ui::EngineUiAction>::new();
    let mut action_bus = irisui::workbench::ActionBus::new();
    dialog_host.open_info(
        irisui::workbench::InfoCardDialogDescriptor::new("About Aeon Engine")
            .with_app_name("Aeon Engine")
            .with_size(480.0, 250.0),
    );
    let root = tree.root().unwrap();
    let mut scope = UiScope::new(&mut tree, root);
    dialog_host.render(
        &mut scope,
        Rect::new(0.0, 0.0, screen_width, screen_height),
        false,
        &mut action_bus,
    );

    let about_dialog_rect = Rect::new(
        (screen_width - 480.0) * 0.5,
        (screen_height - 250.0) * 0.5,
        480.0,
        250.0,
    );

    // Register modals in Z-order: Preferences (Layer 0), About (Layer 1)
    let active_modals = [pref_card_rect, about_dialog_rect];
    let sections =
        irisui::IrisSurfaceHost::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts
            .iter()
            .any(|t| t.contains("About Aeon Engine")),
        "Topmost About header title must be rendered without occlusion"
    );
    assert!(
        rendered_texts.iter().any(|t| t.contains("Aeon Engine")),
        "About dialog title must be rendered without occlusion"
    );

    for section in &sections {
        let visible_bounds = match section.clip_bounds {
            Some(clip) => section.bounds.intersect(clip),
            None => section.bounds,
        };
        let visible_center_x = visible_bounds.x + visible_bounds.width * 0.5;
        let visible_center_y = visible_bounds.y + visible_bounds.height * 0.5;
        let visible_inside_about = visible_bounds.width > 0.0
            && visible_bounds.height > 0.0
            && visible_center_x >= about_dialog_rect.x
            && visible_center_x <= about_dialog_rect.x + about_dialog_rect.width
            && visible_center_y >= about_dialog_rect.y
            && visible_center_y <= about_dialog_rect.y + about_dialog_rect.height;

        if visible_inside_about {
            let is_about_text = section.text.contains("About")
                || section.text.contains("Aeon Engine")
                || section.text.contains("Copyright")
                || section.text.contains("Mozilla")
                || section.text.contains("License")
                || section.text.contains("MPL")
                || section.text.contains("WARRANTY")
                || section.text.contains("Close")
                || section.text == "✕";
            assert!(
                is_about_text,
                "Visible text '{}' inside About dialog bounds must belong to About, not bleed from Preferences",
                section.text
            );
        }
    }
}