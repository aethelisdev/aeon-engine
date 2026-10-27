// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Event Routing & Dragging Verification Suite
//!
//! Validates continuous window dragging calculations across panel boundaries and menubar,
//! reliable clamping behavior, 100% declarative semantic tag dispatching, and slider bounds invariants.

use super::builder::{
    PREF_CARD_HEIGHT, PREF_CARD_WIDTH, SIDEBAR_TABS, TITLEBAR_HEIGHT, build_preferences_dialog,
    calculate_preferences_drag_pos,
};
use super::types::{
    PREF_TAG_CLOSE, PREF_TAG_TITLEBAR, PreferencesAction, PreferencesParams, PreferencesToggleId,
    encode_section_tag, encode_tab_tag, encode_toggle_tag, is_preferences_tag, parse_section_tag,
    parse_tab_tag, parse_toggle_tag,
};
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::{FpsLimit, GraphicsSettings};
use irisui::prelude::*;
use irisui::widgets::declarative::hash_label;
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

    // 3. Dragging cursor into the top menubar boundary (cursor_pos.y = 10.0 <= MENUBAR_HEIGHT)
    let top_cursor = Point::new(500.0, 10.0);
    let top_pos =
        calculate_preferences_drag_pos(top_cursor, drag_offset, screen_width, screen_height);
    let expected_min_y = crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT;
    assert_eq!(
        top_pos.y, expected_min_y,
        "Preferences window y must clamp safely beneath menubar without freezing"
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
    let expected_max_y = (screen_height - PREF_CARD_HEIGHT).max(expected_min_y);
    assert_eq!(max_pos.x, expected_max_x);
    assert_eq!(max_pos.y, expected_max_y);
}

#[test]
fn test_preferences_semantic_tags_encoding_roundtrip() {
    // 1. Tab tag roundtrips
    for tab_idx in 0..=9 {
        let tag = encode_tab_tag(tab_idx);
        assert!(is_preferences_tag(tag));
        assert_eq!(parse_tab_tag(tag), Some(tab_idx));
    }
    assert_eq!(parse_tab_tag(0x1234), None);

    // 2. Section tag roundtrips
    for &sec in &super::types::PREF_SECTIONS {
        let tag = encode_section_tag(sec);
        assert!(is_preferences_tag(tag));
        assert_eq!(parse_section_tag(tag), Some(sec));
    }

    // 3. Toggle tag roundtrips
    let toggles = [
        PreferencesToggleId::LiveUpdatesEnabled,
        PreferencesToggleId::Module(ae_core::modules::EngineModule::Physics),
    ];
    for &toggle in &toggles {
        let tag = encode_toggle_tag(toggle);
        assert!(is_preferences_tag(tag));
        assert_eq!(parse_toggle_tag(tag), Some(toggle));
    }
}

