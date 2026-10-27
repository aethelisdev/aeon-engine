// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Camera Info HUD Builder
//!
//! Renders the bottom-right camera position and orientation angles telemetry overlay badge
//! using an autonomous [`ImmediateCanvas`] without persistent child nodes in the [`UiTree`].
//!

use super::types::{TAG_CAMERA_HUD, ViewportHudParams};
use ae_renderer::camera::Camera;
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;

/// Layout pixel width reserved for the camera telemetry HUD pill canvas.
pub const CAMERA_HUD_WIDTH: f32 = 260.0;

/// Layout pixel height reserved for the camera telemetry HUD pill.
pub const CAMERA_HUD_HEIGHT: f32 = 22.0;

/// Inset offset in pixels from the bottom and right edges of the viewport.
pub const CAMERA_HUD_MARGIN: f32 = 12.0;

/// Draws the camera telemetry pill background, border, and dynamic text directly into an [`ImmediateCanvas`].
///
/// # Arguments
/// * `canvas` - Immediate drawing context.
/// * `camera` - Current active viewport camera snapshot.
/// * `rect` - Bounding rectangle of the camera HUD pill in screen pixels.
pub fn draw_camera_hud_canvas(canvas: &mut ImmediateCanvas<'_>, camera: &Camera, rect: Rect) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let p = camera.position;
    let pos_text = format!("Pos: {:.1}, {:.1}, {:.1}", p.x, p.y, p.z);
    let pitch_deg = camera.pitch.0.to_degrees();
    let yaw_deg = camera.yaw.0.to_degrees();
    let rot_text = format!("Rot: {:.0}°, {:.0}°", pitch_deg, yaw_deg);
    let full_text = format!("{} | {}", pos_text, rot_text);

    let bg_color = Color::rgba(0.07, 0.08, 0.11, 0.82);
    let border_color = Color::rgba(0.24, 0.28, 0.38, 0.55);
    let text_color = Color::rgba(0.85, 0.88, 0.94, 0.95);

    // Measure exact text advance dynamically for pixel-snug backdrop pill
    let text_w = irisui::text::measure_text_advance(&full_text, 10.0).ceil();
    let pad_h = 8.0;
    let pill_w = (text_w + pad_h * 2.0).min(rect.width);
    let pill_x = rect.right() - pill_w;
    let pill_rect = Rect::new(pill_x, rect.y, pill_w, rect.height);

    // 1. Draw rounded backdrop pill and border hugging the text snugly
    canvas.fill_rect(pill_rect, bg_color, 4.0);
    canvas.stroke_rect(pill_rect, border_color, 1.0, 4.0);

    // 2. Draw volatile camera telemetry text
    let text_pos = Point::new(pill_rect.x + pad_h, rect.y + 4.0);
    canvas.draw_text(text_pos, full_text, 10.0, text_color);
}

/// Builds the camera position & rotation angles HUD badge at the bottom-right of the viewport.
///
/// Emits an [`ImmediateCanvas`] node tagged with [`TAG_CAMERA_HUD`] positioned via absolute insets
/// with zero persistent child tree nodes.
pub fn build_camera_hud(scope: &mut UiScope<'_>, params: &ViewportHudParams<'_>) {
    let hud_rect = Rect::new(
        params.viewport_rect.right() - CAMERA_HUD_WIDTH - CAMERA_HUD_MARGIN,
        params.viewport_rect.bottom() - CAMERA_HUD_HEIGHT - CAMERA_HUD_MARGIN,
        CAMERA_HUD_WIDTH,
        CAMERA_HUD_HEIGHT,
    );

    scope.volatile_overlay("CameraHudCanvas", TAG_CAMERA_HUD, hud_rect, |canvas| {
        draw_camera_hud_canvas(canvas, params.camera, hud_rect);
    });
}

/// Updates the camera telemetry HUD volatile primitives in-place in the UI tree.
///
/// Resolves the HUD pill boundary from viewport bounds and updates the canvas primitives
/// without modifying retained tree topology or triggering Taffy layout passes.
///
/// # Arguments
/// * `tree` - Mutable reference to the UI hierarchy.
/// * `camera` - Current camera instance.
/// * `viewport_rect` - Screen bounding box of the active viewport.
pub fn update_camera_hud(tree: &mut UiTree, camera: &Camera, viewport_rect: Rect) -> bool {
    let hud_rect = Rect::new(
        viewport_rect.right() - CAMERA_HUD_WIDTH - CAMERA_HUD_MARGIN,
        viewport_rect.bottom() - CAMERA_HUD_HEIGHT - CAMERA_HUD_MARGIN,
        CAMERA_HUD_WIDTH,
        CAMERA_HUD_HEIGHT,
    );

    irisui::widgets::canvas::update_canvas_by_tag(tree, TAG_CAMERA_HUD, |canvas| {
        draw_camera_hud_canvas(canvas, camera, hud_rect);
    })
}