// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Developer Console unit tests.

use super::*;
use crate::ui::types::ConsoleEntry;
use irisui::prelude::*;

fn build_in_host(
    tree: &mut UiTree,
    root: WidgetId,
    bounds: Rect,
    f: impl FnOnce(&mut UiScope<'_>),
) {
    let mut scope = UiScope::new(tree, root);
    scope.panel("ConsoleTestHost").bounds(bounds).show(|p| {
        crate::ui::panel_layout::PanelId::Console
            .surface()
            .expect("Console surface")
            .show(p, f);
    });
}

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

#[test]
fn test_console_contains_ignore_case_and_log_counts() {
    assert!(contains_ignore_case("RendererPipeline", ""));
    assert!(contains_ignore_case("RendererPipeline", "pipeline"));
    assert!(contains_ignore_case("RendererPipeline", "RENDER"));
    assert!(!contains_ignore_case("Short", "LongerNeedle"));
    assert!(contains_ignore_case("Ümit Güneş", "ümit"));

    let entries = vec![
        ConsoleEntry {
            level: log::Level::Error,
            target: "a".to_string(),
            msg: "err".to_string(),
            timestamp: "00:00:00".to_string(),
        },
        ConsoleEntry {
            level: log::Level::Warn,
            target: "a".to_string(),
            msg: "wrn".to_string(),
            timestamp: "00:00:01".to_string(),
        },
        ConsoleEntry {
            level: log::Level::Trace,
            target: "a".to_string(),
            msg: "trc".to_string(),
            timestamp: "00:00:02".to_string(),
        },
    ];
    let counts = ConsoleLogCounts::from_entries(&entries);
    assert_eq!(counts, ConsoleLogCounts::new(3, 1, 1, 0, 1));
    assert_eq!(counts.count_for(ConsoleFilterLevel::Debug), 1);
}

use super::types::ConsoleFilterExt;

#[test]
fn test_console_click_target_routing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let entries = vec![];
    let mut state = ConsolePanelState::default();
    state.search_input.set_text("test");

    build_in_host(&mut tree, root, Rect::new(0.0, 0.0, 600.0, 300.0), |s| {
        build_console_panel(s, &mut state, &entries);
    });

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
    let mut tree = UiTree::new();
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

    let mut state = ConsolePanelState {
        filter: ConsoleFilterLevel::Error,
        ..Default::default()
    };

    build_in_host(&mut tree, root, Rect::new(0.0, 0.0, 400.0, 200.0), |s| {
        build_console_rows(s, &mut state, &entries);
    });
    let max_scroll = tree.max_scroll(CONSOLE_TAG_VIEWPORT);

    assert_eq!(max_scroll, 0.0);
    let row_count = tree
        .iter()
        .filter(|(_, n)| n.tag == CONSOLE_TAG_ROW)
        .count();
    assert_eq!(
        row_count, 1,
        "Exactly 1 matching error log row must be rendered in tree"
    );

    let mut tree_all = UiTree::new();
    let root_all = tree_all.create_root().unwrap();
    state.filter = ConsoleFilterLevel::All;
    build_in_host(
        &mut tree_all,
        root_all,
        Rect::new(0.0, 0.0, 400.0, 200.0),
        |s| {
            build_console_rows(s, &mut state, &entries);
        },
    );
    let all_row_count = tree_all
        .iter()
        .filter(|(_, n)| n.tag == CONSOLE_TAG_ROW)
        .count();
    assert_eq!(
        all_row_count, 2,
        "All matching log rows must be rendered in tree"
    );
}

