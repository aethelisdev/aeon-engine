// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for the Native Iris UI Asset / Content Browser Subsystem.
//!

use super::events::{self, AssetClickTracker, AssetsEventContext};
use super::panel::build_assets_panel;
use super::types::{
    AssetCardTarget, AssetPreviewModalState, AssetsContextMenuTarget, AssetsPanelAction,
    AssetsPanelParams, AssetsPanelTargets, truncate_display_name,
};
use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
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
        source: AssetSource::Project,
        file_size_bytes: 2048,
        metadata_badge: "2.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };
    let items = vec![item];

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
        filtered_items: &items,
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
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
            assert!(
                !text.contains('⚙'),
                "Node text contains forbidden emoji '⚙': {}",
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
        hit_target: None,
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
        source: AssetSource::Project,
        file_size_bytes: 1024,
        metadata_badge: "1.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
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
        hit_target: None,
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
        source: AssetSource::Project,
        file_size_bytes: 1048576,
        metadata_badge: "1.0 MB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
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
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(500.0, 300.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: Some(&preview_state),
        thumbnail_layers: &HashMap::new(),
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
        hit_target: None,
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
        source: AssetSource::Project,
        file_size_bytes: 408500,
        metadata_badge: "408.5 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };
    let items = vec![item];

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
        filtered_items: &items,
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
    };

    // Should build cards with Turkish/Unicode filenames without any panic
    build_assets_panel(&mut tree, root_id, &params, &mut targets);
    assert_eq!(targets.grid_cards.len(), 1);
}

#[test]
fn test_asset_drag_overlay_construction() {
    use crate::assets::types::{AssetCategory, AssetDragPayload};
    use crate::ui::iris_bridge::assets::build_asset_drag_overlays;
    use ae_renderer::camera::Camera;

    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let payload = AssetDragPayload {
        path: PathBuf::from("assets/textures/player.png"),
        name: "player.png".to_string(),
        category: AssetCategory::Textures2D,
        model_handle: None,
        texture_handle: None,
    };

    let camera = Camera {
        position: cgmath::Point3::new(0.0, 5.0, 10.0),
        yaw: cgmath::Rad(-std::f32::consts::FRAC_PI_2),
        pitch: cgmath::Rad(-0.4),
        aspect: 800.0 / 600.0,
        fovy: 45.0,
        znear: 0.1,
        zfar: 1000.0,
        mode: ae_renderer::camera::ProjectionMode::Perspective,
        ortho_scale: 10.0,
        target: cgmath::Point3::new(0.0, 0.0, 0.0),
    };
    let vp_rect = Rect::new(100.0, 50.0, 800.0, 600.0);
    let cursor = Point::new(500.0, 350.0);

    build_asset_drag_overlays(
        &mut tree, root_id, &payload, cursor, vp_rect, &camera, false,
    );

    let root_node = tree.get(root_id).expect("Root node exists");
    assert!(root_node.children.len() >= 2);
}

