// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Asset Browser UI Components
//!
//! Provides modular, reusable, 100% declarative UI widgets for the Content / Asset Browser panel
//! using [`UiScope`], including interactive breadcrumb bars, folder tree items, responsive
//! grid asset cards, and tabular list rows.
//!

use super::types::{
    AssetsPanelAction, encode_breadcrumb_tag, encode_item_inspect_tag, encode_item_spawn_tag,
    encode_item_tag, encode_tree_chevron_tag, encode_tree_row_tag, truncate_display_name,
};
use crate::assets::types::{AssetCategory, AssetItem};
use crate::ui::iris_bridge::icons::{
    ICON_AUDIO, ICON_CAMERA, ICON_CUBE, ICON_FOLDER, ICON_PLUS, ICON_SPARKLE, ICON_SPHERE,
    ICON_WORLD,
};
use irisui::prelude::*;
use irisui::widgets::declarative::{
    TreeRowProps, TreeRowResponse, TreeViewConfig, WrappedLabelDescriptor,
};
use std::path::Path;

/// Standard width of an asset grid card in logical pixels.
pub const ASSET_CARD_WIDTH: f32 = 115.0;
/// Standard height of an asset grid card in logical pixels.
pub const ASSET_CARD_HEIGHT: f32 = 125.0;
/// Spacing between grid cards in logical pixels.
pub const ASSET_CARD_GAP: f32 = 10.0;
/// Height of a folder tree row in logical pixels.
pub const FOLDER_TREE_ROW_HEIGHT: f32 = 24.0;
/// Height of a table list row in logical pixels.
pub const ASSET_LIST_ROW_HEIGHT: f32 = 28.0;

/// Resolves the canonical RGBA badge color for an asset category.
#[inline]
pub fn resolve_category_color(category: AssetCategory) -> Color {
    match category {
        AssetCategory::Models3D => Color::rgba(0.0, 0.90, 1.0, 1.0), // Aeon Cyan
        AssetCategory::Textures2D => Color::rgba(0.39, 0.86, 0.47, 1.0), // Emerald Green
        AssetCategory::Shaders => Color::rgba(1.0, 0.75, 0.24, 1.0), // Amber / Yellow
        AssetCategory::Materials => Color::rgba(0.86, 0.39, 0.86, 1.0), // Magenta
        AssetCategory::Scenes => Color::rgba(0.31, 0.63, 1.0, 1.0),  // Sky Blue
        AssetCategory::Audio => Color::rgba(1.0, 0.47, 0.39, 1.0),   // Coral
        AssetCategory::All => Color::rgba(0.70, 0.72, 0.78, 1.0),
    }
}

/// Resolves the canonical vector icon texture UV coordinates and color tint for a category.
#[inline]
pub fn resolve_category_icon(category: AssetCategory) -> ([f32; 4], Color) {
    match category {
        AssetCategory::Models3D => (ICON_CUBE, Color::rgba(0.0, 0.90, 1.0, 1.0)),
        AssetCategory::Textures2D => (ICON_WORLD, Color::rgba(0.39, 0.86, 0.47, 1.0)),
        AssetCategory::Shaders => (ICON_SPARKLE, Color::rgba(1.0, 0.75, 0.24, 1.0)),
        AssetCategory::Scenes => (ICON_CAMERA, Color::rgba(0.31, 0.63, 1.0, 1.0)),
        AssetCategory::Materials => (ICON_SPHERE, Color::rgba(0.86, 0.39, 0.86, 1.0)),
        AssetCategory::Audio => (ICON_AUDIO, Color::rgba(1.0, 0.47, 0.39, 1.0)),
        AssetCategory::All => (ICON_FOLDER, Color::WHITE),
    }
}