#[test]
fn test_preferences_dialog_builder_and_declarative_scope() {
    let mut tree = UiTree::new();
    let _root_id = tree.create_root().expect("Root node creation must succeed");

    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(400.0, 200.0)),
        active_tab: &mut 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(420.0, 215.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_widget_id, card_rect, content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Verify card geometry
    assert_eq!(card_rect.x, 400.0);
    assert_eq!(card_rect.y, 200.0);
    assert_eq!(card_rect.width, PREF_CARD_WIDTH);
    assert_eq!(card_rect.height, PREF_CARD_HEIGHT);

    // Verify content view bounds
    assert_eq!(content_rect.x, 400.0 + super::builder::SIDEBAR_WIDTH + 1.0);
    assert_eq!(content_rect.y, 200.0 + TITLEBAR_HEIGHT);
    assert_eq!(
        content_rect.width,
        PREF_CARD_WIDTH - super::builder::SIDEBAR_WIDTH - 1.0
    );
    assert_eq!(content_rect.height, PREF_CARD_HEIGHT - TITLEBAR_HEIGHT);

    // General tab (tab 0) virtual height (680.0) exceeds content height -> scrollable
    assert_eq!(max_scroll_y, 176.0);

    // Verify hit testing: Titlebar contains point for dragging
    let title_point = Point::new(450.0, 215.0);
    let title_hit = tree
        .hit_test_target(title_point)
        .expect("Titlebar must be hit at (450, 215)");
    assert_eq!(title_hit.tag, PREF_TAG_TITLEBAR);

    // Verify hit testing: Close button
    let close_point = Point::new(400.0 + PREF_CARD_WIDTH - 20.0, 215.0);
    let close_hit = tree
        .hit_test_target(close_point)
        .expect("Close button must be hit");
    assert_eq!(close_hit.tag, PREF_TAG_CLOSE);

    // Verify all 10 sidebar navigation tabs are present in UiTree with their semantic tags
    for &(label, tab_idx) in &SIDEBAR_TABS {
        let expected_tag = encode_tab_tag(tab_idx);
        let found = tree
            .iter()
            .any(|(_id, node)| node.tag == expected_tag || node.text.as_deref() == Some(label));
        assert!(
            found,
            "Sidebar tab {} ({}) must exist in declarative UiTree",
            label, tab_idx
        );
    }

    // Verify TabAccentBar nodes exist for sidebar tabs
    let accent_bar_count = tree
        .iter()
        .filter(|(_id, node)| {
            node.name.as_deref() == Some("TabAccentBar")
                || node.name.as_deref() == Some("PrefTabAccentBar")
        })
        .count();
    assert_eq!(
        accent_bar_count,
        SIDEBAR_TABS.len(),
        "Each sidebar tab must contain a dedicated TabAccentBar node"
    );

    // Verify Titlebar Gear Icon from Texture Atlas
    let has_title_icon = tree
        .iter()
        .any(|(_id, node)| node.texture_uv == Some(crate::ui::iris_bridge::icons::ICON_GEAR));
    assert!(
        has_title_icon,
        "Preferences Titlebar must display the canonical ICON_GEAR from the texture atlas"
    );

    // Verify Titlebar Gap between Icon and Text
    let has_title_gap = tree
        .iter()
        .any(|(_id, node)| node.name.as_deref() == Some("PrefTitleGap"));
    assert!(
        has_title_gap,
        "Preferences Titlebar must contain PrefTitleGap spacer between icon and text"
    );

    // Verify TabSpacer nodes exist for sidebar tabs
    let spacer_count = tree
        .iter()
        .filter(|(_id, node)| {
            node.name.as_deref() == Some("TabSpacer")
                || node.name.as_deref() == Some("PrefTabSpacer")
        })
        .count();
    assert_eq!(
        spacer_count,
        SIDEBAR_TABS.len(),
        "Each sidebar tab must contain a dedicated TabSpacer node for 14px left padding"
    );
}

#[test]
fn test_preferences_combobox_mounting() {
    let mut tree = UiTree::new();
    let _root_id = tree.create_root().expect("Root node creation must succeed");

    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    assert_eq!(graphics_settings.fps_limit, FpsLimit::Limit120);

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(400.0, 200.0)),
        active_tab: &mut 1,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(600.0, 300.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_widget_id, _card_rect, _content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    assert!(max_scroll_y > 0.0, "Graphics tab must require scrolling");
    assert!(
        tree.iter().count() > 20,
        "Preferences UiTree must contain declarative nodes mounted for Graphics tab"
    );
}

#[test]
fn test_preferences_scroll_translates_children_upward() {
    let mut tree_0 = UiTree::new();
    let mut tree_100 = UiTree::new();

    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut gs_0 = GraphicsSettings::default();
    let mut gs_100 = GraphicsSettings::default();
    let mut snapping_settings_0 = SnapSettings::default();
    let mut editor_config_0 = EditorConfig::default();
    let mut snapping_settings_100 = SnapSettings::default();
    let mut editor_config_100 = EditorConfig::default();

    let params_0 = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs_0,
        snapping_settings: &mut snapping_settings_0,
        editor_config: &mut editor_config_0,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let params_100 = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs_100,
        snapping_settings: &mut snapping_settings_100,
        editor_config: &mut editor_config_100,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree_0, params_0, &mut Vec::new());
    tree_100.set_scroll_offset(super::types::PREF_TAG_CONTENT_VIEW, 100.0);
    build_preferences_dialog(&mut tree_100, params_100, &mut Vec::new());

    // Find the Shadows section card in both trees
    let tag = encode_section_tag("graphics_shadows");
    let node_0 = tree_0
        .iter()
        .find(|(_, n)| n.tag == tag)
        .map(|(_, n)| n.computed_rect);
    let node_100 = tree_100
        .iter()
        .find(|(_, n)| n.tag == tag)
        .map(|(_, n)| n.computed_rect);

    assert!(
        node_0.is_some(),
        "Shadows section card must exist at scroll 0"
    );
    assert!(
        node_100.is_some(),
        "Shadows section card must exist at scroll 100"
    );

    let rect_0 = node_0.unwrap();
    let rect_100 = node_100.unwrap();

    // The node computed Y must shift upward by exactly 100 pixels
    assert!(
        (rect_0.y - rect_100.y - 100.0).abs() < 1e-3,
        "Expected scroll shift of 100px upward, got rect_0.y={}, rect_100.y={}",
        rect_0.y,
        rect_100.y
    );
}

