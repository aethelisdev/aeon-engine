// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Event Routing & Dragging Verification Suite
//!
//! Validates continuous window dragging calculations across panel boundaries and menubar,
//! reliable clamping behavior, 100% declarative semantic tag dispatching, and slider bounds invariants.

use super::builder::{
    PREF_CARD_HEIGHT, PREF_CARD_WIDTH, SIDEBAR_TABS, TITLEBAR_HEIGHT, build_preferences_dialog,
};
use super::types::{
    PREF_TAG_CLOSE, PREF_TAG_TITLEBAR, PreferencesDropdownId, PreferencesParams,
    PreferencesSliderId, PreferencesToggleId, encode_dropdown_item_tag, encode_dropdown_tag,
    encode_number_tag, encode_section_tag, encode_slider_tag, encode_tab_tag, encode_toggle_tag,
    is_preferences_tag, parse_dropdown_item_tag, parse_dropdown_tag, parse_number_tag,
    parse_section_tag, parse_slider_tag, parse_tab_tag, parse_toggle_tag,
};
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

    // 3. Dropdown tag roundtrips
    let dropdowns = [
        PreferencesDropdownId::UiScale,
        PreferencesDropdownId::ShadowResolution,
        PreferencesDropdownId::ShadowCascades,
        PreferencesDropdownId::ShadowPcf,
        PreferencesDropdownId::FpsLimit,
        PreferencesDropdownId::MsaaSamples,
        PreferencesDropdownId::SkyQuality,
        PreferencesDropdownId::SnapMode,
    ];
    for &dd in &dropdowns {
        let tag = encode_dropdown_tag(dd);
        assert!(is_preferences_tag(tag));
        assert_eq!(parse_dropdown_tag(tag), Some(dd));
    }

    // 4. Dropdown item tag roundtrips
    for idx in 0..10 {
        let tag = encode_dropdown_item_tag(idx);
        assert!(is_preferences_tag(tag));
        assert_eq!(parse_dropdown_item_tag(tag), Some(idx));
    }

    // 5. Toggle tag roundtrips
    let toggles = [
        PreferencesToggleId::ShadowsEnabled,
        PreferencesToggleId::BloomEnabled,
        PreferencesToggleId::FogEnabled,
        PreferencesToggleId::LiveUpdatesEnabled,
    ];
    for &toggle in &toggles {
        let tag = encode_toggle_tag(toggle);
        assert!(is_preferences_tag(tag));
        assert_eq!(parse_toggle_tag(tag), Some(toggle));
    }

    // 6. Slider & Number tag roundtrips
    let sliders = [
        PreferencesSliderId::ShadowBias,
        PreferencesSliderId::BloomIntensity,
        PreferencesSliderId::SunPitch,
        PreferencesSliderId::SunYaw,
        PreferencesSliderId::AtmosphereDensity,
        PreferencesSliderId::OzoneDensity,
        PreferencesSliderId::SunDiscSize,
        PreferencesSliderId::SunGlowStrength,
        PreferencesSliderId::CloudCoverage,
        PreferencesSliderId::CloudDensity,
        PreferencesSliderId::CloudSpeed,
        PreferencesSliderId::CloudEvolution,
        PreferencesSliderId::CloudAltitude,
        PreferencesSliderId::FogDistance,
        PreferencesSliderId::GridSize,
        PreferencesSliderId::UndoHistoryLimit,
        PreferencesSliderId::PhysicsFrequency,
    ];
    for &slider in &sliders {
        let s_tag = encode_slider_tag(slider);
        assert!(is_preferences_tag(s_tag));
        assert_eq!(parse_slider_tag(s_tag), Some(slider));

        let n_tag = encode_number_tag(slider);
        assert!(is_preferences_tag(n_tag));
        assert_eq!(parse_number_tag(n_tag), Some(slider));

        // Verify min < max mathematical invariant
        assert!(
            slider.min_val() < slider.max_val(),
            "Slider {:?} min_val must be strictly less than max_val",
            slider
        );
    }
}

#[test]
fn test_preferences_dialog_builder_and_declarative_scope() {
    let mut tree = UiTree::new();
    let _root_id = tree.create_root().expect("Root node creation must succeed");

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
        is_scrollbar_dragging: false,
        active_dropdown: None,
        dropdown_trigger_rect: None,
        collapsed_sections: &collapsed,
        active_number_input: None,
        blink_caret: false,
        cursor_pos: Point::new(420.0, 215.0),
        hovered_tag: None,
        zoom_factor: 1.0,
        graphics_settings: &graphics_settings,
        snapping_settings: &snapping_settings,
        editor_config: &editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
    };

    let (_widget_id, card_rect, content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params);

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

    // General tab (tab 0) virtual height is smaller than content height -> max_scroll_y is 0
    assert_eq!(max_scroll_y, 0.0);

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
        .filter(|(_id, node)| node.name.as_deref() == Some("PrefTabAccentBar"))
        .count();
    assert_eq!(
        accent_bar_count,
        SIDEBAR_TABS.len(),
        "Each sidebar tab must contain a dedicated PrefTabAccentBar node"
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
        .filter(|(_id, node)| node.name.as_deref() == Some("PrefTabSpacer"))
        .count();
    assert_eq!(
        spacer_count,
        SIDEBAR_TABS.len(),
        "Each sidebar tab must contain a dedicated PrefTabSpacer node for 14px left padding"
    );
}

