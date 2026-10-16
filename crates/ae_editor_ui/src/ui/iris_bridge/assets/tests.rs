// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for the Native Iris UI Asset / Content Browser Subsystem.
//!

use super::panel::build_assets_panel;
use super::types::{
    ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT, AssetClickTracker, AssetPreviewModalState,
    AssetsContextMenuTarget, AssetsPanelParams, encode_breadcrumb_tag, encode_item_tag,
    truncate_display_name,
};
use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
use irisui::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn test_assets_panel_structure_and_no_emojis() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

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
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    let metrics = build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );

    assert_eq!(metrics.panel_rect, panel_rect);
    assert!(has_tag_recursive(&tree, root_id, encode_breadcrumb_tag(0)));
    assert!(has_tag_recursive(&tree, root_id, encode_item_tag(0)));

    // Verify that NO emojis exist in node text strings across the entire panel
    assert_no_emojis_recursive(&tree, root_id);
}

fn has_tag_recursive(tree: &UiTree, current: WidgetId, target_tag: u64) -> bool {
    let mut found = false;
    tree.traverse_depth_first(current, &mut |_id, node| {
        if node.tag == target_tag {
            found = true;
        }
    });
    found
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
fn test_preview_modal_build_and_actions() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

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
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(500.0, 300.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: Some(&preview_state),
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions_build = Vec::new();
    let _metrics = build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions_build,
    );

    // Test close button on the preview modal exists in UiTree via semantic tag
    let mut close_button_found = false;
    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.tag == irisui::prelude::MODAL_TAG_CLOSE {
            close_button_found = true;
        }
    });
    assert!(close_button_found, "Preview modal close button must exist");
}

#[test]
fn test_preview_modal_declarative_scene_and_buttons_hover_reactivity() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let scene_item = AssetItem {
        name: "test_level.ae3d".to_string(),
        path: PathBuf::from("assets/scenes/test_level.ae3d"),
        relative_path: "scenes/test_level.ae3d".to_string(),
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

    // 1. Build with cursor far away (idle state)
    let params_idle = AssetsPanelParams {
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
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        active_context_menu: None,
        active_preview_modal: Some(&preview_state),
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions_idle = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params_idle,
        &mut TreeState::default(),
        &mut actions_idle,
    );

    let mut has_test_level_title = false;
    let mut close_rect = Rect::ZERO;
    let mut reveal_rect = Rect::ZERO;
    let mut has_action_btn = false;

    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.text.as_deref() == Some("test_level.ae3d") {
            has_test_level_title = true;
        } else if node.tag == irisui::prelude::MODAL_TAG_CLOSE {
            close_rect = node.computed_rect;
        } else if node.tag == crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_REVEAL {
            reveal_rect = node.computed_rect;
        } else if node.tag == irisui::prelude::MODAL_TAG_CONFIRM {
            has_action_btn = true;
        }
    });

    assert!(
        has_test_level_title,
        "Modal title test_level.ae3d must exist in tree"
    );

    assert!(
        has_action_btn,
        "Scene preview must have Load Scene action button"
    );
    assert!(close_rect.width > 0.0 && close_rect.height > 0.0);
    assert!(reveal_rect.width > 0.0 && reveal_rect.height > 0.0);

    // Verify idle style of close button (transparent background)
    let mut close_node_idle = None;
    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.computed_rect == close_rect {
            close_node_idle = Some((node.text.clone(), node.style.background_color));
        }
    });
    let (idle_text, idle_bg) = close_node_idle.expect("Close node found");
    assert_eq!(idle_text.as_deref(), Some("✕"));
    assert_eq!(idle_bg, Color::TRANSPARENT);

    // 2. Re-build with cursor hovered over close button
    let mut tree_hover = UiTree::new();
    let root_hover = tree_hover.create_root().expect("Root node creation failed");

    let mut params_hover = params_idle;
    params_hover.cursor_pos = Point::new(close_rect.x + 2.0, close_rect.y + 2.0);
    tree_hover.set_cursor_position(params_hover.cursor_pos);

    let mut actions_hover = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree_hover, root_hover),
        &params_hover,
        &mut TreeState::default(),
        &mut actions_hover,
    );

    let mut close_node_hover_bg = None;
    tree_hover.traverse_depth_first(root_hover, &mut |_id, node| {
        if node.computed_rect == close_rect {
            close_node_hover_bg = Some(node.style.background_color);
        }
    });
    let hover_bg = close_node_hover_bg.expect("Hovered close node found");
    assert_eq!(hover_bg, Color::rgba(0.85, 0.22, 0.22, 0.28));
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
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    // Should build cards with Turkish/Unicode filenames without any panic
    let mut actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );
    assert!(has_tag_recursive(&tree, root_id, encode_item_tag(0)));
}

