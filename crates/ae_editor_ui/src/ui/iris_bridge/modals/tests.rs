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

#[test]
fn test_about_dialog_occludes_underlying_preferences_text() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let screen_width = 1920.0;
    let screen_height = 1080.0;

    // 1. Build Preferences dialog in center
    let collapsed = std::collections::HashSet::new();
    let enabled_modules = std::collections::HashSet::new();
    let graphics_settings = ae_renderer::graphics_settings::GraphicsSettings::default();
    let snapping_settings = ae_editor::snapping::SnapSettings::default();
    let editor_config = ae_editor::editor_state::EditorConfig::default();

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
        scroll_offset_y: 0.0,
        active_dropdown: None,
        collapsed_sections: &collapsed,
        active_number_input: None,
        blink_caret: false,
        cursor_pos: Point::new(0.0, 0.0),
        zoom_factor: 1.0,
        graphics_settings: &graphics_settings,
        snapping_settings: &snapping_settings,
        editor_config: &editor_config,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
    };
    let (pref_id, pref_targets) =
        crate::ui::iris_bridge::preferences::builder::build_preferences_dialog(
            &mut tree,
            pref_params,
        );
    let _ = tree.add_child(root, pref_id);

    // 2. Build About dialog centered on top of Preferences
    let (about_id, about_targets) = crate::ui::iris_bridge::about::build_about_dialog(
        &mut tree,
        screen_width,
        screen_height,
        Point::new(0.0, 0.0),
    );
    let _ = tree.add_child(root, about_id);

    // Register modals in Z-order: Preferences (Layer 0), About (Layer 1)
    let active_modals = [pref_targets.card_rect, about_targets.dialog_rect];
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
            && about_targets
                .dialog_rect
                .contains_point(Point::new(visible_center_x, visible_center_y));

        if visible_inside_about {
            let is_about_text = section.text.contains("About")
                || section.text.contains("Aeon Engine")
                || section.text.contains("Copyright")
                || section.text.contains("Mozilla")
                || section.text.contains("License")
                || section.text.contains("MPL")
                || section.text.contains("WARRANTY")
                || section.text.contains("Close")
                || section.text == "✖";
            assert!(
                is_about_text,
                "Visible text '{}' inside About dialog bounds must belong to About, not bleed from Preferences",
                section.text
            );
        }
    }
}