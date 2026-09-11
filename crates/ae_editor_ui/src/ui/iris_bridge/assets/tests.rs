// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for the Native Iris UI Asset / Content Browser Subsystem.
//!

use super::events::{self, AssetClickTracker, AssetsEventContext};
use super::panel::build_assets_panel;
use super::sync::sync_assets_panel;
use super::types::{
    AssetCardTarget, AssetPreviewModalState, AssetsContextMenuTarget, AssetsPanelAction,
    AssetsPanelParams, AssetsPanelTargets, truncate_display_name,
};
use crate::ui::panels::assets::types::{AssetCategory, AssetItem, AssetViewMode};
use irisui::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn test_assets_panel_structure_and_no_emojis() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");
    let mut targets = AssetsPanelTargets::default();

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
    let current_folder = PathBuf::from("assets");
    let item = AssetItem {
        name: "test_mesh.gltf".to_string(),
        path: PathBuf::from("assets/test_mesh.gltf"),
        relative_path: "test_mesh.gltf".to_string(),
        category: AssetCategory::Models3D,
        file_size_bytes: 2048,
        metadata_badge: "2.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };
    let items = vec![item];
    let item_refs: Vec<&AssetItem> = items.iter().collect();

    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &item_refs,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        revision: 0,
    };

    build_assets_panel(&mut tree, root_id, &params, &mut targets);

    assert_eq!(targets.panel_rect, panel_rect);
    assert!(!targets.breadcrumbs.is_empty());
    assert_eq!(targets.grid_cards.len(), 1);

    // Verify that NO emojis exist in node text strings across the entire panel
    assert_no_emojis_recursive(&tree, root_id);
}

fn assert_no_emojis_recursive(tree: &UiTree, current: WidgetId) {
    if let Some(node) = tree.get(current) {
        if let Some(text) = &node.text {
            assert!(
                !text.contains('📁'),
                "Node text contains forbidden emoji '📁': {}",
                text
            );
            assert!(
                !text.contains('📂'),
                "Node text contains forbidden emoji '📂': {}",
                text
            );
            assert!(
                !text.contains('📦'),
                "Node text contains forbidden emoji '📦': {}",
                text
            );
            assert!(
                !text.contains('🖼'),
                "Node text contains forbidden emoji '🖼': {}",
                text
            );
            assert!(
                !text.contains('⚡'),
                "Node text contains forbidden emoji '⚡': {}",
                text
            );
            assert!(
                !text.contains('🎬'),
                "Node text contains forbidden emoji '🎬': {}",
                text
            );
            assert!(
                !text.contains('🎨'),
                "Node text contains forbidden emoji '🎨': {}",
                text
            );
            assert!(
                !text.contains('🔊'),
                "Node text contains forbidden emoji '🔊': {}",
                text
            );
        }
        for &child in &node.children {
            assert_no_emojis_recursive(tree, child);
        }
    }
}

#[test]
fn test_assets_actions_dispatch() {
    let mut tracker = AssetClickTracker::default();
    let mut actions = Vec::new();
    let targets = AssetsPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 800.0, 400.0),
        grid_toggle_rect: Rect::new(500.0, 5.0, 46.0, 24.0),
        ..Default::default()
    };

    let ctx = AssetsEventContext {
        cursor_pos: Point::new(510.0, 10.0),
        targets: &targets,
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        selected_asset: None,
    };

    let consumed = events::handle_assets_click(&ctx, &mut tracker, &mut actions);

    assert!(consumed);
    assert_eq!(
        actions,
        vec![AssetsPanelAction::SetViewMode(AssetViewMode::Grid)]
    );
}

#[test]
fn test_assets_right_click_context_menu_dispatch() {
    let mut actions = Vec::new();
    let item = AssetItem {
        name: "test.png".to_string(),
        path: PathBuf::from("assets/test.png"),
        relative_path: "test.png".to_string(),
        category: AssetCategory::Textures2D,
        file_size_bytes: 1024,
        metadata_badge: "1.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };

    let card_target = AssetCardTarget {
        rect: Rect::new(200.0, 50.0, 116.0, 134.0),
        path: PathBuf::from("assets/test.png"),
        category: AssetCategory::Textures2D,
        item: item.clone(),
    };

    let targets = AssetsPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 800.0, 400.0),
        grid_cards: vec![card_target],
        ..Default::default()
    };

    let ctx = AssetsEventContext {
        cursor_pos: Point::new(220.0, 70.0),
        targets: &targets,
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        selected_asset: None,
    };

    let consumed = events::handle_assets_right_click(&ctx, &mut actions);

    assert!(consumed);
    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions[0],
        AssetsPanelAction::SelectAsset(Some(PathBuf::from("assets/test.png")))
    );
    assert_eq!(
        actions[1],
        AssetsPanelAction::OpenContextMenu(
            AssetsContextMenuTarget::Asset(item),
            Point::new(220.0, 70.0)
        )
    );
}

