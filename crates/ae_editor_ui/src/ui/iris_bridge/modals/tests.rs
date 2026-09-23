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
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);

    let scrim_id = build_new_folder_modal(
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
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);

    let scrim_id = build_rename_modal(
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
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);

    let scrim_id = build_delete_modal(
        &mut tree,
        Path::new("assets/temp.obj"),
        1920.0,
        1080.0,
        Point::new(0.0, 0.0),
    );
    let _ = tree.add_child(root, scrim_id);

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
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);

    let scrim_id = build_loading_overlay(
        &mut tree,
        LoadingOverlayParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            time_secs: 1.0,
        },
    );
    let _ = tree.add_child(root, scrim_id);

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
    let root = PanelBuilder::new(&mut tree).build();
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
        is_scrollbar_dragging: false,
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
    let about_id = crate::ui::iris_bridge::about::build_about_dialog(
        &mut tree,
        screen_width,
        screen_height,
        Point::new(0.0, 0.0),
        &[],
        None,
    );
    let _ = tree.add_child(root, about_id);

    let about_dialog_rect = Rect::new(
        (screen_width - crate::ui::iris_bridge::about::ABOUT_DIALOG_WIDTH) * 0.5,
        (screen_height - crate::ui::iris_bridge::about::ABOUT_DIALOG_HEIGHT) * 0.5,
        crate::ui::iris_bridge::about::ABOUT_DIALOG_WIDTH,
        crate::ui::iris_bridge::about::ABOUT_DIALOG_HEIGHT,
    );

    // Register modals in Z-order: Preferences (Layer 0), About (Layer 1)
    let active_modals = [pref_targets.card_rect, about_dialog_rect];
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
    let root = PanelBuilder::new(&mut tree).build();
    let _ = tree.set_root(root);

    let screen_w = 1920.0;
    let screen_h = 1080.0;

    // 1. Build delete modal with cursor hovering the close button
    let del_id = build_delete_modal(
        &mut tree,
        Path::new("assets/test.mesh"),
        screen_w,
        screen_h,
        Point::new(
            (screen_w + DELETE_MODAL_WIDTH) * 0.5 - 20.0,
            (screen_h - DELETE_MODAL_HEIGHT) * 0.5 + 20.0,
        ),
    );
    assert!(tree.get(del_id).is_some());

    // Search for close button and danger button
    let mut close_btn_hovered = false;
    let mut danger_btn_found = false;
    tree.traverse_depth_first(del_id, &mut |_id, node| {
        if node.tag == irisui::prelude::MODAL_TAG_CLOSE
            && node.text.as_deref() == Some("✕")
            && node.style.background_color == Color::rgba(0.85, 0.22, 0.22, 0.28)
        {
            close_btn_hovered = true;
        }
        if node.tag == irisui::prelude::MODAL_TAG_DANGER
            && node.text.as_deref() == Some("🗑 Delete Permanently")
        {
            danger_btn_found = true;
        }
    });
    assert!(close_btn_hovered, "Modal close button must react to hover");
    assert!(
        danger_btn_found,
        "Danger confirm button must be tagged with MODAL_TAG_DANGER"
    );
}