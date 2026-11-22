// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Overlays Declarative Card Builder
//!
//! Renders the wireframe mode and grid visibility interactive checkboxes
//! purely using [`UiScope`] and [`ActionBus<EngineUiAction>`].

use super::types::{STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Builds the Viewport Overlays section inside the declarative stats card.
///
/// Emits interactive toggle controls tagged with [`STATS_TAG_TOGGLE_WIREFRAME`]
/// and [`STATS_TAG_TOGGLE_GRID`] and dispatches directly into [`ActionBus`].
pub fn build_viewport_overlays_content(
    scope: &mut UiScope<'_>,
    wireframe_enabled: bool,
    grid_enabled: bool,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.container(Style::new().flex_col().gap(6.0), |s| {
        s.add(
            Checkbox::new("🕸 Wireframe Mode (Edges)", wireframe_enabled)
                .tag(STATS_TAG_TOGGLE_WIREFRAME),
        )
        .on_click(|| action_bus.emit(EngineUiAction::SetWireframe(!wireframe_enabled)));

        s.add(Checkbox::new("🔲 Show Grid", grid_enabled).tag(STATS_TAG_TOGGLE_GRID))
            .on_click(|| {
                action_bus.emit(EngineUiAction::SetGridEnabled(!grid_enabled));
            });
    });
}