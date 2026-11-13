// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous `DialogHost` State Machine
//!
//! Verifies state transitions, modal stacking, Escape dismissals, Enter key confirmations,
//! input submissions, custom modal rendering, sheet drawers, and action dispatch into [`ActionBus`].

use super::DialogHost;
use crate::action_bus::ActionBus;
use crate::dialogs::descriptor::sheet::SheetAnchor;
use crate::dialogs::descriptor::unified::DialogDescriptor;
use iris_core::Color;
use iris_core::event::KeyCode;
use iris_core::geometry::{Point, Rect};
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
    assert_eq!(host.stack_depth(), 0);
    assert!(host.active_dialog().is_none());
    assert!(!host.is_focus_trapped());
    assert!(!host.wants_keyboard_input());
    assert!(!host.wants_pointer_input());

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
    assert_eq!(host.stack_depth(), 1);
    let dialog = host.active_dialog().expect("Active dialog expected");
    assert!(dialog.dismiss_on_escape);
    assert!(dialog.dismiss_on_backdrop);

    host.close();
    assert!(!host.is_active());
    assert_eq!(host.stack_depth(), 0);
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

#[test]
fn test_dialog_host_modal_stacking_lifecycle() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    // 1. Open Base Dialog A
    host.open_confirm("Base Dialog", "Dialog A", TestAction::CustomAction(1));
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(host.active_dialog().unwrap().title(), "Base Dialog");

    // 2. Open Secondary Dialog B on top of A
    host.open_confirm(
        "Unsaved Changes",
        "Save before proceeding?",
        TestAction::CustomAction(2),
    );
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(host.active_dialog().unwrap().title(), "Unsaved Changes");

    // 3. Escape dismisses only topmost Dialog B, revealing Base Dialog A
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(host.active_dialog().unwrap().title(), "Base Dialog");
    assert_eq!(bus.len(), 0);

    // 4. Escape dismisses Base Dialog A, clearing stack completely
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    assert!(host.active_dialog().is_none());
}

#[test]
fn test_dialog_host_stack_enter_confirms_topmost_only() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    // Base dialog 1
    host.open_confirm("Base Dialog", "Message 1", TestAction::CustomAction(10));
    // Nested dialog 2
    host.open_confirm("Nested Dialog", "Message 2", TestAction::CustomAction(20));
    assert_eq!(host.stack_depth(), 2);

    // Enter confirms Nested Dialog 2
    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(20)]);

    // Base dialog 1 remains intact in stack!
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(host.active_dialog().unwrap().title(), "Base Dialog");

    // Enter confirms Base dialog 1
    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(10)]);
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
}

#[test]
fn test_dialog_host_close_all() {
    let mut host = DialogHost::new();
    host.open_confirm("D1", "M1", TestAction::CustomAction(1));
    host.open_confirm("D2", "M2", TestAction::CustomAction(2));
    host.open_confirm("D3", "M3", TestAction::CustomAction(3));
    assert_eq!(host.stack_depth(), 3);

    host.close_all();
    assert_eq!(host.stack_depth(), 0);
    assert!(!host.is_active());
    assert!(host.active_dialog().is_none());
}

