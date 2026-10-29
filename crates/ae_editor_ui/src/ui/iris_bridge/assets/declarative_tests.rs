// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous Declarative Assets Panel Interactions
//!
//! Verifies that [`build_assets_panel`] correctly reads widget responses
//! from autonomous tree interactions and emits canonical [`AssetsPanelAction`]
//! variants without routing through legacy event interceptors.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use irisui::prelude::*;
use irisui::widgets::controls::FocusState;

use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
use crate::ui::iris_bridge::assets::panel::build_assets_panel;
use crate::ui::iris_bridge::assets::types::{
    ASSETS_TAG_IMPORT, AssetsPanelAction, AssetsPanelParams, encode_item_tag,
};

/// Verifies that clicking declarative toolbar buttons and category filter chips
/// synchronously generates the corresponding [`AssetsPanelAction`] actions.
#[test]
fn test_assets_panel_declarative_toolbar_and_chip_actions() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &[],
        filtered_items: &[],
        subfolders: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        cursor_pos: Point::new(10.0, 10.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    // 1. Initial build: constructs the tree and registers tags
    let mut actions = Vec::new();
    let _metrics = build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );
    assert!(
        actions.is_empty(),
        "Initial idle build must emit zero actions"
    );

    // 2. Find Import button node
    let mut import_node_info = None;
    tree.traverse_depth_first(root_id, &mut |id, node| {
        if node.tag == ASSETS_TAG_IMPORT {
            import_node_info = Some((id, node.computed_rect));
        }
    });

    let (import_id, import_rect) = import_node_info.expect("Import button must exist in UI tree");
    assert!(import_rect.width > 0.0 && import_rect.height > 0.0);

    let click_pos = Point::new(import_rect.x + 2.0, import_rect.y + 2.0);

    // 3. Dispatch MouseDown and MouseUp to simulate click on the Import button
    let down_event = UiEvent::MouseDown {
        point: click_pos,
        button: MouseButton::Left,
    };
    tree.dispatch_event(&down_event);

    let up_event = UiEvent::MouseUp {
        point: click_pos,
        button: MouseButton::Left,
    };
    let dispatch_res = tree.dispatch_event(&up_event);
    assert!(
        dispatch_res.consumed,
        "Click event must be consumed by tree"
    );
    assert_eq!(tree.event_state.frame_clicked_id, Some(import_id));

    // 4. Rebuild assets panel with updated event state: must emit AssetsPanelAction::Import
    let mut click_actions = Vec::new();
    let mut params_clicked = params;
    params_clicked.cursor_pos = click_pos;

    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params_clicked,
        &mut TreeState::default(),
        &mut click_actions,
    );
    assert!(
        click_actions.contains(&AssetsPanelAction::OpenImportDialog),
        "Import button click must emit AssetsPanelAction::OpenImportDialog, got: {:?}",
        click_actions
    );
}

/// Verifies that clicking an asset card emits [`AssetsPanelAction::SelectAsset`].
#[test]
fn test_assets_panel_declarative_card_selection_action() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let test_item = AssetItem {
        name: "hero.png".to_string(),
        path: PathBuf::from("assets/textures/hero.png"),
        relative_path: "textures/hero.png".to_string(),
        category: AssetCategory::Textures2D,
        source: AssetSource::Project,
        file_size_bytes: 2048,
        metadata_badge: "2.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };

    let items = [test_item.clone()];

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &items,
        subfolders: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        cursor_pos: Point::new(10.0, 10.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    // 1. Initial build: constructs the card
    let mut actions = Vec::new();
    let _metrics = build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );

    // 2. Find asset card tag
    let card_tag = encode_item_tag(0);
    let mut card_rect = None;
    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.tag == card_tag {
            card_rect = Some(node.computed_rect);
        }
    });

    let rect = card_rect.expect("Asset card 0 must exist in UI tree");
    let click_pos = Point::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);

    // 3. Dispatch click
    tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pos,
        button: MouseButton::Left,
    });
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pos,
        button: MouseButton::Left,
    });
    assert!(up_res.consumed, "Card click event must be consumed by tree");

    // 4. Rebuild: must emit SelectAsset(Some(path))
    let mut select_actions = Vec::new();
    let mut params_clicked = params;
    params_clicked.cursor_pos = click_pos;

    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params_clicked,
        &mut TreeState::default(),
        &mut select_actions,
    );
    assert_eq!(
        select_actions,
        vec![AssetsPanelAction::SelectAsset(Some(test_item.path.clone()))],
        "Asset card single-click must emit SelectAsset(Some(path))"
    );
}