#[test]
fn test_preferences_declarative_text_wrap_and_scope_purity() {
    let mut tree = UiTree::new();
    let collapsed = HashSet::new();
    let mut enabled_modules = HashSet::new();
    enabled_modules.insert(ae_core::modules::EngineModule::Physics);
    let mut graphics_settings = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 9, // Modules tab
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, _card_rect, _content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Dynamic scroll measurement should compute a non-negative scroll limit
    assert!(
        max_scroll_y >= 0.0,
        "Dynamic layout should produce valid max_scroll_y >= 0"
    );

    // Verify all ModuleDesc text nodes have TextWrap::Word set declaratively
    let module_desc_nodes: Vec<_> = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("ModuleDesc"))
        .collect();

    assert!(
        !module_desc_nodes.is_empty(),
        "ModuleDesc nodes must be generated in the Modules tab"
    );

    for (_id, node) in module_desc_nodes {
        assert_eq!(
            node.text_wrap,
            TextWrap::Word,
            "Module description labels must have TextWrap::Word via declarative UiScope"
        );
    }
}

#[test]
fn test_preferences_two_way_data_binding_shadows_and_bloom() {
    let mut tree = UiTree::new();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    // Invert initial state to verify mutation
    gs.shadow_enabled = false;
    gs.bloom_enabled = false;
    gs.fog_enabled = false;
    gs.shadow_bias = 0.001;
    gs.bloom_intensity = 0.5;
    gs.sun_pitch = 0.0;
    gs.cloud_coverage = 0.2;

    // Collect tags for checkboxes and sliders across graphics cards
    let tag_shadows = hash_label("Enable Directional Shadows");
    let tag_bloom = hash_label("Enable Bloom");
    let tag_fog = hash_label("Enable Atmospheric Depth Fog");
    let tag_bias = hash_label("Depth Bias");
    let tag_pitch = hash_label("Sun Pitch");
    let tag_clouds = hash_label("Cloud Coverage");

    let events = [
        (
            tag_shadows,
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            tag_bloom,
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            tag_fog,
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            tag_bias,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            tag_pitch,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            tag_clouds,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
    ];

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics tab
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
        events: &events,
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Verify two-way data binding flipped boolean values in-place!
    assert!(
        gs.shadow_enabled,
        "Shadows enabled must be toggled from false to true in-place"
    );
    assert!(
        gs.bloom_enabled,
        "Bloom enabled must be toggled from false to true in-place"
    );
    assert!(
        gs.fog_enabled,
        "Fog enabled must be toggled from false to true in-place"
    );
    assert!(
        (gs.shadow_bias - (0.001 + 10.0 * 0.0005)).abs() < 1e-4,
        "Shadow bias must be updated in-place by drag delta: expected ~0.006, got {}",
        gs.shadow_bias
    );
    assert!(
        (gs.sun_pitch - 5.0_f32.to_radians()).abs() < 1e-4,
        "Sun pitch must be updated in-place by drag delta: expected ~5 deg in radians, got {}",
        gs.sun_pitch
    );
    assert!(
        (gs.cloud_coverage - (0.2 + 10.0 * 0.01)).abs() < 1e-4,
        "Cloud coverage must be updated in-place by drag delta: expected ~0.3, got {}",
        gs.cloud_coverage
    );
}

#[test]
fn test_preferences_two_way_data_binding_editor_tab() {
    let mut tree = UiTree::new();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    // Initial values
    snapping_settings.grid_size = 1.0;
    editor_config.physics_hz = 60.0;
    editor_config.max_undo_history = 100;

    let tag_grid_size = hash_label("Grid Size");
    let tag_physics = hash_label("Fixed Update Frequency");
    let tag_undo = hash_label("Undo History Limit");

    let events = [
        (
            tag_grid_size,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            tag_physics,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            tag_undo,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
    ];

    let params = PreferencesParams {
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
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &events,
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Verify two-way data binding mutated Editor values in-place!
    // Grid Size: speed = 0.05, delta = 10.0 -> 1.0 + 0.5 = 1.5
    assert!(
        (snapping_settings.grid_size - 1.5).abs() < 1e-4,
        "Grid size must be updated in-place: expected 1.5, got {}",
        snapping_settings.grid_size
    );
    // Physics Hz: speed = 1.0, delta = 10.0 -> 60.0 + 10.0 = 70.0
    assert!(
        (editor_config.physics_hz - 70.0).abs() < 1e-4,
        "Physics frequency must be updated in-place: expected 70.0, got {}",
        editor_config.physics_hz
    );
    // Undo limit: speed = 10.0, delta = 10.0 -> 100 + 100 = 200
    assert_eq!(
        editor_config.max_undo_history, 200,
        "Max undo history must be updated in-place: expected 200, got {}",
        editor_config.max_undo_history
    );
}

#[test]
fn test_preferences_graphics_slider_click_to_type_and_selection_highlight() {
    let mut tree = UiTree::new();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    // 1. Initial State: Graphics tab with active text input on Sun Pitch number box
    gs.sun_pitch = 0.50;
    let tag_pitch = hash_label("Sun Pitch");
    let num_box_pitch = irisui::widgets::declarative::hash_label_with_seed(tag_pitch, "##num_box");

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics tab
        collapsed_sections: &collapsed,
        blink_caret: true,
        active_number_input: Some((num_box_pitch, "0.50", true)), // Selected on click
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree, params, &mut Vec::new());

    // Verify SliderNumberBox in Graphics tab renders active selection highlight
    let num_box = tree
        .iter()
        .find(|(_, n)| n.tag == num_box_pitch)
        .map(|(_, n)| n)
        .expect("Sun Pitch number box must exist");

    assert_eq!(
        num_box.style.background_color,
        Color::rgba(0.0, 0.40, 0.70, 0.85),
        "Number box must have active blue selection background when all selected"
    );

    // 2. Direct Typed Input: user enters "45.0" degrees
    let mut tree_type = UiTree::new();
    let events = [(
        num_box_pitch,
        InteractionEvent::TextInput {
            text: "45.0".to_string(),
        },
    )];

    let params_typed = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics tab
        collapsed_sections: &collapsed,
        blink_caret: true,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &events,
        current_theme: Default::default(),
    };

    build_preferences_dialog(&mut tree_type, params_typed, &mut Vec::new());

    // Verify two-way data binding parsed and mutated gs.sun_pitch in-place!
    assert!(
        (gs.sun_pitch - 45.0_f32.to_radians()).abs() < 1e-4,
        "Sun pitch must be updated directly via typed degree input: expected 45 deg in radians, got {}",
        gs.sun_pitch
    );
}

#[test]
fn test_preferences_scrollbar_thumb_hit_test_and_layer() {
    let mut tree = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let enabled_modules = HashSet::new();
    let collapsed = HashSet::new();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics tab (overflows content height)
        collapsed_sections: &collapsed,
        blink_caret: true,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, _card_rect, _content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    assert!(max_scroll_y > 0.0, "Graphics tab must be scrollable");

    let thumb_id = tree
        .find_node_by_tag(super::types::PREF_TAG_SCROLLBAR_THUMB)
        .expect("Scrollbar thumb node must exist in tree");

    let thumb_node = tree.get(thumb_id).expect("Thumb node must exist");
    assert_eq!(
        thumb_node.layer,
        irisui::core::UiLayer::Floating,
        "Scrollbar thumb must reside on Floating layer matching the preferences card"
    );
    assert_eq!(
        thumb_node.parent,
        Some(_card_id),
        "Scrollbar thumb must be a child of card_id to move with window dragging"
    );
    let thumb_center = Point::new(
        thumb_node.computed_rect.x + thumb_node.computed_rect.width * 0.5,
        thumb_node.computed_rect.y + thumb_node.computed_rect.height * 0.5,
    );

    let hit = tree.hit_test_target(thumb_center);
    assert_eq!(
        hit.map(|h| h.tag),
        Some(super::types::PREF_TAG_SCROLLBAR_THUMB),
        "Clicking scrollbar thumb center must hit PREF_TAG_SCROLLBAR_THUMB, not card background or None"
    );
}

#[test]
fn test_preferences_mouse_wheel_scroll_event_handling() {
    let mut tree = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let enabled_modules = HashSet::new();
    let collapsed = HashSet::new();

    tree.set_scroll_offset(super::types::PREF_TAG_CONTENT_VIEW, 150.0);
    let (_card_id, card_rect, content_rect, max_scroll_y) = build_preferences_dialog(
        &mut tree,
        PreferencesParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            window_pos: None,
            active_tab: &mut 1, // Graphics tab
            collapsed_sections: &collapsed,
            active_number_input: None,
            blink_caret: false,
            cursor_pos: Point::new(750.0, 400.0),
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snapping_settings,
            editor_config: &mut editor_config,
            enable_live_updates: false,
            enabled_modules: &enabled_modules,
            events: &[],
            current_theme: Default::default(),
        },
        &mut Vec::new(),
    );

    let (_card_id_unscrolled, _, _, _) = build_preferences_dialog(
        &mut UiTree::new(),
        PreferencesParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            window_pos: None,
            active_tab: &mut 1,
            collapsed_sections: &collapsed,
            active_number_input: None,
            blink_caret: false,
            cursor_pos: Point::new(750.0, 400.0),
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snapping_settings,
            editor_config: &mut editor_config,
            enable_live_updates: false,
            enabled_modules: &enabled_modules,
            events: &[],
            current_theme: Default::default(),
        },
        &mut Vec::new(),
    );

    assert!(max_scroll_y > 0.0);
    assert_eq!(card_rect.width, PREF_CARD_WIDTH);
    assert_eq!(content_rect.height, PREF_CARD_HEIGHT - TITLEBAR_HEIGHT);
    let tag = encode_section_tag("graphics_shadows");
    let y_scrolled = tree
        .iter()
        .find(|(_, n)| n.tag == tag)
        .unwrap()
        .1
        .computed_rect
        .y;
    assert!(y_scrolled > 0.0);
}

