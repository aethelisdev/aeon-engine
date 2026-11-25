// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Overlays Declarative Card Builder
//!
//! Renders the wireframe mode and grid visibility interactive checkboxes
//! purely using [`UiScope`] and [`ActionBus<EngineUiAction>`].

use super::types::{STATS_TAG_TOGGLE_COLLIDERS, STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Boolean toggles rendered inside the Viewport Overlays telemetry card.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ViewportOverlaySettings {
    /// Whether triangle wireframe shading mode is active in the 3D viewport.
    pub wireframe_enabled: bool,
    /// Whether the world floor grid overlay is drawn in 3D Edit mode.
    pub grid_enabled: bool,
    /// Whether all scene collider wireframes are shown (`false` draws selected entities only).
    pub show_all_colliders: bool,
}

/// Builds the Viewport Overlays section inside the declarative stats card.
///
/// Emits interactive toggle controls tagged with [`STATS_TAG_TOGGLE_WIREFRAME`],
/// [`STATS_TAG_TOGGLE_GRID`], and [`STATS_TAG_TOGGLE_COLLIDERS`] directly into [`ActionBus`].
pub fn build_viewport_overlays_content(
    scope: &mut UiScope<'_>,
    overlays: ViewportOverlaySettings,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.container(Style::new().flex_col().gap(6.0), |s| {
        s.add(
            Checkbox::new("🕸 Wireframe Mode (Edges)", overlays.wireframe_enabled)
                .tag(STATS_TAG_TOGGLE_WIREFRAME),
        )
        .on_click(|| {
            action_bus.emit(EngineUiAction::SetWireframe(!overlays.wireframe_enabled));
        });

        s.add(Checkbox::new("🔲 Show Grid", overlays.grid_enabled).tag(STATS_TAG_TOGGLE_GRID))
            .on_click(|| {
                action_bus.emit(EngineUiAction::SetGridEnabled(!overlays.grid_enabled));
            });

        s.add(
            Checkbox::new("📦 Show All Colliders", overlays.show_all_colliders)
                .tag(STATS_TAG_TOGGLE_COLLIDERS),
        )
        .on_click(|| {
            action_bus.emit(EngineUiAction::SetShowAllColliders(
                !overlays.show_all_colliders,
            ));
        });
    });
}