/// Verifies that right-clicking an asset card declaratively opens the context menu,
/// and clicking a menu item dispatches the corresponding [`AssetsPanelAction`]
/// while autonomously closing the popup.
#[test]
fn test_assets_card_declarative_context_menu_interaction() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let test_item = AssetItem {
        name: "character.glb".to_string(),
        path: PathBuf::from("assets/models/character.glb"),
        relative_path: "models/character.glb".to_string(),
        category: AssetCategory::Models3D,
        source: AssetSource::Project,
        file_size_bytes: 4096,
        metadata_badge: "4.2 MB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };
    let items = [test_item.clone()];

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &items,
        subfolders: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    // 1. Initial build: render card
    let mut actions = Vec::new();
    let mut tree_state = TreeState::default();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );

    let card_tag = encode_item_tag(0);
    let mut card_rect = None;
    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.tag == card_tag {
            card_rect = Some(node.computed_rect);
        }
    });
    let rect = card_rect.expect("Card 0 must exist");

    let click_pos = Point::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);

    // 2. Secondary (Right) click on card
    tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pos,
        button: MouseButton::Right,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pos,
        button: MouseButton::Right,
    });

    // 3. Rebuild: context menu should be open and rendered in UiLayer::Popup
    let mut right_click_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut right_click_actions,
    );
    assert_eq!(
        right_click_actions,
        vec![AssetsPanelAction::SelectAsset(Some(test_item.path.clone()))],
        "Right clicking a card selects it"
    );

    // Popup card must exist in the UI tree
    let popup_node = tree
        .iter()
        .find(|(_, n)| n.layer == UiLayer::Popup && n.role == WidgetRole::DropdownPopup);
    assert!(
        popup_node.is_some(),
        "Context menu popup card must be emitted in UiLayer::Popup"
    );

    // Find "Quick Inspect" item (first ContextMenuItem in popup)
    let inspect_node = tree
        .iter()
        .find(|(_, n)| n.layer == UiLayer::Popup && n.name.as_deref() == Some("ContextMenuItem"));
    assert!(
        inspect_node.is_some(),
        "Quick Inspect context menu item must be present"
    );

    let inspect_rect = inspect_node.unwrap().1.computed_rect;
    let inspect_click = Point::new(
        inspect_rect.x + inspect_rect.width * 0.5,
        inspect_rect.y + inspect_rect.height * 0.5,
    );

    // 4. Click Quick Inspect item
    tree.dispatch_event(&UiEvent::MouseDown {
        point: inspect_click,
        button: MouseButton::Left,
    });
    let _up_res = tree.dispatch_event(&UiEvent::MouseUp {
        point: inspect_click,
        button: MouseButton::Left,
    });

    // 5. Rebuild: must emit OpenInspectModal action
    let mut inspect_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut inspect_actions,
    );
    assert!(
        inspect_actions.contains(&AssetsPanelAction::OpenInspectModal(test_item)),
        "Clicking inspect context menu item must emit OpenInspectModal action: {:?}",
        inspect_actions
    );
}

