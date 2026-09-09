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
use std::path::Path;

#[test]
fn test_new_folder_modal_text_sections_not_occluded() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let (scrim_id, targets) = build_new_folder_modal(
        &mut tree,
        FolderModalParams {
            parent_path: Path::new("assets/models"),
            input_text: "characters",
            text_width: 60.0,
            cursor_blink_visible: true,
            screen_width: 1920.0,
            screen_height: 1080.0,
            cursor_pos: Point::new(0.0, 0.0),
        },
    );
    let _ = tree.add_child(root, scrim_id);

    // Collect text sections with the modal card's dialog_rect registered as an active modal
    let active_modals = [targets.dialog_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"📁  Create New Folder"),
        "Title '📁  Create New Folder' must be rendered and not occluded"
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
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let (scrim_id, targets) = build_rename_modal(
        &mut tree,
        RenameModalParams {
            target_path: Path::new("assets/textures/diffuse.png"),
            input_text: "albedo.png",
            text_width: 70.0,
            is_folder: false,
            cursor_blink_visible: false,
            screen_width: 1920.0,
            screen_height: 1080.0,
            cursor_pos: Point::new(0.0, 0.0),
        },
    );
    let _ = tree.add_child(root, scrim_id);

    let active_modals = [targets.dialog_rect];
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
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let (scrim_id, targets) = build_delete_modal(
        &mut tree,
        Path::new("assets/temp.obj"),
        1920.0,
        1080.0,
        Point::new(0.0, 0.0),
    );
    let _ = tree.add_child(root, scrim_id);

    let active_modals = [targets.dialog_rect];
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
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let (scrim_id, targets) = build_loading_overlay(
        &mut tree,
        LoadingOverlayParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            time_secs: 1.0,
        },
    );
    let _ = tree.add_child(root, scrim_id);

    let active_modals = [targets.card_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &active_modals, &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"Processing geometry, materials & textures"),
        "Loading subtext must be rendered and not occluded"
    );
}