// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Comprehensive Event Coverage Tests for [`Workbench`]
//!
//! Validates all [`UiEvent`] variants, `WorkbenchOutcome` completeness,
//! text-focus global shortcut suppression, and `EventInvalidationRoute` merging.

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::lifecycle::EventInvalidationRoute;
use irisui_core::event::{ImeEvent, KeyCode, MouseButton, UiEvent};
use irisui_core::geometry::{Point, Rect, Size};
use irisui_core::node::{WidgetCursor, WidgetRole};
use irisui_core::style::Style;
use irisui_core::tree::UiTree;
use irisui_dock::chrome::DockChromeFrame;
use irisui_dock::state::DockState;
use irisui_dock::tree::DockTree;
use irisui_winit::CursorIcon;
use irisui_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Inspector,
    Console,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    Zoom(u32),
    OpenPreferences,
    Dropped(Vec<PathBuf>),
}

#[derive(Debug, Default)]
struct MockWindow {
    cursor: Option<CursorIcon>,
    maximized: bool,
    logical_size: Size,
}

impl WindowInterface for MockWindow {
    fn set_cursor(&mut self, icon: CursorIcon) {
        self.cursor = Some(icon);
    }

    fn drag_resize_window(&mut self, _direction: ResizeDirection) -> Result<(), WindowOpError> {
        Ok(())
    }

    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Ok(())
    }

    fn toggle_maximize(&mut self) {
        self.maximized = !self.maximized;
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }

    fn minimize(&mut self) {}

    fn close(&mut self) {}

    fn logical_size(&self) -> Size {
        self.logical_size
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn create_test_workbench() -> Workbench<TestTab, TestAction, ()> {
    let mut dock_tree = DockTree::new();
    let r = dock_tree.create_leaf(vec![TestTab::Inspector, TestTab::Console]);
    dock_tree.set_root(r);

    let mut descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(dock_tree));
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    Workbench::new(descriptor)
        .with_zoom_action(|z| TestAction::Zoom((z * 100.0).round() as u32))
        .with_preferences_action(|| TestAction::OpenPreferences)
        .with_file_drop_action(|paths, _| TestAction::Dropped(paths))
}

#[test]
fn test_workbench_handle_event_covers_all_ui_event_variants() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1280.0, 720.0),
        ..Default::default()
    };

    // 1. MouseMove
    let move_ev = UiEvent::MouseMove {
        point: Point::new(400.0, 300.0),
    };
    let move_out = wb.handle_event(&mut win, &move_ev);
    assert_eq!(move_out.cursor_pos, Some(Point::new(400.0, 300.0)));
    assert!(!move_out.consumed);

    // 2. MouseDown & 3. MouseUp
    let down_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(400.0, 300.0),
    };
    let _ = wb.handle_event(&mut win, &down_ev);
    let up_ev = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(400.0, 300.0),
    };
    let _ = wb.handle_event(&mut win, &up_ev);

    // 4. MouseScroll
    let scroll_ev = UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: 24.0,
        point: Point::new(400.0, 300.0),
    };
    let scroll_out = wb.handle_event(&mut win, &scroll_ev);
    assert_eq!(scroll_out.cursor_pos, Some(Point::new(400.0, 300.0)));

    // 5. KeyboardInput (Ctrl+, triggers Preferences)
    let key_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character(','),
        text: Some(",".into()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let key_out = wb.handle_event(&mut win, &key_ev);
    assert!(key_out.consumed);
    assert!(key_out.toggle_preferences);
    assert!(key_out.shell_dirty);
    assert_eq!(key_out.actions, vec![TestAction::OpenPreferences]);

    // 6. Ime
    let ime_ev = UiEvent::Ime(ImeEvent::Commit("abc".into()));
    let ime_out = wb.handle_event(&mut win, &ime_ev);
    assert!(!ime_out.consumed);

    // 7. FileHovered, 8. FileCancelled, 9. FileDropped
    let hover_ev = UiEvent::FileHovered {
        path: PathBuf::from("assets/hero.glb"),
        point: Point::new(500.0, 360.0),
    };
    let hover_out = wb.handle_event(&mut win, &hover_ev);
    assert_eq!(hover_out.cursor_pos, Some(Point::new(500.0, 360.0)));

    let cancel_out = wb.handle_event(&mut win, &UiEvent::FileCancelled);
    assert!(!cancel_out.consumed);

    let drop_ev = UiEvent::FileDropped {
        paths: vec![PathBuf::from("assets/hero.glb")],
        point: Point::new(500.0, 360.0),
    };
    let drop_out = wb.handle_event(&mut win, &drop_ev);
    assert!(drop_out.consumed);
    assert!(drop_out.shell_dirty);
    assert_eq!(
        drop_out.dropped_files,
        Some((
            vec![PathBuf::from("assets/hero.glb")],
            Point::new(500.0, 360.0)
        ))
    );
    assert_eq!(
        drop_out.actions,
        vec![TestAction::Dropped(vec![PathBuf::from("assets/hero.glb")])]
    );
}

