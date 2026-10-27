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

#[test]
fn test_modal_style_builder_and_defaults() {
    let style = ModalStyle::new()
        .with_backdrop_color(Color::rgba(0.0, 0.0, 0.0, 0.8))
        .with_card_background(Color::rgba(0.1, 0.1, 0.1, 1.0))
        .with_card_border(Color::rgba(0.3, 0.3, 0.3, 1.0), 2.0)
        .with_corner_radius(12.0)
        .with_card_padding(20.0)
        .with_content_gap(16.0)
        .with_footer_gap(10.0)
        .with_header_text_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
        .with_subtitle_color(Color::rgba(0.7, 0.7, 0.7, 1.0))
        .with_danger_color(Color::rgba(0.9, 0.2, 0.2, 1.0))
        .with_cancel_button_width(85.0)
        .with_confirm_button_width(125.0);

    assert_eq!(style.backdrop_color, Color::rgba(0.0, 0.0, 0.0, 0.8));
    assert_eq!(style.card_background, Color::rgba(0.1, 0.1, 0.1, 1.0));
    assert_eq!(style.card_border_color, Color::rgba(0.3, 0.3, 0.3, 1.0));
    assert_eq!(style.card_border_width, 2.0);
    assert_eq!(style.card_corner_radius, 12.0);
    assert_eq!(style.card_padding, 20.0);
    assert_eq!(style.content_gap, 16.0);
    assert_eq!(style.footer_gap, 10.0);
    assert_eq!(style.header_text_color, Color::rgba(1.0, 1.0, 1.0, 1.0));
    assert_eq!(style.subtitle_color, Color::rgba(0.7, 0.7, 0.7, 1.0));
    assert_eq!(style.danger_color, Color::rgba(0.9, 0.2, 0.2, 1.0));
    assert_eq!(style.cancel_button_width, 85.0);
    assert_eq!(style.confirm_button_width, 125.0);

    // Verify light_studio preset
    let light = ModalStyle::light_studio();
    assert_eq!(light.card_border_width, 1.0);
    assert_eq!(light.card_corner_radius, 8.0);
    assert_ne!(light.card_background, style.card_background);
}

#[test]
fn test_confirm_dialog_descriptor_and_rendering() {
    let desc = ConfirmDialogDescriptor::danger("Delete Item", "Are you sure?", "Delete")
        .with_detail("assets/test.obj")
        .with_cancel_label("Abort")
        .with_size(440.0, 200.0);

    assert_eq!(desc.title, "Delete Item");
    assert_eq!(desc.message, "Are you sure?");
    assert_eq!(desc.detail, Some("assets/test.obj".to_string()));
    assert_eq!(desc.confirm_label, "Delete");
    assert_eq!(desc.cancel_label, "Abort");
    assert!(desc.is_danger);
    assert_eq!(desc.size, (440.0, 200.0));

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    // Test Escape dismissal
    let outcome = ModalController::render_confirm_dialog(&mut scope, &desc, viewport, true);
    assert_eq!(
        outcome,
        ModalOutcome::Dismissed(ModalDismissReason::EscapeKey)
    );

    // Test Active state without escape
    let active_outcome = ModalController::render_confirm_dialog(&mut scope, &desc, viewport, false);
    assert_eq!(active_outcome, ModalOutcome::Active);
}

#[test]
fn test_input_dialog_descriptor_and_rendering() {
    let desc = InputDialogDescriptor::new("New Folder", "Folder Name:")
        .with_placeholder("MyFolder")
        .with_detail("Location: assets/")
        .with_confirm_label("Create")
        .with_cancel_label("Cancel")
        .with_size(410.0, 195.0);

    assert_eq!(desc.title, "New Folder");
    assert_eq!(desc.prompt, "Folder Name:");
    assert_eq!(desc.placeholder, "MyFolder");
    assert_eq!(desc.detail, Some("Location: assets/".to_string()));
    assert_eq!(desc.confirm_label, "Create");
    assert_eq!(desc.cancel_label, "Cancel");
    assert_eq!(desc.size, (410.0, 195.0));

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let mut buffer = "NewDirectory".to_string();

    let outcome =
        ModalController::render_input_dialog(&mut scope, &desc, &mut buffer, viewport, false);
    assert_eq!(outcome, ModalOutcome::Active);
}

#[test]
fn test_workbench_ui_scope_ext_modal_methods() {
    use crate::hooks::WorkbenchUiScopeExt;

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let modal = WorkbenchModal::new("Preferences", 700.0, 500.0)
        .with_position(iris_core::geometry::Point::new(100.0, 80.0))
        .with_icon("⚙️")
        .with_subtitle("Editor Settings");

    let outcome: ModalOutcome<i32> =
        scope.workbench_modal(&modal, viewport, false, |_card| ModalOutcome::Action(42));
    assert_eq!(outcome, ModalOutcome::Action(42));

    let confirm = ConfirmDialogDescriptor::new("Save Scene", "Save changes to current scene?");
    let confirm_outcome = scope.confirm_modal(&confirm, viewport, false);
    assert_eq!(confirm_outcome, ModalOutcome::Active);

    let input_desc = InputDialogDescriptor::new("Rename", "New Name:");
    let mut name = "Scene1".to_string();
    let input_outcome = scope.input_modal(&input_desc, &mut name, viewport, false);
    assert_eq!(input_outcome, ModalOutcome::Active);
}