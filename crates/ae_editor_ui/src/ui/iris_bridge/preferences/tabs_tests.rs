// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences All Tabs & Two-Way Data Binding Verification Suite
//!
//! Validates that all ten preference tabs (`0..=9`) render robustly without errors,
//! and that Navigation (Tab 3), Editor (Tab 2), and Input (Tab 7) property sliders
//! mutate [`EditorConfig`] directly via two-way declarative bindings.

use super::builder::{SIDEBAR_WIDTH, build_preferences_view};
use super::types::{
    PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLL_CONTAINER, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PreferencesParams, preferences_scroll_style,
};
use crate::ui::iris_bridge::theme::ThemePreset;
use crate::ui::types::EngineUiAction;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::widgets::declarative::hash_label;
use irisui::workbench::ActionBus;
use std::collections::HashSet;

const TEST_PANEL_RECT: Rect = Rect::new(0.0, 0.0, 760.0, 540.0);

fn build_tab_harness(
    tree: &mut UiTree,
    tab_idx: u8,
    gs: &mut GraphicsSettings,
    snap: &mut SnapSettings,
    cfg: &mut EditorConfig,
    events: &[(u64, InteractionEvent)],
) -> f32 {
    let root = tree.create_root().expect("Root creation must succeed");
    let enabled_modules = HashSet::new();
    let mut cur_tab = tab_idx;
    let mut live_updates = false;
    let mut theme = ThemePreset::default();
    let mut bus = ActionBus::<EngineUiAction>::new();
    let content_w = TEST_PANEL_RECT.width - SIDEBAR_WIDTH - 1.0;

    let mut scope = UiScope::with_tagged_interactions(tree, root, events, None);
    let mut params = PreferencesParams {
        active_tab: &mut cur_tab,
        zoom_factor: 1.0,
        graphics_settings: gs,
        snapping_settings: snap,
        editor_config: cfg,
        enable_live_updates: &mut live_updates,
        enabled_modules: &enabled_modules,
        current_theme: &mut theme,
    };
    scope.container_named(
        "PrefTabHarness",
        Style::new()
            .position_absolute()
            .left(TEST_PANEL_RECT.x)
            .top(TEST_PANEL_RECT.y)
            .width(TEST_PANEL_RECT.width)
            .height(TEST_PANEL_RECT.height)
            .flex_row(),
        |panel| {
            build_preferences_view(
                panel,
                &mut params,
                &mut bus,
                content_w,
                TEST_PANEL_RECT.height,
            );
        },
    );
    scope.finish_layout(TEST_PANEL_RECT);
    scope.sync_and_render_vertical_scrollbar(
        VerticalScrollbarSpec::new(
            PREF_TAG_CONTENT_VIEW,
            PREF_TAG_SCROLL_CONTAINER,
            PREF_TAG_SCROLLBAR_TRACK,
            PREF_TAG_SCROLLBAR_THUMB,
            Rect::new(SIDEBAR_WIDTH + 1.0, 0.0, content_w, TEST_PANEL_RECT.height),
        )
        .min_virtual_height(super::builder::tab_virtual_height(cur_tab))
        .style(preferences_scroll_style()),
    )
}

#[test]
fn test_preferences_all_ten_tabs_render_cleanly() {
    for tab_idx in 0..=9 {
        let mut tree = UiTree::new();
        let mut gs = GraphicsSettings::default();
        let mut snap = SnapSettings::default();
        let mut cfg = EditorConfig::default();

        let max_scroll_y = build_tab_harness(&mut tree, tab_idx, &mut gs, &mut snap, &mut cfg, &[]);

        assert!(
            max_scroll_y >= 0.0,
            "Tab {tab_idx} max_scroll_y must be non-negative"
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
fn test_preferences_two_way_data_binding_navigation_input_and_editor_tabs() {
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig {
        camera_base_speed: 5.0,
        camera_shift_multiplier: 3.0,
        camera_scroll_speed: 1.5,
        mouse_sensitivity: 0.005,
        max_undo_history: 100,
        physics_hz: 60.0,
        autosave_enabled: true,
        autosave_interval_secs: 300.0,
    };

    let nav_events = [
        (
            hash_label("Base Fly Speed"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Shift Boost Multiplier"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Scroll Zoom Speed"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
    ];
    build_tab_harness(
        &mut UiTree::new(),
        3,
        &mut gs,
        &mut snap,
        &mut cfg,
        &nav_events,
    );

    assert!((cfg.camera_base_speed - 7.0).abs() < 1e-4);
    assert!((cfg.camera_shift_multiplier - 4.0).abs() < 1e-4);
    assert!((cfg.camera_scroll_speed - 2.0).abs() < 1e-4);

    let input_events = [(
        hash_label("Look Sensitivity"),
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];
    build_tab_harness(
        &mut UiTree::new(),
        7,
        &mut gs,
        &mut snap,
        &mut cfg,
        &input_events,
    );
    assert!((cfg.mouse_sensitivity - 0.007).abs() < 1e-5);

    let editor_drag = [(
        hash_label("Auto-Save Interval"),
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];
    let mut editor_tree = UiTree::new();
    build_tab_harness(
        &mut editor_tree,
        2,
        &mut gs,
        &mut snap,
        &mut cfg,
        &editor_drag,
    );
    assert!((cfg.autosave_interval_secs - 350.0).abs() < 1e-3);

    let editor_click = [(
        hash_label("Enable Periodic Auto-Save"),
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    build_tab_harness(
        &mut UiTree::new(),
        2,
        &mut gs,
        &mut snap,
        &mut cfg,
        &editor_click,
    );
    assert!(!cfg.autosave_enabled);
}

#[test]
fn test_preferences_keymap_scroll_covers_full_content() {
    let mut tree = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();

    let max_scroll_y = build_tab_harness(&mut tree, 4, &mut gs, &mut snap, &mut cfg, &[]);
    assert!(
        max_scroll_y >= 500.0,
        "Keymap tab max_scroll_y must be at least 500px, got {max_scroll_y}"
    );
}