#[test]
fn test_console_panel_clipping_and_clear_action() {
    let mut tree = UiTree::new();
    let parent_id = tree.create_root().unwrap();

    let entries = vec![];
    let mut state = ConsolePanelState::default();
    build_in_host(
        &mut tree,
        parent_id,
        Rect::new(0.0, 0.0, 500.0, 300.0),
        |s| {
            build_console_panel(s, &mut state, &entries);
        },
    );

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
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let mut entries = Vec::new();
    for i in 0..50 {
        entries.push(ConsoleEntry {
            timestamp: "00:00:00".to_string(),
            level: log::Level::Info,
            target: "test".to_string(),
            msg: format!("Log entry {i}"),
        });
    }

    let mut state = ConsolePanelState {
        auto_scroll: false,
        ..Default::default()
    };
    build_in_host(&mut tree, root, Rect::new(0.0, 0.0, 400.0, 200.0), |s| {
        build_console_panel(s, &mut state, &entries);
    });
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

    let mut state = ConsolePanelState::default();
    build_in_host(&mut tree, root, Rect::new(0.0, 0.0, 600.0, 300.0), |s| {
        build_console_panel(s, &mut state, &[]);
    });

    let (_, clear_node) = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_CLEAR)
        .expect("Clear button must exist in tree");

    assert_eq!(
        clear_node.style.hover_background,
        Some(tree.theme().button_hover_bg),
        "Clear button must have declarative hover background"
    );
    assert_eq!(
        clear_node.hover_text_color,
        Some(tree.theme().text_primary),
        "Clear button must have declarative hover text"
    );

    let (_, search_node) = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_SEARCH_INPUT && n.role == WidgetRole::TextInput)
        .expect("Search input node must exist in tree");

    let expected_style = irisui::widgets::InputBoxStyle::from_theme(&tree.theme());
    assert_eq!(
        search_node.style.hover_background,
        Some(expected_style.hover_background),
        "Search input must have declarative hover background"
    );
    assert_eq!(
        search_node.style.hover_border,
        Some(Border::uniform(1.0, expected_style.hover_border_color)),
        "Search input must have declarative illuminated hover border"
    );
}

#[test]
fn test_console_search_focused_caret_positioning() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");
    tree.caret_blink = true;
    tree.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);

    let mut state = ConsolePanelState::default();
    state.search_input.set_text("error_trace");
    state.search_input.cursor_byte_idx = 5;
    build_in_host(&mut tree, root, Rect::new(0.0, 0.0, 800.0, 300.0), |s| {
        build_console_panel(s, &mut state, &[]);
    });

    let (_, search_box) = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_SEARCH_INPUT && n.interactive)
        .expect("Search box must exist");

    assert_eq!(search_box.tag, CONSOLE_TAG_SEARCH_INPUT);

    let caret_node = search_box
        .children
        .iter()
        .filter_map(|&cid| tree.get(cid))
        .find(|n| n.role == WidgetRole::TextInputCaret)
        .expect("Caret must exist when focused and blinking");

    let expected_offset = measure_text_advance("error", 11.0);
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
fn test_console_declarative_direct_state_mutation() {
    use crate::ui::panel_layout::PanelId;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");
    let bounds = Rect::new(0.0, 0.0, 600.0, 300.0);

    let mut state = ConsolePanelState::default();
    let mut notifier = DeferredNotifier::<PanelId>::default();
    build_in_host(&mut tree, root, bounds, |s| {
        super::panel::build_console_panel_notified(s, &mut state, &[], Some(&mut notifier));
    });
    assert!(!notifier.pending_panels().contains(&PanelId::Console));

    tree.event_state.frame_clicked_tag = Some(CONSOLE_TAG_FILTER_ERROR);
    build_in_host(&mut tree, root, bounds, |s| {
        super::panel::build_console_panel_notified(s, &mut state, &[], Some(&mut notifier));
    });
    assert_eq!(state.filter, ConsoleFilterLevel::Error);
    assert!(notifier.pending_panels().contains(&PanelId::Console));
}