/// Emits a declarative interactive breadcrumb navigation bar showing the active directory hierarchy.
///
/// Emits a cyan folder icon at the root, followed by interactive button segments separated by chevrons.
/// Clicking a segment appends [`super::types::AssetsPanelAction::NavigateFolder`] to the action stream.
#[inline]
fn breadcrumb_button_style() -> Style {
    Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .padding_insets(Insets::symmetric(0.0, 6.0))
        .height(22.0)
        .background(Color::TRANSPARENT)
        .hover_background(Color::rgba(0.18, 0.22, 0.30, 0.80))
        .border_radius(3.0)
}

pub fn asset_breadcrumb_bar(
    scope: &mut UiScope<'_>,
    current_folder: &Path,
    actions: &mut Vec<super::types::AssetsPanelAction>,
) {
    scope.container_named(
        "AssetBreadcrumbBar",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(4.0)
            .height(26.0),
        |bar| {
            // Root Folder Atlas Logo (Layer 6: ICON_FOLDER)
            bar.icon(ICON_FOLDER, Color::rgba(0.0, 0.90, 1.0, 1.0), 16.0);
            bar.spacing(4.0);

            for (i, segment) in current_folder
                .components()
                .filter_map(|comp| comp.as_os_str().to_str().filter(|s| !s.is_empty()))
                .enumerate()
            {
                if i > 0 {
                    bar.label_styled_passive(
                        "BreadcrumbSeparator",
                        ">",
                        11.0,
                        Color::rgba(0.40, 0.45, 0.55, 1.0),
                        TextAlign::Center,
                        Style::new().width(8.0),
                    );
                }

                bar.container_tagged_resp(
                    "BreadcrumbButton",
                    breadcrumb_button_style(),
                    WidgetRole::Button,
                    encode_breadcrumb_tag(i as u8),
                    |crumb| {
                        crumb.label_styled_hover_passive(
                            "BreadcrumbText",
                            segment,
                            11.5,
                            (
                                Color::rgba(0.70, 0.74, 0.84, 1.0),
                                Color::rgba(0.0, 0.90, 1.0, 1.0),
                            ),
                            TextAlign::Center,
                            Style::new(),
                        );
                    },
                )
                .on_click(|| {
                    actions.push(super::types::AssetsPanelAction::NavigateFolder(
                        current_folder.components().take(i + 1).collect(),
                    ));
                });
            }

            if current_folder.as_os_str().is_empty() {
                bar.label_styled_passive(
                    "BreadcrumbRootLabel",
                    "assets",
                    11.5,
                    Color::WHITE,
                    TextAlign::Left,
                    Style::new().width(38.0),
                );
            }
        },
    );
}

/// Parameters for rendering a folder tree item in the asset browser sidebar.
#[derive(Debug, Clone, Copy)]
pub struct FolderTreeItemParams<'a> {
    /// Zero-based sequential node index used for encoding semantic tags.
    pub node_idx: u32,
    /// Directory display name.
    pub name: &'a str,
    /// Tree hierarchy nesting depth.
    pub depth: u16,
    /// Whether this folder contains subdirectories.
    pub has_children: bool,
    /// Whether this folder branch is expanded.
    pub is_expanded: bool,
    /// Whether this folder is currently selected.
    pub is_selected: bool,
}

