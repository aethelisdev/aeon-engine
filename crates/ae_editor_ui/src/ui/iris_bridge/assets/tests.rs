// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for the Native Iris UI Asset / Content Browser Subsystem.

use super::panel::build_assets_panel;
use super::types::{
    ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT, AssetClickTracker, AssetPreviewModalState,
    AssetsPanelState, encode_breadcrumb_tag, encode_item_tag, truncate_display_name,
};
use crate::assets::types::{AssetBrowserState, AssetCategory, AssetItem, AssetSource};
use irisui::prelude::*;
use irisui::widgets::modal::ModalDialogAction;
use std::path::PathBuf;

fn build_in_host(
    tree: &mut UiTree,
    root: WidgetId,
    bounds: Rect,
    f: impl FnOnce(&mut UiScope<'_>),
) {
    let mut scope = UiScope::new(tree, root);
    scope.panel("AssetsTestHost").bounds(bounds).show(f);
}

#[test]
fn test_assets_panel_structure_and_no_emojis() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
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
    let asset_browser = AssetBrowserState {
        cached_items: vec![item],
        ..Default::default()
    };
    let mut panel_state = AssetsPanelState::default();
    let mut actions = Vec::new();
    build_in_host(&mut tree, root_id, panel_rect, |s| {
        crate::ui::panel_layout::PanelId::Assets
            .surface()
            .expect("Assets surface")
            .show(s, |panel_scope| {
                build_assets_panel(
                    panel_scope,
                    &asset_browser,
                    false,
                    &mut panel_state,
                    &mut actions,
                );
            });
    });

    assert_eq!(
        tree.get(tree.find_node_by_name("AssetsPanelRoot").unwrap())
            .unwrap()
            .computed_rect,
        panel_rect
    );
    assert!(has_tag_recursive(&tree, root_id, encode_breadcrumb_tag(0)));
    assert!(has_tag_recursive(&tree, root_id, encode_item_tag(0)));

    assert_no_emojis_recursive(&tree, root_id);
}

pub(super) fn has_tag_recursive(tree: &UiTree, current: WidgetId, target_tag: u64) -> bool {
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
            for forbidden in ['📁', '📂', '📦', '🖼', '⚡', '🎬', '🎨', '🔊', '⚙']
            {
                assert!(
                    !text.contains(forbidden),
                    "Node text contains forbidden emoji '{forbidden}': {text}"
                );
            }
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
        item,
        orbit_yaw: 0.5,
        orbit_pitch: 0.2,
        zoom_distance: 1.2,
        show_wireframe: true,
    };

    let asset_browser = AssetBrowserState::default();
    let mut panel_state = AssetsPanelState {
        preview_modal: Some(preview_state),
        ..Default::default()
    };

    let mut actions_build = Vec::new();
    build_in_host(
        &mut tree,
        root_id,
        Rect::new(0.0, 0.0, 1000.0, 600.0),
        |s| {
            build_assets_panel(
                s,
                &asset_browser,
                false,
                &mut panel_state,
                &mut actions_build,
            );
        },
    );

    assert!(tree.is_any_modal_open());
    let mut close_button_found = false;
    tree.traverse_depth_first(root_id, &mut |_id, node| {
        if node.tag == ModalDialogAction::TAG_CLOSE {
            close_button_found = true;
        }
    });
    assert!(close_button_found, "Preview modal close button must exist");
}

#[test]
fn test_truncate_display_name_utf8_boundary_safety() {
    let short = "cube.png";
    assert_eq!(truncate_display_name(short, 14, 11), "cube.png");

    let long_ascii = "very_long_texture_filename.png";
    assert_eq!(truncate_display_name(long_ascii, 14, 11), "very_long_t...");

    let turkish = "dokusu_öğütülmüş_yüzey.png";
    let truncated_turkish = truncate_display_name(turkish, 14, 11);
    assert_eq!(truncated_turkish, "dokusu_öğüt...");
    assert!(truncated_turkish.ends_with("..."));

    let cjk = "こんにちは世界_テクスチャ_2026.png";
    let truncated_cjk = truncate_display_name(cjk, 14, 11);
    assert_eq!(truncated_cjk, "こんにちは世界_テクス...");
}

#[test]
fn test_assets_card_rendering_with_unicode_filenames() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);
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
    let asset_browser = AssetBrowserState {
        cached_items: vec![item],
        ..Default::default()
    };
    let mut panel_state = AssetsPanelState::default();

    let mut actions = Vec::new();
    build_in_host(&mut tree, root_id, panel_rect, |s| {
        crate::ui::panel_layout::PanelId::Assets
            .surface()
            .expect("Assets surface")
            .show(s, |panel_scope| {
                build_assets_panel(
                    panel_scope,
                    &asset_browser,
                    false,
                    &mut panel_state,
                    &mut actions,
                );
            });
    });
    assert!(has_tag_recursive(&tree, root_id, encode_item_tag(0)));
}