#[test]
fn test_handle_event_with_tree_suppresses_global_shortcuts_during_text_focus() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1280.0, 720.0),
        ..Default::default()
    };

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let leaf = tree.create_node();
    let input = tree.create_node();
    const INPUT_TAG: u64 = 0x55AA;
    if let Some(node) = tree.get_mut(leaf) {
        node.is_boundary = true;
        node.computed_rect = Rect::new(0.0, 30.0, 400.0, 400.0);
    }
    if let Some(node) = tree.get_mut(input) {
        node.tag = INPUT_TAG;
        node.interactive = true;
        node.role = WidgetRole::TextInput;
        node.cursor = Some(WidgetCursor::Text);
        node.computed_rect = Rect::new(20.0, 60.0, 200.0, 28.0);
    }
    let _ = tree.add_child(root, leaf);
    let _ = tree.add_child(leaf, input);
    tree.event_state
        .start_text_edit(Some(input), INPUT_TAG, "cube", 0, None);

    let mut frame = DockChromeFrame::default();
    frame.leaf_containers.push((TestTab::Inspector, leaf));
    wb.dock_frame = Some(frame);

    // Ctrl+, while a UiTree TextInput is actively focused must NOT fire global Preferences shortcut
    let pref_shortcut = UiEvent::KeyboardInput {
        key: KeyCode::Character(','),
        text: Some(",".into()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let out = wb.handle_event_with_tree(&mut win, &pref_shortcut, &mut tree);
    assert!(
        !out.toggle_preferences,
        "Global Ctrl+, shortcut must be suppressed while UiTree text input is focused"
    );
    assert!(
        out.wants_keyboard_input,
        "WorkbenchOutcome must report wants_keyboard_input while UiTree text input is focused"
    );

    // Clicking scroll inside panel yields Panel(Inspector) route, layout_changed, and consumed
    if let Some(node) = tree.get_mut(leaf) {
        node.tag = 0x7777;
        node.style = Style::new().clip_children(true);
        node.content_size = Size::new(400.0, 1200.0);
    }
    let scroll_ev = UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: -30.0,
        point: Point::new(100.0, 150.0),
    };
    let scroll_out = wb.handle_event_with_tree(&mut win, &scroll_ev, &mut tree);
    assert!(scroll_out.consumed);
    assert!(scroll_out.layout_changed);
    assert_eq!(scroll_out.invalidated_panel, Some(TestTab::Inspector));
    assert_eq!(
        scroll_out.invalidation_route,
        Some(EventInvalidationRoute::Panel(TestTab::Inspector))
    );

    // FileHovered -> FileCancelled sets needs_redraw / layout_changed when clearing active hover
    let hover_ev = UiEvent::FileHovered {
        path: PathBuf::from("scene.json"),
        point: Point::new(100.0, 150.0),
    };
    let _ = wb.handle_event_with_tree(&mut win, &hover_ev, &mut tree);
    let cancel_out = wb.handle_event_with_tree(&mut win, &UiEvent::FileCancelled, &mut tree);
    assert!(
        cancel_out.layout_changed,
        "FileCancelled clearing hovered_file must mark layout_changed for redraw"
    );

    // FileDropped over interactive widget preserves outcome.dropped_files even when tree consumes
    let drop_ev = UiEvent::FileDropped {
        paths: vec![PathBuf::from("mesh.glb")],
        point: Point::new(40.0, 70.0),
    };
    let drop_out = wb.handle_event_with_tree(&mut win, &drop_ev, &mut tree);
    assert!(drop_out.consumed);
    assert_eq!(
        drop_out.dropped_files,
        Some((vec![PathBuf::from("mesh.glb")], Point::new(40.0, 70.0)))
    );
}