/// Emits an individual directory row in the folder tree sidebar using declarative [`UiScope::tree_row`].
///
/// Returns [`TreeRowResponse`] containing click and foldout toggle interaction states.
pub fn asset_folder_tree_item(
    scope: &mut UiScope<'_>,
    params: &FolderTreeItemParams<'_>,
) -> TreeRowResponse {
    scope.tree_row(
        "FolderTreeRow",
        TreeRowProps::new(
            params.depth,
            params.has_children,
            params.is_expanded,
            params.is_selected,
            encode_tree_row_tag(params.node_idx),
            encode_tree_chevron_tag(params.node_idx),
        ),
        &TreeViewConfig::new()
            .with_row_height(FOLDER_TREE_ROW_HEIGHT)
            .with_row_gap(0.0),
        |row| {
            row.icon(
                ICON_FOLDER,
                if params.is_selected {
                    Color::rgba(0.0, 0.90, 1.0, 1.0)
                } else {
                    Color::rgba(0.95, 0.76, 0.28, 1.0)
                },
                16.0,
            );

            row.spacing(6.0);

            if params.is_selected {
                row.label_styled_passive(
                    "TreeFolderName",
                    params.name,
                    11.5,
                    Color::WHITE,
                    TextAlign::Left,
                    Style::new().flex_grow(1.0),
                );
            } else {
                row.label_styled_hover_passive(
                    "TreeFolderName",
                    params.name,
                    11.5,
                    (
                        Color::rgba(0.75, 0.78, 0.85, 1.0),
                        Color::rgba(0.90, 0.92, 0.96, 1.0),
                    ),
                    TextAlign::Left,
                    Style::new().flex_grow(1.0),
                );
            }
        },
    )
}

#[inline]
fn asset_card_container_style(
    rel_pos: Point,
    is_selected: bool,
    cat_color: Color,
    card_bg: Color,
    card_border: Color,
) -> Style {
    Style::new()
        .position_absolute()
        .left(rel_pos.x)
        .top(rel_pos.y)
        .width(ASSET_CARD_WIDTH)
        .height(ASSET_CARD_HEIGHT)
        .flex_col()
        .align_items(AlignItems::Center)
        .padding_insets(Insets::new(6.0, 6.0, 6.0, 6.0))
        .border_radius(6.0)
        .clip_children(true)
        .background(if is_selected {
            Color::rgba(0.0, 0.65, 0.85, 0.12)
        } else {
            card_bg
        })
        .border(
            if is_selected { 1.5 } else { 1.0 },
            if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else {
                card_border
            },
        )
        .hover_background(if is_selected {
            Color::rgba(0.0, 0.65, 0.85, 0.12)
        } else {
            Color::rgba(0.14, 0.16, 0.22, 0.90)
        })
        .hover_border(
            1.0,
            if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else {
                cat_color.with_alpha(0.60)
            },
        )
}