/// Verifies that dragging an asset card emits [`AssetsPanelAction::StartAssetDrag`]
/// and releasing the drag emits [`AssetsPanelAction::EndAssetDrag`].
#[test]
fn test_assets_panel_declarative_drag_and_drop_actions() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let test_item = AssetItem {
        name: "character.glb".to_string(),
        path: PathBuf::from("assets/models/character.glb"),
        relative_path: "models/character.glb".to_string(),
        category: AssetCategory::Models3D,
        file_size_bytes: 4096,
        source: AssetSource::Project,
        is_loaded_in_memory: false,
        metadata_badge: "4.0 KB".to_string(),
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };

    let items = [test_item.clone()];
    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &items,
        subfolders: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        cursor_pos: Point::new(10.0, 10.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    // 1. Initial build: constructs the asset card
    let mut tree_state = TreeState::new();
    let mut actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut actions,
    );

    let card_node = tree
        .iter()
        .find(|(_, n)| n.tag == encode_item_tag(0))
        .expect("Asset card node with tag encode_item_tag(0) not found");
    let card_rect = card_node.1.computed_rect;
    let card_center = Point::new(
        card_rect.x + card_rect.width * 0.5,
        card_rect.y + card_rect.height * 0.5,
    );

    // 2. MouseDown on card
    tree.dispatch_event(&UiEvent::MouseDown {
        point: card_center,
        button: MouseButton::Left,
    });

    // 3. Move mouse past threshold (> 3.0 px) -> Triggers drag
    let move_point = Point::new(card_center.x + 20.0, card_center.y + 20.0);
    tree.dispatch_event(&UiEvent::MouseMove { point: move_point });

    // 4. Rebuild: must emit StartAssetDrag
    let mut drag_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut drag_actions,
    );
    let expected_payload =
        TypedDragDropPayload::new(test_item.clone(), test_item.name.clone(), "Assets")
            .with_badge(test_item.category.badge(), test_item.category.badge_color())
            .into_erased();
    assert!(
        drag_actions.contains(&AssetsPanelAction::StartAssetDrag(expected_payload)),
        "Dragging card must emit AssetsPanelAction::StartAssetDrag, got: {:?}",
        drag_actions
    );

    // 5. MouseUp over viewport area -> Release drag
    let drop_point = Point::new(card_center.x + 200.0, card_center.y + 100.0);
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        point: drop_point,
        button: MouseButton::Left,
    });
    eprintln!("UP RES: {:?}", up_res);
    eprintln!("UP EVENT STATE: {:?}", tree.event_state);

    // 6. Rebuild: must emit EndAssetDrag
    let mut release_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut release_actions,
    );
    eprintln!("RELEASE ACTIONS: {:?}", release_actions);
    assert!(
        release_actions.contains(&AssetsPanelAction::EndAssetDrag),
        "Releasing drag must emit AssetsPanelAction::EndAssetDrag, got: {:?}",
        release_actions
    );
}