#[test]
fn test_asset_drag_overlay_construction() {
    use crate::assets::types::{AssetCategory, AssetDragPayload};
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

    super::drag_overlay::build_asset_drag_overlays_scope(
        &mut UiScope::new(&mut tree, root_id),
        &payload,
        cursor,
        vp_rect,
        &camera,
        false,
    );

    let child_ids = tree
        .get(root_id)
        .expect("Root node exists")
        .children
        .clone();
    assert!(child_ids.len() >= 2);

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

    let current = Point::new(110.0, 110.0);
    let start_pos = tracker.drag_start_pos.unwrap();
    let dx = current.x - start_pos.x;
    let dy = current.y - start_pos.y;
    assert!((dx * dx + dy * dy) > 25.0);
    tracker.is_dragging_asset = true;
    tracker.potential_drag_item = None;
    tracker.drag_start_pos = None;
    assert!(tracker.is_dragging_asset);

    tracker.is_dragging_asset = false;
    tracker.potential_drag_item = None;
    tracker.drag_start_pos = None;
    assert!(!tracker.is_dragging_asset);
}

#[test]
fn test_asset_drag_viewport_boundary_check() {
    let viewport_rect = Rect::new(200.0, 100.0, 800.0, 600.0);
    assert!(viewport_rect.contains_point(Point::new(300.0, 200.0)));
    assert!(!viewport_rect.contains_point(Point::new(300.0, 800.0)));
    assert!(!viewport_rect.contains_point(Point::new(50.0, 200.0)));
    assert!(!viewport_rect.contains_point(Point::new(400.0, 15.0)));
}

#[test]
fn test_asset_source_classification_and_filtering() {
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

    let all_items = [project_item, engine_item];
    let filtered_hidden: Vec<_> = all_items
        .iter()
        .filter(|item| item.source != AssetSource::Engine)
        .cloned()
        .collect();
    assert_eq!(filtered_hidden.len(), 1);
    assert_eq!(filtered_hidden[0].name, "player.png");
}

#[test]
fn test_engine_and_sidebar_vector_icons() {
    use crate::ui::iris_bridge::icons::{
        FIRST_THUMBNAIL_LAYER, ICON_CHEVRON_DOWN, ICON_CHEVRON_UP, ICON_GEAR, ICON_SPARKLE,
    };

    assert_eq!(ICON_GEAR, [0.0, 0.0, 1.0, 16.0]);
    assert_eq!(ICON_SPARKLE, [0.0, 0.0, 1.0, 17.0]);
    assert_eq!(ICON_CHEVRON_DOWN, [0.0, 0.0, 1.0, 18.0]);
    assert_eq!(ICON_CHEVRON_UP, [0.0, 0.0, 1.0, 19.0]);
    assert_eq!(FIRST_THUMBNAIL_LAYER, 32);

    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 400.0);

    let asset_browser = AssetBrowserState {
        show_engine_content: true,
        ..Default::default()
    };
    let mut panel_state = AssetsPanelState::default();

    let mut actions = Vec::new();
    build_in_host(&mut tree, root_id, panel_rect, |s| {
        crate::ui::panel_layout::PanelId::Assets
            .surface()
            .expect("Assets surface")
            .show(s, |panel_scope| {
                build_assets_panel(
                    panel_scope,
                    &asset_browser,
                    false,
                    &mut panel_state,
                    &mut actions,
                );
            });
    });

    let gear_id = tree
        .find_node_by_name("EngineGearIcon")
        .expect("EngineGearIcon node must be present");
    assert_eq!(tree.get(gear_id).unwrap().texture_uv, Some(ICON_GEAR));

    let side_id = tree
        .find_node_by_name("SidebarToggleButton")
        .expect("SidebarToggleButton node must be present");
    assert_eq!(tree.get(side_id).unwrap().text.as_deref(), Some("◀"));
}

#[test]
fn test_bidirectional_scene_filtering_and_world_bytes() {
    use crate::assets::scanner::is_world_bytes_3d;

    let mut ids = ae_world::GuidGenerator::from_seed(101);
    let world_3d = ae_world::WorldDocument::new(ids.next_entity_guid(), false, 128.0)
        .encode_full()
        .expect("encode 3D binary world");
    assert!(is_world_bytes_3d(&world_3d));

    let world_2d = ae_world::WorldDocument::new(ids.next_entity_guid(), true, 128.0)
        .encode_full()
        .expect("encode 2D binary world");
    assert!(!is_world_bytes_3d(&world_2d));
    assert!(!is_world_bytes_3d(b"corrupted-not-a-world"));
}

#[test]
fn test_assets_toolbar_width_and_search_ux() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node creation failed");

    let panel_rect = Rect::new(0.0, 0.0, 960.0, 480.0);
    let asset_browser = AssetBrowserState {
        search_query: "test_query".to_string(),
        ..Default::default()
    };
    let mut panel_state = AssetsPanelState::default();
    panel_state.search_input.set_text("test_query");

    let mut actions = Vec::new();
    build_in_host(&mut tree, root_id, panel_rect, |s| {
        crate::ui::panel_layout::PanelId::Assets
            .surface()
            .expect("Assets surface")
            .show(s, |panel_scope| {
                build_assets_panel(
                    panel_scope,
                    &asset_browser,
                    false,
                    &mut panel_state,
                    &mut actions,
                );
            });
    });

    assert!(has_tag_recursive(&tree, root_id, ASSETS_TAG_SEARCH_INPUT));
    assert!(has_tag_recursive(&tree, root_id, ASSETS_TAG_SEARCH_CLEAR));

    let toolbar_id = tree
        .find_node_by_name("AssetsTopToolbar")
        .expect("AssetsTopToolbar node must exist in UiTree");
    assert_eq!(tree.get(toolbar_id).unwrap().computed_rect.width, 960.0);
}