/// Emits an individual grid asset card.
///
/// Features category badge, VRAM resident status dot, thumbnail preview/atlas vector icon,
/// truncated display title, and human-readable metadata badge.
/// Dispatches drag, spawn (double click), and selection directly into the provided `actions` buffer,
/// returning [`WidgetResponse`] for declarative chaining (e.g. `.context_menu(...)`).
pub fn asset_grid_card(
    scope: &mut UiScope<'_>,
    item_idx: u32,
    item: &AssetItem,
    thumbnail_layer: Option<u32>,
    is_selected: bool,
    rel_pos: Point,
    actions: &mut Vec<AssetsPanelAction>,
) -> WidgetResponse {
    let card_bg = scope.theme().card_bg;
    let card_border = scope.theme().card_border;
    let resp = scope
        .container_tagged_resp(
            "AssetGridCard",
            asset_card_container_style(
                rel_pos,
                is_selected,
                resolve_category_color(item.category),
                card_bg,
                card_border,
            ),
            WidgetRole::Button,
            encode_item_tag(item_idx),
            |card| {
                // Header Row: Category Badge + Status Dot
                card.container_passive_named(
                    "CardHeader",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .width(ASSET_CARD_WIDTH - 12.0)
                        .height(14.0)
                        .margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    |hdr| {
                        // Category Badge Pill
                        hdr.container_passive_named(
                            "CardCatBadge",
                            Style::new()
                                .padding_insets(Insets::new(1.0, 4.0, 1.0, 4.0))
                                .border_radius(3.0)
                                .background(resolve_category_color(item.category).with_alpha(0.20)),
                            |b| {
                                b.label_styled_passive(
                                    "CardCatText",
                                    item.category.badge(),
                                    8.5,
                                    resolve_category_color(item.category),
                                    TextAlign::Center,
                                    Style::new(),
                                );
                            },
                        );

                        // Memory Status Dot (Cyan if loaded in RAM/VRAM)
                        if item.is_loaded_in_memory {
                            hdr.empty_box_passive_named(
                                "CardMemoryDot",
                                Style::new()
                                    .width(5.0)
                                    .height(5.0)
                                    .border_radius(2.5)
                                    .background(Color::rgba(0.0, 0.90, 1.0, 1.0)),
                            );
                        }
                    },
                );

                // Preview Box (54x54 px): Texture Thumbnail or Vector Atlas Icon
                card.container_passive_named(
                    "CardPreviewBox",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .width(54.0)
                        .height(54.0)
                        .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
                        .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.60))
                        .border_radius(4.0)
                        .margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    |box_scope| match thumbnail_layer {
                        Some(layer) => {
                            box_scope.icon([0.0, 0.0, 1.0, layer as f32], Color::WHITE, 54.0);
                        }
                        None => {
                            box_scope.icon_pair(resolve_category_icon(item.category), 28.0);
                        }
                    },
                );

                // Truncated Display Name
                card.label_styled_passive(
                    "CardName",
                    truncate_display_name(&item.name, 14, 11),
                    11.0,
                    Color::rgba(0.92, 0.94, 0.98, 1.0),
                    TextAlign::Center,
                    Style::new().width(ASSET_CARD_WIDTH - 8.0),
                );

                // Metadata / Size Label
                card.label_styled_passive(
                    "CardMetadata",
                    &item.metadata_badge,
                    9.5,
                    Color::rgba(0.50, 0.54, 0.64, 1.0),
                    TextAlign::Center,
                    Style::new().width(ASSET_CARD_WIDTH - 8.0),
                );
            },
        )
        .on_drag(|| actions.push(AssetsPanelAction::StartAssetDrag(item.clone())))
        .on_drag_release(|| actions.push(AssetsPanelAction::EndAssetDrag))
        .on_double_click(|| {
            actions.push(AssetsPanelAction::SpawnAsset(
                item.path.clone(),
                item.category,
            ))
        });

    if (resp.clicked() || resp.secondary_clicked() || (resp.pressed() && !resp.dragged()))
        && !resp.double_clicked()
    {
        actions.push(AssetsPanelAction::SelectAsset(Some(item.path.clone())));
    }

    resp
}