#[test]
fn test_dialog_host_open_generic_descriptors() {
    let mut host = DialogHost::<TestAction>::new();

    // 1. Modal descriptor
    host.open(DialogDescriptor::modal("Settings Modal", 600.0, 400.0));
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(host.active_dialog().unwrap().title(), "Settings Modal");
    assert_eq!(host.active_dialog().unwrap().size, (600.0, 400.0));

    // 2. Sheet descriptor
    host.open(DialogDescriptor::sheet(
        "Drawer Sheet",
        SheetAnchor::Bottom,
        280.0,
    ));
    assert_eq!(host.stack_depth(), 2);
    assert_eq!(host.active_dialog().unwrap().title(), "Drawer Sheet");

    // 3. Confirmation descriptor
    host.open(DialogDescriptor::confirmation(
        "Affirm",
        "Proceed?",
        TestAction::CustomAction(99),
    ));
    assert_eq!(host.stack_depth(), 3);
    assert_eq!(host.active_dialog().unwrap().title(), "Affirm");
    assert!(!host.active_dialog().unwrap().is_danger());

    // 4. Danger confirmation descriptor
    host.open(DialogDescriptor::danger(
        "Delete File",
        "Permanent removal",
        "Delete",
        TestAction::CustomAction(100),
    ));
    assert_eq!(host.stack_depth(), 4);
    assert_eq!(host.active_dialog().unwrap().title(), "Delete File");
    assert!(host.active_dialog().unwrap().is_danger());

    // 5. Prompt descriptor
    host.open(DialogDescriptor::prompt(
        "Asset Name",
        "Enter name:",
        TestAction::RenameAsset,
    ));
    assert_eq!(host.stack_depth(), 5);
    assert_eq!(host.active_dialog().unwrap().title(), "Asset Name");

    host.close_all();
    assert_eq!(host.stack_depth(), 0);
}

#[test]
fn test_dialog_host_focus_trap_queries_and_containment() {
    let mut host = DialogHost::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    // Idle host
    assert!(!host.is_focus_trapped());
    assert!(!host.wants_keyboard_input());
    assert!(!host.wants_pointer_input());
    assert!(!host.contains_point(Point::new(100.0, 100.0), viewport));

    // Active host
    host.open_confirm("Modal", "Trap Test", TestAction::CustomAction(1));
    assert!(host.is_focus_trapped());
    assert!(host.wants_keyboard_input());
    assert!(host.wants_pointer_input());
    assert!(host.contains_point(Point::new(500.0, 500.0), viewport));
    assert!(!host.contains_point(Point::new(2000.0, 500.0), viewport));
}

#[test]
fn test_dialog_host_stacked_rendering() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    // Stack 2 dialogs
    host.open_confirm(
        "Dialog 1",
        "Underlying content",
        TestAction::CustomAction(1),
    );
    host.open_confirm("Dialog 2", "Topmost content", TestAction::CustomAction(2));
    assert_eq!(host.stack_depth(), 2);

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    // Render stack: both dialogs should render cleanly without panics
    host.render(&mut scope, viewport, false, &mut bus);
    assert_eq!(host.stack_depth(), 2);

    // Escape dismisses top dialog during render
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_node();
    let mut scope2 = UiScope::new(&mut tree2, root2);
    host.render(&mut scope2, viewport, true, &mut bus);
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(host.active_dialog().unwrap().title(), "Dialog 1");
}

#[test]
fn test_dialog_host_open_custom_descriptor_and_rendering() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc = crate::dialogs::descriptor::custom::CustomDialogDescriptor::new(
        "Floating Inspector",
        (400.0, 300.0),
        |_scope, action_bus| {
            action_bus.emit(TestAction::CustomAction(123));
        },
    )
    .with_position(Point::new(50.0, 50.0))
    .with_subtitle("Entity Properties")
    .with_icon("🔍")
    .with_dismiss_on_escape(true);

    host.open(desc);
    assert!(host.is_active());
    assert_eq!(host.stack_depth(), 1);

    let active = host.active_dialog().expect("Active dialog expected");
    assert_eq!(active.title(), "Floating Inspector");
    assert_eq!(active.position, Some(Point::new(50.0, 50.0)));
    assert_eq!(active.subtitle.as_deref(), Some("Entity Properties"));
    assert_eq!(active.icon.as_deref(), Some("🔍"));

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    host.render(&mut scope, viewport, false, &mut bus);

    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(123)]);

    // Escape dismisses custom dialog during render
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_node();
    let mut scope2 = UiScope::new(&mut tree2, root2);
    host.render(&mut scope2, viewport, true, &mut bus);
    assert!(!host.is_active());
    assert_eq!(host.stack_depth(), 0);
}

