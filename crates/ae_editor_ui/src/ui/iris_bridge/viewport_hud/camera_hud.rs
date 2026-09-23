// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Camera Info HUD Builder
//!
//! Renders the bottom-right camera position and orientation angles overlay badge using
//! declarative [`UiScope`] and Taffy absolute layout positioning.
//!

use super::types::ViewportHudParams;
use irisui::prelude::*;

/// Builds the camera position & rotation angles HUD badge at the bottom-right of the viewport.
///
/// Configured using [`Style::position_absolute`], [`Style::right`], and [`Style::bottom`]
/// without manual coordinate calculations.
pub fn build_camera_hud(tree: &mut UiTree, parent_id: WidgetId, params: &ViewportHudParams<'_>) {
    let p = params.camera.position;
    let pos_text = format!("Pos: {:.1}, {:.1}, {:.1}", p.x, p.y, p.z);
    let pitch_deg = params.camera.pitch.0.to_degrees();
    let yaw_deg = params.camera.yaw.0.to_degrees();
    let rot_text = format!("Rot: {:.0}°, {:.0}°", pitch_deg, yaw_deg);
    let full_text = format!("{}   |   {}", pos_text, rot_text);

    let hud_w = 210.0;
    let hud_h = 22.0;

    let pill_style = Style::new()
        .position_absolute()
        .right(8.0)
        .bottom(8.0)
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(hud_w)
        .height(hud_h)
        .background(Color::rgba(0.07, 0.08, 0.11, 0.75))
        .border(1.0, Color::rgba(0.24, 0.28, 0.38, 0.50))
        .border_radius(4.0);

    let mut scope = UiScope::new(tree, parent_id);
    scope.container_named("CameraHudPill", pill_style, |pill| {
        pill.label(
            full_text,
            10.0,
            Color::rgba(0.80, 0.83, 0.90, 0.90),
            TextAlign::Center,
        );
    });
}