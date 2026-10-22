// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Iris UI Modal Dialogues
//!
//! Verifies that modal dialogues (New Folder, Rename, Delete, Loading) have correct
//! widget roles (`WidgetRole::ModalWindow`) and that their text elements are never
//! occluded or suppressed by their own dialog bounding boxes during text extraction.

use super::*;
use crate::ui::iris_bridge::types::IrisEditorOverlay;
use irisui::prelude::*;
use irisui::widgets::modal::ModalDialogAction;
use std::path::Path;

#[test]
fn test_new_folder_modal_text_sections_not_occluded() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    let mut folder_name = "characters".to_string();
    build_new_folder_modal(
        &mut tree,
        FolderModalParams {
            parent_path: Path::new("assets/models"),
            input_text: &mut folder_name,
            screen_width: 1920.0,
            screen_height: 1080.0,
            cursor_pos: Point::new(0.0, 0.0),
        },
    );

    // Collect text sections with the modal card's dialog_rect registered as an active modal
    let active_modals = [Rect::new(
        (1920.0 - INPUT_MODAL_WIDTH) * 0.5,
        (1080.0 - INPUT_MODAL_HEIGHT) * 0.5,
        INPUT_MODAL_WIDTH,
        INPUT_MODAL_HEIGHT,
    )];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"Create New Folder"),
        "Title 'Create New Folder' must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Cancel"),
        "Cancel button text must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Create Folder"),
        "Create Folder button text must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"characters"),
        "Input text 'characters' must be rendered and not occluded"
    );
}

#[test]
fn test_rename_modal_text_sections_not_occluded() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    let mut asset_name = "albedo.png".to_string();
    build_rename_modal(
        &mut tree,
        RenameModalParams {
            target_path: Path::new("assets/textures/diffuse.png"),
            input_text: &mut asset_name,
            is_folder: false,
            screen_width: 1920.0,
            screen_height: 1080.0,
            cursor_pos: Point::new(0.0, 0.0),
        },
    );

    let active_modals = [Rect::new(
        (1920.0 - INPUT_MODAL_WIDTH) * 0.5,
        (1080.0 - INPUT_MODAL_HEIGHT) * 0.5,
        INPUT_MODAL_WIDTH,
        INPUT_MODAL_HEIGHT,
    )];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"🔄  Rename Asset"),
        "Title '🔄  Rename Asset' must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Cancel"),
        "Cancel button text must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Apply Rename"),
        "Apply Rename button text must be rendered and not occluded"
    );
}

#[test]
fn test_delete_modal_text_sections_not_occluded() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    build_delete_modal(
        &mut tree,
        Path::new("assets/temp.obj"),
        1920.0,
        1080.0,
        Point::new(0.0, 0.0),
    );

    let active_modals = [Rect::new(
        (1920.0 - DELETE_MODAL_WIDTH) * 0.5,
        (1080.0 - DELETE_MODAL_HEIGHT) * 0.5,
        DELETE_MODAL_WIDTH,
        DELETE_MODAL_HEIGHT,
    )];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"⚠️  Confirm Deletion"),
        "Title '⚠️  Confirm Deletion' must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Cancel"),
        "Cancel button text must be rendered and not occluded"
    );
    assert!(
        rendered_texts.contains(&"🗑 Delete Permanently"),
        "Delete Permanently button text must be rendered and not occluded"
    );
}

#[test]
fn test_loading_overlay_text_sections_not_occluded() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    build_loading_overlay(
        &mut tree,
        LoadingOverlayParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            time_secs: 1.0,
        },
    );

    let active_modals = [Rect::new(
        (1920.0 - 420.0) * 0.5,
        (1080.0 - 180.0) * 0.5,
        420.0,
        180.0,
    )];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"Processing geometry, materials & textures"),
        "Loading subtext must be rendered and not occluded"
    );
}