#[test]
fn test_dialog_host_open_confirm_custom_labels() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open_confirm_custom(
        "Silme Onayı",
        "Bu öğeyi silmek istediğinize emin misiniz?",
        "Sil",
        "Vazgeç",
        true,
        TestAction::DeleteAsset("mesh.obj".into()),
    );

    assert!(host.is_active());
    let active = host.active_dialog().expect("Active dialog expected");
    assert_eq!(active.title(), "Silme Onayı");
    assert!(active.is_danger());

    // Enter confirms and emits action
    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert!(!host.is_active());
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::DeleteAsset("mesh.obj".into())]);
}

#[test]
fn test_dialog_host_open_info_card_and_render() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc = crate::dialogs::descriptor::info::InfoCardDialogDescriptor::new("System Diagnostic")
        .with_app_name("Aeon Creative Studio")
        .with_version("0.8.2")
        .with_entry("Vulkan", "Enabled")
        .with_on_close(TestAction::CustomAction(777));

    host.open_info(desc);
    assert!(host.is_active());
    assert_eq!(host.stack_depth(), 1);

    let active = host.active_dialog().expect("Active dialog expected");
    assert_eq!(active.title(), "System Diagnostic");

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    // Normal render: dialog stays open
    host.render(&mut scope, viewport, false, &mut bus);
    assert!(host.is_active());
    assert_eq!(bus.len(), 0);

    // Escape dismisses info dialog without action
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_node();
    let mut scope2 = UiScope::new(&mut tree2, root2);
    host.render(&mut scope2, viewport, true, &mut bus);
    assert!(!host.is_active());
    assert_eq!(host.stack_depth(), 0);
    assert_eq!(bus.len(), 0);
}

#[test]
fn test_dialog_host_info_card_enter_triggers_on_close() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    let desc = crate::dialogs::descriptor::info::InfoCardDialogDescriptor::new("Notice")
        .with_on_close(TestAction::CustomAction(888));

    host.open_info(desc);
    assert!(host.is_active());

    // Enter confirms and emits on_close action
    let consumed = host.handle_keyboard_input(&KeyCode::Enter, true, &mut bus);
    assert!(consumed);
    assert!(!host.is_active());
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(888)]);
}

#[test]
fn test_dialog_host_info_card_click_confirm_dismisses() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc = crate::dialogs::descriptor::info::InfoCardDialogDescriptor::new("About")
        .with_app_name("App")
        .with_on_close(TestAction::CustomAction(123));

    host.open_info(desc);
    assert!(host.is_active());

    let mut tree = UiTree::new();
    let root = tree.create_node();
    tree.event_state.frame_clicked_tag = Some(iris_widgets::modal::MODAL_TAG_CONFIRM);
    let mut scope = UiScope::new(&mut tree, root);
    host.render(&mut scope, viewport, false, &mut bus);

    assert!(!host.is_active());
    assert_eq!(bus.len(), 1);
}

#[test]
fn test_dialog_host_info_card_click_header_close_dismisses() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc = crate::dialogs::descriptor::info::InfoCardDialogDescriptor::new("About")
        .with_app_name("App");

    host.open_info(desc);
    assert!(host.is_active());

    let mut tree = UiTree::new();
    let root = tree.create_node();
    tree.event_state.frame_clicked_tag = Some(iris_widgets::modal::MODAL_TAG_CLOSE);
    let mut scope = UiScope::new(&mut tree, root);
    host.render(&mut scope, viewport, false, &mut bus);

    assert!(!host.is_active());
}

#[test]
fn test_dialog_host_info_card_with_link_and_custom_styling() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc = crate::dialogs::descriptor::info::InfoCardDialogDescriptor::new("About Test")
        .with_app_name("Custom App")
        .with_app_name_color(Color::rgba(0.0, 0.8, 1.0, 1.0))
        .with_link("License Link", "https://example.com/license")
        .with_footer_notice_colored("Warning Notice", Color::rgba(1.0, 0.3, 0.3, 1.0));

    host.open_info(desc);
    assert!(host.is_active());

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    host.render(&mut scope, viewport, false, &mut bus);

    assert!(host.is_active());
}

