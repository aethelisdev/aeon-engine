// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Modal Dialogue Focus Trap & Event Containment
//!
//! Verifies pointer hit-test occlusion, keyboard event containment, shortcut isolation,
//! non-cancellable barrier preservation, and modal layer protection across [`DialogHost`].

use crate::action_bus::ActionBus;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::dialogs::descriptor::sheet::SheetAnchor;
use crate::dialogs::host::stack::DialogHost;
use crate::dialogs::host::trap::DialogFocusTrap;
use crate::workbench::runner::Workbench;
use iris_core::event::KeyCode;
use iris_core::geometry::{Point, Rect};
use iris_dock::state::DockState;
use iris_dock::tree::DockTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MockTab {
    Viewport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    DeleteConfirmed,
    DeleteCancelled,
    SubmittedInput(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkbenchShortcut {
    SaveScene,
    DeleteSelection,
    DuplicateEntity,
}

/// Helper creating a minimal test workbench with an interactive 3D viewport canvas.
fn create_test_workbench() -> Workbench<MockTab, TestAction, ()> {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec![MockTab::Viewport]);
    tree.set_root(root);
    let mut descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(tree));
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    let mut wb = Workbench::new(descriptor);
    wb.set_interactive_viewport(Some(Rect::new(300.0, 30.0, 1300.0, 900.0)));
    wb
}

#[test]
fn test_dialog_focus_trap_evaluators() {
    // 1. Depth evaluators
    assert!(!DialogFocusTrap::is_trapped(0));
    assert!(DialogFocusTrap::is_trapped(1));
    assert!(DialogFocusTrap::is_trapped(5));

    // 2. Occlusion zone geometry
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let inside_point = Point::new(960.0, 540.0);
    let outside_point = Point::new(2000.0, 500.0);

    // Inactive state never occludes
    assert!(!DialogFocusTrap::occludes_point(
        false,
        inside_point,
        viewport
    ));
    assert!(!DialogFocusTrap::occludes_point(
        false,
        outside_point,
        viewport
    ));

    // Active state occludes any point falling within the viewport bounds
    assert!(DialogFocusTrap::occludes_point(
        true,
        inside_point,
        viewport
    ));
    assert!(!DialogFocusTrap::occludes_point(
        true,
        outside_point,
        viewport
    ));

    // 3. DialogHost query methods
    let mut host = DialogHost::<TestAction>::new();
    assert!(!host.is_focus_trapped());
    assert!(!host.wants_keyboard_input());
    assert!(!host.wants_pointer_input());
    assert!(!host.contains_point(inside_point, viewport));

    host.open_confirm("Modal Title", "Modal Message", TestAction::DeleteConfirmed);
    assert!(host.is_focus_trapped());
    assert!(host.wants_keyboard_input());
    assert!(host.wants_pointer_input());
    assert!(host.contains_point(inside_point, viewport));
    assert!(!host.contains_point(outside_point, viewport));
}

#[test]
fn test_dialog_host_keyboard_event_swallowing_and_isolation() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    let arbitrary_keys = [
        KeyCode::Character('a'),
        KeyCode::Character('z'),
        KeyCode::Character('w'),
        KeyCode::Space,
        KeyCode::Delete,
        KeyCode::Backspace,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::Other,
    ];

    // Case 1: When modal host is empty, all keyboard inputs must pass through unconsumed (false)
    for key in &arbitrary_keys {
        assert!(
            !host.handle_keyboard_input(key, true, &mut bus),
            "Inactive host must not consume key press {:?}",
            key
        );
        assert!(
            !host.handle_keyboard_input(key, false, &mut bus),
            "Inactive host must not consume key release {:?}",
            key
        );
    }
    assert!(bus.is_empty());

    // Case 2: When a modal dialogue is presented, all keyboard inputs must be captured (true)
    host.open_confirm(
        "Destructive Action",
        "Are you sure?",
        TestAction::DeleteConfirmed,
    );
    assert!(host.is_active());

    for key in &arbitrary_keys {
        assert!(
            host.handle_keyboard_input(key, true, &mut bus),
            "Active modal must capture key press {:?}",
            key
        );
        assert!(
            host.handle_keyboard_input(key, false, &mut bus),
            "Active modal must capture key release {:?}",
            key
        );
    }

    // Crucial check: Swallowed keys must not emit phantom actions to the action bus
    assert!(bus.is_empty());
}

