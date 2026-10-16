// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Overlays Declarative Card Builder
//!
//! Renders the wireframe mode and grid visibility interactive checkboxes
//! purely using [`UiScope`] declarative primitives.
//!

use super::types::{
    STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME, StatsPanelAction, StatsPanelParams,
};
use irisui::prelude::*;

/// Builds the Viewport Overlays section inside the declarative stats card.
///
/// Emits semantic interactive toggle controls tagged with [`STATS_TAG_TOGGLE_WIREFRAME`]
/// and [`STATS_TAG_TOGGLE_GRID`] and enqueues emitted actions directly on click.
pub fn build_viewport_overlays_content(
    scope: &mut UiScope<'_>,
    params: &StatsPanelParams<'_>,
    actions: &mut Vec<StatsPanelAction>,
) {
    let row_style = Style::new().flex_col().gap(6.0);

    scope.container(row_style, |s| {
        // 1. Wireframe Mode Checkbox
        if s.add(
            Checkbox::new("🕸 Wireframe Mode (Edges)", params.wireframe_enabled)
                .tag(STATS_TAG_TOGGLE_WIREFRAME),
        )
        .clicked()
        {
            actions.push(StatsPanelAction::ToggleWireframe);
        }

        // 2. Show Grid Checkbox
        if s.add(Checkbox::new("🔲 Show Grid", params.grid_enabled).tag(STATS_TAG_TOGGLE_GRID))
            .clicked()
        {
            actions.push(StatsPanelAction::ToggleGrid);
        }
    });
}