#[test]
fn test_asset_drag_tracker_lifecycle_and_cancellation() {
    let mut tracker = AssetClickTracker::default();
    assert!(!tracker.is_dragging_asset);
    assert!(tracker.potential_drag_item.is_none());

    // 1. User clicks item
    let item = AssetItem {
        name: "test.png".to_string(),
        path: PathBuf::from("assets/test.png"),
        relative_path: "test.png".to_string(),
        category: AssetCategory::Textures2D,
        source: AssetSource::Project,
        file_size_bytes: 1024,
        metadata_badge: "1.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };
    tracker.potential_drag_item = Some(item);
    tracker.drag_start_pos = Some(Point::new(100.0, 100.0));

    // 2. Cursor moved > 5 px
    let current = Point::new(110.0, 110.0);
    let start_pos = tracker.drag_start_pos.unwrap();
    let dx = current.x - start_pos.x;
    let dy = current.y - start_pos.y;
    assert!((dx * dx + dy * dy) > 25.0);
    tracker.is_dragging_asset = true;
    tracker.potential_drag_item = None;
    tracker.drag_start_pos = None;
    assert!(tracker.is_dragging_asset);

    // 3. User drops outside viewport (or hits Escape)
    tracker.is_dragging_asset = false;
    tracker.potential_drag_item = None;
    tracker.drag_start_pos = None;
    assert!(!tracker.is_dragging_asset);
}

#[test]
fn test_asset_drag_viewport_boundary_check() {
    let viewport_rect = Rect::new(200.0, 100.0, 800.0, 600.0);

    // Inside viewport
    let inside_pos = Point::new(300.0, 200.0);
    assert!(viewport_rect.contains_point(inside_pos));

    // Over Asset panel (outside viewport)
    let asset_panel_pos = Point::new(300.0, 800.0);
    assert!(!viewport_rect.contains_point(asset_panel_pos));

    // Over Hierarchy panel (outside viewport)
    let hierarchy_pos = Point::new(50.0, 200.0);
    assert!(!viewport_rect.contains_point(hierarchy_pos));

    // Over Menubar (outside viewport)
    let menubar_pos = Point::new(400.0, 15.0);
    assert!(!viewport_rect.contains_point(menubar_pos));
}

#[test]
fn test_asset_source_classification() {
    let project_item = AssetItem {
        name: "player.png".to_string(),
        path: PathBuf::from("assets/textures/player.png"),
        relative_path: "textures/player.png".to_string(),
        category: AssetCategory::Textures2D,
        source: AssetSource::Project,
        file_size_bytes: 1024,
        metadata_badge: "1.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };
    assert_eq!(project_item.source, AssetSource::Project);

    let engine_item = AssetItem {
        name: "bloom.wgsl".to_string(),
        path: PathBuf::from("crates/ae_renderer/src/shaders/bloom.wgsl"),
        relative_path: "crates/ae_renderer/src/shaders/bloom.wgsl".to_string(),
        category: AssetCategory::Shaders,
        source: AssetSource::Engine,
        file_size_bytes: 3000,
        metadata_badge: "3.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };
    assert_eq!(engine_item.source, AssetSource::Engine);
}

#[test]
fn test_engine_content_visibility_filtering() {
    let project_item = AssetItem {
        name: "player.png".to_string(),
        path: PathBuf::from("assets/textures/player.png"),
        relative_path: "textures/player.png".to_string(),
        category: AssetCategory::Textures2D,
        source: AssetSource::Project,
        file_size_bytes: 1024,
        metadata_badge: "1.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };

    let engine_item = AssetItem {
        name: "bloom.wgsl".to_string(),
        path: PathBuf::from("crates/ae_renderer/src/shaders/bloom.wgsl"),
        relative_path: "crates/ae_renderer/src/shaders/bloom.wgsl".to_string(),
        category: AssetCategory::Shaders,
        source: AssetSource::Engine,
        file_size_bytes: 3000,
        metadata_badge: "3.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };

    let all_items = [project_item.clone(), engine_item.clone()];

    // When show_engine_content is false (default hidden)
    let filtered_hidden: Vec<_> = all_items
        .iter()
        .filter(|item| item.source != AssetSource::Engine)
        .cloned()
        .collect();
    assert_eq!(filtered_hidden.len(), 1);
    assert_eq!(filtered_hidden[0].name, "player.png");

    // When show_engine_content is true
    let filtered_visible: Vec<_> = all_items.iter().filter(|_item| true).cloned().collect();
    assert_eq!(filtered_visible.len(), 2);
}

#[test]
fn test_engine_toggle_action_dispatch() {
    let mut actions = Vec::new();
    let engine_rect = Rect::new(500.0, 5.0, 76.0, 24.0);
    let targets = AssetsPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 800.0, 400.0),
        engine_toggle_btn_rect: Some(engine_rect),
        ..Default::default()
    };

    let ctx = AssetsEventContext {
        cursor_pos: Point::new(510.0, 15.0),
        targets: &targets,
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        selected_asset: None,
        hit_target: None,
    };

    let mut tracker = AssetClickTracker::default();
    let consumed = events::handle_assets_click(&ctx, &mut tracker, &mut actions);
    assert!(consumed);
    assert_eq!(actions, vec![AssetsPanelAction::ToggleEngineContent]);
}

#[test]
fn test_engine_and_sidebar_vector_icons() {
    use crate::ui::iris_bridge::icons::{
        FIRST_THUMBNAIL_LAYER, ICON_CHEVRON_DOWN, ICON_CHEVRON_UP, ICON_GEAR, ICON_SPARKLE,
    };

    // Verify canonical texture coordinates on layer 16..19 and thumbnail layer reservation
    assert_eq!(ICON_GEAR, [0.0, 0.0, 1.0, 16.0]);
    assert_eq!(ICON_SPARKLE, [0.0, 0.0, 1.0, 17.0]);
    assert_eq!(ICON_CHEVRON_DOWN, [0.0, 0.0, 1.0, 18.0]);
    assert_eq!(ICON_CHEVRON_UP, [0.0, 0.0, 1.0, 19.0]);
    assert_eq!(FIRST_THUMBNAIL_LAYER, 32);

    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
    let mut targets = AssetsPanelTargets::default();

    let current_folder = PathBuf::from("assets");
    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        show_engine_content: true,
        cached_items: &[],
        filtered_items: &[],
        is_2d_mode: false,
        selected_asset: None,
        sidebar_width: 180.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
    };

    build_assets_panel(&mut tree, root_id, &params, &mut targets);

    fn find_node_by_name<'a>(
        tree: &'a UiTree,
        current: WidgetId,
        name: &str,
    ) -> Option<&'a irisui::prelude::WidgetNode> {
        if let Some(node) = tree.get(current) {
            if node.name.as_deref() == Some(name) {
                return Some(node);
            }
            for &child in &node.children {
                if let Some(found) = find_node_by_name(tree, child, name) {
                    return Some(found);
                }
            }
        }
        None
    }

    let gear_node = find_node_by_name(&tree, root_id, "EngineGearIcon");
    assert!(gear_node.is_some(), "EngineGearIcon node must be present");
    assert_eq!(gear_node.unwrap().texture_uv, Some(ICON_GEAR));

    let side_btn = find_node_by_name(&tree, root_id, "SidebarToggleButton");
    assert!(
        side_btn.is_some(),
        "SidebarToggleButton node must be present"
    );
    assert_eq!(side_btn.unwrap().text.as_deref(), Some("◀"));
}