#[test]
fn test_preferences_dropdown_popup_hit_targets_and_item_selection() {
    let mut tree = UiTree::new();
    let _root_id = tree.create_root().expect("Root node creation must succeed");

    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let graphics_settings = GraphicsSettings::default();
    let snapping_settings = SnapSettings::default();
    let editor_config = EditorConfig::default();

    // Open Preferences dialog with Graphics tab (tab 1) and FpsLimit dropdown open
    let params = PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: Some(Point::new(400.0, 200.0)),
        active_tab: 1,
        scroll_offset_y: 0.0,
        is_scrollbar_dragging: false,
        active_dropdown: Some(PreferencesDropdownId::FpsLimit),
        dropdown_trigger_rect: None,
        collapsed_sections: &collapsed,
        active_number_input: None,
        blink_caret: false,
        cursor_pos: Point::new(600.0, 300.0),
        hovered_tag: None,
        zoom_factor: 1.0,
        graphics_settings: &graphics_settings,
        snapping_settings: &snapping_settings,
        editor_config: &editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
    };

    let (_widget_id, _card_rect, _content_rect, max_scroll_y) =
        build_preferences_dialog(&mut tree, params);

    // Graphics tab total content height exceeds viewport height
    assert!(max_scroll_y > 0.0, "Graphics tab must require scrolling");

    // Verify dropdown items exist with semantic tags
    let options = PreferencesDropdownId::FpsLimit.options();
    for (idx, _label) in options.iter().enumerate() {
        let item_tag = encode_dropdown_item_tag(idx);
        let found = tree.iter().any(|(_id, node)| node.tag == item_tag);
        assert!(
            found,
            "Dropdown item {} must exist in declarative UiTree with tag {}",
            idx, item_tag
        );
    }
}

#[test]
fn test_dropdown_options_match_engine_spec() {
    assert_eq!(
        PreferencesDropdownId::ShadowResolution.options(),
        &["Low (512)", "Medium (1024)", "High (2048)", "Ultra (4096)"]
    );
    assert_eq!(
        PreferencesDropdownId::ShadowCascades.options(),
        &["3 Cascades (Default)", "4 Cascades (High Fidelity)"]
    );
    assert_eq!(
        PreferencesDropdownId::ShadowPcf.options(),
        &["Off (Sharp)", "3x3 Soft", "5x5 Ultra Soft"]
    );
    assert_eq!(
        PreferencesDropdownId::FpsLimit.options(),
        &["60 FPS", "120 FPS", "Uncapped"]
    );
    assert_eq!(
        PreferencesDropdownId::MsaaSamples.options(),
        &["Off (1x)", "2x", "4x (Default)"]
    );
    assert_eq!(
        PreferencesDropdownId::SkyQuality.options(),
        &[
            "Low (Gradient)",
            "Medium (Fast HDR)",
            "High (Atmospheric 2.5D)",
        ]
    );
    assert_eq!(
        PreferencesDropdownId::SnapMode.options(),
        &["Off", "Hold (Ctrl)", "Toggle"]
    );
    assert_eq!(PreferencesDropdownId::UiScale.options().len(), 7);
}

#[test]
fn test_preferences_scroll_translates_children_upward() {
    let mut tree_0 = UiTree::new();
    let mut tree_100 = UiTree::new();

    let collapsed = HashSet::new();
    let enabled_modules = HashSet::new();
    let graphics_settings = GraphicsSettings::default();
    let snapping_settings = SnapSettings::default();
    let editor_config = EditorConfig::default();

    let make_params = |scroll_y: f32| PreferencesParams {
        screen_width: 1920.0,
        screen_height: 1080.0,
        window_pos: None,
        active_tab: 1, // Graphics
        scroll_offset_y: scroll_y,
        is_scrollbar_dragging: false,
        active_dropdown: None,
        dropdown_trigger_rect: None,
        collapsed_sections: &collapsed,
        active_number_input: None,
        blink_caret: false,
        cursor_pos: Point::new(0.0, 0.0),
        hovered_tag: None,
        zoom_factor: 1.0,
        graphics_settings: &graphics_settings,
        snapping_settings: &snapping_settings,
        editor_config: &editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
    };

    build_preferences_dialog(&mut tree_0, make_params(0.0));
    build_preferences_dialog(&mut tree_100, make_params(100.0));

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