/// Verifies that dragging an asset card succeeds even when [`UiTree`] is cleared and rebuilt
/// between MouseDown and MouseMove (mimicking runtime immediate-mode frame rebuilds).
#[test]
fn test_assets_panel_drag_and_drop_across_tree_rebuild() {
    let mut tree = UiTree::new();
    let root_id = tree
        .create_root()
        .expect("Failed to create root node in UiTree");

    let test_item = AssetItem {
        path: PathBuf::from("assets/models/character.glb"),
        relative_path: "models/character.glb".to_string(),
        name: "character.glb".to_string(),
        category: AssetCategory::Models3D,
        file_size_bytes: 4096,
        source: AssetSource::Project,
        is_loaded_in_memory: false,
        metadata_badge: "4.0 KB".to_string(),
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };
    let items = [test_item.clone()];

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &items,
        subfolders: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        cursor_pos: Point::new(10.0, 10.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut tree_state = TreeState::new();
    let mut init_actions = Vec::new();

    // 1. Initial build
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut init_actions,
    );
    let card_node = tree
        .iter()
        .find(|(_, n)| n.tag == encode_item_tag(0))
        .expect("Asset card node not found");
    let card_rect = card_node.1.computed_rect;
    let card_center = Point::new(
        card_rect.x + card_rect.width * 0.5,
        card_rect.y + card_rect.height * 0.5,
    );

    // 2. MouseDown on card
    tree.dispatch_event(&UiEvent::MouseDown {
        point: card_center,
        button: MouseButton::Left,
    });

    // 3. TREE CLEAR & REBUILD (simulating next runtime frame before mouse motion)
    tree.clear();
    let root_id = tree.create_root().expect("Failed to recreate root node");
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut init_actions,
    );

    // 4. MouseMove past drag threshold (> 3.0 px) -> Must establish pointer capture via pressed_tag
    let move_point = Point::new(card_center.x + 25.0, card_center.y + 25.0);
    tree.dispatch_event(&UiEvent::MouseMove { point: move_point });

    // 5. Rebuild: must emit StartAssetDrag
    let mut drag_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut drag_actions,
    );
    let expected_payload =
        TypedDragDropPayload::new(test_item.clone(), test_item.name.clone(), "Assets")
            .with_badge(test_item.category.badge(), test_item.category.badge_color())
            .into_erased();
    assert!(
        drag_actions.contains(&AssetsPanelAction::StartAssetDrag(expected_payload)),
        "Dragging card across tree rebuild must emit StartAssetDrag, got: {:?}",
        drag_actions
    );

    // 6. TREE CLEAR & REBUILD again (simulating mid-drag frame before mouse release)
    tree.clear();
    let root_id = tree.create_root().expect("Failed to recreate root node");
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut init_actions,
    );

    // 7. MouseUp over viewport area -> Release drag
    let drop_point = Point::new(card_center.x + 200.0, card_center.y + 100.0);
    tree.dispatch_event(&UiEvent::MouseUp {
        point: drop_point,
        button: MouseButton::Left,
    });

    // 8. Rebuild: must emit EndAssetDrag
    let mut release_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut release_actions,
    );
    assert!(
        release_actions.contains(&AssetsPanelAction::EndAssetDrag),
        "Releasing drag across tree rebuild must emit EndAssetDrag, got: {:?}",
        release_actions
    );
}

/// Verifies that dropping OS files onto the declarative Asset Browser content area
/// captures the drop target and emits [`AssetsPanelAction::ImportDroppedFiles`].
#[test]
fn test_assets_content_drop_target_emits_import_action() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &[],
        filtered_items: &[],
        subfolders: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &std::collections::HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };
    let mut tree_state = TreeState::new();
    let mut init_actions = Vec::new();

    // 1. Initial build & layout
    let metrics = build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut init_actions,
    );
    let mut layout_engine = irisui::layout::LayoutEngine::new();
    layout_engine
        .compute_layout(&mut tree, Size::new(1000.0, 600.0))
        .expect("Layout calculation failed");

    // 2. Dispatch FileHovered over the content viewport
    let content_pt = Point::new(
        metrics.content_viewport_rect.x + 50.0,
        metrics.content_viewport_rect.y + 50.0,
    );
    let test_file = PathBuf::from("/home/user/downloads/tree.gltf");
    let hover_res = tree.dispatch_event(&UiEvent::FileHovered {
        point: content_pt,
        path: test_file.clone(),
    });
    assert!(
        hover_res.target.is_some(),
        "FileHovered over AssetsContentArea must hit a target"
    );
    assert_eq!(
        tree.event_state.hovered_file,
        Some((test_file.clone(), content_pt)),
        "Hovered file must be recorded in event state"
    );

    // 3. Dispatch FileDropped over the content viewport
    let drop_res = tree.dispatch_event(&UiEvent::FileDropped {
        point: content_pt,
        paths: vec![test_file.clone()],
    });
    assert!(
        drop_res.is_consumed(),
        "FileDropped over AssetsContentArea must be consumed"
    );

    // 4. Rebuild scope: must emit AssetsPanelAction::ImportDroppedFiles
    let mut drop_actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut tree_state,
        &mut drop_actions,
    );
    assert_eq!(
        drop_actions,
        vec![AssetsPanelAction::ImportDroppedFiles(vec![test_file])],
        "Drop target must emit AssetsPanelAction::ImportDroppedFiles"
    );
}