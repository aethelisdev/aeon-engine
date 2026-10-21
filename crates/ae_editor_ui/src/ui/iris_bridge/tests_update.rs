// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained UI update loop, always-rebuild and typography collection tests.
//!

use super::IrisEditorOverlay;
use super::types::IrisChromeState;
use irisui::prelude::{Button, Color, FocusManager, Rect, UiScope, UiTree};

#[test]
fn test_always_rebuild_default_and_setter() {
    let mut chrome = IrisChromeState::default();
    assert!(
        !chrome.always_rebuild,
        "always_rebuild must default to false for retained multi-tree architecture"
    );
    chrome.always_rebuild = true;
    assert!(chrome.always_rebuild);
}

#[test]
fn test_collect_text_sections_with_hover_state() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn_tag = 888u64;

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.add(Button::new("Export").tag(btn_tag));
        scope.finish_layout(Rect::new(0.0, 0.0, 800.0, 600.0));
    }

    // Without hover
    let normal = IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &[], &[]);
    assert_eq!(normal.len(), 1);
    assert_eq!(normal[0].color, Color::rgba(0.0, 0.88, 1.0, 1.0));

    // With hover on btn_tag
    let hovered = IrisEditorOverlay::collect_text_sections_from_tree_with_hover(
        &tree,
        &[],
        &[],
        &[],
        Some(btn_tag),
    );
    assert_eq!(hovered.len(), 1);
    assert_eq!(hovered[0].color, Color::WHITE);
}

#[test]
fn test_focus_manager_text_input_focus() {
    use super::hierarchy::HIERARCHY_TAG_SEARCH_INPUT;

    let mut focus_mgr = FocusManager::new();
    assert!(!focus_mgr.has_focus());
    assert_eq!(focus_mgr.focused_tag(), None);

    focus_mgr.set_focus_tag(HIERARCHY_TAG_SEARCH_INPUT);
    assert!(focus_mgr.has_focus());
    assert_eq!(focus_mgr.focused_tag(), Some(HIERARCHY_TAG_SEARCH_INPUT));
    assert!(focus_mgr.is_tag_focused(HIERARCHY_TAG_SEARCH_INPUT));
    assert!(!focus_mgr.is_tag_focused(99999));

    focus_mgr.clear_focus();
    assert!(!focus_mgr.has_focus());
    assert_eq!(focus_mgr.focused_tag(), None);
}

#[test]
fn test_live_search_query_filtering_sync() {
    use super::hierarchy::{HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT};
    use irisui::prelude::UiTree;

    let mut tree = UiTree::new();
    tree.event_state
        .start_text_edit(None, HIERARCHY_TAG_SEARCH_INPUT, "aadsadsa", 8, false);

    // Verify buffer resolution
    let buf = tree
        .event_state
        .text_edit_buffer(HIERARCHY_TAG_SEARCH_INPUT);
    assert_eq!(buf, Some("aadsadsa"));

    // Verify filtering predicate
    let query_lower = buf.unwrap().trim().to_lowercase();
    let entity_names = ["Ground Plane", "Dynamic Cube", "Sun"];
    let matching: Vec<_> = entity_names
        .iter()
        .filter(|name| name.to_lowercase().contains(&query_lower))
        .copied()
        .collect();
    assert!(
        matching.is_empty(),
        "None of the scene entities should match 'aadsadsa'"
    );

    // Verify clear tag wipes EventState buffer
    tree.event_state.frame_clicked_tag = Some(HIERARCHY_TAG_SEARCH_CLEAR);
    tree.event_state.clear_text_edit();
    assert_eq!(
        tree.event_state
            .text_edit_buffer(HIERARCHY_TAG_SEARCH_INPUT),
        None
    );
}

#[test]
fn test_search_input_focus_blur_lifecycle() {
    use super::assets::ASSETS_TAG_SEARCH_INPUT;
    use super::console::CONSOLE_TAG_SEARCH_INPUT;
    use super::hierarchy::HIERARCHY_TAG_SEARCH_INPUT;
    use irisui::prelude::UiTree;

    let mut tree = UiTree::new();
    assert!(!tree.is_text_input_focused());

    // 1. Assets search editing starts
    tree.event_state
        .start_text_edit(None, ASSETS_TAG_SEARCH_INPUT, "mesh", 4, false);
    assert!(tree.is_text_input_focused());

    // Blur / click outside clears text edit
    tree.event_state.clear_text_edit();
    assert!(
        !tree.is_text_input_focused(),
        "Blurring assets search must immediately release text input focus"
    );

    // 2. Console search editing starts
    tree.event_state
        .start_text_edit(None, CONSOLE_TAG_SEARCH_INPUT, "warn", 4, false);
    assert!(tree.is_text_input_focused());

    tree.event_state.clear_text_edit();
    assert!(
        !tree.is_text_input_focused(),
        "Blurring console search must immediately release text input focus"
    );

    // 3. Hierarchy search editing starts
    tree.event_state
        .start_text_edit(None, HIERARCHY_TAG_SEARCH_INPUT, "cube", 4, false);
    assert!(tree.is_text_input_focused());

    tree.event_state.clear_text_edit();
    assert!(
        !tree.is_text_input_focused(),
        "Blurring hierarchy search must immediately release text input focus"
    );
}

