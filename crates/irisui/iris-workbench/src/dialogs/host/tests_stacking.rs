// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Multi-Modal Dialog Stacking & Sequential Cancellation
//!
//! Verifies hierarchical modal stacking lifecycles, Z-order layering,
//! non-cancellable barrier preservation, sequential cancellation action dispatching,
//! and empty-stack invariants within [`DialogHost`].

use crate::action_bus::ActionBus;
use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::custom::CustomDialogDescriptor;
use crate::dialogs::descriptor::info::InfoCardDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::dialogs::descriptor::progress::ProgressModalDescriptor;
use crate::dialogs::descriptor::sheet::{SheetAnchor, SheetDialogDescriptor};
use crate::dialogs::host::stack::DialogHost;
use iris_core::event::KeyCode;
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;
use iris_widgets::declarative::UiScope;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    ConfirmBase,
    CancelBase,
    SubmitPrompt(String),
    CancelPrompt,
    CancelProgress,
    CloseInfo,
    CustomCancelled,
}

#[test]
fn test_dialog_host_modal_stacking_and_cancellation() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // 1. Initially empty
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());

    // 2. Open Base Dialog 1 (Confirmation)
    let desc1 =
        ConfirmationDialogDescriptor::new("Save Project", "Save changes?", TestAction::ConfirmBase)
            .with_cancel_action(TestAction::CancelBase);
    host.open(desc1);

    assert_eq!(host.stack_depth(), 1);
    assert!(host.is_active());
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Save Project")
    );

    // 3. Open Dialog 2 (Prompt) on top of Dialog 1
    let desc2 =
        PromptDialogDescriptor::new("Enter Commit Message", "Message:", TestAction::SubmitPrompt)
            .with_cancel_action(TestAction::CancelPrompt);
    host.open(desc2);

    assert_eq!(host.stack_depth(), 2);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Enter Commit Message")
    );

    // 4. Open Dialog 3 (Progress Modal) on top of Dialog 2
    let desc3 = ProgressModalDescriptor::new("Uploading to Remote")
        .with_cancellable(true)
        .with_on_cancel(TestAction::CancelProgress);
    host.open(desc3);

    assert_eq!(host.stack_depth(), 3);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Uploading to Remote")
    );

    // 5. Cancel topmost Dialog 3 via cancel_top()
    let cancelled = host.cancel_top(&mut bus);
    assert!(cancelled);
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Enter Commit Message")
    );

    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelProgress]);

    // 6. Cancel Dialog 2 via Escape key input
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Save Project")
    );

    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelPrompt]);

    // 7. Confirm Base Dialog 1 via Enter key input
    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    assert!(host.active_dialog().is_none());

    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::ConfirmBase]);
}

#[test]
fn test_dialog_host_mixed_stack_with_uncancellable_barrier() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // Dialog 1 (Bottom, cancellable)
    let d1 = ConfirmationDialogDescriptor::new("D1", "Bottom", TestAction::ConfirmBase)
        .with_cancel_action(TestAction::CancelBase);
    host.open(d1);

    // Dialog 2 (Middle, non-cancellable progress barrier)
    let d2 = ProgressModalDescriptor::new("Critical Asset Migration").with_cancellable(false);
    host.open(d2);

    // Dialog 3 (Top, cancellable prompt)
    let d3 = PromptDialogDescriptor::new("D3", "Top", TestAction::SubmitPrompt)
        .with_cancel_action(TestAction::CancelPrompt);
    host.open(d3);

    assert_eq!(host.stack_depth(), 3);

    // Escape dismisses top Dialog 3
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Critical Asset Migration")
    );
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelPrompt]);

    // Escape on non-cancellable Dialog 2: absorbed, triggers attention, does NOT pop!
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(bus.len(), 0);
    assert_eq!(host.active_dialog().unwrap().attention_ticks, 8);

    // Dialog 1 remains safely preserved underneath the barrier
    assert_eq!(host.dialog_stack[0].title(), "D1");

    // When the migration operation finishes, host closes Dialog 2
    host.close();
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(host.active_dialog().map(|d| d.title()), Some("D1"));

    // Now Dialog 1 can be cancelled
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelBase]);
}

