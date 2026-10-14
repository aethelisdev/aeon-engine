// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Developer Console unit tests.
//!

use super::*;
use crate::ui::types::ConsoleEntry;
use irisui::prelude::*;

#[test]
fn test_console_filter_level_matching() {
    assert!(ConsoleFilterLevel::All.matches(log::Level::Error));
    assert!(ConsoleFilterLevel::All.matches(log::Level::Info));

    assert!(ConsoleFilterLevel::Error.matches(log::Level::Error));
    assert!(!ConsoleFilterLevel::Error.matches(log::Level::Warn));

    assert!(ConsoleFilterLevel::Warn.matches(log::Level::Warn));
    assert!(!ConsoleFilterLevel::Warn.matches(log::Level::Info));

    assert!(ConsoleFilterLevel::Info.matches(log::Level::Info));
    assert!(!ConsoleFilterLevel::Info.matches(log::Level::Debug));

    assert!(ConsoleFilterLevel::Debug.matches(log::Level::Debug));
    assert!(ConsoleFilterLevel::Debug.matches(log::Level::Trace));
    assert!(!ConsoleFilterLevel::Debug.matches(log::Level::Info));
}

use super::types::ConsoleFilterExt;

#[test]
fn test_console_click_target_routing() {
    let mut tree = irisui::prelude::UiTree::new();
    let root = tree.create_root().unwrap();

    let entries = vec![];
    let panel_rect = Rect::new(0.0, 0.0, 600.0, 300.0);
    let params = ConsolePanelParams {
        panel_rect,
        entries: &entries,
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::All,
        search_query: "test",
        auto_scroll: true,
        cursor_pos: Point::new(15.0, 15.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut actions);

    // Verify Clear button exists in tree with correct tag
    let mut clear_node_found = false;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.tag == CONSOLE_TAG_CLEAR {
            clear_node_found = true;
            assert_eq!(node.role, WidgetRole::Button);
        }
    });
    assert!(clear_node_found);
}

#[test]
fn test_console_virtualized_empty_and_matching() {
    let mut tree = irisui::prelude::UiTree::new();
    let root = tree.create_root().unwrap();

    let entries = vec![
        ConsoleEntry {
            level: log::Level::Info,
            target: "ae_engine".to_string(),
            msg: "Engine started".to_string(),
            timestamp: "12:00:00".to_string(),
        },
        ConsoleEntry {
            level: log::Level::Error,
            target: "ae_audio".to_string(),
            msg: "Device disconnected".to_string(),
            timestamp: "12:00:01".to_string(),
        },
    ];

    let params = ConsolePanelParams {
        panel_rect: Rect::new(0.0, 0.0, 400.0, 200.0),
        entries: &entries,
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::Error,
        search_query: "",
        auto_scroll: true,
        cursor_pos: Point::new(50.0, 50.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut scope = UiScope::new(&mut tree, root);
    build_console_rows(&mut scope, &params);
    let max_scroll = tree.max_scroll(CONSOLE_TAG_VIEWPORT);

    // Only 1 log matches Error filter
    assert_eq!(max_scroll, 0.0);
}

#[test]
fn test_console_panel_clipping_and_clear_action() {
    let mut tree = irisui::prelude::UiTree::new();
    let parent_id = tree.create_root().unwrap();

    let entries = vec![];
    let panel_rect = Rect::new(0.0, 0.0, 500.0, 300.0);
    let params = ConsolePanelParams {
        panel_rect,
        entries: &entries,
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::All,
        search_query: "",
        auto_scroll: true,
        cursor_pos: Point::new(15.0, 15.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, parent_id, &params, &mut actions);

    // Find ConsolePanelRoot and ConsoleViewport and verify clip_children is true
    let mut found_root = false;
    let mut found_vp = false;

    tree.traverse_depth_first(parent_id, &mut |_id, node| {
        if node.name.as_deref() == Some("ConsolePanelRoot") {
            assert!(node.style.clip_children);
            found_root = true;
        } else if node.name.as_deref() == Some("ConsoleViewport") {
            assert!(node.style.clip_children);
            found_vp = true;
        }
    });

    assert!(found_root, "ConsolePanelRoot must exist");
    assert!(found_vp, "ConsoleViewport must exist");
}

#[test]
fn test_console_scrollbar_rendered_when_content_overflows() {
    let mut tree = irisui::prelude::UiTree::new();
    let root = tree.create_root().unwrap();

    let mut entries = Vec::new();
    for i in 0..50 {
        entries.push(ConsoleEntry {
            timestamp: "00:00:00".to_string(),
            level: log::Level::Info,
            target: "test".to_string(),
            msg: format!("Log entry {}", i),
        });
    }

    let params = ConsolePanelParams {
        panel_rect: Rect::new(0.0, 0.0, 400.0, 200.0),
        entries: &entries,
        scroll_y: 50.0,
        filter: ConsoleFilterLevel::All,
        search_query: "",
        auto_scroll: false,
        cursor_pos: Point::new(395.0, 80.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut actions);
    let max_scroll = tree.max_scroll(CONSOLE_TAG_VIEWPORT);
    assert!(
        max_scroll > 0.0,
        "max_scroll must be positive when content overflows"
    );

    let mut found_track = false;
    let mut found_thumb = false;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.tag == CONSOLE_TAG_SCROLLBAR_TRACK {
            found_track = true;
            assert!(node.computed_rect.height > 0.0);
        } else if node.tag == CONSOLE_TAG_SCROLLBAR_THUMB {
            found_thumb = true;
            assert!(node.computed_rect.height > 0.0);
        }
    });

    assert!(
        found_track,
        "Console scrollbar track must be rendered when overflowing"
    );
    assert!(
        found_thumb,
        "Console scrollbar thumb must be rendered when overflowing"
    );
}

#[test]
fn test_console_toolbar_buttons_and_search_hover_styling() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");

    let params = ConsolePanelParams {
        panel_rect: Rect::new(0.0, 0.0, 600.0, 300.0),
        entries: &[],
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::All,
        search_query: "",
        auto_scroll: true,
        cursor_pos: Point::new(10.0, 10.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut actions);

    let (_, clear_node) = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_CLEAR)
        .expect("Clear button must exist in tree");

    assert_eq!(
        clear_node.style.hover_background,
        Some(Color::rgba(0.24, 0.29, 0.39, 1.0)),
        "Clear button must have declarative hover background"
    );
    assert_eq!(
        clear_node.hover_text_color,
        Some(Color::WHITE),
        "Clear button must have declarative white hover text"
    );

    let (_, search_node) = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_SEARCH_INPUT && n.role == WidgetRole::TextInput)
        .expect("Search input node must exist in tree");

    assert_eq!(
        search_node.style.hover_background,
        Some(Color::rgba(0.08, 0.09, 0.12, 0.98)),
        "Search input must have declarative hover background"
    );
    assert_eq!(
        search_node.style.hover_border,
        Some(irisui::prelude::Border::uniform(
            1.0,
            Color::rgba(0.35, 0.40, 0.52, 0.95)
        )),
        "Search input must have declarative illuminated hover border"
    );
}

