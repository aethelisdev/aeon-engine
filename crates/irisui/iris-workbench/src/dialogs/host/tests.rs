// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous `DialogHost` State Machine
//!
//! Verifies state transitions, Escape dismissals, Enter key confirmations,
//! input submissions, custom modal rendering, and action dispatch into [`ActionBus`].

use super::DialogHost;
use crate::action_bus::ActionBus;
use iris_core::event::KeyCode;
use iris_core::geometry::Rect;
use iris_core::tree::UiTree;
use iris_widgets::declarative::UiScope;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    DeleteAsset(String),
    RenameAsset(String),
    CustomAction(u32),
}

#[test]
fn test_dialog_host_initial_state() {
    let host: DialogHost<TestAction> = DialogHost::new();
    assert!(!host.is_active());
    assert!(host.active_dialog().is_none());

    let default_host: DialogHost<TestAction> = DialogHost::default();
    assert!(!default_host.is_active());
}

#[test]
fn test_dialog_host_open_confirm_and_close() {
    let mut host = DialogHost::new();
    host.open_confirm(
        "Delete Item",
        "Are you sure?",
        TestAction::DeleteAsset("test.png".into()),
    );

    assert!(host.is_active());
    let dialog = host.active_dialog().expect("Active dialog expected");
    assert!(dialog.dismiss_on_escape);
    assert!(dialog.dismiss_on_backdrop);

    host.close();
    assert!(!host.is_active());
    assert!(host.active_dialog().is_none());
}

#[test]
fn test_dialog_host_escape_dismisses_dialog_without_action() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open_confirm(
        "Warning",
        "Discard unsaved changes?",
        TestAction::CustomAction(42),
    );
    assert!(host.is_active());

    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert!(!host.is_active());
    assert_eq!(bus.len(), 0);
}

#[test]
fn test_dialog_host_enter_confirms_action() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open_confirm(
        "Confirm",
        "Proceed with operation?",
        TestAction::CustomAction(100),
    );

    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert!(!host.is_active());
    assert_eq!(bus.len(), 1);

    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(100)]);
}

#[test]
fn test_dialog_host_enter_submits_input_dialog() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open_input("Rename", "New name:", "character_model", |s| {
        TestAction::RenameAsset(s)
    });
    assert!(host.is_active());

    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert!(!host.is_active());
    assert_eq!(bus.len(), 1);

    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(
        drained,
        vec![TestAction::RenameAsset("character_model".into())]
    );
}

#[test]
fn test_dialog_host_danger_styling() {
    let mut host = DialogHost::new();
    host.open_confirm_danger(
        "Delete Directory",
        "This action cannot be undone.",
        TestAction::DeleteAsset("assets/textures".into()),
    );

    let dialog = host.active_dialog().expect("Active dialog expected");
    if let super::DialogKind::Confirm {
        is_danger,
        confirm_label,
        ..
    } = &dialog.kind
    {
        assert!(is_danger);
        assert_eq!(confirm_label, "Delete");
    } else {
        panic!("Expected Confirm dialog kind");
    }
}

#[test]
fn test_dialog_host_keyboard_captures_when_active() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    // Inactive: keys are NOT consumed
    assert!(!host.handle_keyboard_input(&KeyCode::Character('a'), true, &mut bus));

    // Active: all keys ARE consumed to isolate underlying views
    host.open_confirm("Test", "Prompt", TestAction::CustomAction(1));
    assert!(host.handle_keyboard_input(&KeyCode::Character('a'), true, &mut bus));
    assert!(host.handle_keyboard_input(&KeyCode::Backspace, true, &mut bus));
    assert!(host.is_active());
}

#[test]
fn test_dialog_host_render_empty_is_noop() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::new();
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    host.render(
        &mut scope,
        Rect::new(0.0, 0.0, 1920.0, 1080.0),
        false,
        &mut bus,
    );
    assert!(!host.is_active());
    assert_eq!(bus.len(), 0);
}

#[test]
fn test_dialog_host_render_escape_dismisses() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();
    host.open_confirm("Title", "Message", TestAction::CustomAction(5));

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    host.render(
        &mut scope,
        Rect::new(0.0, 0.0, 1920.0, 1080.0),
        true,
        &mut bus,
    );
    assert!(!host.is_active());
    assert_eq!(bus.len(), 0);
}

#[test]
fn test_dialog_host_open_custom() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    let rendered = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let rendered_clone = rendered.clone();
    host.open_custom(
        "Custom Dialog",
        (500.0, 300.0),
        move |_scope, action_bus| {
            rendered_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            action_bus.emit(TestAction::CustomAction(777));
        },
    );

    assert!(host.is_active());

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    host.render(
        &mut scope,
        Rect::new(0.0, 0.0, 1920.0, 1080.0),
        false,
        &mut bus,
    );
    assert!(host.is_active());
    assert!(rendered.load(std::sync::atomic::Ordering::Relaxed));
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(777)]);
}