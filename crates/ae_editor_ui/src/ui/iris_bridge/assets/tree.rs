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
use super::types::{ASSETS_TAG_NEW_SUBFOLDER, AssetsPanelParams};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;
use std::path::{Path, PathBuf};

/// Height of an individual folder tree node row in pixels.
pub const FOLDER_ROW_HEIGHT: f32 = 24.0;

/// Header height for the "FOLDERS" label and new subfolder button.
pub const FOLDER_HEADER_HEIGHT: f32 = 28.0;

/// Constructs the complete hierarchical folder tree sidebar into the Iris `UiTree`.
pub fn build_folder_tree_sidebar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    sidebar_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    let mut scope = UiScope::new(tree, parent_id);
    build_folder_tree_sidebar_scope(&mut scope, sidebar_rect, params);
}

/// Constructs the complete hierarchical folder tree sidebar directly via a declarative [`UiScope`].
pub fn build_folder_tree_sidebar_scope(
    scope: &mut UiScope<'_>,
    sidebar_rect: Rect,
    params: &AssetsPanelParams<'_>,
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
            let is_plus_hovered = params.hovered_tag == Some(ASSETS_TAG_NEW_SUBFOLDER);

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

                    hdr.container_tagged(
                        "NewSubfolderBtn",
                        Style::new()
                            .width(20.0)
                            .height(20.0)
                            .border_radius(3.0)
                            .background(if is_plus_hovered {
                                Color::rgba(0.20, 0.24, 0.32, 1.0)
                            } else {
                                Color::rgba(0.12, 0.14, 0.18, 0.80)
                            })
                            .border(
                                1.0,
                                if is_plus_hovered {
                                    Color::rgba(0.35, 0.42, 0.55, 0.80)
                                } else {
                                    Color::rgba(0.20, 0.23, 0.30, 0.40)
                                },
                            )
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center),
                        WidgetRole::Button,
                        ASSETS_TAG_NEW_SUBFOLDER,
                        |btn| {
                            btn.icon(
                                ICON_PLUS,
                                if is_plus_hovered {
                                    Color::WHITE
                                } else {
                                    Color::rgba(0.70, 0.75, 0.85, 1.0)
                                },
                                12.0,
                            );
                        },
                    );
                },
            );

            // 3. Scrollable Tree Viewport
            let vp_rect = Rect::new(
                sidebar_rect.x,
                sidebar_rect.y + FOLDER_HEADER_HEIGHT,
                sidebar_rect.width,
                sidebar_rect.height - FOLDER_HEADER_HEIGHT,
            );

            sidebar.container_named(
                "FolderTreeViewport",
                Style::new()
                    .flex_col()
                    .clip_children(true)
                    .flex_grow(1.0)
                    .width(sidebar_rect.width)
                    .padding_insets(Insets::new(4.0, 0.0, 0.0, 0.0)),
                |viewport| {
                    let root_path = PathBuf::from("assets");
                    let mut cur_y = vp_rect.y + 4.0 - params.tree_scroll_y;
                    let mut tree_idx = 0;
                    let mut ctx = FolderTreeContext { vp_rect, params };
                    render_folder_recursive(
                        viewport,
                        &root_path,
                        0,
                        &mut cur_y,
                        &mut tree_idx,
                        &mut ctx,
                    );
                },
            );
        },
    );
}

/// Context descriptor bundling tree traversal layout parameters.
struct FolderTreeContext<'a, 'p> {
    /// Scissor-clipped scrollable viewport bounding box.
    pub vp_rect: Rect,
    /// Read-only panel rendering parameters.
    pub params: &'a AssetsPanelParams<'p>,
}

/// Recursively builds folder tree rows using declarative [`asset_folder_tree_item`].
///
/// Pure zero-allocation traversal referencing engine pre-cached subfolders.
fn render_folder_recursive(
    scope: &mut UiScope<'_>,
    path: &Path,
    depth: usize,
    cur_y: &mut f32,
    tree_idx: &mut usize,
    ctx: &mut FolderTreeContext<'_, '_>,
) {
    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("assets");

    // Discover child directories from pre-cached engine state (ZERO DISK I/O)
    let has_children = ctx
        .params
        .subfolders
        .iter()
        .any(|p| p.parent() == Some(path));

    let is_selected = ctx.params.current_folder == path;
    let is_expanded = path == Path::new("assets") || ctx.params.current_folder.starts_with(path);

    let row_y = *cur_y;
    *cur_y += FOLDER_ROW_HEIGHT;

    let this_idx = *tree_idx;
    *tree_idx += 1;

    // Viewport scissor cull: skip generating node if completely outside viewport
    if row_y + FOLDER_ROW_HEIGHT < ctx.vp_rect.y || row_y > ctx.vp_rect.bottom() {
        if is_expanded {
            for child in ctx
                .params
                .subfolders
                .iter()
                .filter(|p| p.parent() == Some(path))
            {
                render_folder_recursive(scope, child, depth + 1, cur_y, tree_idx, ctx);
            }
        }
        return;
    }

    // Render pure declarative tree item (Zero Target Collections, Zero Allocations)
    let item_params = FolderTreeItemParams {
        node_idx: this_idx as u32,
        name: folder_name,
        depth,
        has_children,
        is_expanded,
        is_selected,
        hovered_tag: ctx.params.hovered_tag,
    };
    asset_folder_tree_item(scope, &item_params);

    // Recurse children if expanded
    if is_expanded {
        for child in ctx
            .params
            .subfolders
            .iter()
            .filter(|p| p.parent() == Some(path))
        {
            render_folder_recursive(scope, child, depth + 1, cur_y, tree_idx, ctx);
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

        let params = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
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
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &subfolders,
            hovered_tag: None,
            thumbnail_layers: &HashMap::new(),
        };

        build_folder_tree_sidebar(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 180.0, 500.0),
            &params,
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
}