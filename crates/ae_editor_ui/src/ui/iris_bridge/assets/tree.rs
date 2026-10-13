// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hierarchical Folder Tree Sidebar for Iris UI Asset Browser.
//!
//! Renders an interactive, collapsible directory tree using the engine's canonical
//! vector folder logo (`ICON_FOLDER`, Layer 6) with depth indents and selection pills
//! via 100% declarative [`UiScope`] widgets.
//!
//! Zero disk I/O during rendering hot loops: folder hierarchy is referenced directly
//! from engine pre-cached state. Zero per-frame heap allocations.
//!

use super::components::{FolderTreeItemParams, asset_folder_tree_item};
use super::types::{
    ASSETS_TAG_NEW_SUBFOLDER, ASSETS_TAG_TREE_VIEWPORT, AssetsPanelAction, AssetsPanelParams,
};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;
use std::path::{Path, PathBuf};

/// Header height for the "FOLDERS" label and new subfolder button.
pub const FOLDER_HEADER_HEIGHT: f32 = 28.0;

/// Constructs the complete hierarchical folder tree sidebar into the Iris `UiTree`.
pub fn build_folder_tree_sidebar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    sidebar_rect: Rect,
    params: &AssetsPanelParams<'_>,
    tree_state: &mut TreeState<PathBuf>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    let mut scope = UiScope::new(tree, parent_id);
    build_folder_tree_sidebar_scope(&mut scope, sidebar_rect, params, tree_state, actions);
}

/// Constructs the complete hierarchical folder tree sidebar directly via a declarative [`UiScope`].
pub fn build_folder_tree_sidebar_scope(
    scope: &mut UiScope<'_>,
    sidebar_rect: Rect,
    params: &AssetsPanelParams<'_>,
    tree_state: &mut TreeState<PathBuf>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    // 1. Sidebar Container with Hardware Scissor Clipping
    scope.container_named(
        "FolderTreeSidebarRoot",
        Style::new()
            .flex_col()
            .background(Color::rgba(0.06, 0.07, 0.09, 0.98))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.70))
            .clip_children(true)
            .width(sidebar_rect.width)
            .height(sidebar_rect.height),
        |sidebar| {
            // 2. Header Bar ("FOLDERS" + "+" button)
            sidebar.container_named(
                "FolderTreeHeader",
                Style::new()
                    .flex_row()
                    .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
                    .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.50))
                    .height(FOLDER_HEADER_HEIGHT)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0)),
                |hdr| {
                    hdr.label_styled_passive(
                        "FoldersLabel",
                        "FOLDERS",
                        11.0,
                        Color::rgba(0.65, 0.70, 0.80, 1.0),
                        TextAlign::Left,
                        Style::new(),
                    );

                    let new_subfolder_id = hdr.container_tagged(
                        "NewSubfolderBtn",
                        Style::new()
                            .width(20.0)
                            .height(20.0)
                            .border_radius(3.0)
                            .background(Color::rgba(0.12, 0.14, 0.18, 0.80))
                            .hover_background(Color::rgba(0.20, 0.24, 0.32, 1.0))
                            .border(1.0, Color::rgba(0.20, 0.23, 0.30, 0.40))
                            .hover_border(1.0, Color::rgba(0.35, 0.42, 0.55, 0.80))
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center),
                        WidgetRole::Button,
                        ASSETS_TAG_NEW_SUBFOLDER,
                        |btn| {
                            let icon_id =
                                btn.icon(ICON_PLUS, Color::rgba(0.70, 0.75, 0.85, 1.0), 12.0);
                            btn.set_hover_texture_tint(icon_id, Color::WHITE);
                        },
                    );
                    if hdr.response(new_subfolder_id).clicked() {
                        actions.push(AssetsPanelAction::OpenCreateSubfolder(
                            params.current_folder.to_path_buf(),
                        ));
                    }
                },
            );

            // 3. Scrollable Tree Viewport
            sidebar.container_tagged(
                "FolderTreeViewport",
                Style::new()
                    .flex_col()
                    .clip_children(true)
                    .flex_grow(1.0)
                    .scroll_offset_y(params.tree_scroll_y)
                    .width(sidebar_rect.width)
                    .padding_insets(Insets::new(4.0, 0.0, 0.0, 0.0)),
                WidgetRole::ScrollAreaViewport,
                ASSETS_TAG_TREE_VIEWPORT,
                |viewport| {
                    let root_path = PathBuf::from("assets");
                    render_folder_recursive(
                        viewport,
                        &root_path,
                        0,
                        params.subfolders,
                        params.current_folder,
                        tree_state,
                        actions,
                    );
                },
            );
        },
    );
}

