// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use iris_core::Color;
use iris_core::UiTree;
use iris_core::geometry::{Point, Rect};
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    DeleteAsset(String),
    Rename(String),
    Close,
}

#[test]
fn test_dialog_descriptor_variants_and_queries() {
    // 1. Modal
    let modal_desc: DialogDescriptor<TestAction> =
        DialogDescriptor::modal("Settings", 600.0, 450.0);
    assert_eq!(modal_desc.title(), "Settings");
    assert_eq!(modal_desc.size(), (600.0, 450.0));
    assert!(modal_desc.dismiss_on_escape());
    assert!(modal_desc.dismiss_on_backdrop());
    assert!(modal_desc.show_close_button());

    // 2. Sheet
    let sheet_desc: DialogDescriptor<TestAction> =
        DialogDescriptor::sheet("Drawer", SheetAnchor::Bottom, 320.0);
    assert_eq!(sheet_desc.title(), "Drawer");
    assert_eq!(sheet_desc.size(), (800.0, 320.0));

    // 3. Confirmation
    let confirm_desc =
        DialogDescriptor::confirmation("Save File", "Save before quitting?", TestAction::Close);
    assert_eq!(confirm_desc.title(), "Save File");
    assert_eq!(confirm_desc.size(), (420.0, 180.0));

    // 4. Danger Confirmation
    let danger_desc = DialogDescriptor::danger(
        "Delete",
        "Delete asset permanently?",
        "Delete",
        TestAction::DeleteAsset("tex.png".to_string()),
    );
    assert_eq!(danger_desc.title(), "Delete");

    // 5. Prompt
    let prompt_desc = DialogDescriptor::prompt("Rename Asset", "New Name:", TestAction::Rename);
    assert_eq!(prompt_desc.title(), "Rename Asset");
    assert_eq!(prompt_desc.size(), (400.0, 190.0));

    // 6. Custom View
    let custom_desc = DialogDescriptor::<TestAction>::custom_view(
        "Inspector Overlay",
        (500.0, 400.0),
        |_s, _b| {},
    );
    assert_eq!(custom_desc.title(), "Inspector Overlay");
    assert_eq!(custom_desc.size(), (500.0, 400.0));
}

#[test]
fn test_sheet_dialog_descriptor_anchors_and_extents() {
    let bottom: SheetDialogDescriptor<TestAction> = SheetDialogDescriptor::bottom("Console", 250.0);
    assert_eq!(bottom.anchor, SheetAnchor::Bottom);
    assert!(bottom.anchor.is_vertical());
    assert!(!bottom.anchor.is_horizontal());
    assert_eq!(bottom.extent, 250.0);

    let left: SheetDialogDescriptor<TestAction> =
        SheetDialogDescriptor::left("Tool Palette", 300.0);
    assert_eq!(left.anchor, SheetAnchor::Left);
    assert!(left.anchor.is_horizontal());
    assert!(!left.anchor.is_vertical());
    assert_eq!(left.extent, 300.0);

    let top: SheetDialogDescriptor<TestAction> =
        SheetDialogDescriptor::top("Command Palette", 200.0);
    assert_eq!(top.anchor, SheetAnchor::Top);

    let right: SheetDialogDescriptor<TestAction> =
        SheetDialogDescriptor::right("Inspector Drawer", 350.0);
    assert_eq!(right.anchor, SheetAnchor::Right);
}

#[test]
fn test_dialog_descriptor_from_conversions() {
    let modal = ModalDescriptor::new("Modal Title", 500.0, 400.0);
    let d1: DialogDescriptor<TestAction> = modal.into();
    assert_eq!(d1.title(), "Modal Title");

    let sheet: SheetDialogDescriptor<TestAction> =
        SheetDialogDescriptor::bottom("Sheet Title", 300.0);
    let d2: DialogDescriptor<TestAction> = sheet.into();
    assert_eq!(d2.title(), "Sheet Title");

    let confirm = ConfirmationDialogDescriptor::new("Confirm Title", "Msg", TestAction::Close);
    let d3: DialogDescriptor<TestAction> = confirm.into();
    assert_eq!(d3.title(), "Confirm Title");

    let prompt = PromptDialogDescriptor::new("Prompt Title", "Prompt", TestAction::Rename);
    let d4: DialogDescriptor<TestAction> = prompt.into();
    assert_eq!(d4.title(), "Prompt Title");

    let custom =
        CustomDialogDescriptor::<TestAction>::new("Custom Title", (300.0, 200.0), |_s, _b| {});
    let d5: DialogDescriptor<TestAction> = custom.into();
    assert_eq!(d5.title(), "Custom Title");
}