#[test]
fn test_dialog_host_non_cancellable_modal_escape_containment() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // Present a non-cancellable dialogue (e.g. mandatory update or critical safety warning)
    let desc = ConfirmationDialogDescriptor::new(
        "Mandatory Alert",
        "System operation must complete",
        TestAction::DeleteConfirmed,
    )
    .with_cancellable(false);

    host.open(desc);
    assert!(host.is_active());
    assert_eq!(host.stack_depth(), 1);

    // Escape key press MUST be captured (true) but MUST NOT dismiss the dialogue or leak
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed, "Escape must be consumed by non-cancellable modal");
    assert!(
        host.is_active(),
        "Non-cancellable dialogue must remain active after Escape"
    );
    assert_eq!(host.stack_depth(), 1);
    assert!(bus.is_empty(), "No cancellation action should be emitted");

    // Verify attention shake animation was triggered
    assert!(
        host.is_demanding_attention(),
        "Attention shake must be engaged"
    );
    let active_dlg = host.active_dialog().expect("Active dialogue must exist");
    assert_eq!(active_dlg.attention_ticks, 8);
    assert!(active_dlg.attention_offset().abs() > 0.0);
}

#[test]
fn test_dialog_host_cancellable_modal_escape_and_enter_confirm() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // Test 1: Cancellable modal dismissed via Escape
    let desc_escape = ConfirmationDialogDescriptor::new(
        "Discard Changes",
        "Revert all unsaved modifications?",
        TestAction::DeleteConfirmed,
    )
    .with_cancel_action(TestAction::DeleteCancelled);

    host.open(desc_escape);
    assert!(host.is_active());

    let escape_handled = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(escape_handled);
    assert!(!host.is_active(), "Modal must be dismissed by Escape");
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![TestAction::DeleteCancelled]
    );

    // Test 2: Modal confirmed via Enter key
    let desc_confirm = ConfirmationDialogDescriptor::new(
        "Commit Workspace",
        "Write changes to disk?",
        TestAction::DeleteConfirmed,
    );
    host.open(desc_confirm);
    assert!(host.is_active());

    let enter_handled = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(enter_handled);
    assert!(!host.is_active(), "Modal must be dismissed by Enter");
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![TestAction::DeleteConfirmed]
    );
}

#[test]
fn test_dialog_host_shortcut_isolation_across_stack() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // Helper simulating application shortcut dispatcher logic
    let dispatch_shortcut = |host: &mut DialogHost<TestAction>,
                             key: &KeyCode,
                             bus: &mut ActionBus<TestAction>|
     -> Option<WorkbenchShortcut> {
        if host.handle_keyboard_input(key, true, bus) {
            return None;
        }
        match key {
            KeyCode::Character('s') => Some(WorkbenchShortcut::SaveScene),
            KeyCode::Delete => Some(WorkbenchShortcut::DeleteSelection),
            KeyCode::Character('d') => Some(WorkbenchShortcut::DuplicateEntity),
            _ => None,
        }
    };

    // 1. Without modal dialog: shortcuts dispatch normally
    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Character('s'), &mut bus),
        Some(WorkbenchShortcut::SaveScene)
    );
    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Delete, &mut bus),
        Some(WorkbenchShortcut::DeleteSelection)
    );
    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Character('d'), &mut bus),
        Some(WorkbenchShortcut::DuplicateEntity)
    );

    // 2. Open a modal dialog: all shortcut attempts must be blocked
    host.open_confirm(
        "Confirm Operation",
        "Please confirm",
        TestAction::DeleteConfirmed,
    );
    assert!(host.is_active());

    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Character('s'), &mut bus),
        None,
        "Save shortcut must be isolated while modal is active"
    );
    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Delete, &mut bus),
        None,
        "Delete shortcut must be isolated while modal is active"
    );
    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Character('d'), &mut bus),
        None,
        "Duplicate shortcut must be isolated while modal is active"
    );
    assert!(bus.is_empty());

    // 3. Dismiss modal: shortcuts immediately resume functionality
    let _ = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(!host.is_active());

    assert_eq!(
        dispatch_shortcut(&mut host, &KeyCode::Character('s'), &mut bus),
        Some(WorkbenchShortcut::SaveScene)
    );
}