#[test]
fn test_pointer_capture_state_machine() {
    use crate::ui::panel_layout::PanelId;

    let mut chrome = IrisChromeState::default();
    assert_eq!(chrome.pointer_capture, None);

    chrome.pointer_capture = Some(PanelId::AnimationTimeline);
    assert_eq!(chrome.pointer_capture, Some(PanelId::AnimationTimeline));

    chrome.pointer_capture = None;
    assert_eq!(chrome.pointer_capture, None);
}

#[test]
fn test_panel_tree_resolution_and_hit_testing() {
    use irisui::prelude::{Button, Color, Point, Rect, UiScope, UiTree};

    let panel_bounds = Rect::new(100.0, 50.0, 300.0, 200.0);
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut scope = UiScope::new(&mut tree, root);
    scope
        .panel("ConsoleTestRoot")
        .bounds(panel_bounds)
        .tagged(0xCC00_0001)
        .background(Color::BLACK)
        .show(|s| {
            s.add(Button::new("Btn").name("TestBtn").tag(0xCC00_0002));
        });

    // Hit-test target query with screen coordinates: (150.0, 70.0) is inside panel_bounds
    let screen_point = Point::new(150.0, 70.0);
    assert!(panel_bounds.contains_point(screen_point));
    let hit = tree.hit_test_target(screen_point);
    assert!(hit.is_some());

    // Hit-test outside screen bounds
    let outside_point = Point::new(50.0, 20.0);
    assert!(!panel_bounds.contains_point(outside_point));
}

#[test]
fn test_chrome_is_dirty_state_machine() {
    let mut chrome = IrisChromeState {
        last_dimensions: (1920.0, 1080.0),
        last_zoom_factor: 1.0,
        last_floating_count: 0,
        last_has_drag_payload: false,
        ..Default::default()
    };

    // Clean state
    assert!(!chrome.is_dirty((1920.0, 1080.0), 1.0, 0, false));

    // Dimension delta
    assert!(chrome.is_dirty((2560.0, 1440.0), 1.0, 0, false));

    // Zoom delta
    assert!(chrome.is_dirty((1920.0, 1080.0), 1.25, 0, false));

    // Floating count delta
    assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 1, false));

    // Drag payload
    assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 0, true));

    // Explicit layout rebuild
    chrome.needs_layout_rebuild = true;
    assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 0, false));
    chrome.needs_layout_rebuild = false;

    // Always rebuild override
    chrome.always_rebuild = true;
    assert!(chrome.is_dirty((1920.0, 1080.0), 1.0, 0, false));
}

#[test]
fn test_unified_tree_layout_and_layer_compilation() {
    use irisui::layout::LayoutEngine;
    use irisui::prelude::{Button, Size, Style, UiLayer, UiScope, UiTree, WidgetRole};
    use irisui::text::{TextCollectionOptions, collect_text_sections_with_options};
    use irisui::wgpu_backend::{
        DrawCommandList, TreeCompilerOptions, compile_tree_draw_commands_into,
    };

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");

    {
        let mut scope = UiScope::new(&mut tree, root);
        // Shell base
        scope.add(Button::new("TopMenubarBtn").tag(0xBB00_0001));
        // Panel container
        scope.container_tagged(
            "HierarchyContainer",
            Style::new().width(300.0).height(400.0).flex_col(),
            WidgetRole::Default,
            0xAA00_0001,
            |s| {
                s.add(Button::new("EntityRow1").tag(0xAA00_0002));
                s.add(Button::new("EntityRow2").tag(0xAA00_0003));
            },
        );
        // Modal dropdown overlay
        scope.add(
            Button::new("DropdownItem")
                .tag(0xCC00_0001)
                .layer(UiLayer::Modal),
        );
    }

    // Layout computation
    let mut text_system = irisui::text::TextSystem::new();
    text_system.measure_subtree_text(&mut tree, root);
    let mut layout_engine = LayoutEngine::new();
    let layout_res = layout_engine.compute_layout(&mut tree, Size::new(1920.0, 1080.0));
    assert!(layout_res.is_ok(), "Layout computation must succeed");

    // Native tree compilation into DrawCommandList
    let mut command_list = DrawCommandList::new();
    let mut options = TreeCompilerOptions::new();
    compile_tree_draw_commands_into(&tree, root, &mut options, &mut command_list);

    assert!(
        !command_list.quads.is_empty(),
        "Command list must contain quads"
    );

    // Unified typography collection across all layers
    let text_options = TextCollectionOptions::default();
    let sections = collect_text_sections_with_options(&tree, &text_options);
    assert_eq!(
        sections.len(),
        4,
        "Unified text sections must encompass all nodes (menubar + 2 panel rows + modal)"
    );
}