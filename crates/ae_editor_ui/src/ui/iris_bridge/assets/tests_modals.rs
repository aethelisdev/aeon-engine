// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modal, Context Menu, and Single DialogHost Integration Tests for the Asset Browser.

use super::panel::build_assets_panel;
use super::types::{AssetPreviewModalState, AssetsPanelAction, AssetsPanelState};
use crate::ui::types::EngineUiAction;
use ae_assets::types::{AssetBrowserState, AssetCategory, AssetItem, AssetSource};
use irisui::prelude::*;
use irisui::widgets::modal::ModalDialogAction;
use irisui::workbench::{ActionBus, DialogHost, WorkbenchUiScopeExt};
use std::path::PathBuf;

fn build_in_host(
    tree: &mut UiTree,
    root: WidgetId,
    bounds: Rect,
    f: impl FnOnce(&mut UiScope<'_>),
) {
    let mut scope = UiScope::new(tree, root);
    scope.panel("AssetsModalTestHost").bounds(bounds).show(f);
}

#[test]
fn test_preview_modal_declarative_scene_and_buttons_hover_reactivity() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let scene_item = AssetItem {
        name: "test_level.aeworld".to_string(),
        path: PathBuf::from("assets/scenes/test_level.aeworld"),
        relative_path: "scenes/test_level.aeworld".to_string(),
        category: AssetCategory::Scenes,
        source: AssetSource::Project,
        file_size_bytes: 4096,
        metadata_badge: "4.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };

    let preview_state = AssetPreviewModalState {
        item: scene_item,
        orbit_yaw: 0.0,
        orbit_pitch: 0.0,
        zoom_distance: 1.0,
        show_wireframe: false,
    };

    let asset_browser = AssetBrowserState::default();
    let mut panel_state = AssetsPanelState {
        preview_modal: Some(preview_state),
        ..Default::default()
    };

    let mut actions_idle = Vec::new();
    build_in_host(
        &mut tree,
        root_id,
        Rect::new(0.0, 0.0, 1280.0, 720.0),
        |s| {
            build_assets_panel(
                s,
                &asset_browser,
                false,
                &mut panel_state,
                &mut actions_idle,
            );
        },
    );

    let mut has_test_level_title = false;
    let mut close_rect = Rect::ZERO;
    let mut reveal_rect = Rect::ZERO;
    let mut has_action_btn = false;

    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.text.as_deref() == Some("test_level.aeworld") {
            has_test_level_title = true;
        } else if node.tag == ModalDialogAction::TAG_CLOSE {
            close_rect = node.computed_rect;
        } else if node.tag == crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_REVEAL {
            reveal_rect = node.computed_rect;
        } else if node.tag == ModalDialogAction::TAG_CONFIRM {
            has_action_btn = true;
        }
    });

    assert!(has_test_level_title);
    assert!(has_action_btn);
    assert!(close_rect.width > 0.0 && close_rect.height > 0.0);
    assert!(reveal_rect.width > 0.0 && reveal_rect.height > 0.0);

    let mut tree_hover = UiTree::new();
    let root_hover = tree_hover.create_root().expect("Root node creation failed");

    let hover_pos = Point::new(close_rect.x + 2.0, close_rect.y + 2.0);
    tree_hover.set_cursor_position(hover_pos);

    let mut actions_hover = Vec::new();
    build_in_host(
        &mut tree_hover,
        root_hover,
        Rect::new(0.0, 0.0, 1280.0, 720.0),
        |s| {
            build_assets_panel(
                s,
                &asset_browser,
                false,
                &mut panel_state,
                &mut actions_hover,
            );
        },
    );

    let mut close_node_info = None;
    tree_hover.traverse_depth_first(root_hover, &mut |id, node| {
        if node.computed_rect == close_rect {
            close_node_info = Some((id, node.style.hover_background));
        }
    });
    let (close_id, hover_bg) = close_node_info.expect("Hovered close node found");
    assert_eq!(tree_hover.event_state.hovered, Some(close_id));
    let expected_danger = tree_hover.theme().danger;
    assert_eq!(
        hover_bg,
        Some(Color::rgba(
            expected_danger.r,
            expected_danger.g,
            expected_danger.b,
            0.28,
        ))
    );
}

#[test]
fn test_unified_workbench_dialog_host_delete_and_rename_modals_render() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let mut host = DialogHost::<EngineUiAction>::new();
    let mut bus = ActionBus::<EngineUiAction>::new();
    let target = PathBuf::from("assets/models/character.gltf");
    let screen_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    // 1. Open delete confirmation danger dialog on the unified DialogHost<EngineUiAction>
    host.open_confirm_danger(
        "Delete Asset",
        "Are you sure you want to permanently delete 'character.gltf'?",
        EngineUiAction::DeleteAssetOrFolder(target.clone()),
    );
    assert!(host.is_active());

    {
        let mut scope = UiScope::new(&mut tree, root_id);
        scope.dialog_host(&mut host, screen_rect, false, &mut bus);
    }
    assert!(tree.len() > 1);
    assert!(
        tree.iter()
            .any(|(_, node)| node.layer == irisui::core::UiLayer::Modal)
    );

    // 2. Escape dismissal
    assert!(host.cancel_top(&mut bus));
    assert!(!host.is_active());

    // 3. Open rename dialog on the unified DialogHost<EngineUiAction>
    let target_rename = PathBuf::from("assets/textures/albedo.png");
    host.open_input("Rename Asset", "New Name:", "albedo.png", move |new_name| {
        EngineUiAction::RenameAssetOrFolder {
            target: target_rename.clone(),
            new_name,
        }
    });
    assert!(host.is_active());

    {
        let mut scope = UiScope::new(&mut tree, root_id);
        scope.dialog_host(&mut host, screen_rect, false, &mut bus);
    }
    assert!(tree.len() > 1);
}

#[test]
fn test_assets_preview_modal_emits_close_inspect_modal_on_close_or_scrim() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let item = AssetItem {
        name: "preview.png".to_string(),
        path: PathBuf::from("assets/textures/preview.png"),
        relative_path: "textures/preview.png".to_string(),
        category: AssetCategory::Textures2D,
        source: AssetSource::Project,
        file_size_bytes: 2048,
        metadata_badge: "2 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };

    let preview_state = AssetPreviewModalState {
        item,
        orbit_yaw: 0.0,
        orbit_pitch: 0.0,
        zoom_distance: 1.0,
        show_wireframe: false,
    };

    let asset_browser = AssetBrowserState::default();
    let mut panel_state = AssetsPanelState {
        preview_modal: Some(preview_state),
        ..Default::default()
    };

    for tag in [
        irisui::widgets::modal::MODAL_TAG_CLOSE,
        irisui::widgets::modal::MODAL_TAG_SCRIM,
    ] {
        let mut actions = Vec::new();
        tree.event_state.frame_clicked_tag = Some(tag);
        build_in_host(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 1280.0, 720.0),
            |s| {
                build_assets_panel(s, &asset_browser, false, &mut panel_state, &mut actions);
            },
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, AssetsPanelAction::CloseInspectModal)),
            "Expected CloseInspectModal for tag {tag}"
        );
        assert!(!tree.is_any_modal_open());
    }
}