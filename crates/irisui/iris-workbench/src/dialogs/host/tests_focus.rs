// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Modal Dialogue Focus Cycling & Keyboard Navigation
//!
//! Verifies cyclic focus traversal (`Tab` / `Shift+Tab`), semantic focus tags,
//! and contextual Enter-key activation across modal dialogue controls.

use crate::action_bus::ActionBus;
use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::dialogs::descriptor::progress::ProgressModalDescriptor;
use crate::dialogs::host::focus::{DialogFocusCycle, DialogFocusItem};
use crate::dialogs::host::stack::DialogHost;
use iris_core::event::KeyCode;
use iris_widgets::controls::focus::FocusTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestAction {
    ConfirmDelete,
    CancelDelete,
    SubmitName,
    CancelPrompt,
    CancelProgress,
}

#[test]
fn test_dialog_focus_item_semantic_tags_and_targets() {
    let confirm = DialogFocusItem::Confirm;
    let cancel = DialogFocusItem::Cancel;
    let input = DialogFocusItem::Input(2);
    let custom = DialogFocusItem::Custom(0x1234_5678);

    assert_eq!(confirm.to_semantic_tag(), 0xF0C0_0000_C011_F111);
    assert_eq!(cancel.to_semantic_tag(), 0xF0C0_0000_CA11_CE01);
    assert_eq!(input.to_semantic_tag(), 0xF0C0_1000_0000_0002);
    assert_eq!(custom.to_semantic_tag(), 0x1234_5678);

    assert_eq!(
        confirm.to_focus_target(),
        FocusTarget::Tag(confirm.to_semantic_tag())
    );
    assert_eq!(
        cancel.to_focus_target(),
        FocusTarget::Tag(cancel.to_semantic_tag())
    );
    assert_eq!(
        input.to_focus_target(),
        FocusTarget::Tag(input.to_semantic_tag())
    );
    assert_eq!(
        custom.to_focus_target(),
        FocusTarget::Tag(custom.to_semantic_tag())
    );
}

#[test]
fn test_dialog_focus_cycle_state_machine() {
    let mut cycle = DialogFocusCycle::with_targets(vec![
        DialogFocusItem::Input(0),
        DialogFocusItem::Confirm,
        DialogFocusItem::Cancel,
    ]);

    assert_eq!(cycle.current(), Some(DialogFocusItem::Input(0)));
    assert!(cycle.is_focused(DialogFocusItem::Input(0)));
    assert!(!cycle.is_focused(DialogFocusItem::Confirm));

    // Forward cycle (Tab)
    assert_eq!(cycle.focus_next(), Some(DialogFocusItem::Confirm));
    assert!(cycle.is_focused(DialogFocusItem::Confirm));

    assert_eq!(cycle.focus_next(), Some(DialogFocusItem::Cancel));
    assert!(cycle.is_focused(DialogFocusItem::Cancel));

    // Wrap around to start
    assert_eq!(cycle.focus_next(), Some(DialogFocusItem::Input(0)));
    assert!(cycle.is_focused(DialogFocusItem::Input(0)));

    // Backward cycle (Shift+Tab)
    assert_eq!(cycle.focus_prev(), Some(DialogFocusItem::Cancel));
    assert!(cycle.is_focused(DialogFocusItem::Cancel));

    assert_eq!(cycle.focus_prev(), Some(DialogFocusItem::Confirm));
    assert_eq!(cycle.focus_prev(), Some(DialogFocusItem::Input(0)));
    assert_eq!(cycle.focus_prev(), Some(DialogFocusItem::Cancel));

    // Direct selection
    cycle.set_current(DialogFocusItem::Confirm);
    assert_eq!(cycle.current(), Some(DialogFocusItem::Confirm));

    // Clear
    cycle.clear();
    assert_eq!(cycle.current(), None);

    // Empty cycle edge cases
    let mut empty_cycle = DialogFocusCycle::new();
    assert_eq!(empty_cycle.focus_next(), None);
    assert_eq!(empty_cycle.focus_prev(), None);
    assert_eq!(empty_cycle.current(), None);
}

