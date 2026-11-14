// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous Workbench Global Escape Key Resolution
//!
//! Validates hierarchical Escape key dismissal across [`crate::dialogs::host::DialogHost`]
//! modal dialogues, contextual overlay hooks, [`iris_core::UiTree`] tagged modals,
//! popups, tooltips, and widget keyboard focus.

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::dialogs::descriptor::ConfirmationDialogDescriptor;
use crate::hooks::overlay::{OverlayHook, OverlayId};
use iris_core::WidgetRole;
use iris_core::event::{KeyCode, UiEvent};
use iris_core::geometry::{Rect, Size};
use iris_core::tree::UiTree;
use iris_dock::state::DockState;
use iris_dock::tree::DockTree;
use iris_widgets::declarative::scope::core::UiScope;
use iris_winit::CursorIcon;
use iris_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MockTab {
    Viewport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MockAction {
    ModalConfirmed,
    ModalCancelled,
    OverlayDismissed,
}

#[derive(Debug, Default)]
struct MockAppState {
    overlay_active: bool,
    overlay_dismissed: bool,
}

struct MockOverlayHook;

impl OverlayHook<MockAppState, MockAction> for MockOverlayHook {
    fn id(&self) -> OverlayId {
        OverlayId::new(0x1234)
    }

    fn is_active(&self, state: &MockAppState) -> bool {
        state.overlay_active
    }

    fn dismiss_on_escape(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        _scope: &mut UiScope<'_>,
        _workspace: Rect,
        _state: &mut MockAppState,
        _action_bus: &mut crate::action_bus::ActionBus<MockAction>,
    ) {
    }

    fn on_dismiss(
        &mut self,
        state: &mut MockAppState,
        action_bus: &mut crate::action_bus::ActionBus<MockAction>,
    ) {
        state.overlay_active = false;
        state.overlay_dismissed = true;
        action_bus.emit(MockAction::OverlayDismissed);
    }
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
        if self.logical_size.width > 0.0 {
            self.logical_size
        } else {
            Size::new(1024.0, 768.0)
        }
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn create_mock_workbench() -> Workbench<MockTab, MockAction, MockAppState> {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec![MockTab::Viewport]);
    tree.set_root(root);
    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(tree));
    Workbench::new(descriptor)
}

#[test]
fn test_workbench_escape_key_dismisses_dialog_host_modal() {
    let mut wb = create_mock_workbench();
    assert!(!wb.can_handle_escape(None, None));

    // Open a confirm modal dialog on DialogHost
    let dialog = ConfirmationDialogDescriptor::new(
        "Confirm Delete",
        "Are you sure you want to delete this asset?",
        MockAction::ModalConfirmed,
    )
    .with_cancel_action(MockAction::ModalCancelled);

    wb.dialog_host.open(dialog);
    assert!(wb.dialog_host.is_active());
    assert!(wb.can_handle_escape(None, None));

    // Handle Escape key
    let handled = wb.handle_escape_key(None, None);
    assert!(handled, "Escape key must be handled by active DialogHost");
    assert!(!wb.dialog_host.is_active(), "Modal must be dismissed");

    // Action bus should receive the cancellation action
    let actions: Vec<MockAction> = wb.action_bus.drain().collect();
    assert_eq!(actions, vec![MockAction::ModalCancelled]);
}

#[test]
fn test_workbench_escape_key_dismisses_tree_tagged_modal() {
    let mut wb = create_mock_workbench();
    let mut tree = UiTree::new();

    const ABOUT_MODAL_TAG: u64 = 0x5001;
    tree.open_modal(ABOUT_MODAL_TAG);
    assert!(tree.is_any_modal_open());
    assert!(wb.can_handle_escape(Some(&tree), None));

    // Handle Escape key
    let handled = wb.handle_escape_key(Some(&mut tree), None);
    assert!(handled, "Escape key must be handled by open tagged modal");
    assert!(!tree.is_any_modal_open(), "Tagged modal must be closed");
}