#[test]
fn test_console_scratchpad_tab_rendering_and_interaction() {
    use crate::ui::panel_layout::PanelId;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);

    let mut state = ConsolePanelState {
        active_tab: ConsoleTab::Scratchpad,
        scratchpad_state: MultilineTextState::new("let a = 10;\nlet b = 20;\na + b"),
        ..Default::default()
    };
    build_in_host(&mut tree, root, panel_rect, |s| {
        build_console_panel(s, &mut state, &[]);
    });

    let editor_id = tree
        .find_node_by_tag(CONSOLE_TAG_SCRATCHPAD_EDITOR)
        .expect("Scratchpad editor container must exist");
    let editor_node = tree.get(editor_id).expect("Editor node exists");
    assert_eq!(editor_node.role, WidgetRole::TextInput);
    assert_eq!(editor_node.children.len(), 3);

    assert!(
        tree.find_node_by_tag(CONSOLE_TAG_SCRATCHPAD_CLEAR)
            .is_some(),
        "Scratchpad clear button exists"
    );
    assert!(
        tree.find_node_by_tag(CONSOLE_TAG_SCRATCHPAD_RESET)
            .is_some(),
        "Scratchpad reset button exists"
    );
    assert!(
        tree.find_node_by_tag(CONSOLE_TAG_VIEWPORT).is_none(),
        "Logs viewport must not be rendered when Scratchpad is active"
    );

    let mut layout_engine = irisui::layout::LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, panel_rect.size());
    let clear_rect = tree
        .get(tree.find_node_by_tag(CONSOLE_TAG_SCRATCHPAD_CLEAR).unwrap())
        .unwrap()
        .computed_rect;
    let clear_center = Point::new(
        clear_rect.x + clear_rect.width * 0.5,
        clear_rect.y + clear_rect.height * 0.5,
    );
    assert_eq!(
        tree.hit_test_target(clear_center).map(|h| h.tag),
        Some(CONSOLE_TAG_SCRATCHPAD_CLEAR)
    );
    tree.dispatch_event(&UiEvent::MouseDown {
        point: clear_center,
        button: MouseButton::Left,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        point: clear_center,
        button: MouseButton::Left,
    });
    let rebuild_root = tree.root().unwrap();
    let mut notifier = DeferredNotifier::<PanelId>::default();
    build_in_host(&mut tree, rebuild_root, panel_rect, |s| {
        super::panel::build_console_panel_notified(s, &mut state, &[], Some(&mut notifier));
    });
    assert_eq!(state.scratchpad_state.text, "");
    assert!(notifier.pending_panels().contains(&PanelId::Console));
}

#[test]
fn test_console_auto_scroll_disabled_on_scroll_up_declarative_hook() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation failed");

    let entries = vec![
        ConsoleEntry {
            level: log::Level::Info,
            target: "ae_engine".to_string(),
            msg: "Frame 1".to_string(),
            timestamp: "00:00:01".to_string(),
        },
        ConsoleEntry {
            level: log::Level::Info,
            target: "ae_engine".to_string(),
            msg: "Frame 2".to_string(),
            timestamp: "00:00:02".to_string(),
        },
    ];

    tree.event_state.frame_scrolled_tag = Some(CONSOLE_TAG_VIEWPORT);
    tree.event_state.frame_scroll_delta = Some((0.0, 15.0));

    let mut state = ConsolePanelState::default();
    build_in_host(&mut tree, root, Rect::new(0.0, 0.0, 600.0, 300.0), |s| {
        build_console_panel(s, &mut state, &entries);
    });

    assert!(
        !state.auto_scroll,
        "Auto-scroll must be automatically disabled when scrolled up via the declarative hook"
    );
}

