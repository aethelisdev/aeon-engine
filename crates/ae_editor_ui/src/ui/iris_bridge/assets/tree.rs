// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hierarchical Folder Tree Sidebar for Iris UI Asset Browser.
//!
//! Binds cached folder paths to `irisui`'s [`UiScope::virtual_tree_retained`] adapter with
//! autonomous scrolling, foldout expansion state inside [`UiTree`](irisui::core::UiTree),
//! and zero per-frame consumer traversal.

use super::components::FOLDER_TREE_ROW_HEIGHT;
use super::types::{
    ASSETS_TAG_NEW_SUBFOLDER, ASSETS_TAG_TREE_SCROLLBAR_THUMB_ID,
    ASSETS_TAG_TREE_SCROLLBAR_TRACK_ID, ASSETS_TAG_TREE_VIEWPORT_ID, AssetsPanelAction,
    TREE_CHEVRON_BASE_TAG, TREE_ROW_BASE_TAG,
};
use crate::ui::iris_bridge::icons::{ICON_FOLDER, ICON_PLUS};
use irisui::prelude::*;
use irisui::widgets::tree_view::{TreeDataSource, TreeNodeCollector, TreeRowStyle};
use std::path::{Path, PathBuf};

/// Header height for the "FOLDERS" label and new subfolder button.
pub const FOLDER_HEADER_HEIGHT: f32 = 28.0;

const ASSETS_ROOT_KEY: u64 = UiTag::from_str("assets").raw();
const INITIAL_EXPANDED_FOLDERS: &[u64] = &[ASSETS_ROOT_KEY];

/// Computes a deterministic 64-bit key for any relative directory path without heap allocations.
#[inline]
#[must_use]
fn folder_path_key(path: &Path) -> u64 {
    if path == Path::new("assets") {
        ASSETS_ROOT_KEY
    } else {
        path.to_string_lossy()
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, &byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            })
            | 1
    }
}

#[inline]
#[must_use]
fn subfolders_revision(subfolders: &[PathBuf]) -> u64 {
    subfolders.iter().fold(
        0xcbf2_9ce4_8422_2325_u64 ^ (subfolders.len() as u64),
        |hash, path| (hash ^ folder_path_key(path)).wrapping_mul(0x0100_0000_01b3),
    )
}

#[inline]
fn resolve_folder_path(subfolders: &[PathBuf], key: u64) -> &Path {
    if key == ASSETS_ROOT_KEY {
        Path::new("assets")
    } else {
        subfolders
            .iter()
            .find(|p| folder_path_key(p) == key)
            .map_or(Path::new("assets"), PathBuf::as_path)
    }
}

struct AssetsFolderSource<'a>(&'a [PathBuf]);

impl TreeDataSource for AssetsFolderSource<'_> {
    type Id = u64;

    fn node_key(&self, id: Self::Id) -> u64 {
        id
    }

    fn id_from_key(&self, key: u64) -> Option<Self::Id> {
        Some(key)
    }

    fn collect_nodes(&self, collector: &mut TreeNodeCollector<Self::Id>) {
        collector.reserve(self.0.len() + 1);
        collector.push_node(ASSETS_ROOT_KEY, None);
        for path in self.0 {
            collector.push_node(folder_path_key(path), path.parent().map(folder_path_key));
        }
    }

    fn search_text(&self, id: Self::Id, out: &mut String) {
        out.push_str(
            resolve_folder_path(self.0, id)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("assets"),
        );
    }
}

fn folder_tree_config(current_folder: &Path) -> VirtualTreeConfig<u64> {
    VirtualTreeConfig::new(
        "FolderTreeViewport",
        "FolderTreeRow",
        ASSETS_TAG_TREE_VIEWPORT_ID,
        (
            ASSETS_TAG_TREE_SCROLLBAR_TRACK_ID,
            ASSETS_TAG_TREE_SCROLLBAR_THUMB_ID,
        ),
        TREE_ROW_BASE_TAG,
        TREE_CHEVRON_BASE_TAG,
    )
    .with_default_expanded(false)
    .with_initial_expanded_keys(INITIAL_EXPANDED_FOLDERS)
    .with_external_selected(Some(folder_path_key(current_folder)))
    .with_tree_config(
        TreeViewConfig::new()
            .with_name("FolderTreeRow")
            .with_row_height(FOLDER_TREE_ROW_HEIGHT)
            .with_row_gap(0.0)
            .with_style(TreeRowStyle::dark_default()),
    )
}