#[test]
fn test_workbench_escape_key_dismisses_open_popup_and_tooltip() {
    let mut wb = create_mock_workbench();
    let mut tree = UiTree::new();

    // Create an active combobox popup in tree
    tree.set_active_combobox(Some(0x1234));
    assert!(tree.has_open_popup());
    assert!(wb.can_handle_escape(Some(&tree), None));

    // Handle Escape key
    let handled = wb.handle_escape_key(Some(&mut tree), None);
    assert!(handled, "Escape key must dismiss open popups");
    assert!(!tree.has_open_popup(), "All popups must be closed");
}

#[test]
fn test_workbench_escape_key_clears_text_edit_and_focus() {
    let mut wb = create_mock_workbench();
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must be created");

    // Create a text input node
    let input_id = tree.create_node();
    if let Some(node) = tree.get_mut(input_id) {
        node.set_role(WidgetRole::TextInput);
    }
    let _ = tree.add_child(root, input_id);

    // Set focus and active text edit
    tree.event_state
        .start_text_edit(Some(input_id), 0x9999, "draft text", 5, None);
    assert!(tree.event_state.has_text_input_focus());
    assert!(wb.can_handle_escape(Some(&tree), None));

    // Handle Escape key
    let handled = wb.handle_escape_key(Some(&mut tree), None);
    assert!(handled, "Escape key must clear active text edit and focus");
    assert!(!tree.event_state.has_text_input_focus());
    assert_eq!(tree.event_state.focused, None);
}

#[test]
fn test_workbench_escape_key_dismisses_registered_overlay_hook() {
    let mut wb = create_mock_workbench();
    wb.overlay_host.register(MockOverlayHook);

    let mut state = MockAppState {
        overlay_active: true,
        overlay_dismissed: false,
    };
    assert!(wb.can_handle_escape(None, Some(&state)));

    // Handle Escape key with AppState
    let handled = wb.handle_escape_key(None, Some(&mut state));
    assert!(handled, "Escape key must dismiss active overlay hook");
    assert!(!state.overlay_active);
    assert!(state.overlay_dismissed);

    let actions: Vec<MockAction> = wb.action_bus.drain().collect();
    assert_eq!(actions, vec![MockAction::OverlayDismissed]);
}

#[test]
fn test_workbench_handle_event_full_auto_consumes_escape() {
    let mut wb = create_mock_workbench();
    let mut window = MockWindow::default();
    let mut tree = UiTree::new();
    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![MockTab::Viewport]);
    dock_tree.set_root(root);
    let mut dock_state = DockState::new(dock_tree);

    // Open a modal tag on tree
    tree.open_modal(0xCAFE);

    let escape_event = UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        is_pressed: true,
        text: None,
        ctrl_held: false,
        shift_held: false,
    };

    let outcome =
        wb.handle_event_full(&mut window, &escape_event, &mut dock_state, Some(&mut tree));

    assert!(outcome.consumed, "Outcome must be consumed");
    assert!(outcome.escape_handled, "Escape must be marked as handled");
    assert!(outcome.layout_changed, "Layout must be marked dirty");
    assert!(!tree.is_any_modal_open(), "Tree modal must be closed");
}

#[test]
fn test_workbench_idle_escape_not_swallowed_when_nothing_active() {
    let mut wb = create_mock_workbench();
    let mut window = MockWindow::default();
    let mut tree = UiTree::new();
    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![MockTab::Viewport]);
    dock_tree.set_root(root);
    let mut dock_state = DockState::new(dock_tree);

    let escape_event = UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        is_pressed: true,
        text: None,
        ctrl_held: false,
        shift_held: false,
    };

    let outcome =
        wb.handle_event_full(&mut window, &escape_event, &mut dock_state, Some(&mut tree));

    assert!(
        !outcome.escape_handled,
        "Idle escape must not report escape_handled"
    );
}