#[test]
fn test_about_dialog_occludes_underlying_preferences_text() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    let screen_width = 1920.0;
    let screen_height = 1080.0;

    // 1. Build Preferences dialog in center
    let collapsed = std::collections::HashSet::new();
    let enabled_modules = std::collections::HashSet::new();
    let mut graphics_settings = ae_renderer::graphics_settings::GraphicsSettings::default();
    let mut snapping_settings = ae_editor::snapping::SnapSettings::default();
    let mut editor_config = ae_editor::editor_state::EditorConfig::default();

    let pref_left =
        ((screen_width - crate::ui::iris_bridge::preferences::builder::PREF_CARD_WIDTH) * 0.5)
            .round();
    let pref_top =
        ((screen_height - crate::ui::iris_bridge::preferences::builder::PREF_CARD_HEIGHT) * 0.5)
            .round();

    let pref_params = crate::ui::iris_bridge::preferences::types::PreferencesParams {
        screen_width,
        screen_height,
        window_pos: Some(Point::new(pref_left, pref_top)),
        active_tab: 0,
        collapsed_sections: &collapsed,
        blink_caret: false,
        active_number_input: None,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &mut graphics_settings,
        snapping_settings: &mut snapping_settings,
        editor_config: &mut editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
        events: &[],
    };
    let (_pref_id, pref_card_rect, _, _) =
        crate::ui::iris_bridge::preferences::builder::build_preferences_dialog(
            &mut tree,
            pref_params,
            &mut Vec::new(),
        );

    // 2. Build About dialog centered on top of Preferences
    crate::ui::iris_bridge::about::build_about_dialog(
        &mut tree,
        screen_width,
        screen_height,
        Point::new(0.0, 0.0),
        &[],
        None,
    );

    let about_dialog_rect = Rect::new(
        (screen_width - crate::ui::iris_bridge::about::ABOUT_DIALOG_WIDTH) * 0.5,
        (screen_height - crate::ui::iris_bridge::about::ABOUT_DIALOG_HEIGHT) * 0.5,
        crate::ui::iris_bridge::about::ABOUT_DIALOG_WIDTH,
        crate::ui::iris_bridge::about::ABOUT_DIALOG_HEIGHT,
    );

    // Register modals in Z-order: Preferences (Layer 0), About (Layer 1)
    let active_modals = [pref_card_rect, about_dialog_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    // Topmost About dialog text must render cleanly
    assert!(
        rendered_texts
            .iter()
            .any(|t| t.contains("About Aeon Engine")),
        "Topmost About header title must be rendered without occlusion"
    );
    assert!(
        rendered_texts.iter().any(|t| t.contains("Aeon Engine")),
        "About dialog title must be rendered without occlusion"
    );

    // Any Preferences text that falls inside About's dialog_rect must either be fully occluded
    // or scissor-clipped so that no visible glyph falls inside the About dialog boundaries.
    for section in &sections {
        let visible_bounds = match section.clip_bounds {
            Some(clip) => section.bounds.intersect(clip),
            None => section.bounds,
        };

        let visible_center_x = visible_bounds.x + visible_bounds.width * 0.5;
        let visible_center_y = visible_bounds.y + visible_bounds.height * 0.5;
        let visible_inside_about = visible_bounds.width > 0.0
            && visible_bounds.height > 0.0
            && visible_center_x >= about_dialog_rect.x
            && visible_center_x <= about_dialog_rect.x + about_dialog_rect.width
            && visible_center_y >= about_dialog_rect.y
            && visible_center_y <= about_dialog_rect.y + about_dialog_rect.height;

        if visible_inside_about {
            let is_about_text = section.text.contains("About")
                || section.text.contains("Aeon Engine")
                || section.text.contains("Copyright")
                || section.text.contains("Mozilla")
                || section.text.contains("License")
                || section.text.contains("MPL")
                || section.text.contains("WARRANTY")
                || section.text.contains("Close")
                || section.text == "✕";
            assert!(
                is_about_text,
                "Visible text '{}' inside About dialog bounds must belong to About, not bleed from Preferences",
                section.text
            );
        }
    }
}

#[test]
fn test_declarative_modals_buttons_hover_reactivity() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().expect("Root node must be created");

    let screen_w = 1920.0;
    let screen_h = 1080.0;

    // 1. Build delete modal with cursor hovering the close button
    build_delete_modal(
        &mut tree,
        Path::new("assets/test.mesh"),
        screen_w,
        screen_h,
        Point::new(
            (screen_w + DELETE_MODAL_WIDTH) * 0.5 - 20.0,
            (screen_h - DELETE_MODAL_HEIGHT) * 0.5 + 20.0,
        ),
    );
    let del_id = tree
        .find_node_by_tag(ModalDialogAction::TAG_SCRIM)
        .expect("Modal scrim must exist");
    assert!(tree.get(del_id).is_some());

    // Search for close button and danger button
    assert!(
        tree.any_in_subtree(del_id, |node| {
            node.tag == ModalDialogAction::TAG_CLOSE
                && node.text.as_deref() == Some("✕")
                && node.style.background_color == Color::rgba(0.85, 0.22, 0.22, 0.28)
        }),
        "Modal close button must react to hover"
    );
    assert!(
        tree.any_in_subtree(del_id, |node| {
            node.tag == ModalDialogAction::TAG_DANGER
                && node.text.as_deref() == Some("🗑 Delete Permanently")
        }),
        "Danger confirm button must be tagged with MODAL_TAG_DANGER"
    );
}