#[test]
fn test_custom_dialog_descriptor_builder_and_options() {
    let custom = CustomDialogDescriptor::<TestAction>::new(
        "Editor Preferences",
        (800.0, 600.0),
        |_s, _b| {},
    )
    .with_position(Point::new(100.0, 80.0))
    .with_subtitle("Configure graphics and controls")
    .with_icon("⚙️")
    .with_backdrop_color(Color::rgba(0.0, 0.0, 0.0, 0.4))
    .with_header(false)
    .with_dismiss_on_backdrop(false)
    .with_dismiss_on_escape(true)
    .with_close_button(false)
    .with_size(860.0, 640.0);

    assert_eq!(custom.title, "Editor Preferences");
    assert_eq!(custom.size, (860.0, 640.0));
    assert_eq!(custom.position, Some(Point::new(100.0, 80.0)));
    assert_eq!(
        custom.subtitle.as_deref(),
        Some("Configure graphics and controls")
    );
    assert_eq!(custom.icon.as_deref(), Some("⚙️"));
    assert_eq!(custom.backdrop_color, Some(Color::rgba(0.0, 0.0, 0.0, 0.4)));
    assert!(!custom.show_header);
    assert!(!custom.dismiss_on_backdrop);
    assert!(custom.dismiss_on_escape);
    assert!(!custom.show_close_button);
}

#[test]
fn test_info_card_descriptor_builder_and_defaults() {
    let card: InfoCardDialogDescriptor<TestAction> =
        InfoCardDialogDescriptor::new("About Application")
            .with_app_name("Aeon Creative Suite")
            .with_version("0.8.2")
            .with_copyright("Copyright © 2026 AethelisDEV")
            .with_description("Professional 3D real-time simulation and world editing engine.")
            .with_entry("Renderer", "WGPU Vulkan Backend")
            .with_entry("Physics", "Rapier3D Multithreaded")
            .with_entry("Platform", "Linux x86_64")
            .with_footer_notice("Licensed under Mozilla Public License 2.0")
            .with_close_label("Dismiss")
            .with_on_close(TestAction::Close)
            .with_dismiss_on_backdrop(true)
            .with_dismiss_on_escape(true)
            .with_close_button(true);

    assert_eq!(card.title, "About Application");
    assert_eq!(card.app_name.as_deref(), Some("Aeon Creative Suite"));
    assert_eq!(card.version.as_deref(), Some("0.8.2"));
    assert_eq!(
        card.copyright.as_deref(),
        Some("Copyright © 2026 AethelisDEV")
    );
    assert_eq!(
        card.description.as_deref(),
        Some("Professional 3D real-time simulation and world editing engine.")
    );
    assert_eq!(card.entries.len(), 3);
    assert_eq!(card.entries[0].label, "Renderer");
    assert_eq!(card.entries[0].value, "WGPU Vulkan Backend");
    assert_eq!(
        card.footer_notice.as_deref(),
        Some("Licensed under Mozilla Public License 2.0")
    );
    assert_eq!(card.close_label, "Dismiss");
    assert_eq!(card.on_close, Some(TestAction::Close));
    assert!(card.dismiss_on_backdrop);
    assert!(card.dismiss_on_escape);
    assert!(card.show_close_button);
}

#[test]
fn test_info_card_dialog_from_conversion() {
    let card =
        InfoCardDialogDescriptor::<TestAction>::new("Diagnostics Card").with_entry("FPS", "60.0");
    let desc: DialogDescriptor<TestAction> = card.into();
    assert_eq!(desc.title(), "Diagnostics Card");
    assert_eq!(
        desc.size(),
        InfoCardDialogDescriptor::<TestAction>::DEFAULT_SIZE
    );
}