#[test]
fn test_handle_event_with_tree_preserves_shell_invalidation_when_dock_tab_changes() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1280.0, 720.0),
        ..Default::default()
    };

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let leaf = tree.create_node();
    if let Some(node) = tree.get_mut(leaf) {
        node.is_boundary = true;
        node.computed_rect = Rect::new(0.0, 60.0, 1280.0, 640.0);
    }
    let _ = tree.add_child(root, leaf);
    let mut frame = DockChromeFrame::default();
    frame.leaf_containers.push((TestTab::Inspector, leaf));
    wb.dock_frame = Some(frame);

    // Clicking the second tab inside the dock tab bar (below menubar_height=24.0, near x=130, y=36)
    let tab_click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(140.0, 34.0),
    };
    let out = wb.handle_event_with_tree(&mut win, &tab_click, &mut tree);
    assert!(out.consumed);
    assert!(out.layout_changed);
    assert!(out.shell_dirty);
    assert_eq!(
        out.invalidation_route,
        Some(EventInvalidationRoute::Shell),
        "Dock tab switch must retain Shell invalidation rather than being overwritten by tree None/Panel route"
    );
}

#[test]
fn test_workbench_handle_event_full_preserves_drag_delta_and_cursor_resets() {
    let mut wb = create_test_workbench();
    let mut dock_state = wb.descriptor.dock_state.clone();
    let mut win = MockWindow {
        logical_size: Size::new(1280.0, 720.0),
        ..Default::default()
    };

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let leaf = tree.create_node();
    let num = tree.create_node();
    let btn = tree.create_node();
    const NUM_TAG: u64 = 0x9001;
    const BTN_TAG: u64 = 0x9002;
    if let Some(n) = tree.get_mut(leaf) {
        n.is_boundary = true;
        n.computed_rect = Rect::new(0.0, 40.0, 1280.0, 650.0);
    }
    if let Some(n) = tree.get_mut(num) {
        n.tag = NUM_TAG;
        n.interactive = true;
        n.role = WidgetRole::NumericInput;
        n.cursor = Some(WidgetCursor::Default);
        n.computed_rect = Rect::new(100.0, 100.0, 60.0, 22.0);
    }
    if let Some(n) = tree.get_mut(btn) {
        n.tag = BTN_TAG;
        n.interactive = true;
        n.role = WidgetRole::Button;
        n.cursor = Some(WidgetCursor::Pointer);
        n.computed_rect = Rect::new(220.0, 100.0, 80.0, 24.0);
    }
    let _ = tree.add_child(root, leaf);
    let _ = tree.add_child(leaf, num);
    let _ = tree.add_child(leaf, btn);

    let mut frame = DockChromeFrame::default();
    frame.leaf_containers.push((TestTab::Inspector, leaf));
    wb.dock_frame = Some(frame);

    // 1. Hover button -> OS cursor becomes Pointer; move to neutral space -> OS cursor resets to Default
    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove {
            point: Point::new(240.0, 110.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(win.cursor, Some(CursorIcon::Pointer));

    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove {
            point: Point::new(500.0, 300.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(
        win.cursor,
        Some(CursorIcon::Default),
        "Moving off a custom-cursor widget onto neutral background must reset OS cursor to Default"
    );

    // 2. Press and drag NumericInput across a frame tree.clear() -> delta MUST NOT be zeroed
    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(120.0, 110.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    tree.clear();
    let leaf2 = tree.create_node();
    let num2 = tree.create_node();
    if let Some(n) = tree.get_mut(leaf2) {
        n.is_boundary = true;
        n.computed_rect = Rect::new(0.0, 40.0, 1280.0, 650.0);
    }
    if let Some(n) = tree.get_mut(num2) {
        n.tag = NUM_TAG;
        n.interactive = true;
        n.role = WidgetRole::NumericInput;
        n.cursor = Some(WidgetCursor::Default);
        n.computed_rect = Rect::new(100.0, 100.0, 60.0, 22.0);
    }
    let root2 = tree.create_root().expect("root2");
    let _ = tree.add_child(root2, leaf2);
    let _ = tree.add_child(leaf2, num2);

    let move_out = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove {
            point: Point::new(145.0, 110.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(move_out.cursor, Some(WidgetCursor::EwResize));
    assert_eq!(win.cursor, Some(CursorIcon::EwResize));
    assert_eq!(
        tree.event_state.frame_drag_delta,
        Some(Point::new(25.0, 0.0)),
        "Workbench MouseMove dispatch must preserve previous last_pointer_pos so NumericInput scrub delta is non-zero"
    );

    // 3. Releasing drag outside pill resets OS cursor back to Default
    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(500.0, 300.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(win.cursor, Some(CursorIcon::Default));
}

#[test]
fn test_workbench_dock_tab_chrome_hover_and_click_passthrough() {
    let mut wb = create_test_workbench();
    let mut dock_state = wb.descriptor.dock_state.clone();
    let mut win = MockWindow {
        logical_size: Size::new(1280.0, 720.0),
        ..Default::default()
    };

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let workspace = wb.compute_workspace_rect(1280.0, 720.0);
    let viewer = wb.panel_registry.tab_viewer();
    let dock_style = wb.dock_style;
    let params = irisui_dock::DockChromeParams {
        dock_tree: &dock_state.tree,
        workspace_rect: workspace,
        cursor_pos: Point::new(0.0, 0.0),
        is_cursor_occluded: false,
        is_dragging_splitter: false,
        active_splitter_node: None,
        is_dragging_tab: false,
        viewer: &viewer,
        style: &dock_style,
    };
    let frame = irisui_dock::build_dock_chrome(&mut tree, root, &params);
    assert_eq!(frame.tab_targets.len(), 2);
    let second_tab = frame.tab_targets[1];
    let tab_pt = Point::new(
        second_tab.rect.x + 12.0,
        second_tab.rect.y + second_tab.rect.height * 0.5,
    );

    // Hovering inactive tab hits DockTab in UiTree, updates cursor & marks strip dirty
    let hover_out = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove { point: tab_pt },
        &mut dock_state,
        Some(&mut tree),
    );
    assert!(irisui_dock::is_dock_tab_tag(
        hover_out.hovered_tag().unwrap_or(0)
    ));
    assert_eq!(win.cursor, Some(CursorIcon::Pointer));

    // Clicking inactive tab passes through to DockArea and switches active_tab to 1
    let click_out = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            point: tab_pt,
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert!(click_out.consumed);
    assert!(click_out.shell_dirty);
    let active_tab = match dock_state.tree.get(second_tab.leaf) {
        Some(irisui_dock::DockNode::Leaf { active_tab, .. }) => Some(*active_tab),
        _ => None,
    };
    assert_eq!(
        active_tab,
        Some(1),
        "Clicking interactive IrisDockTab must pass through to DockArea tab switcher"
    );
}