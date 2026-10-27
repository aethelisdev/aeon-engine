// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use iris_core::Color;
use iris_core::UiTree;
use iris_core::geometry::Rect;
use iris_widgets::declarative::UiScope;

#[test]
fn test_modal_descriptor_defaults_and_builder() {
    let desc = ModalDescriptor::new("Settings", 500.0, 350.0)
        .with_backdrop_color(Color::rgba(0.1, 0.1, 0.1, 0.8))
        .with_dismiss_on_backdrop(false)
        .with_dismiss_on_escape(false)
        .with_close_button(false);

    assert_eq!(desc.title, "Settings");
    assert_eq!(desc.size, (500.0, 350.0));
    assert_eq!(desc.backdrop_color, Color::rgba(0.1, 0.1, 0.1, 0.8));
    assert!(!desc.dismiss_on_backdrop);
    assert!(!desc.dismiss_on_escape);
    assert!(!desc.show_close_button);
}

#[test]
fn test_floating_window_descriptor_clamping() {
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    // 1. Off-screen to the top-left (negative coordinates)
    let win_off_left =
        FloatingWindowDescriptor::new("panel_1", "Panel 1", Rect::new(-100.0, -50.0, 400.0, 300.0))
            .clamp_to_viewport(viewport);

    assert_eq!(win_off_left.rect.x, 0.0);
    assert_eq!(win_off_left.rect.y, 0.0);
    assert_eq!(win_off_left.rect.width, 400.0);
    assert_eq!(win_off_left.rect.height, 300.0);

    // 2. Off-screen to the bottom-right (pushed past viewport bounds)
    let win_off_right = FloatingWindowDescriptor::new(
        "panel_2",
        "Panel 2",
        Rect::new(2000.0, 1200.0, 400.0, 300.0),
    )
    .clamp_to_viewport(viewport);

    assert_eq!(win_off_right.rect.x, 1920.0 - 400.0);
    assert_eq!(win_off_right.rect.y, 1080.0 - 300.0);

    // 3. Smaller than minimum dimensions
    let win_too_small =
        FloatingWindowDescriptor::new("panel_3", "Panel 3", Rect::new(100.0, 100.0, 50.0, 40.0))
            .clamp_to_viewport(viewport);

    assert_eq!(
        win_too_small.rect.width,
        FloatingWindowDescriptor::<&str>::DEFAULT_MIN_SIZE.0
    );
    assert_eq!(
        win_too_small.rect.height,
        FloatingWindowDescriptor::<&str>::DEFAULT_MIN_SIZE.1
    );
}

#[test]
fn test_modal_dismiss_on_escape() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let desc = ModalDescriptor::new("Alert", 300.0, 200.0);
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);

    let outcome: ModalOutcome<()> = ModalController::render_modal(
        &mut scope,
        &desc,
        viewport,
        true, // Escape key pressed
        |_card| ModalOutcome::Active,
    );

    assert_eq!(
        outcome,
        ModalOutcome::Dismissed(ModalDismissReason::EscapeKey)
    );
    assert!(outcome.is_dismissed());
    assert!(!outcome.is_active());
}

#[test]
fn test_modal_escape_disabled_keeps_modal_active() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let desc = ModalDescriptor::new("Mandatory Dialog", 300.0, 200.0).with_dismiss_on_escape(false);
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);

    let outcome: ModalOutcome<()> = ModalController::render_modal(
        &mut scope,
        &desc,
        viewport,
        true, // Escape pressed, but ignored
        |_card| ModalOutcome::Active,
    );

    assert_eq!(outcome, ModalOutcome::Active);
    assert!(outcome.is_active());
    assert!(!outcome.is_dismissed());
}

#[test]
fn test_modal_action_propagation() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let desc = ModalDescriptor::new("Confirm", 300.0, 200.0);
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);

    let outcome: ModalOutcome<String> =
        ModalController::render_modal(&mut scope, &desc, viewport, false, |_card| {
            ModalOutcome::Action("Confirmed".to_string())
        });

    assert_eq!(outcome, ModalOutcome::Action("Confirmed".to_string()));
    assert_eq!(outcome.action(), Some("Confirmed".to_string()));
}

#[test]
fn test_show_confirm_dialog_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let desc = ModalDescriptor::new("Delete Entity", 380.0, 160.0);
    let viewport = Rect::new(0.0, 0.0, 1280.0, 720.0);

    let outcome = ModalController::show_confirm_dialog(
        &mut scope,
        &desc,
        viewport,
        false,
        "Are you sure you want to delete this entity?",
        "Delete",
        true,
    );

    // Initial pass with no button click returns Active
    assert_eq!(outcome, ModalOutcome::Active);
}