#[test]
fn test_dialog_host_multi_layered_render_scrim_and_action_pipeline() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let d1 = ConfirmationDialogDescriptor::new("D1", "M1", TestAction::ConfirmBase);
    let d2 = ConfirmationDialogDescriptor::new("D2", "M2", TestAction::ConfirmBase);
    let d3 = ConfirmationDialogDescriptor::new("D3", "M3", TestAction::ConfirmBase);

    host.open(d1);
    host.open(d2);
    host.open(d3);
    assert_eq!(host.stack_depth(), 3);

    // Render 3-dialog stack: background dialogs get dimmed scrim, topmost renders active
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    host.render(&mut scope, viewport, false, &mut bus);
    assert_eq!(host.stack_depth(), 3);

    // Escape during render dismisses topmost dialog D3
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_node();
    let mut scope2 = UiScope::new(&mut tree2, root2);

    host.render(&mut scope2, viewport, true, &mut bus);
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(host.active_dialog().map(|d| d.title()), Some("D2"));
}

#[test]
fn test_dialog_host_multi_modal_custom_and_sheet_stacking() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // 1. Push a Sheet Drawer at bottom
    let sheet = SheetDialogDescriptor::new("Log Inspector Drawer", SheetAnchor::Bottom, 240.0)
        .with_cancellable(true)
        .with_cancel_action(TestAction::CancelBase);
    host.open(sheet);

    // 2. Push a Custom View Modal
    let custom = CustomDialogDescriptor::new("Color Palette Picker", (300.0, 400.0), |_, _| {})
        .with_position(Point::new(100.0, 100.0))
        .with_cancellable(true)
        .with_cancel_action(TestAction::CustomCancelled);
    host.open(custom);

    // 3. Push an Info Card
    let info = InfoCardDialogDescriptor::new("About Studio").with_on_close(TestAction::CloseInfo);
    host.open(info);

    assert_eq!(host.stack_depth(), 3);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("About Studio")
    );

    // Enter confirms Info Card
    host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Color Palette Picker")
    );
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CloseInfo]);

    // Escape cancels Custom View
    host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(
        host.active_dialog().map(|d| d.title()),
        Some("Log Inspector Drawer")
    );
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CustomCancelled]);

    // Escape cancels Sheet Drawer
    host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::CancelBase]);
}

#[test]
fn test_dialog_host_pop_and_cancel_empty_stack_invariants() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();

    // All empty stack queries must be deterministic and safe
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    assert!(host.active_dialog().is_none());
    assert!(!host.is_top_cancellable());
    assert_eq!(host.focused_item(), None);
    assert_eq!(host.cycle_focus(false), None);
    assert_eq!(host.cycle_focus(true), None);

    // Pop on empty stack returns None
    assert!(host.pop().is_none());

    // cancel_top on empty stack returns false
    assert!(!host.cancel_top(&mut bus));

    // Keyboard inputs on empty stack return false (not consumed)
    assert!(!host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus));
    assert!(!host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus));
    assert!(!host.handle_keyboard_input(&KeyCode::Tab, true, &mut bus));
    assert_eq!(bus.len(), 0);

    // close and close_all on empty stack are safe no-ops
    host.close();
    host.close_all();
    assert_eq!(host.stack_depth(), 0);
}

#[test]
fn test_dialog_host_stack_attention_propagation_only_topmost() {
    let mut host = DialogHost::<TestAction>::new();

    host.open_confirm("Bottom", "M1", TestAction::ConfirmBase);
    host.open_confirm("Top", "M2", TestAction::ConfirmBase);

    assert_eq!(host.dialog_stack[0].attention_ticks, 0);
    assert_eq!(host.dialog_stack[1].attention_ticks, 0);

    // Trigger attention on host
    host.trigger_attention();

    // Only topmost dialog receives attention ticks
    assert_eq!(host.dialog_stack[0].attention_ticks, 0);
    assert_eq!(host.dialog_stack[1].attention_ticks, 8);

    // Tick attention
    host.active_dialog_mut().unwrap().tick_attention();
    assert_eq!(host.dialog_stack[0].attention_ticks, 0);
    assert_eq!(host.dialog_stack[1].attention_ticks, 7);
}