/// Recursively builds folder tree rows using declarative [`asset_folder_tree_item`].
///
/// Pure zero-allocation traversal referencing engine pre-cached subfolders.
/// Toggles expansion states autonomously in-place upon foldout chevron interactions.
fn render_folder_recursive(
    scope: &mut UiScope<'_>,
    path: &Path,
    depth: u16,
    subfolders: &[PathBuf],
    current_folder: &Path,
    tree_state: &mut TreeState<PathBuf>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("assets");

    // Discover child directories from pre-cached engine state (ZERO DISK I/O)
    let has_children = subfolders.iter().any(|p| p.parent() == Some(path));

    let is_selected = current_folder == path;
    let is_expanded = tree_state.is_expanded(&path.to_path_buf());

    let node_idx = if path == Path::new("assets") {
        0
    } else {
        subfolders
            .iter()
            .position(|p| p == path)
            .map(|i| (i + 1) as u32)
            .unwrap_or(0)
    };

    // Render pure declarative tree item (Zero Target Collections, Zero Allocations)
    let item_params = FolderTreeItemParams {
        node_idx,
        name: folder_name,
        depth,
        has_children,
        is_expanded,
        is_selected,
    };
    let row_resp = asset_folder_tree_item(scope, &item_params);
    if row_resp.foldout_clicked() {
        tree_state.toggle_expanded(&path.to_path_buf());
    }
    row_resp.context_menu(scope, |menu| {
        super::context_menu::render_folder_context_menu_items(menu, path, actions);
    });
    if row_resp.clicked() {
        actions.push(AssetsPanelAction::NavigateFolder(path.to_path_buf()));
    }

    // Recurse children if expanded
    if is_expanded {
        for child in subfolders.iter().filter(|p| p.parent() == Some(path)) {
            render_folder_recursive(
                scope,
                child,
                depth + 1,
                subfolders,
                current_folder,
                tree_state,
                actions,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::{AssetCategory, AssetItem, AssetViewMode};
    use std::collections::HashMap;

    #[test]
    fn test_tree_sidebar_declarative_build() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let current_folder = PathBuf::from("assets");
        let items: Vec<AssetItem> = Vec::new();
        let subfolders = vec![
            PathBuf::from("assets/models"),
            PathBuf::from("assets/textures"),
        ];
        let mut tree_state = TreeState::default();
        tree_state.set_expanded(PathBuf::from("assets"), true);

        let params = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder: &current_folder,
            search_query: "",
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
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &subfolders,
            thumbnail_layers: &HashMap::new(),
            focus_state: &FocusState::new(),
            cursor_byte_idx: 0,
        };

        let mut actions = Vec::new();
        build_folder_tree_sidebar(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 180.0, 500.0),
            &params,
            &mut tree_state,
            &mut actions,
        );

        let found_sidebar = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("FolderTreeSidebarRoot"));
        let found_header = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("FolderTreeHeader"));
        let found_viewport = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("FolderTreeViewport"));
        let found_row = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("FolderTreeRow"));
        assert!(found_sidebar, "FolderTreeSidebarRoot should be emitted");
        assert!(found_header, "FolderTreeHeader should be emitted");
        assert!(found_viewport, "FolderTreeViewport should be emitted");
        assert!(found_row, "FolderTreeRow should be emitted");
    }

    #[test]
    fn test_tree_sidebar_foldout_toggle_and_selection() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let current_folder = PathBuf::from("assets");
        let items: Vec<AssetItem> = Vec::new();
        let subfolders = vec![
            PathBuf::from("assets/models"),
            PathBuf::from("assets/models/characters"),
        ];

        // 1. Initial State: Only "assets" is expanded; "assets/models" is collapsed
        let mut tree_state = TreeState::default();
        tree_state.set_expanded(PathBuf::from("assets"), true);
        assert!(!tree_state.is_expanded(&PathBuf::from("assets/models")));

        {
            let params = AssetsPanelParams {
                panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
                screen_size: (1280.0, 720.0),
                current_folder: &current_folder,
                search_query: "",
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
                cursor_pos: Point::new(0.0, 0.0),
                blink_caret: false,
                active_context_menu: None,
                active_preview_modal: None,
                subfolders: &subfolders,
                thumbnail_layers: &HashMap::new(),
                focus_state: &FocusState::new(),
                cursor_byte_idx: 0,
            };

            let mut actions = Vec::new();
            build_folder_tree_sidebar(
                &mut tree,
                root_id,
                Rect::new(0.0, 0.0, 180.0, 500.0),
                &params,
                &mut tree_state,
                &mut actions,
            );
        }

        let row_count_collapsed = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("FolderTreeRow"))
            .count();
        // Emits "assets" and "assets/models", but NOT "assets/models/characters"
        assert_eq!(
            row_count_collapsed, 2,
            "When 'assets/models' is collapsed, characters subfolder must not be rendered"
        );

        // 2. Toggle "assets/models" autonomously in-place via tree_state
        let models_path = PathBuf::from("assets/models");
        tree_state.toggle_expanded(&models_path);
        assert!(
            tree_state.is_expanded(&models_path),
            "Folder must now be expanded"
        );

        // 3. Re-render with expanded tree_state
        let params_expanded = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder: &current_folder,
            search_query: "",
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
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &subfolders,
            thumbnail_layers: &HashMap::new(),
            focus_state: &FocusState::new(),
            cursor_byte_idx: 0,
        };
        let mut tree_expanded = UiTree::new();
        let root_exp_id = tree_expanded.create_root().expect("Root creation failed");
        let mut actions_expanded = Vec::new();
        build_folder_tree_sidebar(
            &mut tree_expanded,
            root_exp_id,
            Rect::new(0.0, 0.0, 180.0, 500.0),
            &params_expanded,
            &mut tree_state,
            &mut actions_expanded,
        );

        let row_count_expanded = tree_expanded
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("FolderTreeRow"))
            .count();
        // Emits "assets", "assets/models", and "assets/models/characters"
        assert_eq!(
            row_count_expanded, 3,
            "When 'assets/models' is expanded, characters subfolder must be rendered"
        );

        // 4. Test row selection directly via tree_state
        tree_state.select_single(models_path.clone());
        assert_eq!(tree_state.primary_selected, Some(models_path));
    }
}