#[test]
fn test_sync_scroll_in_place_translates_children() {
    let mut tree = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let enabled_modules = HashSet::new();
    let collapsed = HashSet::new();

    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 1, // Graphics tab
        collapsed_sections: &collapsed,
        blink_caret: true,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
        current_theme: Default::default(),
    };

    let (_card_id, _card_rect, _content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params, &mut Vec::new());

    let tag = encode_section_tag("graphics_shadows");
    let y_before = tree
        .iter()
        .find(|(_, n)| n.tag == tag)
        .unwrap()
        .1
        .computed_rect
        .y;

    // Now call ScrollArea::sync_scroll_in_place
    let updated = ScrollArea::sync_scroll_in_place(
        &mut tree,
        super::types::PREF_TAG_CONTENT_VIEW,
        Some(super::types::PREF_TAG_SCROLLBAR_TRACK),
        Some(super::types::PREF_TAG_SCROLLBAR_THUMB),
        100.0,
        max_scroll_y,
    );

    assert!(updated, "sync_scroll_in_place must return true");

    let y_after = tree
        .iter()
        .find(|(_, n)| n.tag == tag)
        .unwrap()
        .1
        .computed_rect
        .y;
    eprintln!(
        "Before: {}, After: {}, Delta: {}",
        y_before,
        y_after,
        y_before - y_after
    );
    assert_eq!(y_after, y_before - 100.0);

    // Also verify text sections collected from overlay tree
    let options = irisui::text::TextCollectionOptions::default();
    let sections = irisui::text::collect_text_sections_with_options(&tree, &options);
    for s in &sections {
        eprintln!("Section text: '{}', bounds: {:?}", s.text, s.bounds);
    }
    let shadow_text = sections
        .iter()
        .find(|s| s.text.contains("Shadows"))
        .expect("Shadows text must exist");
    eprintln!("Shadows text section y: {}", shadow_text.bounds.y);
    assert!(
        shadow_text.bounds.y < 350.0,
        "Shadows text must be shifted up by scroll"
    );
}

