// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Asset Browser UI Components
//!
//! Provides modular, reusable, 100% declarative UI widgets for the Content / Asset Browser panel
//! using [`UiScope`], including interactive breadcrumb bars, folder tree items, responsive
//! grid asset cards, and tabular list rows.
//!

use super::types::{
    encode_breadcrumb_tag, encode_item_inspect_tag, encode_item_spawn_tag, encode_item_tag,
    encode_tree_chevron_tag, encode_tree_row_tag, truncate_display_name,
};
use crate::assets::types::{AssetCategory, AssetItem};
use crate::ui::iris_bridge::icons::{
    ICON_AUDIO, ICON_CAMERA, ICON_CUBE, ICON_FOLDER, ICON_PLUS, ICON_SPARKLE, ICON_SPHERE,
    ICON_WORLD,
};
use irisui::prelude::*;
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
pub fn asset_breadcrumb_bar(
    scope: &mut UiScope<'_>,
    current_folder: &Path,
    hovered_tag: Option<u64>,
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

            bar.empty_box_passive_named(
                "BreadcrumbLogoSpacer",
                Style::new().width(4.0).height(1.0),
            );

            let mut has_segments = false;
            for (i, comp) in current_folder.components().enumerate() {
                let segment = comp.as_os_str().to_str().unwrap_or("");
                if segment.is_empty() {
                    continue;
                }
                has_segments = true;

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

                let tag = encode_breadcrumb_tag(i as u8);
                let is_hovered = hovered_tag == Some(tag);

                let text_color = if is_hovered {
                    Color::rgba(0.0, 0.90, 1.0, 1.0)
                } else {
                    Color::rgba(0.70, 0.74, 0.84, 1.0)
                };

                let bg_color = if is_hovered {
                    Color::rgba(0.18, 0.22, 0.30, 0.80)
                } else {
                    Color::TRANSPARENT
                };

                let style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .padding_insets(Insets::symmetric(0.0, 6.0))
                    .height(22.0)
                    .background(bg_color)
                    .border_radius(3.0);

                bar.container_tagged(
                    "BreadcrumbButton",
                    style,
                    WidgetRole::Button,
                    tag,
                    |crumb| {
                        crumb.label_styled_passive(
                            "BreadcrumbText",
                            segment,
                            11.5,
                            text_color,
                            TextAlign::Center,
                            Style::new(),
                        );
                    },
                );
            }

            if !has_segments {
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
    pub depth: usize,
    /// Whether this folder contains subdirectories.
    pub has_children: bool,
    /// Whether this folder branch is expanded.
    pub is_expanded: bool,
    /// Whether this folder is currently selected.
    pub is_selected: bool,
    /// Currently hovered semantic tag in the UI tree.
    pub hovered_tag: Option<u64>,
}

/// Emits an individual directory row in the folder tree sidebar.
pub fn asset_folder_tree_item(scope: &mut UiScope<'_>, params: &FolderTreeItemParams<'_>) {
    let row_tag = encode_tree_row_tag(params.node_idx);
    let chev_tag = encode_tree_chevron_tag(params.node_idx);

    let is_row_hovered = params.hovered_tag == Some(row_tag);
    let is_chev_hovered = params.hovered_tag == Some(chev_tag);

    let (bg_color, border_color, border_w) = if params.is_selected {
        (
            Color::rgba(0.08, 0.22, 0.32, 0.90),
            Color::rgba(0.0, 0.85, 1.0, 0.80),
            1.0,
        )
    } else if is_row_hovered {
        (Color::rgba(0.14, 0.16, 0.22, 0.70), Color::TRANSPARENT, 0.0)
    } else {
        (Color::TRANSPARENT, Color::TRANSPARENT, 0.0)
    };

    let text_color = if params.is_selected {
        Color::WHITE
    } else if is_row_hovered {
        Color::rgba(0.90, 0.92, 0.96, 1.0)
    } else {
        Color::rgba(0.75, 0.78, 0.85, 1.0)
    };

    let folder_tint = if params.is_selected {
        Color::rgba(0.0, 0.90, 1.0, 1.0) // Cyan when selected
    } else {
        Color::rgba(0.95, 0.76, 0.28, 1.0) // Warm folder amber
    };

    let chev_col = if params.is_selected || is_chev_hovered {
        Color::rgba(0.0, 0.90, 1.0, 1.0)
    } else {
        Color::rgba(0.60, 0.65, 0.75, 1.0)
    };

    scope.container_tagged(
        "FolderTreeRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(22.0)
            .margin_insets(Insets::new(1.0, 4.0, 1.0, 4.0))
            .border_radius(4.0)
            .border(border_w, border_color)
            .background(bg_color),
        WidgetRole::Button,
        row_tag,
        |row| {
            // Indent spacing based on hierarchy tree depth
            let indent_w = params.depth as f32 * 14.0 + 4.0;
            if indent_w > 0.0 {
                row.empty_box_passive_named("TreeIndent", Style::new().width(indent_w).height(1.0));
            }

            // Expand / Collapse Chevron Button
            if params.has_children {
                let chev_icon = if params.is_expanded { "▾" } else { "▸" };

                row.container_tagged(
                    "TreeChevronBtn",
                    Style::new()
                        .width(14.0)
                        .height(18.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                    WidgetRole::Button,
                    chev_tag,
                    |c| {
                        c.label_styled_passive(
                            "TreeChevronIcon",
                            chev_icon,
                            11.0,
                            chev_col,
                            TextAlign::Center,
                            Style::new(),
                        );
                    },
                );
            } else {
                row.empty_box_passive_named(
                    "TreeNoChevronSpacer",
                    Style::new().width(14.0).height(1.0),
                );
            }

            // Folder Atlas Icon (Layer 6: ICON_FOLDER)
            row.icon(ICON_FOLDER, folder_tint, 16.0);

            row.empty_box_passive_named("TreeFolderGap", Style::new().width(6.0).height(1.0));

            // Folder Name Label
            row.label_styled_passive(
                "TreeFolderName",
                params.name,
                11.5,
                text_color,
                TextAlign::Left,
                Style::new().flex_grow(1.0),
            );
        },
    );
}

/// Emits an individual grid asset card.
///
/// Features category badge, VRAM resident status dot, thumbnail preview/atlas vector icon,
/// truncated display title, and human-readable metadata badge.
pub fn asset_grid_card(
    scope: &mut UiScope<'_>,
    item_idx: u32,
    item: &AssetItem,
    thumbnail_layer: Option<u32>,
    is_selected: bool,
    hovered_tag: Option<u64>,
    rel_pos: Point,
) {
    let tag = encode_item_tag(item_idx);
    let is_hovered = hovered_tag == Some(tag);
    let cat_color = resolve_category_color(item.category);

    let border_color = if is_selected {
        Color::rgba(0.0, 0.90, 1.0, 1.0)
    } else if is_hovered {
        cat_color.with_alpha(0.60)
    } else {
        Color::rgba(0.18, 0.20, 0.26, 0.75)
    };

    let border_w = if is_selected { 1.5 } else { 1.0 };

    let bg_color = if is_selected {
        Color::rgba(0.0, 0.65, 0.85, 0.12)
    } else if is_hovered {
        Color::rgba(0.14, 0.16, 0.22, 0.90)
    } else {
        Color::rgba(0.08, 0.09, 0.12, 0.90)
    };

    scope.container_tagged(
        "AssetGridCard",
        Style::new()
            .position_absolute()
            .left(rel_pos.x)
            .top(rel_pos.y)
            .width(ASSET_CARD_WIDTH)
            .height(ASSET_CARD_HEIGHT)
            .flex_col()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(6.0, 6.0, 6.0, 6.0))
            .background(bg_color)
            .border(border_w, border_color)
            .border_radius(6.0)
            .clip_children(true),
        WidgetRole::Button,
        tag,
        |card| {
            // Header Row: Category Badge + Status Dot
            card.container_named(
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
                    hdr.container_named(
                        "CardCatBadge",
                        Style::new()
                            .padding_insets(Insets::new(1.0, 4.0, 1.0, 4.0))
                            .border_radius(3.0)
                            .background(cat_color.with_alpha(0.20)),
                        |b| {
                            b.label_styled_passive(
                                "CardCatText",
                                item.category.badge(),
                                8.5,
                                cat_color,
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
            card.container_named(
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
                |box_scope| {
                    if let Some(layer) = thumbnail_layer {
                        // Render hardware texture layer thumbnail
                        box_scope.icon([0.0, 0.0, 1.0, layer as f32], Color::WHITE, 54.0);
                    } else {
                        // Fallback to canonical category vector icon
                        let (icon_uv, icon_tint) = resolve_category_icon(item.category);
                        box_scope.icon(icon_uv, icon_tint, 28.0);
                    }
                },
            );

            // Truncated Display Name
            let display_name = truncate_display_name(&item.name, 14, 11);
            card.label_styled_passive(
                "CardName",
                display_name,
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
    );
}

/// Emits an individual table list row with multi-column metrics and action buttons.
pub fn asset_list_row(
    scope: &mut UiScope<'_>,
    item_idx: u32,
    item: &AssetItem,
    is_selected: bool,
    hovered_tag: Option<u64>,
) {
    let row_tag = encode_item_tag(item_idx);
    let spawn_tag = encode_item_spawn_tag(item_idx);
    let inspect_tag = encode_item_inspect_tag(item_idx);

    let is_row_hovered = hovered_tag == Some(row_tag);
    let is_spawn_hovered = hovered_tag == Some(spawn_tag);
    let is_inspect_hovered = hovered_tag == Some(inspect_tag);

    let cat_color = resolve_category_color(item.category);
    let (icon_uv, icon_tint) = resolve_category_icon(item.category);

    let bg_color = if is_selected {
        Color::rgba(0.0, 0.65, 0.85, 0.14)
    } else if is_row_hovered {
        Color::rgba(0.14, 0.16, 0.22, 0.60)
    } else {
        Color::TRANSPARENT
    };

    scope.container_tagged(
        "AssetListRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(ASSET_LIST_ROW_HEIGHT)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .margin_insets(Insets::new(0.0, 0.0, 1.0, 0.0))
            .border_radius(4.0)
            .background(bg_color),
        WidgetRole::Button,
        row_tag,
        |row| {
            // Category Icon (14px)
            row.icon(icon_uv, icon_tint, 14.0);

            row.empty_box_passive_named("ListIconGap", Style::new().width(6.0).height(1.0));

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
            row.container_named(
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
                        cat_color,
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
            row.container_named(
                "ListStatus",
                Style::new()
                    .width(64.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
                |st| {
                    let (status_text, status_col) = if item.is_loaded_in_memory {
                        ("Loaded", Color::rgba(0.0, 0.90, 1.0, 1.0))
                    } else {
                        ("Disk", Color::rgba(0.50, 0.53, 0.62, 1.0))
                    };
                    st.label_styled_passive(
                        "ListStatusText",
                        status_text,
                        9.5,
                        status_col,
                        TextAlign::Center,
                        Style::new(),
                    );
                },
            );

            // Direct Action: Spawn Button ("+ Spawn")
            let spawn_bg = if is_spawn_hovered {
                Color::rgba(0.0, 0.70, 0.90, 0.25)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.80)
            };
            row.container_tagged(
                "ListSpawnBtn",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(20.0)
                    .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
                    .margin_insets(Insets::new(0.0, 4.0, 0.0, 0.0))
                    .border_radius(3.0)
                    .background(spawn_bg),
                WidgetRole::Button,
                spawn_tag,
                |btn| {
                    btn.icon(ICON_PLUS, Color::rgba(0.0, 0.90, 1.0, 1.0), 9.0);
                    btn.empty_box_passive_named("SpawnGap", Style::new().width(3.0).height(1.0));
                    btn.label_styled_passive(
                        "ListSpawnText",
                        "Spawn",
                        9.5,
                        Color::rgba(0.85, 0.92, 1.0, 1.0),
                        TextAlign::Center,
                        Style::new(),
                    );
                },
            );

            // Direct Action: Inspect Button ("Inspect")
            let inspect_bg = if is_inspect_hovered {
                Color::rgba(0.25, 0.28, 0.38, 0.60)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.80)
            };
            row.container_tagged(
                "ListInspectBtn",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(20.0)
                    .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
                    .margin_insets(Insets::new(0.0, 4.0, 0.0, 0.0))
                    .border_radius(3.0)
                    .background(inspect_bg),
                WidgetRole::Button,
                inspect_tag,
                |btn| {
                    btn.label_styled_passive(
                        "ListInspectText",
                        "Inspect",
                        9.5,
                        Color::rgba(0.80, 0.84, 0.92, 1.0),
                        TextAlign::Center,
                        Style::new(),
                    );
                },
            );
        },
    );
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
                    .background(Color::rgba(0.07, 0.08, 0.11, 0.95))
                    .border(1.0, Color::rgba(0.18, 0.22, 0.30, 0.60))
                    .border_radius(6.0)
                    .clip_children(true),
                |notice| {
                    // Large Vector Folder Logo (Warm Amber Yellow: Layer 6 ICON_FOLDER)
                    notice.icon(ICON_FOLDER, Color::rgba(0.95, 0.76, 0.28, 1.0), 30.0);

                    notice.empty_box_passive_named("EmptyLogoGap", Style::new().height(8.0));

                    let title_text = if !query.is_empty() {
                        format!("No assets matching \"{}\"", query)
                    } else {
                        "No Assets Found in Active Directory".to_string()
                    };

                    notice.label_styled_passive(
                        "EmptyTitle",
                        title_text,
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
            // 1. Breadcrumbs
            asset_breadcrumb_bar(&mut scope, Path::new("assets/models"), None);

            // 2. Tree Item
            let tree_params = FolderTreeItemParams {
                node_idx: 0,
                name: "models",
                depth: 1,
                has_children: true,
                is_expanded: true,
                is_selected: false,
                hovered_tag: None,
            };
            asset_folder_tree_item(&mut scope, &tree_params);

            // 3. Grid Card
            asset_grid_card(
                &mut scope,
                0,
                &item,
                Some(32),
                false,
                None,
                Point::new(10.0, 10.0),
            );

            // 4. List Row
            asset_list_row(&mut scope, 0, &item, true, None);

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