/// Emits an individual table list row with multi-column metrics and action buttons.
///
/// Dispatches quick actions (`+ Spawn`, `Inspect`), drag tracking, and row selection
/// directly into the provided `actions` buffer, returning the row container's [`WidgetResponse`]
/// for fluent declarative chaining (e.g. `.context_menu(...)`).
pub fn asset_list_row(
    scope: &mut UiScope<'_>,
    item_idx: u32,
    item: &AssetItem,
    is_selected: bool,
    actions: &mut Vec<AssetsPanelAction>,
) -> WidgetResponse {
    let resp = scope
        .container_tagged_resp(
            "AssetListRow",
            asset_list_row_style(is_selected),
            WidgetRole::Button,
            encode_item_tag(item_idx),
            |row| {
                // Category Icon (14px)
                row.icon_pair(resolve_category_icon(item.category), 14.0);

                row.spacing(6.0);

                // Name (flex grow)
                row.label_styled_passive(
                    "ListName",
                    &item.name,
                    11.5,
                    if is_selected {
                        Color::rgba(0.0, 0.90, 1.0, 1.0)
                    } else {
                        Color::rgba(0.90, 0.92, 0.96, 1.0)
                    },
                    TextAlign::Left,
                    Style::new().flex_grow(1.0),
                );

                // Category Badge
                row.container_passive_named(
                    "ListBadge",
                    Style::new()
                        .width(60.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                    |b| {
                        b.label_styled_passive(
                            "ListBadgeText",
                            item.category.badge(),
                            9.0,
                            resolve_category_color(item.category),
                            TextAlign::Center,
                            Style::new(),
                        );
                    },
                );

                // File Size / Metadata
                row.label_styled_passive(
                    "ListSize",
                    &item.metadata_badge,
                    10.5,
                    Color::rgba(0.60, 0.64, 0.74, 1.0),
                    TextAlign::Right,
                    Style::new().width(70.0),
                );

                // Status Indicator (VRAM / Memory)
                row.container_passive_named(
                    "ListStatus",
                    Style::new()
                        .width(64.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                    |st| {
                        st.label_styled_passive(
                            "ListStatusText",
                            if item.is_loaded_in_memory {
                                "Loaded"
                            } else {
                                "Disk"
                            },
                            9.5,
                            if item.is_loaded_in_memory {
                                Color::rgba(0.0, 0.90, 1.0, 1.0)
                            } else {
                                Color::rgba(0.50, 0.53, 0.62, 1.0)
                            },
                            TextAlign::Center,
                            Style::new(),
                        );
                    },
                );

                // Direct Action: Spawn Button ("+ Spawn")
                row.small_button_with_icon_tagged(
                    ICON_PLUS,
                    "Spawn",
                    encode_item_spawn_tag(item_idx),
                )
                .on_click(|| {
                    actions.push(AssetsPanelAction::SpawnAsset(
                        item.path.clone(),
                        item.category,
                    ));
                });

                row.spacing(4.0);

                // Direct Action: Inspect Button ("Inspect")
                row.add(
                    Button::new("Inspect")
                        .small()
                        .tag(encode_item_inspect_tag(item_idx)),
                )
                .on_click(|| {
                    actions.push(AssetsPanelAction::InspectAsset(item.clone()));
                });
            },
        )
        .on_drag(|| actions.push(AssetsPanelAction::StartAssetDrag(item.clone())))
        .on_drag_release(|| actions.push(AssetsPanelAction::EndAssetDrag));

    if (resp.clicked() || resp.secondary_clicked() || (resp.pressed() && !resp.dragged()))
        && !resp.double_clicked()
    {
        actions.push(AssetsPanelAction::SelectAsset(Some(item.path.clone())));
    }

    resp
}

#[inline]
fn asset_list_row_style(is_selected: bool) -> Style {
    Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(ASSET_LIST_ROW_HEIGHT)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .margin_insets(Insets::new(0.0, 0.0, 1.0, 0.0))
        .border_radius(4.0)
        .background(if is_selected {
            Color::rgba(0.0, 0.65, 0.85, 0.14)
        } else {
            Color::TRANSPARENT
        })
        .hover_background(if is_selected {
            Color::rgba(0.0, 0.65, 0.85, 0.14)
        } else {
            Color::rgba(0.14, 0.16, 0.22, 0.60)
        })
}

/// Emits an empty state notice when no matching assets are found.
///
/// Features large warm amber vector folder logo, title, and clear format guidance centered in the viewport.
pub fn asset_empty_notice(scope: &mut UiScope<'_>, vp_rect: Rect, query: &str) {
    scope.container_named(
        "EmptyAssetsCenterWrapper",
        Style::new()
            .flex_col()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(vp_rect.width)
            .height(vp_rect.height),
        |center_scope| {
            center_scope.container_named(
                "EmptyAssetsNotice",
                Style::new()
                    .flex_col()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(440.0)
                    .height(135.0)
                    .padding_insets(Insets::new(14.0, 20.0, 14.0, 20.0))
                    .background(center_scope.theme().card_bg)
                    .border(1.0, center_scope.theme().card_border)
                    .border_radius(6.0)
                    .clip_children(true),
                |notice| {
                    // Large Vector Folder Logo (Warm Amber Yellow: Layer 6 ICON_FOLDER)
                    notice.icon(ICON_FOLDER, Color::rgba(0.95, 0.76, 0.28, 1.0), 30.0);

                    notice.empty_box_passive_named("EmptyLogoGap", Style::new().height(8.0));

                    notice.label_styled_passive(
                        "EmptyTitle",
                        if query.is_empty() {
                            "No Assets Found in Active Directory".to_string()
                        } else {
                            format!("No assets matching \"{}\"", query)
                        },
                        12.5,
                        Color::WHITE,
                        TextAlign::Center,
                        Style::new(),
                    );

                    notice.empty_box_passive_named("EmptyTextGap", Style::new().height(6.0));

                    notice.label_styled_passive_wrapped(
                        "EmptySubtitle",
                        "Place 3D models (.gltf, .glb, .fbx), textures (.png), shaders (.wgsl), or scenes (.ae3d, .ae2d) into this folder.",
                        WrappedLabelDescriptor::new(
                            10.0,
                            Color::rgba(0.55, 0.60, 0.70, 1.0),
                            Style::new().width(400.0),
                        )
                        .align(TextAlign::Center),
                    );
                },
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::{AssetCategory, AssetItem, AssetSource};
    use std::path::PathBuf;

    #[test]
    fn test_asset_components_build_declaratively() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation must succeed");

        let item = AssetItem {
            name: "hero_model.gltf".to_string(),
            path: PathBuf::from("assets/models/hero_model.gltf"),
            relative_path: "models/hero_model.gltf".to_string(),
            category: AssetCategory::Models3D,
            source: AssetSource::Project,
            file_size_bytes: 4096,
            metadata_badge: "4.0 KB".to_string(),
            is_loaded_in_memory: true,
            model_handle: None,
            texture_handle: None,
            shader_handle: None,
            is_3d: true,
        };

        {
            let mut scope = UiScope::new(&mut tree, root_id);
            let mut actions = Vec::new();
            // 1. Breadcrumbs
            asset_breadcrumb_bar(&mut scope, Path::new("assets/models"), &mut actions);

            // 2. Tree Item
            let tree_params = FolderTreeItemParams {
                node_idx: 0,
                name: "models",
                depth: 1,
                has_children: true,
                is_expanded: true,
                is_selected: false,
            };
            asset_folder_tree_item(&mut scope, &tree_params);

            // 3. Grid Card
            asset_grid_card(
                &mut scope,
                0,
                &item,
                Some(32),
                false,
                Point::new(10.0, 10.0),
                &mut actions,
            );

            // 4. List Row
            asset_list_row(&mut scope, 0, &item, true, &mut actions);

            // 5. Empty Notice
            asset_empty_notice(&mut scope, Rect::new(0.0, 0.0, 600.0, 400.0), "search_term");
        }

        // Verify emitted tagged elements
        let has_breadcrumb = tree.iter().any(|(_, n)| n.tag == encode_breadcrumb_tag(0));
        let has_tree_row = tree.iter().any(|(_, n)| n.tag == encode_tree_row_tag(0));
        let has_tree_chev = tree
            .iter()
            .any(|(_, n)| n.tag == encode_tree_chevron_tag(0));
        let has_card = tree.iter().any(|(_, n)| n.tag == encode_item_tag(0));
        let has_spawn = tree.iter().any(|(_, n)| n.tag == encode_item_spawn_tag(0));
        let has_inspect = tree
            .iter()
            .any(|(_, n)| n.tag == encode_item_inspect_tag(0));

        assert!(has_breadcrumb, "Breadcrumb must have semantic tag");
        assert!(has_tree_row, "Tree row must have semantic tag");
        assert!(has_tree_chev, "Tree chevron must have semantic tag");
        assert!(has_card, "Asset card must have semantic tag");
        assert!(has_spawn, "List spawn button must have semantic tag");
        assert!(has_inspect, "List inspect button must have semantic tag");
    }
}