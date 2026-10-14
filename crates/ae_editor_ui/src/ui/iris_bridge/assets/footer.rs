// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bottom Status Bar & Telemetry Footer for the Iris UI Content / Asset Browser.
//!
//! Provides a declarative Taffy flexbox footer bar featuring:
//! - Interactive sidebar toggle button ("◀" / "▶") with semantic tag `ASSETS_TAG_TOGGLE_SIDEBAR`
//! - Current active folder path indicator paired with the canonical vector folder icon (`ICON_FOLDER`)
//! - Real-time zero-allocation asset telemetry (items in scope, total count, VRAM loaded count, aggregate disk size)
//!

use super::panel::ASSETS_FOOTER_HEIGHT;
use super::types::{ASSETS_TAG_TOGGLE_SIDEBAR, AssetsPanelAction, AssetsPanelParams};
use crate::assets::types::AssetBrowserState;
use crate::ui::iris_bridge::icons::ICON_FOLDER;
use irisui::prelude::*;

/// Declarative construction of the Asset Browser bottom status bar and telemetry footer.
pub fn build_asset_footer_scope(
    scope: &mut UiScope<'_>,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    scope.container_named(
        "AssetsFooter",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding_insets(Insets::symmetric(0.0, 8.0))
            .height(ASSETS_FOOTER_HEIGHT)
            .background(Color::rgba(0.06, 0.07, 0.09, 0.98))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.70)),
        |footer| {
            // 1. Left Group: Sidebar Toggle + Folder Icon + Folder Path
            footer.container_named(
                "FooterLeftGroup",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(8.0),
                |left| {
                    left.button_named_styled_tagged(
                        "SidebarToggleButton",
                        if params.sidebar_collapsed {
                            "▶"
                        } else {
                            "◀"
                        },
                        sidebar_toggle_button_style(),
                        ASSETS_TAG_TOGGLE_SIDEBAR,
                    )
                    .on_click(|| {
                        actions.push(AssetsPanelAction::ToggleSidebar);
                    });

                    // Folder Logo (`ICON_FOLDER`) + Path
                    left.icon_named(
                        "FooterFolderLogo",
                        ICON_FOLDER,
                        Color::rgba(0.95, 0.76, 0.28, 0.80),
                        14.0,
                    );

                    left.label_styled_passive(
                        "FooterFolderPath",
                        params.current_folder.display().to_string(),
                        10.5,
                        Color::rgba(0.55, 0.60, 0.70, 1.0),
                        TextAlign::Left,
                        Style::new().width(240.0),
                    );
                },
            );

            // 2. Right Group: Real-time Telemetry (Single-pass zero-allocation iterator)
            footer.label_styled_passive(
                "FooterTelemetry",
                format_footer_telemetry(params),
                10.5,
                Color::rgba(0.50, 0.54, 0.64, 1.0),
                TextAlign::Right,
                Style::new(),
            );
        },
    );
}

/// Computes the visual style for the sidebar collapse/expand toggle button.
#[inline]
fn sidebar_toggle_button_style() -> Style {
    Style::new()
        .width(26.0)
        .height(20.0)
        .background(Color::rgba(0.12, 0.14, 0.18, 0.80))
        .hover_background(Color::rgba(0.20, 0.24, 0.32, 1.0))
        .border_radius(3.0)
}

/// Computes the telemetry summary text for the asset browser footer.
///
/// Single-pass fold iterator referencing pre-cached item buffers with zero heap allocations.
#[inline]
fn format_footer_telemetry(params: &AssetsPanelParams<'_>) -> String {
    let (total_count, in_memory_count, total_size) = params
        .cached_items
        .iter()
        .filter(|item| {
            params.show_engine_content || item.source != crate::assets::types::AssetSource::Engine
        })
        .fold((0usize, 0usize, 0u64), |(count, mem, size), item| {
            (
                count + 1,
                mem + usize::from(item.is_loaded_in_memory),
                size + item.file_size_bytes,
            )
        });

    format!(
        "{} Items in Scope ({} Total, {} in VRAM)  •  Disk: {}",
        params.filtered_items.len(),
        total_count,
        in_memory_count,
        AssetBrowserState::format_file_size(total_size)
    )
}