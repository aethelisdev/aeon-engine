// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Native Iris UI Asset / Content Browser Subsystem
//!
//! Provides a 100% GPU SDF-accelerated replacement for the egui Asset Browser panel,
//! completely free of emojis, featuring breadcrumb navigation, live search,
//! canonical vector icons, folder tree sidebar, floating right-click context menus,
//! interactive quick asset preview modal, and responsive grid/table views.
//!

pub mod cards;
pub mod context_menu;
pub mod events;
pub mod list;
pub mod panel;
pub mod preview;
pub mod tree;
pub mod types;

pub use events::{
    AssetClickTracker, AssetsEventContext, handle_assets_click, handle_assets_panel_event,
    handle_assets_right_click, handle_assets_scroll,
};
pub use panel::build_assets_panel;
pub use types::{
    AssetCardTarget, AssetPreviewModalState, AssetPreviewModalTargets, AssetRowTarget,
    AssetsContextMenuTarget, AssetsContextMenuTargets, AssetsPanelAction, AssetsPanelParams,
    AssetsPanelTargets, BreadcrumbTarget, FolderTreeNodeTarget,
};

#[cfg(test)]
mod tests {
    use super::*;
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
}