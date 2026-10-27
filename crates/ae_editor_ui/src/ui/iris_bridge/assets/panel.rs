// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Main Panel Orchestrator for Iris UI.
//!
//! Ties together top breadcrumb and action navigation, category filter chips,
//! left folder tree sidebar, scrollable card/table views, and telemetry footer
//! using a pure declarative Taffy flexbox layout hierarchy without manual pixel math.
use std::path::PathBuf;

use super::cards::build_asset_grid_cards_scope;
use super::chips::build_category_chips_scope;
use super::footer::build_asset_footer_scope;
use super::list::build_asset_list_table_scope;
use super::toolbar::build_asset_toolbar_scope;
use super::tree::build_folder_tree_sidebar_scope;
use super::types::{
    ASSETS_TAG_PANEL_ROOT, AssetsPanelAction, AssetsPanelLayoutMetrics, AssetsPanelParams,
};
use crate::assets::types::AssetViewMode;
use irisui::prelude::*;

pub use super::types::{ASSETS_CHIPS_BAR_HEIGHT, ASSETS_FOOTER_HEIGHT, ASSETS_TOP_BAR_HEIGHT};

/// Constructs the complete Content / Asset Browser panel widget hierarchy into the Iris `UiTree`.
///
/// Uses an idiomatic nested `UiScope` closure tree adhering strictly to modern Taffy flexbox
/// standards with zero imperative node allocations, zero target collections, and zero per-frame clones.
/// Gathers user interaction events into `actions` for Redux/Action-driven state updates.
///
/// Returns [`AssetsPanelLayoutMetrics`] describing the outer panel bounds, sidebar geometry,
/// asset content viewport, and optional floating context menu card bounds.
pub fn build_assets_panel(
    scope: &mut UiScope<'_>,
    params: &AssetsPanelParams<'_>,
    tree_state: &mut TreeState<PathBuf>,
    actions: &mut Vec<AssetsPanelAction>,
) -> AssetsPanelLayoutMetrics {
    let metrics = params.layout_metrics();

    scope
        .panel("AssetsPanelRoot")
        .tagged(ASSETS_TAG_PANEL_ROOT)
        .background_themed(|t| t.panel_bg)
        .bounds(params.panel_rect)
        .show(|panel_scope| {
            // 2. Top Header Toolbar (Pure Flexbox Breadcrumbs + Controls)
            build_asset_toolbar_scope(panel_scope, params, actions);

            // 3. Category Filter Chips Row (Fixed height, horizontal flex row)
            build_category_chips_scope(
                panel_scope,
                Rect::new(
                    params.panel_rect.x,
                    params.panel_rect.y + ASSETS_TOP_BAR_HEIGHT,
                    params.panel_rect.width,
                    ASSETS_CHIPS_BAR_HEIGHT,
                ),
                params,
                actions,
            );

            // 4. Middle Body (Pure Taffy flex_row with flex_grow(1.0) taking all available space)
            panel_scope.container_named(
                "AssetsPanelBody",
                Style::new()
                    .flex_row()
                    .flex_grow(1.0)
                    .width(params.panel_rect.width),
                |body_scope| {
                    if let Some(sb_rect) = metrics.sidebar_rect {
                        build_folder_tree_sidebar_scope(
                            body_scope, sb_rect, params, tree_state, actions,
                        );
                    }

                    let content_bg = body_scope.theme().panel_bg;
                    body_scope.container_named(
                        "AssetsContentArea",
                        Style::new()
                            .flex_col()
                            .flex_grow(1.0)
                            .background(content_bg),
                        |content_scope| match params.view_mode {
                            AssetViewMode::Grid => {
                                build_asset_grid_cards_scope(
                                    content_scope,
                                    metrics.content_viewport_rect,
                                    params,
                                    actions,
                                );
                            }
                            AssetViewMode::List => {
                                build_asset_list_table_scope(
                                    content_scope,
                                    metrics.content_viewport_rect,
                                    params,
                                    actions,
                                );
                            }
                        },
                    );
                },
            );

            // 5. Bottom Status Footer (Pure Flexbox Telemetry + Sidebar Toggle)
            build_asset_footer_scope(panel_scope, params, actions);
        });

    // 6. Right-Click Floating Context Menu is rendered on top-level UiTree (Layer 2)
    // to prevent clipping and preserve precise absolute screen coordinates.

    // 7. Interactive Quick Asset Preview Modal (Z-Order Highest)
    super::preview::build_asset_preview_modal(scope, params);

    metrics
}