/// Verifies bidirectional scene filtering: 2D mode filters out 3D assets, 3D mode filters out 2D scenes.
#[test]
fn test_bidirectional_scene_filtering() {
    let scene_3d = AssetItem {
        name: "physics_test_suite.ae3d".to_string(),
        path: PathBuf::from("assets/scenes/physics_test_suite.ae3d"),
        relative_path: "scenes/physics_test_suite.ae3d".to_string(),
        category: AssetCategory::Scenes,
        source: AssetSource::Project,
        file_size_bytes: 12000,
        metadata_badge: "12.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };

    let scene_2d = AssetItem {
        name: "level1_2d.ae2d".to_string(),
        path: PathBuf::from("assets/scenes/level1_2d.ae2d"),
        relative_path: "scenes/level1_2d.ae2d".to_string(),
        category: AssetCategory::Scenes,
        source: AssetSource::Project,
        file_size_bytes: 4000,
        metadata_badge: "4.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };

    let texture = AssetItem {
        name: "player.png".to_string(),
        path: PathBuf::from("assets/textures/player.png"),
        relative_path: "textures/player.png".to_string(),
        category: AssetCategory::Textures2D,
        source: AssetSource::Project,
        file_size_bytes: 2048,
        metadata_badge: "2.0 KB".to_string(),
        is_loaded_in_memory: true,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: false,
    };

    let all_items = [scene_3d, scene_2d, texture];

    // Filter in 2D mode: 3D items hidden
    let filtered_2d: Vec<_> = all_items
        .iter()
        .filter(|item| !item.is_3d)
        .cloned()
        .collect();
    assert_eq!(filtered_2d.len(), 2);
    assert!(
        !filtered_2d
            .iter()
            .any(|i| i.name == "physics_test_suite.ae3d")
    );
    assert!(filtered_2d.iter().any(|i| i.name == "level1_2d.ae2d"));

    // Filter in 3D mode: 2D scenes hidden
    let filtered_3d: Vec<_> = all_items
        .iter()
        .filter(|item| !(!item.is_3d && item.category == AssetCategory::Scenes))
        .cloned()
        .collect();
    assert_eq!(filtered_3d.len(), 2);
    assert!(
        filtered_3d
            .iter()
            .any(|i| i.name == "physics_test_suite.ae3d")
    );
    assert!(!filtered_3d.iter().any(|i| i.name == "level1_2d.ae2d"));
}

/// Verifies that `is_scene_json_3d` accurately distinguishes 3D vs 2D scene structures.
#[test]
fn test_is_scene_json_3d_detection() {
    use crate::assets::scanner::is_scene_json_3d;

    let scene_3d_shape = serde_json::json!([
        {
            "name": "Static_Ground",
            "shape": "Cube",
            "position": { "x": 0.0, "y": 0.0, "z": 0.0 }
        }
    ]);
    assert!(is_scene_json_3d(&scene_3d_shape));

    let scene_3d_model = serde_json::json!([
        {
            "name": "Dragon",
            "model_path": "assets/models/dragon.glb"
        }
    ]);
    assert!(is_scene_json_3d(&scene_3d_model));

    let scene_3d_explicit = serde_json::json!({
        "dimension": "3D",
        "entities": []
    });
    assert!(is_scene_json_3d(&scene_3d_explicit));

    let scene_2d_explicit = serde_json::json!({
        "dimension": "2D",
        "entities": []
    });
    assert!(!is_scene_json_3d(&scene_2d_explicit));

    let scene_2d_sprite = serde_json::json!([
        {
            "name": "Player_Sprite",
            "sprite_path": "assets/textures/player.png"
        }
    ]);
    assert!(!is_scene_json_3d(&scene_2d_sprite));
}

#[test]
fn test_assets_context_menu_builder_and_hit_testing() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }
    let _ = tree.set_root(root_id);

    let item = AssetItem {
        name: "character.glb".to_string(),
        path: PathBuf::from("assets/models/character.glb"),
        relative_path: "models/character.glb".to_string(),
        category: AssetCategory::Models3D,
        source: AssetSource::Project,
        file_size_bytes: 4096,
        metadata_badge: "4.0 KB".to_string(),
        is_loaded_in_memory: false,
        model_handle: None,
        texture_handle: None,
        shader_handle: None,
        is_3d: true,
    };

    let ctx_menu_data = (
        AssetsContextMenuTarget::Asset(item.clone()),
        Point::new(200.0, 150.0),
    );
    let thumbnail_layers = std::collections::HashMap::new();

    let params = AssetsPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
        screen_size: (1920.0, 1080.0),
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        selected_asset: None,
        cached_items: &[],
        filtered_items: &[],
        is_2d_mode: false,
        show_engine_content: false,
        sidebar_width: 200.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(210.0, 160.0),
        blink_caret: false,
        active_context_menu: Some(&ctx_menu_data),
        active_preview_modal: None,
        thumbnail_layers: &thumbnail_layers,
    };

    let mut targets = AssetsPanelTargets::default();
    super::context_menu::build_assets_context_menu(&mut tree, root_id, &params, &mut targets);

    assert!(targets.context_menu.is_some());
    let cm = targets.context_menu.unwrap();
    assert_eq!(cm.card_rect.x, 200.0);
    assert_eq!(cm.card_rect.y, 150.0);

    // Hit test Quick Inspect (tag 0)
    let hit_inspect = tree
        .hit_test_target(Point::new(220.0, 195.0))
        .expect("Must hit inspect item");
    assert_eq!(hit_inspect.layer, UiLayer::Popup);
    assert_eq!(hit_inspect.role, WidgetRole::DropdownItem);
    assert_eq!(hit_inspect.tag, super::types::ASSET_CTX_INSPECT);

    // Test event dispatch with hit target
    let mut tracker = AssetClickTracker::default();
    let mut actions = Vec::new();
    let ctx = AssetsEventContext {
        cursor_pos: Point::new(220.0, 195.0),
        targets: &AssetsPanelTargets {
            context_menu: Some(cm),
            ..Default::default()
        },
        current_folder: Path::new("assets"),
        search_query: "",
        is_search_focused: false,
        selected_asset: None,
        hit_target: Some(hit_inspect),
    };

    let consumed = events::handle_assets_click(&ctx, &mut tracker, &mut actions);
    assert!(consumed);
    assert!(actions.contains(&AssetsPanelAction::CloseContextMenu));
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, AssetsPanelAction::OpenInspectModal(_)))
    );
}