#[test]
fn test_confirmation_dialog_keyboard_tab_cycling_and_enter_confirm() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    let desc = ConfirmationDialogDescriptor::new(
        "Delete Scene",
        "Are you sure?",
        TestAction::ConfirmDelete,
    )
    .with_cancel_action(TestAction::CancelDelete);

    host.open(desc);
    assert!(host.is_active());

    // Initial focus on Confirm button
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Confirm));

    // Press Tab -> focus moves to Cancel button
    let consumed = host.handle_keyboard_input_with_modifiers(&KeyCode::Tab, true, false, &mut bus);
    assert!(consumed);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Cancel));

    // Press Tab again -> wraps back to Confirm button
    host.handle_keyboard_input_with_modifiers(&KeyCode::Tab, true, false, &mut bus);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Confirm));

    // Press Enter while Confirm is focused -> pops dialog and emits ConfirmDelete
    host.handle_keyboard_input_with_modifiers(&KeyCode::Enter, true, false, &mut bus);
    assert!(!host.is_active());
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::ConfirmDelete]);
}

#[test]
fn test_confirmation_dialog_keyboard_enter_on_focused_cancel() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    let desc = ConfirmationDialogDescriptor::new(
        "Delete Asset",
        "Permanently remove?",
        TestAction::ConfirmDelete,
    )
    .with_cancel_action(TestAction::CancelDelete);

    host.open(desc);

    // Cycle focus to Cancel button
    host.handle_keyboard_input_with_modifiers(&KeyCode::Tab, true, false, &mut bus);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Cancel));

    // Press Enter while Cancel is focused -> pops dialog and emits CancelDelete
    host.handle_keyboard_input_with_modifiers(&KeyCode::Enter, true, false, &mut bus);
    assert!(!host.is_active());
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelDelete]);
}

#[test]
fn test_prompt_dialog_shift_tab_reverse_cycling() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    let desc =
        PromptDialogDescriptor::new("Rename Node", "Enter new name:", |_| TestAction::SubmitName)
            .with_cancel_action(TestAction::CancelPrompt);

    host.open(desc);

    // Starts on Input(0)
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Input(0)));

    // Shift+Tab -> wraps backward directly to Cancel
    let consumed = host.handle_keyboard_input_with_modifiers(&KeyCode::Tab, true, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Cancel));

    // Shift+Tab again -> moves backward to Confirm
    host.handle_keyboard_input_with_modifiers(&KeyCode::Tab, true, true, &mut bus);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Confirm));

    // Shift+Tab again -> moves backward to Input(0)
    host.handle_keyboard_input_with_modifiers(&KeyCode::Tab, true, true, &mut bus);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Input(0)));

    // Enter on Input(0) submits the prompt
    host.handle_keyboard_input_with_modifiers(&KeyCode::Enter, true, false, &mut bus);
    assert!(!host.is_active());
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::SubmitName]);
}

#[test]
fn test_cancellable_progress_modal_focus_and_enter_cancel() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    let desc = ProgressModalDescriptor::new("Baking Lightmaps")
        .with_cancellable(true)
        .with_on_cancel(TestAction::CancelProgress);

    host.open(desc);
    assert_eq!(host.focused_item(), Some(DialogFocusItem::Cancel));

    // Enter on focused Cancel cancels the progress task
    host.handle_keyboard_input_with_modifiers(&KeyCode::Enter, true, false, &mut bus);
    assert!(!host.is_active());
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelProgress]);
}

#[test]
fn test_uncancellable_progress_modal_ignores_enter() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    let desc = ProgressModalDescriptor::new("Compiling Shaders").with_cancellable(false);

    host.open(desc);
    assert_eq!(host.focused_item(), None);

    // Enter does not cancel
    host.handle_keyboard_input_with_modifiers(&KeyCode::Enter, true, false, &mut bus);
    assert!(host.is_active());
    assert_eq!(bus.len(), 0);
}