#[test]
fn test_floating_console_toolbar_text_sections_not_culled() {
    use irisui::dock::{DockArea, DockState, DockTree, FloatingWindow, TabViewer};
    use irisui::layout::LayoutEngine;
    use irisui::text::{TextCollectionOptions, TextSystem, collect_text_sections_with_options};

    struct ConsoleFloatingViewer<'a> {
        entries: &'a [ConsoleEntry],
    }

    impl TabViewer<u8> for ConsoleFloatingViewer<'_> {
        fn title(&self, _tab: &u8) -> String {
            "Console".to_string()
        }

        fn ui(&mut self, scope: &mut UiScope<'_>, _tab: &u8) {
            let mut state = ConsolePanelState::default();
            build_console_panel(scope, &mut state, self.entries);
        }
    }

    let entries = vec![ConsoleEntry {
        level: log::Level::Info,
        target: "ae_engine".to_string(),
        msg: "Hello floating console".to_string(),
        timestamp: "12:00:00".to_string(),
    }];

    let mut dock_tree = DockTree::new();
    let leaf = dock_tree.create_leaf(vec![0u8]);
    dock_tree.set_root(leaf);
    let mut dock_state = DockState::new(dock_tree);
    dock_state.floating_windows.push(FloatingWindow::new(
        1,
        "Console",
        Rect::new(870.0, 466.0, 1771.0, 276.0),
        vec![1u8],
    ));

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut viewer = ConsoleFloatingViewer { entries: &entries };

    let frame = {
        let mut scope = UiScope::new(&mut tree, root);
        let frame = DockArea::new(&mut dock_state)
            .bounds(Rect::new(0.0, 26.0, 2516.0, 1392.0))
            .show(&mut scope, &mut viewer);
        scope.finish_layout(Rect::new(0.0, 0.0, 2516.0, 1440.0));
        frame
    };
    text_system.measure_subtree_text(&mut tree, root);
    layout_engine
        .compute_layout(&mut tree, Size::new(2516.0, 1440.0))
        .expect("layout");
    for &(_, cid) in &frame.leaf_containers {
        text_system.measure_subtree_text(&mut tree, cid);
        layout_engine
            .compute_boundary_subtree(&mut tree, cid)
            .expect("boundary layout");
    }

    let sections = collect_text_sections_with_options(&tree, &TextCollectionOptions::new());
    let texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();
    assert!(
        texts.iter().any(|t| t.contains("Logs")),
        "Floating Console toolbar button text 'Logs' missing! Collected texts: {texts:?}"
    );
}

#[test]
fn test_console_toolbar_wraps_without_overlap_in_narrow_pane() {
    use irisui::layout::LayoutEngine;
    use irisui::text::TextSystem;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let narrow_rect = Rect::new(0.0, 0.0, 350.0, 500.0);

    let entries = vec![ConsoleEntry {
        level: log::Level::Info,
        target: "ae_engine".to_string(),
        msg: "Narrow console log row".to_string(),
        timestamp: "12:00:00".to_string(),
    }];
    let mut state = ConsolePanelState::default();
    build_in_host(&mut tree, root, narrow_rect, |s| {
        build_console_panel(s, &mut state, &entries);
    });

    let mut text_system = TextSystem::new();
    let mut layout_engine = LayoutEngine::new();
    text_system.measure_subtree_text(&mut tree, root);
    layout_engine
        .compute_layout(&mut tree, Size::new(350.0, 500.0))
        .expect("narrow console layout");

    let toolbar_id = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_TOOLBAR)
        .map(|(id, _)| id)
        .expect("ConsoleToolbar must exist");
    let toolbar_rect = tree.get(toolbar_id).unwrap().computed_rect;
    assert!(
        toolbar_rect.height > CONSOLE_TOOLBAR_HEIGHT + 20.0,
        "Narrow ConsoleToolbar must wrap onto multiple rows instead of staying 34px tall (got {})",
        toolbar_rect.height
    );

    let item_rects: Vec<Rect> = tree
        .get(toolbar_id)
        .unwrap()
        .children
        .iter()
        .filter_map(|&cid| tree.get(cid))
        .filter(|n| n.interactive && n.computed_rect.width > 0.0 && n.computed_rect.height > 0.0)
        .map(|n| n.computed_rect)
        .collect();
    assert!(item_rects.len() >= 8);
    for (i, a) in item_rects.iter().enumerate() {
        for b in &item_rects[i + 1..] {
            let horiz_overlap = a.x < b.right() && a.right() > b.x;
            let vert_overlap = a.y < b.bottom() && a.bottom() > b.y;
            assert!(
                !(horiz_overlap && vert_overlap),
                "Toolbar items overlapped in narrow pane: {a:?} vs {b:?}"
            );
        }
    }

    let vp_rect = tree
        .iter()
        .find(|(_, n)| n.tag == CONSOLE_TAG_VIEWPORT)
        .map(|(_, n)| n.computed_rect)
        .expect("ConsoleViewport must exist");
    assert!(
        vp_rect.y >= toolbar_rect.bottom() - 0.5,
        "ConsoleViewport must start below wrapped toolbar ({} vs {})",
        vp_rect.y,
        toolbar_rect.bottom()
    );
}