#[test]
fn test_preferences_mouse_wheel_and_drag() {
    let mut tree = UiTree::new();
    let screen_width = 1920.0;
    let screen_height = 1080.0;
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();
    let enabled_modules = HashSet::new();
    let collapsed = HashSet::new();

    let (_card_id, _card_rect, _content_rect, max_scroll_y) = build_preferences_dialog(
        &mut tree,
        PreferencesParams {
            screen_width,
            screen_height,
            window_pos: None,
            active_tab: &mut 1, // Graphics tab
            collapsed_sections: &collapsed,
            active_number_input: None,
            blink_caret: false,
            cursor_pos: Point::new(500.0, 500.0),
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snapping_settings,
            editor_config: &mut editor_config,
            enable_live_updates: false,
            enabled_modules: &enabled_modules,
            events: &[],
            current_theme: Default::default(),
        },
        &mut Vec::new(),
    );

    assert!(
        max_scroll_y > 100.0,
        "Graphics tab must have max_scroll_y > 100, got {}",
        max_scroll_y
    );

    let tag = encode_section_tag("graphics_shadows");
    let y_at_0 = tree
        .iter()
        .find(|(_, n)| n.tag == tag)
        .unwrap()
        .1
        .computed_rect
        .y;

    // Now build with scroll_offset_y: 150.0 via tree.set_scroll_offset
    let mut tree_scrolled = UiTree::new();
    tree_scrolled.set_scroll_offset(super::types::PREF_TAG_CONTENT_VIEW, 150.0);
    let (_card_id, _card_rect, _content_rect, _max_scroll_y) = build_preferences_dialog(
        &mut tree_scrolled,
        PreferencesParams {
            screen_width,
            screen_height,
            window_pos: None,
            active_tab: &mut 1, // Graphics tab
            collapsed_sections: &collapsed,
            active_number_input: None,
            blink_caret: false,
            cursor_pos: Point::new(500.0, 500.0),
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snapping_settings,
            editor_config: &mut editor_config,
            enable_live_updates: false,
            enabled_modules: &enabled_modules,
            events: &[],
            current_theme: Default::default(),
        },
        &mut Vec::new(),
    );

    let y_at_150 = tree_scrolled
        .iter()
        .find(|(_, n)| n.tag == tag)
        .unwrap()
        .1
        .computed_rect
        .y;
    eprintln!(
        "y_at_0 = {}, y_at_150 = {}, diff = {}",
        y_at_0,
        y_at_150,
        y_at_0 - y_at_150
    );
    assert_eq!(
        y_at_150,
        y_at_0 - 150.0,
        "Building with scroll_offset_y: 150.0 must shift content up by 150px"
    );
}

