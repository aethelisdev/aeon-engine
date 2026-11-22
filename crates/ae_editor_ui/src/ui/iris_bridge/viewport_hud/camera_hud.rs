// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Camera Info HUD Builder
//!
//! Renders the bottom-right camera position and orientation telemetry badge using
//! [`UiScope::live_canvas_anchored`] (`CanvasOrigin::NodeTopLeft`) and stack-formatted text.

use super::types::TAG_CAMERA_HUD;
use ae_renderer::camera::Camera;
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;
use std::fmt::Write as _;

/// Layout pixel width reserved for the camera telemetry HUD pill canvas.
pub const CAMERA_HUD_WIDTH: f32 = 260.0;

/// Layout pixel height reserved for the camera telemetry HUD pill.
pub const CAMERA_HUD_HEIGHT: f32 = 22.0;

/// Inset offset in pixels from the bottom and right edges of the viewport.
pub const CAMERA_HUD_MARGIN: f32 = 12.0;

struct StackTelemetryText {
    buf: [u8; 96],
    len: usize,
}

impl StackTelemetryText {
    const fn new() -> Self {
        Self {
            buf: [0u8; 96],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    fn format_camera(&mut self, camera: &Camera) -> &str {
        if write!(
            self,
            "Pos: {:.1}, {:.1}, {:.1} | Rot: {:.0}°, {:.0}°",
            camera.position.x,
            camera.position.y,
            camera.position.z,
            camera.pitch.0.to_degrees(),
            camera.yaw.0.to_degrees(),
        )
        .is_err()
        {
            self.len = 0;
        }
        self.as_str()
    }
}

impl std::fmt::Write for StackTelemetryText {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.len +=
            s.bytes()
                .take(self.buf.len() - self.len)
                .enumerate()
                .fold(0, |count, (idx, byte)| {
                    self.buf[self.len + idx] = byte;
                    count + 1
                });
        Ok(())
    }
}

#[inline]
fn camera_hud_local_rect() -> Rect {
    Rect::new(0.0, 0.0, CAMERA_HUD_WIDTH, CAMERA_HUD_HEIGHT)
}

#[inline]
fn draw_camera_hud_pill(canvas: &mut ImmediateCanvas<'_>, rect: Rect, text: &str, pill_w: f32) {
    canvas.fill_rect(
        Rect::new(rect.right() - pill_w, rect.y, pill_w, rect.height),
        Color::rgba(0.07, 0.08, 0.11, 0.82),
        4.0,
    );
    canvas.stroke_rect(
        Rect::new(rect.right() - pill_w, rect.y, pill_w, rect.height),
        Color::rgba(0.24, 0.28, 0.38, 0.55),
        1.0,
        4.0,
    );
    canvas.draw_text(
        Point::new(rect.right() - pill_w + 8.0, rect.y + 4.0),
        text,
        10.0,
        Color::rgba(0.85, 0.88, 0.94, 0.95),
    );
}

/// Draws the camera telemetry pill background, border, and dynamic text into `canvas` without intermediate `let`s.
pub fn draw_camera_hud_canvas(canvas: &mut ImmediateCanvas<'_>, camera: &Camera, rect: Rect) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    draw_camera_hud_from_text(
        canvas,
        rect,
        StackTelemetryText::new().format_camera(camera),
    );
}

#[inline]
fn draw_camera_hud_from_text(canvas: &mut ImmediateCanvas<'_>, rect: Rect, text: &str) {
    draw_camera_hud_pill(
        canvas,
        rect,
        text,
        (irisui::text::measure_text_advance(text, 10.0).ceil() + 16.0).min(rect.width),
    );
}

/// Emits the camera telemetry HUD badge anchored cleanly via Taffy `.right`/`.bottom` and `CanvasOrigin::NodeTopLeft`.
pub fn build_camera_hud(scope: &mut UiScope<'_>, camera: &Camera) {
    scope.live_canvas_anchored(
        "CameraHudCanvas",
        TAG_CAMERA_HUD,
        CanvasOrigin::NodeTopLeft,
        Style::new()
            .position_absolute()
            .right(CAMERA_HUD_MARGIN)
            .bottom(CAMERA_HUD_MARGIN)
            .width(CAMERA_HUD_WIDTH)
            .height(CAMERA_HUD_HEIGHT),
        |canvas| draw_camera_hud_canvas(canvas, camera, camera_hud_local_rect()),
    );
}

/// Updates the camera telemetry HUD volatile primitives in-place via [`UiScope::update_live_canvas`].
pub fn update_camera_hud(scope: &mut UiScope<'_>, camera: &Camera) -> bool {
    scope.update_live_canvas(TAG_CAMERA_HUD, |canvas| {
        draw_camera_hud_canvas(canvas, camera, camera_hud_local_rect());
    })
}