#[test]
fn test_modals_state_sync_from_dialogs_authoritative_retention() {
    let mut state = ModalsOverlayState::default();
    assert!(!state.is_about_active);
    assert!(!state.is_delete_active);
    assert!(!state.is_new_folder_active);
    assert!(!state.is_rename_active);
    assert!(!state.is_loading_active);

    // 1. Synchronize with About, Delete, and Loading active
    let path = Path::new("test/path");
    let dialogs = crate::ui::iris_bridge::types::DialogParams {
        show_about: true,
        show_preferences: false,
        delete_target: Some(path),
        new_folder_parent: None,
        rename_target: None,
        is_loading_assets: true,
    };
    state.sync_from_dialogs(&dialogs);
    assert!(state.is_about_active);
    assert!(state.is_delete_active);
    assert!(!state.is_new_folder_active);
    assert!(!state.is_rename_active);
    assert!(state.is_loading_active);

    // 2. Synchronize with all dialogs closed
    let empty_dialogs = crate::ui::iris_bridge::types::DialogParams {
        show_about: false,
        show_preferences: false,
        delete_target: None,
        new_folder_parent: None,
        rename_target: None,
        is_loading_assets: false,
    };
    state.sync_from_dialogs(&empty_dialogs);
    assert!(!state.is_about_active);
    assert!(!state.is_delete_active);
    assert!(!state.is_new_folder_active);
    assert!(!state.is_rename_active);
    assert!(!state.is_loading_active);
}

#[test]
fn test_modal_close_button_tag_and_layer() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal_close_button();
        resp.id
    };
    let node = tree.get(btn_id).unwrap();
    assert_eq!(node.tag, ModalDialogAction::TAG_CLOSE);
    assert_eq!(node.layer, UiLayer::Modal);
    assert!(node.interactive);
    assert_eq!(node.role, WidgetRole::Button);
}

#[test]
fn test_modal_action_buttons_layer_and_tags() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let (confirm_id, cancel_id, danger_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let confirm = scope.modal_confirm_button("Confirm", 100.0);
        let cancel = scope.modal_cancel_button("Cancel", 100.0);
        let danger = scope.modal_danger_button("Delete", 100.0);
        (confirm.id, cancel.id, danger.id)
    };

    let c_node = tree.get(confirm_id).unwrap();
    assert_eq!(c_node.tag, ModalDialogAction::TAG_CONFIRM);
    assert_eq!(c_node.layer, UiLayer::Modal);
    assert!(c_node.interactive);

    let can_node = tree.get(cancel_id).unwrap();
    assert_eq!(can_node.tag, ModalDialogAction::TAG_CANCEL);
    assert_eq!(can_node.layer, UiLayer::Modal);
    assert!(can_node.interactive);

    let d_node = tree.get(danger_id).unwrap();
    assert_eq!(d_node.tag, ModalDialogAction::TAG_DANGER);
    assert_eq!(d_node.layer, UiLayer::Modal);
    assert!(d_node.interactive);
}

#[test]
fn test_modal_input_box_tag_and_layer() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().unwrap();
    let mut folder_name = "NewFolder".to_string();
    build_new_folder_modal(
        &mut tree,
        FolderModalParams {
            parent_path: Path::new("assets/scenes"),
            input_text: &mut folder_name,
            screen_width: 1920.0,
            screen_height: 1080.0,
            cursor_pos: Point::new(0.0, 0.0),
        },
    );

    let input_node = tree
        .node_named("TextInput")
        .expect("Node with name 'TextInput' must exist");

    assert_eq!(input_node.role, WidgetRole::TextInput);
    assert!(
        input_node.interactive,
        "Modal text input must be interactive"
    );
    assert_eq!(
        input_node.layer,
        UiLayer::Modal,
        "Modal text input must inherit Modal layer"
    );
}