/// Constructs the hierarchical folder tree sidebar using [`UiScope::virtual_tree_retained`].
pub fn build_folder_tree_sidebar_scope(
    scope: &mut UiScope<'_>,
    sidebar_width: f32,
    current_folder: &Path,
    subfolders: &[PathBuf],
    actions: &mut Vec<AssetsPanelAction>,
) {
    scope.container_named(
        "FolderTreeSidebarRoot",
        Style::new()
            .flex_col()
            .background(scope.theme().panel_bg)
            .border(1.0, scope.theme().panel_border)
            .clip_children(true)
            .width(sidebar_width)
            .flex_shrink(0.0),
        |sidebar| {
            sidebar.container_named(
                "FolderTreeHeader",
                Style::new()
                    .flex_row()
                    .background(sidebar.theme().card_header_bg)
                    .border(1.0, sidebar.theme().divider)
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

                    hdr.container_tagged_resp(
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
                            btn.icon_hover(
                                ICON_PLUS,
                                Color::rgba(0.70, 0.75, 0.85, 1.0),
                                Color::WHITE,
                                12.0,
                            );
                        },
                    )
                    .on_click(|| {
                        actions.push(AssetsPanelAction::OpenCreateSubfolder(
                            current_folder.to_path_buf(),
                        ));
                    });
                },
            );

            sidebar
                .virtual_tree_retained(
                    &AssetsFolderSource(subfolders),
                    subfolders_revision(subfolders),
                    subfolders.len() + 1,
                    "",
                    &folder_tree_config(current_folder),
                    actions,
                )
                .on_selection(|acts, event| match event {
                    TreeSelectionEvent::Primary(Some(key)) | TreeSelectionEvent::Secondary(key) => {
                        acts.push(AssetsPanelAction::NavigateFolder(
                            resolve_folder_path(subfolders, key).to_path_buf(),
                        ));
                    }
                    TreeSelectionEvent::Primary(None) => {}
                })
                .context_menu(|menu, acts, key| {
                    super::context_menu::render_folder_context_menu_items(
                        menu,
                        resolve_folder_path(subfolders, key),
                        acts,
                    );
                })
                .render_row(|row_s, _acts, entry, _master_idx, is_selected| {
                    render_folder_row_contents(
                        row_s,
                        resolve_folder_path(subfolders, entry.id),
                        is_selected,
                    );
                });
        },
    );
}

fn render_folder_row_contents(row: &mut UiScope<'_>, path: &Path, is_selected: bool) {
    row.icon(
        ICON_FOLDER,
        if is_selected {
            Color::rgba(0.0, 0.90, 1.0, 1.0)
        } else {
            Color::rgba(0.95, 0.76, 0.28, 1.0)
        },
        16.0,
    );

    row.spacing(6.0);

    render_folder_name(
        row,
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("assets"),
        is_selected,
    );
}

fn render_folder_name(row: &mut UiScope<'_>, folder_name: &str, is_selected: bool) {
    if is_selected {
        row.label_styled_passive(
            "TreeFolderName",
            folder_name,
            11.5,
            Color::WHITE,
            TextAlign::Left,
            Style::new().flex_grow(1.0),
        );
    } else {
        row.label_styled_hover_passive(
            "TreeFolderName",
            folder_name,
            11.5,
            (
                Color::rgba(0.75, 0.78, 0.85, 1.0),
                Color::rgba(0.90, 0.92, 0.96, 1.0),
            ),
            TextAlign::Left,
            Style::new().flex_grow(1.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sidebar_declarative_build() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let current_folder = PathBuf::from("assets");
        let subfolders = vec![
            PathBuf::from("assets/models"),
            PathBuf::from("assets/textures"),
        ];

        let mut actions = Vec::new();
        build_folder_tree_sidebar_scope(
            &mut UiScope::new(&mut tree, root_id),
            180.0,
            &current_folder,
            &subfolders,
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
        let subfolders = vec![
            PathBuf::from("assets/models"),
            PathBuf::from("assets/models/characters"),
        ];

        {
            let mut actions = Vec::new();
            build_folder_tree_sidebar_scope(
                &mut UiScope::new(&mut tree, root_id),
                180.0,
                &current_folder,
                &subfolders,
                &mut actions,
            );
        }

        let row_count_collapsed = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("FolderTreeRow"))
            .count();
        assert_eq!(
            row_count_collapsed, 2,
            "When 'assets/models' is collapsed, characters subfolder must not be rendered"
        );

        // Toggle 'assets/models' foldout (row index 1) directly via retained tree click.
        tree.clear();
        let root_toggle = tree.create_root().expect("Root toggle");
        tree.event_state.frame_clicked_tag = Some(TREE_CHEVRON_BASE_TAG.indexed(1).raw());
        {
            let mut actions = Vec::new();
            build_folder_tree_sidebar_scope(
                &mut UiScope::new(&mut tree, root_toggle),
                180.0,
                &current_folder,
                &subfolders,
                &mut actions,
            );
        }

        tree.clear();
        let root_expanded = tree.create_root().expect("Root expanded");
        tree.event_state.frame_clicked_tag = Some(TREE_ROW_BASE_TAG.indexed(1).raw());
        let mut actions_expanded = Vec::new();
        build_folder_tree_sidebar_scope(
            &mut UiScope::new(&mut tree, root_expanded),
            180.0,
            &current_folder,
            &subfolders,
            &mut actions_expanded,
        );

        let row_count_expanded = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("FolderTreeRow"))
            .count();
        assert_eq!(
            row_count_expanded, 3,
            "When 'assets/models' is expanded in UiTree, characters subfolder must be rendered"
        );
        assert!(
            actions_expanded.contains(&AssetsPanelAction::NavigateFolder(PathBuf::from(
                "assets/models"
            ))),
            "Clicking folder row must emit NavigateFolder action"
        );
    }
}