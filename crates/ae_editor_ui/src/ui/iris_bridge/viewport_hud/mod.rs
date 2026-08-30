// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Viewport HUD Subsystem
//!
//! Provides the root orchestration entry points for building 3D Viewport Toolbar,
//! Orientation Compass, Camera HUD, Billboard Icons, and Play Mode overlays in Iris UI.

pub mod billboards;
pub mod camera_hud;
pub mod compass;
pub mod play_hud;
pub mod popup;
pub mod toolbar;
pub mod types;

pub use types::{ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets};

use irisui::prelude::*;

/// Builds the complete Viewport HUD overlay hierarchy into the UI Tree.
pub fn build_viewport_hud(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    if params.viewport_rect.width < 20.0 || params.viewport_rect.height < 20.0 {
        return;
    }

    if params.is_editing {
        // 1. Top-left floating toolbar
        toolbar::build_viewport_toolbar(tree, parent_id, params, targets);

        // 2. Top-right 3D Scene Navigation Compass
        compass::build_scene_navigation_compass(tree, parent_id, params, targets);

        // 3. Bottom-right Camera Info HUD
        camera_hud::build_camera_hud(tree, parent_id, params);

        // 4. 3D projected billboard icons
        billboards::build_billboard_icons(tree, parent_id, params, targets);

        // 5. Active dropdown popup if open
        if let Some(active_dd) = params.active_dropdown {
            popup::render_viewport_hud_dropdown_popup(tree, parent_id, active_dd, params, targets);
        }
    } else {
        // Play Mode HUD
        play_hud::build_play_hud(tree, parent_id, params);
    }
}