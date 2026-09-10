// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Root builder orchestrating toolbar, compass, camera HUD, and billboard overlays.
//!

use super::billboards;
use super::camera_hud;
use super::compass;
use super::play_hud;
use super::popup;
use super::toolbar;
use super::types::{ViewportHudParams, ViewportHudTargets};
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

        // In 2D dimension mode, 3D compass and 3D camera Euler angle telemetry are hidden
        if !params.is_2d {
            // 2. Top-right 3D Scene Navigation Compass (visible across all 3D projection modes)
            compass::build_scene_navigation_compass(tree, parent_id, params, targets);

            // 3. Bottom-right Camera Info HUD
            camera_hud::build_camera_hud(tree, parent_id, params);

            // 4. 3D projected billboard icons
            billboards::build_billboard_icons(tree, parent_id, params, targets);
        }

        // 5. Active dropdown popup if open
        if let Some(active_dd) = params.active_dropdown {
            popup::render_viewport_hud_dropdown_popup(tree, parent_id, active_dd, params, targets);
        }
    } else {
        // Play Mode HUD & In-Game Pause Menu Overlay
        play_hud::build_play_hud(tree, parent_id, params, targets);
    }
}