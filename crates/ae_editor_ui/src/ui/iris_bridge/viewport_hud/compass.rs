// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Scene Navigation Compass Builder
//!
//! Renders the interactive 3D Orientation Gizmo (Compass) in the top-right corner of the Viewport
//! using [`UiScope::live_canvas_anchored`] (`CanvasOrigin::NodeTopLeft`) and analytical
//! oriented GPU SDF line segments (`canvas.draw_line`).

use super::types::TAG_COMPASS_CANVAS;
use ae_gizmos::SceneNavigationGizmo;
use ae_renderer::camera::Camera;
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;

/// Diameter of the 3D scene compass canvas in logical pixels.
pub const COMPASS_SIZE: f32 = 72.0;

/// Radius of the compass axis projection sphere.
pub const COMPASS_RADIUS: f32 = 25.0;

/// Top and right margin offsets from the viewport edge in pixels.
pub const COMPASS_MARGIN: f32 = 8.0;

#[inline]
fn compass_local_rect() -> Rect {
    Rect::new(0.0, 0.0, COMPASS_SIZE, COMPASS_SIZE)
}

#[inline]
fn axis_color(rgb: [u8; 3], is_positive: bool) -> Color {
    Color::rgba(
        f32::from(rgb[0]) / 255.0,
        f32::from(rgb[1]) / 255.0,
        f32::from(rgb[2]) / 255.0,
        if is_positive { 0.95 } else { 0.45 },
    )
}

#[inline]
fn draw_compass_axis_entry(
    canvas: &mut ImmediateCanvas<'_>,
    center: Point,
    (dx, dy, label, color_rgb, is_positive): (f32, f32, &'static str, [u8; 3], bool),
) {
    if (dx * dx + dy * dy).sqrt() > 1.0 {
        canvas.draw_line(
            center,
            Point::new(center.x + dx, center.y + dy),
            axis_color(color_rgb, is_positive),
            if is_positive { 1.8 } else { 1.0 },
        );
    }
    if is_positive {
        draw_positive_knob(
            canvas,
            center.x + dx - 7.0,
            center.y + dy - 7.0,
            label,
            axis_color(color_rgb, true),
        );
    } else {
        draw_negative_dot(
            canvas,
            center.x + dx - 3.0,
            center.y + dy - 3.0,
            axis_color(color_rgb, false),
        );
    }
}

#[inline]
fn draw_positive_knob(
    canvas: &mut ImmediateCanvas<'_>,
    knob_x: f32,
    knob_y: f32,
    label: &'static str,
    color: Color,
) {
    canvas.fill_rect(Rect::new(knob_x, knob_y, 14.0, 14.0), color, 7.0);
    canvas.draw_text(
        Point::new(knob_x + 3.8, knob_y + 1.8),
        label,
        9.0,
        Color::WHITE,
    );
}

#[inline]
fn draw_negative_dot(canvas: &mut ImmediateCanvas<'_>, dot_x: f32, dot_y: f32, color: Color) {
    canvas.fill_rect(
        Rect::new(dot_x, dot_y, 6.0, 6.0),
        Color::rgba(color.r * 0.45, color.g * 0.45, color.b * 0.45, 0.75),
        3.0,
    );
}

/// Draws the complete 3D orientation compass (circular backdrop, analytical lines, knobs and dots)
/// directly into an [`ImmediateCanvas`].
pub fn draw_compass_canvas(canvas: &mut ImmediateCanvas<'_>, camera: &Camera, rect: Rect) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    canvas.fill_rect(rect, Color::rgba(0.04, 0.05, 0.08, 0.65), rect.width * 0.5);
    canvas.stroke_rect(
        rect,
        Color::rgba(1.0, 1.0, 1.0, 0.08),
        1.0,
        rect.width * 0.5,
    );

    SceneNavigationGizmo::compute_axis_endpoints(camera.pitch.0, camera.yaw.0, COMPASS_RADIUS)
        .into_iter()
        .for_each(|endpoint| {
            draw_compass_axis_entry(
                canvas,
                Point::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5),
                endpoint,
            );
        });
}

/// Emits the 3D Scene Navigation Compass anchored cleanly via Taffy `.right`/`.top` and `CanvasOrigin::NodeTopLeft`.
pub fn build_scene_navigation_compass(scope: &mut UiScope<'_>, camera: &Camera) {
    scope.live_canvas_anchored(
        "SceneNavCompass",
        TAG_COMPASS_CANVAS,
        CanvasOrigin::NodeTopLeft,
        Style::new()
            .position_absolute()
            .right(COMPASS_MARGIN)
            .top(COMPASS_MARGIN)
            .width(COMPASS_SIZE)
            .height(COMPASS_SIZE),
        |canvas| draw_compass_canvas(canvas, camera, compass_local_rect()),
    );
}

/// Updates the 3D scene compass volatile canvas primitives in-place via [`UiScope::update_live_canvas`].
pub fn update_compass_canvas(scope: &mut UiScope<'_>, camera: &Camera) -> bool {
    scope.update_live_canvas(TAG_COMPASS_CANVAS, |canvas| {
        draw_compass_canvas(canvas, camera, compass_local_rect());
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_renderer::camera::{Camera, ProjectionMode};

    fn make_test_camera() -> Camera {
        Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_compass_zero_fake_dot_nodes() {
        let camera = make_test_camera();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        build_scene_navigation_compass(&mut UiScope::new(&mut tree, root), &camera);

        let compass_node = tree
            .node_tagged(TAG_COMPASS_CANVAS)
            .expect("Compass host node must exist");
        assert_eq!(compass_node.canvas_origin, CanvasOrigin::NodeTopLeft);

        let names: Vec<_> = tree.iter().filter_map(|(_, n)| n.name.as_deref()).collect();
        assert!(!names.contains(&"AxisLineDot"));
        let compass_node_count = names
            .iter()
            .filter(|n| n.starts_with("Compass") || n.starts_with("SceneNav"))
            .count();
        assert_eq!(compass_node_count, 1);
    }

    #[test]
    fn test_compass_semantic_tags_assigned() {
        let camera = make_test_camera();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        build_scene_navigation_compass(&mut UiScope::new(&mut tree, root), &camera);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(tags.contains(&TAG_COMPASS_CANVAS));
    }

    #[test]
    fn test_compass_canvas_draws_analytical_lines_and_updates_in_place() {
        let camera = make_test_camera();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        build_scene_navigation_compass(&mut UiScope::new(&mut tree, root), &camera);

        let compass_id = tree
            .find_node_by_tag(TAG_COMPASS_CANVAS)
            .expect("Compass canvas node should be registered with TAG_COMPASS_CANVAS");

        let primitives = tree
            .canvas_primitives(compass_id)
            .expect("primitives should exist for compass");
        let line_count = primitives
            .iter()
            .filter(|p| matches!(p, irisui::core::CanvasPrimitive::Line { .. }))
            .count();
        assert!(
            line_count >= 4,
            "Compass axes must use real CanvasPrimitive::Line segments, got {}",
            line_count
        );

        let mut scope = UiScope::new(&mut tree, root);
        let unchanged = update_compass_canvas(&mut scope, &camera);
        assert!(
            !unchanged,
            "update_compass_canvas must skip dirty marking when camera did not move"
        );

        let mut rotated_camera = make_test_camera();
        rotated_camera.yaw = cgmath::Rad(0.75);
        let mut scope = UiScope::new(&mut tree, root);
        let updated = update_compass_canvas(&mut scope, &rotated_camera);
        assert!(
            updated,
            "update_compass_canvas must succeed when camera orientation changes"
        );
    }
}