#[test]
fn test_preview_modal_build_and_actions() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");
    let mut targets = AssetsPanelTargets::default();

    let item = AssetItem {
        name: "dragon.gltf".to_string(),
        path: PathBuf::from("assets/models/dragon.gltf"),
        relative_path: "models/dragon.gltf".to_string(),
        category: AssetCategory::Models3D,
        file_size_bytes: 1048576,
        metadata_badge: "1.0 MB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };

    let preview_state = AssetPreviewModalState {
        item: item.clone(),
        orbit_yaw: 0.5,
        orbit_pitch: 0.2,
        zoom_distance: 1.2,
        show_wireframe: true,
    };

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1280.0, 720.0),
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &[],
        filtered_items: &[],
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(500.0, 300.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: Some(&preview_state),
        thumbnail_layers: &HashMap::new(),
        revision: 0,
    };

    build_assets_panel(&mut tree, root_id, &params, &mut targets);

    assert!(targets.preview_modal.is_some());
    let pm = targets.preview_modal.as_ref().unwrap();
    assert_eq!(pm.item.name, "dragon.gltf");
    assert!(pm.orbit_canvas_rect.is_some());
    assert!(pm.action_btn_rect.is_some());

    // Test clicking the close button on the preview modal
    let mut tracker = AssetClickTracker::default();
    let mut actions = Vec::new();
    let ctx = AssetsEventContext {
        cursor_pos: Point::new(pm.close_btn_rect.x + 5.0, pm.close_btn_rect.y + 5.0),
        targets: &targets,
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        selected_asset: None,
    };

    let consumed = events::handle_assets_click(&ctx, &mut tracker, &mut actions);
    assert!(consumed);
    assert_eq!(actions, vec![AssetsPanelAction::CloseInspectModal]);
}

#[test]
fn test_truncate_display_name_utf8_boundary_safety() {
    // 1. Short string within limits
    let short = "cube.png";
    assert_eq!(truncate_display_name(short, 14, 11), "cube.png");

    // 2. ASCII string longer than limits
    let long_ascii = "very_long_texture_filename.png";
    assert_eq!(truncate_display_name(long_ascii, 14, 11), "very_long_t...");

    // 3. Multi-byte UTF-8 test: Turkish characters ('ö', 'ğ', 'ü') crossing byte boundary 11
    // 'd'(0), 'o'(1), 'k'(2), 'u'(3), 's'(4), 'u'(5), '_'(6), 'ö'(7..9), 'ğ'(9..11), 'ü'(11..13)
    // Byte 11 is right inside 'ü' (bytes 11..13). Naive byte slicing [..11] panics.
    let turkish = "dokusu_öğütülmüş_yüzey.png";
    let truncated_turkish = truncate_display_name(turkish, 14, 11);
    assert_eq!(truncated_turkish, "dokusu_öğüt...");
    assert!(truncated_turkish.ends_with("..."));

    // 4. Multi-byte CJK and emojis
    let cjk = "こんにちは世界_テクスチャ_2026.png";
    let truncated_cjk = truncate_display_name(cjk, 14, 11);
    assert_eq!(truncated_cjk, "こんにちは世界_テクス...");
}

#[test]
fn test_assets_card_rendering_with_unicode_filenames() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");
    let mut targets = AssetsPanelTargets::default();

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
    let current_folder = PathBuf::from("assets");
    let item = AssetItem {
        name: "dokusu_öğütülmüş_yüzey.png".to_string(),
        path: PathBuf::from("assets/textures/dokusu_öğütülmüş_yüzey.png"),
        relative_path: "dokusu_öğütülmüş_yüzey.png".to_string(),
        category: AssetCategory::Textures2D,
        file_size_bytes: 408500,
        metadata_badge: "408.5 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };
    let items = vec![item];
    let item_refs: Vec<&AssetItem> = items.iter().collect();

    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &item_refs,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        revision: 0,
    };

    // Should build cards with Turkish/Unicode filenames without any panic
    build_assets_panel(&mut tree, root_id, &params, &mut targets);
    assert_eq!(targets.grid_cards.len(), 1);
}

