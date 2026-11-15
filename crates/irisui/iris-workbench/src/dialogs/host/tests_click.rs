// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Modal Tag Click Handling & `ActionBus` Dispatch
//!
//! Validates [`DialogHost::handle_clicked_tag`] and [`Workbench`] modal button/scrim
//! click routing into [`ActionBus`].

use super::DialogHost;
use crate::action_bus::ActionBus;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::info::InfoCardDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::workbench::Workbench;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_core::tree::UiTree;
use iris_dock::{DockState, DockTree};
use iris_widgets::modal::{
    MODAL_TAG_CANCEL, MODAL_TAG_CARD, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER,
    MODAL_TAG_SCRIM,
};
use iris_winit::{CursorIcon, ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, PartialEq, Eq)]
enum MockModalAction {
    Confirmed,
    Cancelled,
    Submitted(String),
    InfoClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MockTab {
    Scene,
}

#[derive(Debug, Default)]
struct MockWindow {
    logical_size: Size,
}

impl WindowInterface for MockWindow {
    fn set_cursor(&mut self, _icon: CursorIcon) {}

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

#[test]
fn test_dialog_host_handle_clicked_tag_confirm_and_danger_push_action_bus() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open(
        ConfirmationDialogDescriptor::new("Delete", "Delete item?", MockModalAction::Confirmed)
            .with_cancel_action(MockModalAction::Cancelled),
    );
    assert!(host.handle_clicked_tag(MODAL_TAG_CONFIRM, &mut bus));
    assert!(!host.is_active());
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockModalAction::Confirmed]
    );

    host.open(
        ConfirmationDialogDescriptor::new(
            "Delete",
            "Delete permanently?",
            MockModalAction::Confirmed,
        )
        .with_danger(true)
        .with_cancel_action(MockModalAction::Cancelled),
    );
    assert!(host.handle_clicked_tag(MODAL_TAG_DANGER, &mut bus));
    assert!(!host.is_active());
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockModalAction::Confirmed]
    );
}

#[test]
fn test_dialog_host_handle_clicked_tag_input_submit_and_cancel() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open(
        PromptDialogDescriptor::new("Rename", "Name:", MockModalAction::Submitted)
            .with_initial_text("player_v2")
            .with_cancellable(true)
            .with_cancel_action(MockModalAction::Cancelled),
    );
    assert!(host.handle_clicked_tag(MODAL_TAG_CONFIRM, &mut bus));
    assert!(!host.is_active());
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockModalAction::Submitted("player_v2".to_owned())]
    );

    host.open(
        PromptDialogDescriptor::new("Rename", "Name:", MockModalAction::Submitted)
            .with_cancellable(true)
            .with_cancel_action(MockModalAction::Cancelled),
    );
    assert!(host.handle_clicked_tag(MODAL_TAG_CANCEL, &mut bus));
    assert!(!host.is_active());
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockModalAction::Cancelled]
    );
}

#[test]
fn test_dialog_host_handle_clicked_tag_info_close_scrim_and_card_invariants() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    host.open_info(
        InfoCardDialogDescriptor::new("About")
            .with_dismiss_on_backdrop(false)
            .with_on_close(MockModalAction::InfoClosed),
    );

    // Clicking card body consumes without dismissing
    assert!(host.handle_clicked_tag(MODAL_TAG_CARD, &mut bus));
    assert!(host.is_active());
    assert!(bus.is_empty());

    // Clicking scrim when dismiss_on_backdrop is false triggers attention and keeps dialog open
    assert!(host.handle_clicked_tag(MODAL_TAG_SCRIM, &mut bus));
    assert!(host.is_active());
    assert!(host.is_demanding_attention());
    assert!(bus.is_empty());

    // Clicking close emits on_close into ActionBus
    assert!(host.handle_clicked_tag(MODAL_TAG_CLOSE, &mut bus));
    assert!(!host.is_active());
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockModalAction::InfoClosed]
    );
}

#[test]
fn test_dialog_host_handle_clicked_tag_non_cancellable_and_unmatched() {
    let mut host = DialogHost::new();
    let mut bus = ActionBus::new();

    assert!(!host.handle_clicked_tag(MODAL_TAG_CONFIRM, &mut bus));
    assert!(!host.handle_clicked_tag(0, &mut bus));

    host.open(
        ConfirmationDialogDescriptor::new(
            "Required",
            "Confirm required",
            MockModalAction::Confirmed,
        )
        .with_cancellable(false)
        .with_cancel_action(MockModalAction::Cancelled),
    );

    assert!(!host.handle_clicked_tag(0xDEAD_BEEF, &mut bus));
    assert!(host.handle_clicked_tag(MODAL_TAG_CANCEL, &mut bus));
    assert!(host.is_active());
    assert!(host.is_demanding_attention());
    assert!(bus.is_empty());
}

#[test]
fn test_workbench_modal_click_dispatches_action_into_outcome() {
    let mut dock_tree = DockTree::new();
    let leaf = dock_tree.create_leaf(vec![MockTab::Scene]);
    dock_tree.set_root(leaf);
    let mut dock_state = DockState::new(dock_tree);
    let descriptor = WorkbenchDescriptor::new("Test", dock_state.clone());
    let mut workbench: Workbench<MockTab, MockModalAction, ()> = Workbench::new(descriptor);
    let mut window = MockWindow {
        logical_size: Size::new(1280.0, 720.0),
    };

    workbench.dialog_host.open(
        ConfirmationDialogDescriptor::new("Confirm", "Proceed?", MockModalAction::Confirmed)
            .with_cancel_action(MockModalAction::Cancelled),
    );

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let btn = tree.create_node();
    let _ = tree.add_child(root, btn);
    {
        let node = tree.get_mut(btn).expect("modal button node");
        node.tag = MODAL_TAG_CONFIRM;
        node.interactive = true;
        node.computed_rect = Rect::new(500.0, 400.0, 100.0, 32.0);
    }

    let down = UiEvent::MouseDown {
        point: Point::new(520.0, 415.0),
        button: MouseButton::Left,
    };
    let _ = workbench.handle_event_with_content_tree(
        &mut window,
        &down,
        &mut dock_state,
        Some(&mut tree),
        |_| (false, None, Vec::new()),
    );

    let up = UiEvent::MouseUp {
        point: Point::new(520.0, 415.0),
        button: MouseButton::Left,
    };
    let outcome = workbench.handle_event_with_content_tree(
        &mut window,
        &up,
        &mut dock_state,
        Some(&mut tree),
        |_| (false, None, Vec::new()),
    );

    assert!(outcome.consumed);
    assert!(outcome.layout_changed);
    assert!(!workbench.dialog_host.is_active());
    assert_eq!(outcome.actions, vec![MockModalAction::Confirmed]);
    assert!(
        tree.event_state.frame_clicked_tag.is_none(),
        "Consumed modal click tag must be cleared to prevent double-firing on render"
    );
}