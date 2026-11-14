// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Outside-Click Popup Dismissal Unit Verification Suite
//!
//! Verifies the declarative outside-click popup dismissal subsystem on [`Workbench`],
//! ensuring that clicks outside open popups, combobox dropdowns, and elevated context menus
//! safely dismiss transient menus, protect background widgets, and trigger layout refresh.

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::tree::UiTree;
use iris_dock::{DockState, DockTree};
use iris_winit::CursorIcon;
use iris_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Viewport,
}

#[derive(Debug, Default)]
struct MockWindow {
    cursor: Option<CursorIcon>,
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

    fn toggle_maximize(&mut self) {}

    fn is_maximized(&self) -> bool {
        false
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

fn create_test_workbench() -> Workbench<TestTab, (), ()> {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec![TestTab::Viewport]);
    tree.set_root(root);
    let desc = WorkbenchDescriptor::new("Test Studio", DockState::new(tree));
    Workbench::new(desc)
}

#[test]
fn test_workbench_can_dismiss_popups_on_outside_click() {
    let wb = create_test_workbench();

    // 1. None tree returns false
    assert!(!wb.can_dismiss_popups_on_outside_click(None, Point::new(100.0, 100.0)));

    // 2. Empty tree returns false
    let mut tree = UiTree::new();
    assert!(!wb.can_dismiss_popups_on_outside_click(Some(&tree), Point::new(100.0, 100.0)));

    // 3. Tree with open combobox
    tree.set_active_combobox(Some(9999));
    let root = tree.create_root().expect("Root node creation");
    let combo_btn = tree.create_node();
    tree.add_child(root, combo_btn).expect("Add combo button");
    if let Some(node) = tree.get_mut(combo_btn) {
        node.tag = 9999;
        node.computed_rect = Rect::new(50.0, 50.0, 120.0, 32.0);
    }

    // Inside combo trigger -> false (should not dismiss on its own trigger)
    assert!(!wb.can_dismiss_popups_on_outside_click(Some(&tree), Point::new(70.0, 60.0)));

    // Outside combo trigger -> true (should dismiss)
    assert!(wb.can_dismiss_popups_on_outside_click(Some(&tree), Point::new(300.0, 300.0)));
}

#[test]
fn test_workbench_dismiss_popups_on_outside_click() {
    let mut wb = create_test_workbench();
    let mut tree = UiTree::new();

    // 1. No open popups -> false
    assert!(!wb.dismiss_popups_on_outside_click(Some(&mut tree), Point::new(200.0, 200.0)));

    // 2. Open combobox dismissed on outside click
    tree.set_active_combobox(Some(42));
    assert!(tree.has_open_popup());
    assert!(wb.dismiss_popups_on_outside_click(Some(&mut tree), Point::new(500.0, 500.0)));
    assert!(!tree.has_open_popup());
    assert_eq!(tree.active_combobox_tag, None);

    // 3. Registered active popup tag dismissed on outside click
    tree.set_popup_tag_active(1001, true);
    assert!(tree.has_open_popup());
    assert!(tree.is_popup_tag_active(1001));
    assert!(wb.dismiss_popups_on_outside_click(Some(&mut tree), Point::new(500.0, 500.0)));
    assert!(!tree.has_open_popup());
    assert!(!tree.is_popup_tag_active(1001));
}

#[test]
fn test_workbench_handle_outside_click_event_returns_outcome() {
    let mut wb = create_test_workbench();
    let mut tree = UiTree::new();
    tree.set_active_combobox(Some(888));

    // Non-mouse event returns None
    let key_event = UiEvent::Ime(iris_core::event::ImeEvent::Disabled);
    assert!(
        wb.handle_outside_click_event(&key_event, Some(&mut tree))
            .is_none()
    );

    // MouseDown outside returns Some(outcome) with flags set
    let outside_click = UiEvent::MouseDown {
        point: Point::new(800.0, 600.0),
        button: MouseButton::Left,
    };
    let outcome = wb
        .handle_outside_click_event(&outside_click, Some(&mut tree))
        .expect("Outside click produces outcome");

    assert!(outcome.consumed);
    assert!(outcome.popups_dismissed);
    assert!(outcome.layout_changed);
    assert_eq!(tree.active_combobox_tag, None);
}

#[test]
fn test_workbench_handle_event_full_auto_dismisses_popups_on_outside_click() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow::default();
    let mut dock_state = wb.descriptor.dock_state.clone();
    let mut tree = UiTree::new();

    let root = tree.create_root().expect("Root node creation");
    let popup_node = tree.create_node();
    tree.add_child(root, popup_node).expect("Add child popup");
    if let Some(node) = tree.get_mut(popup_node) {
        node.tag = 555;
        node.layer = UiLayer::Popup;
        node.role = WidgetRole::DropdownPopup;
        node.computed_rect = Rect::new(100.0, 100.0, 160.0, 120.0);
    }
    tree.set_popup_tag_active(555, true);

    // 1. MouseDown INSIDE popup boundary does NOT trigger outside click dismissal
    let inside_click = UiEvent::MouseDown {
        point: Point::new(150.0, 150.0),
        button: MouseButton::Left,
    };
    let inside_outcome =
        wb.handle_event_full(&mut win, &inside_click, &mut dock_state, Some(&mut tree));
    assert!(!inside_outcome.popups_dismissed);
    assert!(tree.is_popup_tag_active(555));

    // 2. MouseDown OUTSIDE popup boundary triggers outside-click dismissal
    let outside_click = UiEvent::MouseDown {
        point: Point::new(600.0, 600.0),
        button: MouseButton::Left,
    };
    let outside_outcome =
        wb.handle_event_full(&mut win, &outside_click, &mut dock_state, Some(&mut tree));
    assert!(outside_outcome.consumed);
    assert!(outside_outcome.popups_dismissed);
    assert!(outside_outcome.layout_changed);
    assert!(!tree.is_popup_tag_active(555));
}

#[test]
fn test_workbench_outside_click_preserves_clicks_on_child_of_popup_trigger() {
    let mut wb = create_test_workbench();
    let mut tree = UiTree::new();

    let root = tree.create_root().expect("Root node");
    let trigger_btn = tree.create_node();
    let child_label = tree.create_node();

    tree.add_child(root, trigger_btn).expect("Add trigger");
    tree.add_child(trigger_btn, child_label)
        .expect("Add child label");

    if let Some(btn) = tree.get_mut(trigger_btn) {
        btn.tag = 777;
        btn.computed_rect = Rect::new(100.0, 100.0, 200.0, 40.0);
    }
    if let Some(lbl) = tree.get_mut(child_label) {
        lbl.tag = 0; // Child label is untagged
        lbl.computed_rect = Rect::new(110.0, 110.0, 80.0, 20.0);
    }

    tree.set_active_combobox(Some(777));

    // Clicking child label inside trigger button should resolve ancestor tag 777 and NOT be dismissed
    let label_click_point = Point::new(120.0, 115.0);
    assert!(tree.is_point_inside_popup(label_click_point));
    assert!(!wb.can_dismiss_popups_on_outside_click(Some(&tree), label_click_point));
    assert!(!wb.dismiss_popups_on_outside_click(Some(&mut tree), label_click_point));
    assert_eq!(tree.active_combobox_tag, Some(777));
}