#[test]
fn test_dialog_host_info_card_click_scrim_dismisses() {
    let mut host = DialogHost::<TestAction>::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc = crate::dialogs::descriptor::info::InfoCardDialogDescriptor::new("About")
        .with_dismiss_on_backdrop(true);

    host.open_info(desc);
    assert!(host.is_active());

    let mut tree = UiTree::new();
    let root = tree.create_node();
    tree.event_state.frame_clicked_tag = Some(iris_widgets::modal::MODAL_TAG_SCRIM);
    let mut scope = UiScope::new(&mut tree, root);
    host.render(&mut scope, viewport, false, &mut bus);

    assert!(!host.is_active());
}

#[test]
fn test_dialog_host_open_progress_and_render() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc =
        crate::dialogs::descriptor::progress::ProgressModalDescriptor::new("Compiling Shaders")
            .with_status_message("PBR pipeline (4/18)...")
            .with_progress(0.40)
            .with_on_cancel(TestAction::CustomAction(404));

    host.open_progress(desc);
    assert!(host.is_active());
    assert_eq!(host.stack_depth(), 1);

    let active = host.active_dialog().expect("Active dialog expected");
    assert_eq!(active.title(), "Compiling Shaders");

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    // Normal render: modal stays active
    host.render(&mut scope, viewport, false, &mut bus);
    assert!(host.is_active());
    assert_eq!(bus.len(), 0);

    // Escape dismisses cancellable progress and emits on_cancel action
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_node();
    let mut scope2 = UiScope::new(&mut tree2, root2);
    host.render(&mut scope2, viewport, true, &mut bus);
    assert!(!host.is_active());
    assert_eq!(host.stack_depth(), 0);
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(404)]);
}

#[test]
fn test_dialog_host_update_progress_in_place() {
    let mut host: DialogHost<TestAction> = DialogHost::new();

    let desc =
        crate::dialogs::descriptor::progress::ProgressModalDescriptor::new("Asset Pack Export")
            .with_status_message("Step 1")
            .with_progress(0.10);

    host.open_progress(desc);

    // In-place dynamic mutation
    let updated = host.update_progress(0.85, Some("Step 2: Compressing audio..."));
    assert!(updated);

    let active = host.active_dialog().expect("Active dialog expected");
    let desc = active
        .build_progress_desc(true)
        .expect("Progress descriptor expected");
    assert_eq!(desc.progress, 0.85);
    assert_eq!(
        desc.status_message.as_deref(),
        Some("Step 2: Compressing audio...")
    );

    // In-place mutation with None message keeps old message
    let updated_no_msg = host.update_progress(0.99, None::<String>);
    assert!(updated_no_msg);

    let active2 = host.active_dialog().expect("Active dialog expected");
    let desc2 = active2
        .build_progress_desc(true)
        .expect("Progress descriptor expected");
    assert_eq!(desc2.progress, 0.99);
    assert_eq!(
        desc2.status_message.as_deref(),
        Some("Step 2: Compressing audio...")
    );

    assert!(host.has_progress());
    assert!(host.close_progress());
    assert!(!host.has_progress());
    assert!(!host.close_progress());
}

#[test]
fn test_dialog_host_progress_non_cancellable_blocks_escape() {
    let mut host: DialogHost<TestAction> = DialogHost::new();
    let mut bus = ActionBus::new();
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let desc =
        crate::dialogs::descriptor::progress::ProgressModalDescriptor::new("Critical Saving")
            .with_status_message("Flushing disk buffers...")
            .with_progress(0.50)
            .with_cancellable(false);

    host.open_progress(desc);
    assert!(host.is_active());

    // Keyboard Escape input is consumed but does NOT pop stack when non-cancellable
    let consumed = host.handle_keyboard_input(&KeyCode::Escape, true, &mut bus);
    assert!(consumed);
    assert!(host.is_active());
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(bus.len(), 0);

    // Render with escape_pressed also keeps non-cancellable modal active
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    host.render(&mut scope, viewport, true, &mut bus);
    assert!(host.is_active());
    assert_eq!(host.stack_depth(), 1);
    assert_eq!(bus.len(), 0);
}