#[test]
fn test_preferences_declarative_actions_dispatch() {
    let mut tree = UiTree::new();
    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let mut gs = GraphicsSettings::default();
    let mut snapping_settings = SnapSettings::default();
    let mut editor_config = EditorConfig::default();

    // 1. Test Close button declarative dispatch
    let events_close = [(
        PREF_TAG_CLOSE,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let params_close = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 0,
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
        events: &events_close,
        current_theme: Default::default(),
    };
    let mut actions = Vec::new();
    build_preferences_dialog(&mut tree, params_close, &mut actions);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, PreferencesAction::Close)),
        "Clicking Close button must dispatch PreferencesAction::Close"
    );

    // 2. Test Sidebar tab selection declarative dispatch
    let tab_3_tag = encode_tab_tag(3);
    let events_tab = [(
        tab_3_tag,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let params_tab = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: &mut 0,
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
        events: &events_tab,
        current_theme: Default::default(),
    };
    let mut actions_tab = Vec::new();
    build_preferences_dialog(&mut UiTree::new(), params_tab, &mut actions_tab);
    assert!(
        actions_tab
            .iter()
            .any(|a| matches!(a, PreferencesAction::SelectTab(3))),
        "Clicking tab 3 must dispatch PreferencesAction::SelectTab(3)"
    );

    // 3. Test Titlebar Drag through FloatingPanelState controller
    let mut floating = irisui::widgets::FloatingPanelState::new();
    floating.rect = Some(Rect::new(100.0, 100.0, 760.0, 540.0));
    floating.pos = Some(Point::new(100.0, 100.0));
    let down_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(150.0, 115.0),
    };
    let down_disp = tree.dispatch_event(&down_ev);
    let outcome_down = floating.handle_event(&mut tree, &down_ev, &down_disp, 1920.0, 1080.0);
    assert!(outcome_down.consumed);
    assert!(floating.is_dragging());

    let move_ev = UiEvent::MouseMove {
        point: Point::new(160.0, 130.0),
    };
    let move_disp = tree.dispatch_event(&move_ev);
    let outcome_move = floating.handle_event(&mut tree, &move_ev, &move_disp, 1920.0, 1080.0);
    assert!(outcome_move.consumed);
    assert_eq!(
        outcome_move.action,
        Some(irisui::widgets::FloatingPanelAction::Move(Point::new(
            110.0, 115.0
        )))
    );
}