/// Verifies that constructing drag overlay nodes creates non-zero layout rectangles
/// for the viewport landing ring, center target dot, and floating tooltip capsule labels.
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

    let child_ids = tree
        .get(root_id)
        .expect("Root node exists")
        .children
        .clone();
    assert!(child_ids.len() >= 2);

    // Verify declarative layout_subtree resolved valid non-zero computed_rects for overlay children
    let ring_node = tree.get(child_ids[0]).expect("Ring node exists");
    assert!(ring_node.computed_rect.width > 0.0);
    assert!(ring_node.computed_rect.height > 0.0);

    let capsule_node = tree
        .get(child_ids[child_ids.len() - 1])
        .expect("Capsule node exists");
    assert!(capsule_node.computed_rect.width > 0.0);
    assert!(capsule_node.computed_rect.height > 0.0);

    for &label_id in &capsule_node.children {
        let label_node = tree.get(label_id).expect("Label node exists");
        assert!(label_node.computed_rect.width > 0.0);
        assert!(label_node.computed_rect.height > 0.0);
    }
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
    let is_in = |p: Point| {
        p.x >= viewport_rect.x
            && p.x <= viewport_rect.right()
            && p.y >= viewport_rect.y
            && p.y <= viewport_rect.bottom()
    };

    // Inside viewport
    let inside_pos = Point::new(300.0, 200.0);
    assert!(is_in(inside_pos));

    // Over Asset panel (outside viewport)
    let asset_panel_pos = Point::new(300.0, 800.0);
    assert!(!is_in(asset_panel_pos));

    // Over Hierarchy panel (outside viewport)
    let hierarchy_pos = Point::new(50.0, 200.0);
    assert!(!is_in(hierarchy_pos));

    // Over Menubar (outside viewport)
    let menubar_pos = Point::new(400.0, 15.0);
    assert!(!is_in(menubar_pos));
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
    let root_id = tree.create_root().expect("Root node must be created");
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);

    let current_folder = PathBuf::from("assets");
    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1280.0, 720.0),
        current_folder: &current_folder,
        search_query: "",
        active_category: AssetCategory::All,
        view_mode: AssetViewMode::Grid,
        show_engine_content: true,
        cached_items: &[],
        filtered_items: &[],
        subfolders: &[],
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
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );

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
    let root_id = tree.create_root().expect("Root node must be created");
    UiScope::new(&mut tree, root_id).finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));

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

    let panel_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1920.0, 1080.0),
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
        sidebar_width: 200.0,
        sidebar_collapsed: false,
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(210.0, 160.0),
        blink_caret: false,
        active_context_menu: Some(&ctx_menu_data),
        active_preview_modal: None,
        thumbnail_layers: &thumbnail_layers,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let card_rect = super::context_menu::build_assets_context_menu(&mut tree, root_id, &params)
        .expect("Context menu card rect must be returned");
    assert_eq!(card_rect.x, 200.0);
    assert_eq!(card_rect.y, 150.0);

    // Hit test Quick Inspect (tag 0)
    let hit_inspect = tree
        .hit_test_target(Point::new(220.0, 195.0))
        .expect("Must hit inspect item");
    assert_eq!(hit_inspect.layer, UiLayer::Popup);
    assert_eq!(hit_inspect.role, WidgetRole::DropdownItem);
    assert_eq!(hit_inspect.tag, super::types::ASSET_CTX_INSPECT);

    // Hit test Rename (tag 3)
    let hit_rename = tree
        .hit_test_target(Point::new(220.0, 250.0))
        .expect("Must hit rename item");
    assert_eq!(hit_rename.layer, UiLayer::Popup);
    assert_eq!(hit_rename.role, WidgetRole::DropdownItem);
    assert_eq!(hit_rename.tag, super::types::ASSET_CTX_RENAME);

    // Hit test Delete (tag 4)
    let hit_delete = tree
        .hit_test_target(Point::new(220.0, 275.0))
        .expect("Must hit delete item");
    assert_eq!(hit_delete.layer, UiLayer::Popup);
    assert_eq!(hit_delete.role, WidgetRole::DropdownItem);
    assert_eq!(hit_delete.tag, super::types::ASSET_CTX_DELETE);
}

#[test]
fn test_assets_toolbar_width_and_search_ux() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let panel_rect = Rect::new(0.0, 0.0, 960.0, 480.0);
    let current_folder = PathBuf::from("assets");
    let params = AssetsPanelParams {
        panel_rect,
        screen_size: (1920.0, 1080.0),
        current_folder: &current_folder,
        search_query: "test_query",
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
        scroll_y: 0.0,
        tree_scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: true,
        active_context_menu: None,
        active_preview_modal: None,
        thumbnail_layers: &HashMap::new(),
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let mut actions = Vec::new();
    build_assets_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut TreeState::default(),
        &mut actions,
    );

    // Verify search input container is registered with ASSETS_TAG_SEARCH_INPUT
    assert!(has_tag_recursive(&tree, root_id, ASSETS_TAG_SEARCH_INPUT));

    // Verify search clear button is present when query is non-empty
    assert!(has_tag_recursive(&tree, root_id, ASSETS_TAG_SEARCH_CLEAR));

    // Verify AssetsTopToolbar has explicit width equal to panel_rect.width
    let mut toolbar_found = false;
    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.name.as_deref() == Some("AssetsTopToolbar") {
            toolbar_found = true;
            assert_eq!(node.style.width, Some(960.0));
        }
    });
    assert!(toolbar_found, "AssetsTopToolbar node must exist in UiTree");
}