#[test]
fn test_dialog_host_pointer_capture_and_3d_viewport_occlusion() {
    let mut wb = create_test_workbench();
    let dock_state = wb.descriptor.dock_state.clone();
    let canvas_center = [800.0, 450.0];

    // 1. Baseline without modal dialog:
    // Canvas center is in the interactive 3D viewport, so wants_pointer_at_full returns false
    // (meaning 3D camera / orbit / entity selection receives the pointer).
    assert!(
        !wb.wants_pointer_at_full(canvas_center, None, &dock_state, None),
        "Interactive 3D canvas must receive pointer when no modal is active"
    );

    // 2. Open a confirmation dialogue:
    wb.dialog_host.open_confirm(
        "Delete Asset",
        "Permanently delete selected mesh?",
        TestAction::DeleteConfirmed,
    );
    assert!(wb.dialog_host.is_active());
    assert!(wb.dialog_host.wants_pointer_input());

    // CRITICAL FOCUS TRAP INVARIANT:
    // When modal is active, the entire canvas center MUST be occluded and captured by the UI layer (true).
    assert!(
        wb.wants_pointer_at_full(canvas_center, None, &dock_state, None),
        "Interactive 3D canvas must be occluded by modal dialog scrim"
    );

    // 3. Dismiss modal dialogue:
    let mut bus = ActionBus::new();
    let _ = wb
        .dialog_host
        .handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(!wb.dialog_host.is_active());

    // Canvas center immediately becomes transparent to 3D pointer events again
    assert!(
        !wb.wants_pointer_at_full(canvas_center, None, &dock_state, None),
        "Interactive 3D canvas must receive pointer again once modal is dismissed"
    );
}

#[test]
fn test_dialog_host_custom_and_sheet_pointer_and_keyboard_containment() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let center = Point::new(960.0, 540.0);

    // 1. Test Sheet dialog focus trap
    host.open_sheet("Properties Drawer", SheetAnchor::Right, 360.0);
    assert!(host.is_active());
    assert!(host.is_focus_trapped());
    assert!(host.wants_pointer_input());
    assert!(host.contains_point(center, viewport));

    // Arbitrary key swallowed by sheet
    assert!(host.handle_keyboard_input(&KeyCode::Character('t'), true, &mut bus));

    // Dismiss sheet via Escape
    assert!(host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus));
    assert!(!host.is_active());
    assert!(!host.is_focus_trapped());
    assert!(!host.wants_pointer_input());

    // 2. Test Custom modal dialog focus trap
    host.open_custom("Custom Settings", (500.0, 400.0), |_scope, _bus| {});
    assert!(host.is_active());
    assert!(host.is_focus_trapped());
    assert!(host.wants_pointer_input());
    assert!(host.contains_point(center, viewport));

    // Arbitrary key swallowed by custom dialog
    assert!(host.handle_keyboard_input(&KeyCode::Character('m'), true, &mut bus));

    // Dismiss custom dialog via Escape
    assert!(host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus));
    assert!(!host.is_active());
    assert!(!host.is_focus_trapped());
    assert!(!host.wants_pointer_input());
}

#[test]
fn test_dialog_host_hierarchical_nested_modal_focus_trap_transfer() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // Push Layer 1: Prompt Dialog
    let desc1 =
        PromptDialogDescriptor::new("New Folder", "Folder Name:", TestAction::SubmittedInput)
            .with_cancel_action(TestAction::DeleteCancelled);
    host.open(desc1);

    // Push Layer 2: Confirmation Dialog on top of Prompt
    let desc2 = ConfirmationDialogDescriptor::new(
        "Confirm Override",
        "Directory already exists. Overwrite?",
        TestAction::DeleteConfirmed,
    )
    .with_cancel_action(TestAction::DeleteCancelled);
    host.open(desc2);

    assert_eq!(host.stack_depth(), 2);
    assert!(host.is_focus_trapped());
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Confirm Override")
    );

    // Swallowing applies to top modal (Layer 2)
    assert!(host.handle_keyboard_input(&KeyCode::Character('g'), true, &mut bus));

    // Dismiss top modal (Layer 2) via Escape
    let top_dismissed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(top_dismissed);
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![TestAction::DeleteCancelled]
    );

    // Stack depth drops to 1, but focus trap is STILL securely engaged for Layer 1
    assert_eq!(host.stack_depth(), 1);
    assert!(host.is_focus_trapped());
    assert_eq!(host.active_dialog().map(|d| d.title()), Some("New Folder"));

    // Layer 1 continues swallowing keyboard events
    assert!(host.handle_keyboard_input(&KeyCode::Character('h'), true, &mut bus));

    // Dismiss bottom modal (Layer 1) via Escape
    let bottom_dismissed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(bottom_dismissed);
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![TestAction::DeleteCancelled]
    );

    // Entire stack empty: focus trap is finally disengaged
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    assert!(!host.is_focus_trapped());
    assert!(!host.wants_keyboard_input());
    assert!(!host.wants_pointer_input());

    // Keyboard events now pass through freely
    assert!(!host.handle_keyboard_input(&KeyCode::Character('h'), true, &mut bus));
}