#[test]
fn test_console_toolbar_action_unification() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");
    let panel_rect = Rect::new(0.0, 0.0, 1000.0, 400.0);
    let params = ConsolePanelParams {
        panel_rect,
        entries: &[],
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::All,
        search_query: "existing_query",
        auto_scroll: true,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut actions);
    let mut layout_engine = irisui::layout::LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, Size::new(1000.0, 400.0));

    let get_center = |target_tag: u64| -> Point {
        let (_, node) = tree
            .iter()
            .find(|(_, n)| n.tag == target_tag && n.interactive)
            .unwrap_or_else(|| panic!("Node with tag {:#x} not found", target_tag));
        let r = node.computed_rect;
        Point::new(r.x + r.width * 0.5, r.y + r.height * 0.5)
    };

    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_CLEAR)),
        Some(ConsoleAction::ClearLogs)
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_FILTER_ALL)),
        Some(ConsoleAction::SetFilter(ConsoleFilterLevel::All))
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_FILTER_ERROR)),
        Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Error))
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_FILTER_WARN)),
        Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Warn))
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_FILTER_INFO)),
        Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Info))
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_FILTER_DEBUG)),
        Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Debug))
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_AUTOSCROLL)),
        Some(ConsoleAction::ToggleAutoScroll)
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_SEARCH_CLEAR)),
        Some(ConsoleAction::ClearSearch)
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, get_center(CONSOLE_TAG_SEARCH_INPUT)),
        Some(ConsoleAction::FocusSearch)
    );
    assert_eq!(
        evaluate_console_toolbar_click(&tree, Point::new(9999.0, 9999.0)),
        None
    );
}

#[test]
fn test_console_search_focused_caret_positioning() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 300.0);
    let mut focus_state = FocusState::new();
    focus_state.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
    let params = ConsolePanelParams {
        panel_rect,
        entries: &[],
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::All,
        search_query: "error_trace",
        auto_scroll: true,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: true,
        is_scrollbar_dragging: false,
        focus_state: &focus_state,
        cursor_byte_idx: 5,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut actions);

    let (_, search_box) = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_SEARCH_INPUT && n.interactive)
        .expect("Search box must exist");

    assert_eq!(search_box.tag, CONSOLE_TAG_SEARCH_INPUT);

    // Verify caret exists and is offset by cursor_byte_idx
    let caret_node = search_box
        .children
        .iter()
        .filter_map(|&cid| tree.get(cid))
        .find(|n| n.role == irisui::prelude::WidgetRole::TextInputCaret)
        .expect("Caret must exist when focused and blinking");

    let expected_offset = irisui::prelude::measure_text_advance("error", 11.0);
    let text_start = search_box
        .children
        .iter()
        .filter_map(|&cid| tree.get(cid))
        .find(|n| n.text.as_deref() == Some("error_trace"))
        .expect("Text node must exist")
        .computed_rect
        .x;

    assert_eq!(caret_node.computed_rect.x, text_start + expected_offset);
}

#[test]
fn test_console_declarative_action_generation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");
    let panel_rect = Rect::new(0.0, 0.0, 600.0, 300.0);
    let params = ConsolePanelParams {
        panel_rect,
        entries: &[],
        scroll_y: 0.0,
        filter: ConsoleFilterLevel::All,
        search_query: "",
        auto_scroll: true,
        cursor_pos: Point::new(10.0, 10.0),
        blink_caret: false,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut actions);
    assert!(actions.is_empty());

    tree.event_state.frame_clicked_tag = Some(CONSOLE_TAG_CLEAR);

    let mut clicked_actions = Vec::new();
    build_console_panel(&mut tree, root, &params, &mut clicked_actions);

    assert_eq!(clicked_actions.len(), 1);
    assert_eq!(clicked_actions[0], ConsoleAction::ClearLogs);
}