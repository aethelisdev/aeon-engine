// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Main Panel Orchestrator for Iris UI.
//!
//! Ties together top breadcrumb and action navigation, category filter chips,
//! left folder tree sidebar, scrollable card/table views, and telemetry footer
//! using a pure declarative Taffy flexbox layout hierarchy without manual pixel math.
//!

use super::cards::build_asset_grid_cards_scope;
use super::chips::build_category_chips_scope;
use super::footer::build_asset_footer_scope;
use super::list::build_asset_list_table_scope;
use super::toolbar::build_asset_toolbar_scope;
use super::tree::build_folder_tree_sidebar_scope;
use super::types::{ASSETS_TAG_PANEL_ROOT, AssetsPanelLayoutMetrics, AssetsPanelParams};
use crate::assets::types::AssetViewMode;
use crate::ui::iris_bridge::theme::*;
use irisui::prelude::*;

/// Height of the top navigation header toolbar in physical pixels.
pub const ASSETS_TOP_BAR_HEIGHT: f32 = 34.0;

/// Height of the category filter chips toolbar in physical pixels.
pub const ASSETS_CHIPS_BAR_HEIGHT: f32 = 28.0;

/// Height of the bottom telemetry status footer in physical pixels.
pub const ASSETS_FOOTER_HEIGHT: f32 = 24.0;

/// Constructs the complete Content / Asset Browser panel widget hierarchy into the Iris `UiTree`.
///
/// Uses an idiomatic nested `UiScope` closure tree adhering strictly to modern Taffy flexbox
/// standards with zero imperative node allocations, zero target collections, and zero per-frame clones.
///
/// Returns [`AssetsPanelLayoutMetrics`] describing the outer panel bounds, sidebar geometry,
/// asset content viewport, and optional floating context menu card bounds.
pub fn build_assets_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AssetsPanelParams<'_>,
) -> AssetsPanelLayoutMetrics {
    let mut resolved_sidebar_rect = None;
    let mut resolved_content_rect = Rect::default();

    // 1. Panel Root Container (Taffy flex_col layout with clipping)
    let mut root_scope = UiScope::new(tree, parent_id);
    let _root_node_id = root_scope.panel_tagged(
        "AssetsPanelRoot",
        params.panel_rect,
        ASSETS_TAG_PANEL_ROOT,
        ELEVATION_1_PANEL,
        |panel_scope| {
            // 2. Top Header Toolbar (Pure Flexbox Breadcrumbs + Controls)
            build_asset_toolbar_scope(panel_scope, params);

            // 3. Category Filter Chips Row (Fixed height, horizontal flex row)
            let chips_rect = Rect::new(
                params.panel_rect.x,
                params.panel_rect.y + ASSETS_TOP_BAR_HEIGHT,
                params.panel_rect.width,
                ASSETS_CHIPS_BAR_HEIGHT,
            );
            build_category_chips_scope(panel_scope, chips_rect, params);

            // 4. Middle Body (Pure Taffy flex_row with flex_grow(1.0) taking all available space)
            panel_scope.container_named(
                "AssetsPanelBody",
                Style::new()
                    .flex_row()
                    .flex_grow(1.0)
                    .width(params.panel_rect.width)
                    .clip_children(true),
                |body_scope| {
                    let body_h = (params.panel_rect.height
                        - ASSETS_TOP_BAR_HEIGHT
                        - ASSETS_CHIPS_BAR_HEIGHT
                        - ASSETS_FOOTER_HEIGHT)
                        .max(40.0);

                    // Left Side: Folder Tree Sidebar
                    if !params.sidebar_collapsed {
                        let sb_rect = Rect::new(
                            params.panel_rect.x,
                            params.panel_rect.y + ASSETS_TOP_BAR_HEIGHT + ASSETS_CHIPS_BAR_HEIGHT,
                            params.sidebar_width,
                            body_h,
                        );
                        resolved_sidebar_rect = Some(sb_rect);
                        build_folder_tree_sidebar_scope(body_scope, sb_rect, params);
                    }

                    // Right Side: Content Viewport (flex_col with flex_grow(1.0))
                    let content_w = if !params.sidebar_collapsed {
                        (params.panel_rect.width - params.sidebar_width).max(60.0)
                    } else {
                        params.panel_rect.width
                    };
                    let content_x = if !params.sidebar_collapsed {
                        params.panel_rect.x + params.sidebar_width
                    } else {
                        params.panel_rect.x
                    };
                    let content_rect = Rect::new(
                        content_x,
                        params.panel_rect.y + ASSETS_TOP_BAR_HEIGHT + ASSETS_CHIPS_BAR_HEIGHT,
                        content_w,
                        body_h,
                    );
                    resolved_content_rect = content_rect;

                    body_scope.container_tagged(
                        "AssetsContentViewport",
                        Style::new()
                            .flex_col()
                            .flex_grow(1.0)
                            .background(Color::rgba(0.04, 0.05, 0.07, 0.98))
                            .clip_children(true),
                        WidgetRole::Default,
                        super::types::ASSETS_TAG_CONTENT_VIEWPORT,
                        |content_scope| match params.view_mode {
                            AssetViewMode::Grid => {
                                build_asset_grid_cards_scope(content_scope, content_rect, params);
                            }
                            AssetViewMode::List => {
                                build_asset_list_table_scope(content_scope, content_rect, params);
                            }
                        },
                    );
                },
            );

            // 5. Bottom Status Footer (Pure Flexbox Telemetry + Sidebar Toggle)
            build_asset_footer_scope(panel_scope, params);
        },
    );

    // 6. Right-Click Floating Context Menu is rendered on top-level OverlayTree (Layer 2)
    // to prevent clipping and preserve precise absolute screen coordinates.

    // 7. Interactive Quick Asset Preview Modal (Z-Order Highest)
    super::preview::build_asset_preview_modal(tree, parent_id, params);

    AssetsPanelLayoutMetrics {
        panel_rect: params.panel_rect,
        sidebar_rect: resolved_sidebar_rect,
        content_viewport_rect: resolved_content_rect,
        context_menu_card_rect: None,
    }
}