#[test]
fn test_progress_modal_descriptor_builder_and_defaults() {
    let desc: ProgressModalDescriptor<TestAction> =
        ProgressModalDescriptor::new("Baking Lightmaps")
            .with_status_message("Rasterizing irradiance probes (3/12)...")
            .with_progress(0.25)
            .with_progress_color(Color::rgb(0.2, 0.8, 1.0))
            .with_progress_height(8.0)
            .with_cancellable(true)
            .with_cancel_label("Abort Task")
            .with_on_cancel(TestAction::Close)
            .with_subtitle("Lighting Compilation")
            .with_icon("💡")
            .with_size(500.0, 220.0)
            .with_backdrop_color(Color::rgba(0.0, 0.0, 0.0, 0.7))
            .with_dismiss_on_backdrop(false);

    assert_eq!(desc.title, "Baking Lightmaps");
    assert_eq!(
        desc.status_message.as_deref(),
        Some("Rasterizing irradiance probes (3/12)...")
    );
    assert_eq!(desc.progress, 0.25);
    assert_eq!(desc.progress_color, Some(Color::rgb(0.2, 0.8, 1.0)));
    assert_eq!(desc.progress_height, 8.0);
    assert!(desc.is_cancellable);
    assert_eq!(desc.cancel_label, "Abort Task");
    assert_eq!(desc.on_cancel, Some(TestAction::Close));
    assert_eq!(desc.subtitle.as_deref(), Some("Lighting Compilation"));
    assert_eq!(desc.icon.as_deref(), Some("💡"));
    assert_eq!(desc.size, (500.0, 220.0));
    assert_eq!(desc.backdrop_color, Some(Color::rgba(0.0, 0.0, 0.0, 0.7)));
    assert!(!desc.dismiss_on_backdrop);
}

#[test]
fn test_progress_modal_descriptor_clamping() {
    let underflow: ProgressModalDescriptor<TestAction> =
        ProgressModalDescriptor::new("Underflow").with_progress(-0.5);
    assert_eq!(underflow.progress, 0.0);

    let overflow: ProgressModalDescriptor<TestAction> =
        ProgressModalDescriptor::new("Overflow").with_progress(1.5);
    assert_eq!(overflow.progress, 1.0);
}

#[test]
fn test_progress_modal_dialog_from_conversion() {
    let prog = ProgressModalDescriptor::<TestAction>::new("Importing Mesh")
        .with_progress(0.60)
        .with_cancellable(false);

    let desc: DialogDescriptor<TestAction> = prog.into();
    assert_eq!(desc.title(), "Importing Mesh");
    assert_eq!(
        desc.size(),
        ProgressModalDescriptor::<TestAction>::DEFAULT_SIZE
    );
    assert!(!desc.dismiss_on_escape());
    assert!(!desc.dismiss_on_backdrop());
    assert!(!desc.show_close_button());
}

#[test]
fn test_modal_style_attention_configuration() {
    let style = ModalStyle::dark_studio()
        .with_attention_border_color(Color::rgba(1.0, 0.5, 0.0, 1.0))
        .with_attention_shake_amplitude(10.0);

    assert_eq!(
        style.attention_border_color,
        Color::rgba(1.0, 0.5, 0.0, 1.0)
    );
    assert_eq!(style.attention_shake_amplitude, 10.0);
}

#[test]
fn test_modal_outcome_attention_query() {
    let outcome: ModalOutcome<()> = ModalOutcome::Attention;
    assert!(outcome.is_attention());
    assert!(!outcome.is_dismissed());
    assert!(!outcome.is_action());
    assert!(!outcome.is_active());
}

#[test]
fn test_dialog_descriptor_is_cancellable_propagation() {
    let cancellable_conf = ConfirmationDialogDescriptor::new(
        "Delete",
        "Confirm delete?",
        TestAction::DeleteAsset("test.png".into()),
    )
    .with_cancel_action(TestAction::Close);
    let desc: DialogDescriptor<TestAction> = cancellable_conf.into();
    assert!(desc.is_cancellable());
    assert!(desc.dismiss_on_escape());
    assert!(desc.dismiss_on_backdrop());
    assert!(desc.show_close_button());

    let non_cancellable_conf = ConfirmationDialogDescriptor::new(
        "Deleting...",
        "Operation cannot be aborted.",
        TestAction::DeleteAsset("test.png".into()),
    )
    .with_cancellable(false);
    let desc_non_canc: DialogDescriptor<TestAction> = non_cancellable_conf.into();
    assert!(!desc_non_canc.is_cancellable());
    assert!(!desc_non_canc.dismiss_on_escape());
    assert!(!desc_non_canc.dismiss_on_backdrop());
    assert!(!desc_non_canc.show_close_button());
}

#[test]
fn test_prompt_descriptor_cancellable_and_action() {
    let prompt = PromptDialogDescriptor::new("Rename Asset", "New Name:", TestAction::Rename)
        .with_cancellable(false)
        .with_cancel_action(TestAction::Close);

    let desc: DialogDescriptor<TestAction> = prompt.into();
    assert!(!desc.is_cancellable());
    assert!(!desc.dismiss_on_escape());
    assert!(!desc.dismiss_on_backdrop());
    assert!(!desc.show_close_button());
}