#[test]
fn test_retained_assets_panel_zero_allocation_when_idle() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");
    let mut retained = None;

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
    let current_folder = PathBuf::from("assets");
    let item = AssetItem {
        name: "cube.gltf".to_string(),
        path: PathBuf::from("assets/cube.gltf"),
        relative_path: "cube.gltf".to_string(),
        category: AssetCategory::Models3D,
        file_size_bytes: 1024,
        metadata_badge: "1.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };
    let items = vec![item];
    let item_refs: Vec<&AssetItem> = items.iter().collect();

    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &item_refs,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        revision: 0,
    };

    // Frame 1: Initial Mount
    sync_assets_panel(&mut tree, root_id, &mut retained, &params);
    assert!(retained.is_some());
    let initial_node_count = tree.len();
    assert!(initial_node_count > 0);

    // Clear all dirty flags as if a layout and render pass completed
    tree.clear_all_dirty(DirtyFlags::ALL);
    assert!(!tree.has_dirty_nodes(DirtyFlags::ALL));

    // Frame 2: Identical parameters (Panel completely idle and motionless)
    sync_assets_panel(&mut tree, root_id, &mut retained, &params);

    // Assert zero allocations, zero node count changes, and zero dirty flags marked
    assert_eq!(tree.len(), initial_node_count);
    assert!(
        !tree.has_dirty_nodes(DirtyFlags::ALL),
        "Idle frame must not mark any dirty flags"
    );
}

#[test]
fn test_retained_assets_panel_hover_paint_dirty_only() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");
    let mut retained = None;

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
    let current_folder = PathBuf::from("assets");
    let item = AssetItem {
        name: "texture.png".to_string(),
        path: PathBuf::from("assets/texture.png"),
        relative_path: "texture.png".to_string(),
        category: AssetCategory::Textures2D,
        file_size_bytes: 4096,
        metadata_badge: "4.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };
    let items = vec![item];
    let item_refs: Vec<&AssetItem> = items.iter().collect();

    let mut params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &item_refs,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        revision: 0,
    };

    // Frame 1: Initial Mount
    sync_assets_panel(&mut tree, root_id, &mut retained, &params);
    let initial_node_count = tree.len();
    let card_rect = retained.as_ref().unwrap().cached_targets.grid_cards[0].rect;

    // Clear all dirty flags
    tree.traverse_depth_first_mut(root_id, &mut |_, node| {
        node.clear_dirty(DirtyFlags::ALL);
    });

    // Frame 2: Hover cursor over card center
    params.cursor_pos = Point::new(
        card_rect.x + card_rect.width * 0.5,
        card_rect.y + card_rect.height * 0.5,
    );
    sync_assets_panel(&mut tree, root_id, &mut retained, &params);

    // Node count remains strictly identical (zero new allocations)
    assert_eq!(tree.len(), initial_node_count);

    // PAINT must be marked on the hovered card, but LAYOUT must NOT be dirty
    assert!(
        tree.has_dirty_nodes(DirtyFlags::PAINT),
        "Hovered card must mark PAINT dirty flag"
    );
}

#[test]
fn test_retained_assets_panel_clear_all_dirty_invariant() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");
    let mut retained = None;

    let panel_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let current_folder = PathBuf::from("assets");
    let item = AssetItem {
        name: "test.png".to_string(),
        path: PathBuf::from("assets/test.png"),
        relative_path: "test.png".to_string(),
        category: AssetCategory::Textures2D,
        file_size_bytes: 4096,
        metadata_badge: "4 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
    };
    let items = vec![item];
    let item_refs: Vec<&AssetItem> = items.iter().collect();

    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &items,
        filtered_items: &item_refs,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        revision: 0,
    };

    // Frame 1: Build
    sync_assets_panel(&mut tree, root_id, &mut retained, &params);
    assert!(tree.has_dirty_nodes(DirtyFlags::ALL));

    // Clear all dirty at end of frame
    tree.clear_all_dirty(DirtyFlags::ALL);
    assert!(!tree.has_dirty_nodes(DirtyFlags::ALL));

    // Frame 2: Completely idle frame
    sync_assets_panel(&mut tree, root_id, &mut retained, &params);
    assert!(
        !tree.has_dirty_nodes(DirtyFlags::ALL),
        "Idle frame must produce zero dirty nodes across the entire tree"
    );
}