#[test]
fn test_dialog_host_attention_trigger_and_decay() {
    let mut host: DialogHost<TestAction> = DialogHost::new();
    let desc =
        crate::dialogs::descriptor::modal::ModalDescriptor::new("Attention Modal", 400.0, 300.0)
            .with_dismiss_on_backdrop(false);

    host.open(desc);
    assert!(!host.is_demanding_attention());

    // Trigger attention shake
    let triggered = host.trigger_attention();
    assert!(triggered);
    assert!(host.is_demanding_attention());

    let top = host.active_dialog_mut().expect("Top dialog expected");
    assert_eq!(top.attention_ticks, 8);
    assert!(top.attention_offset().abs() > 0.0);

    // Decay across 8 ticks
    for _ in 0..8 {
        top.tick_attention();
    }
    assert_eq!(top.attention_ticks, 0);
    assert_eq!(top.attention_offset(), 0.0);
    assert!(!top.is_demanding_attention());
    assert!(!host.is_demanding_attention());
}

#[test]
fn test_dialog_host_attention_highlight_styling_in_desc() {
    let mut host: DialogHost<TestAction> = DialogHost::new();
    let desc =
        crate::dialogs::descriptor::modal::ModalDescriptor::new("Locked Dialog", 400.0, 300.0)
            .with_dismiss_on_backdrop(false);

    host.open(desc);
    host.trigger_attention();

    let top = host.active_dialog().expect("Top dialog expected");
    let modal_desc = top.build_modal_desc(true);
    let expected_highlight = top.style.attention_border_color;
    assert_eq!(
        modal_desc.style.map(|s| s.card_border_color),
        Some(expected_highlight)
    );
}

#[test]
fn test_dialog_host_cancellation_escape_with_action() {
    let mut host: DialogHost<TestAction> = DialogHost::new();
    let mut action_bus = ActionBus::<TestAction>::new();

    let conf = crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor::new(
        "Discard Changes?",
        "Unsaved work will be lost.",
        TestAction::CustomAction(1),
    )
    .with_cancel_action(TestAction::CustomAction(99));

    host.open(conf);
    assert!(host.is_active());
    assert!(host.is_top_cancellable());

    // Press Escape via DialogFocusTrap
    let consumed = crate::dialogs::host::trap::DialogFocusTrap::handle_key(
        &mut host.dialog_stack,
        &iris_core::event::KeyCode::Escape,
        true,
        &mut action_bus,
    );
    assert!(consumed);
    assert!(!host.is_active());

    // Verify cancellation action was emitted
    let drained: Vec<TestAction> = action_bus.drain().collect();
    assert_eq!(drained, vec![TestAction::CustomAction(99)]);
}

#[test]
fn test_dialog_host_non_cancellable_escape_triggers_attention() {
    let mut host: DialogHost<TestAction> = DialogHost::new();
    let mut action_bus = ActionBus::<TestAction>::new();

    let conf = crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor::new(
        "Critical System Update",
        "Applying critical patches. Do not turn off.",
        TestAction::CustomAction(1),
    )
    .with_cancellable(false)
    .with_cancel_action(TestAction::CustomAction(99));

    host.open(conf);
    assert!(host.is_active());
    assert!(!host.is_top_cancellable());

    // Press Escape via DialogFocusTrap
    let consumed = crate::dialogs::host::trap::DialogFocusTrap::handle_key(
        &mut host.dialog_stack,
        &iris_core::event::KeyCode::Escape,
        true,
        &mut action_bus,
    );
    assert!(consumed);

    // Dialog must NOT close, attention feedback must be active!
    assert!(host.is_active());
    assert!(host.is_demanding_attention());

    // No cancellation action emitted
    assert_eq!(action_bus.len(), 0);

    // cancel_top API should also reject and return false
    let cancelled = host.cancel_top(&mut action_bus);
    assert!(!cancelled);
    assert!(host.is_active());
}