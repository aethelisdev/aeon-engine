// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Main Panel Orchestrator for Iris UI.
//!
//! Ties together top breadcrumb and action navigation, category filter chips,
//! left folder tree sidebar, scrollable card/table views, and telemetry footer
//! using a pure declarative Taffy flexbox layout hierarchy without manual pixel math.

use super::cards::build_asset_grid_cards_scope;
use super::chips::build_category_chips_scope;
use super::footer::build_asset_footer_scope;
use super::list::build_asset_list_table_scope;
use super::toolbar::build_asset_toolbar_scope;
use super::tree::build_folder_tree_sidebar_scope;
pub use super::types::{ASSETS_CHIPS_BAR_HEIGHT, ASSETS_FOOTER_HEIGHT, ASSETS_TOP_BAR_HEIGHT};
use super::types::{
    ASSETS_TAG_PANEL_ROOT, AssetsPanelAction, AssetsPanelLayoutMetrics, AssetsPanelParams,
};
use crate::assets::types::AssetViewMode;
use irisui::prelude::*;
use std::path::PathBuf;

/// Constructs the complete Content / Asset Browser panel widget hierarchy into the Iris `UiTree`.
pub fn build_assets_panel(
    scope: &mut UiScope<'_>,
    params: &AssetsPanelParams<'_>,
    tree_state: &mut TreeState<PathBuf>,
    actions: &mut Vec<AssetsPanelAction>,
) -> AssetsPanelLayoutMetrics {
    scope
        .panel("AssetsPanelRoot")
        .tagged(ASSETS_TAG_PANEL_ROOT)
        .background_themed(|t| t.panel_bg)
        .bounds(params.panel_rect)
        .show(|panel_scope| {
            // 1. Top Header Toolbar
            build_asset_toolbar_scope(panel_scope, params, actions);

            // 2. Category Filter Chips Row
            build_category_chips_scope(panel_scope, params, actions);

            // 3. Middle Body
            panel_scope.container_named(
                "AssetsPanelBody",
                Style::new()
                    .flex_row()
                    .flex_grow(1.0)
                    .width(params.panel_rect.width),
                |body_scope| {
                    params
                        .layout_metrics()
                        .sidebar_rect
                        .into_iter()
                        .for_each(|sb_rect| {
                            build_folder_tree_sidebar_scope(
                                body_scope, sb_rect, params, tree_state, actions,
                            );
                        });

                    body_scope
                        .drop_target_styled(
                            "AssetsContentArea",
                            Style::new()
                                .flex_col()
                                .flex_grow(1.0)
                                .background(body_scope.theme().panel_bg),
                            |target_scope, is_hovered| {
                                if is_hovered {
                                    target_scope.container_named(
                                        "AssetDropIndicator",
                                        Style::new()
                                            .width(
                                                params.layout_metrics().content_viewport_rect.width,
                                            )
                                            .height(28.0)
                                            .background(
                                                target_scope.theme().accent.with_alpha(0.20),
                                            )
                                            .border(1.0, target_scope.theme().accent)
                                            .align_items(AlignItems::Center)
                                            .justify_content(JustifyContent::Center),
                                        |indicator| {
                                            indicator
                                                .label("📥 Drop files here to import into project");
                                        },
                                    );
                                }

                                match params.view_mode {
                                    AssetViewMode::Grid => {
                                        build_asset_grid_cards_scope(
                                            target_scope,
                                            params.layout_metrics().content_viewport_rect,
                                            params,
                                            actions,
                                        );
                                    }
                                    AssetViewMode::List => {
                                        build_asset_list_table_scope(
                                            target_scope,
                                            params.layout_metrics().content_viewport_rect,
                                            params,
                                            actions,
                                        );
                                    }
                                }
                            },
                        )
                        .on_drop(|paths| {
                            actions.push(AssetsPanelAction::ImportDroppedFiles(paths.to_vec()));
                        });
                },
            );

            // 4. Bottom Status Footer
            build_asset_footer_scope(panel_scope, params, actions);
        });

    // 5. Interactive Quick Asset Preview Modal (Z-Order Highest)
    super::preview::build_asset_preview_modal(scope, params);
    if params.active_preview_modal.is_some()
        && (scope.is_tag_clicked(irisui::widgets::modal::MODAL_TAG_CLOSE)
            || scope.is_tag_clicked(irisui::widgets::modal::MODAL_TAG_SCRIM))
    {
        scope.close_modal(super::types::ASSETS_TAG_PREVIEW_MODAL);
        actions.push(AssetsPanelAction::CloseInspectModal);
    }

    params.layout_metrics()
}