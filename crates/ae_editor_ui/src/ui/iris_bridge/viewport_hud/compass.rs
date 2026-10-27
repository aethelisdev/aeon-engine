// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Scene Navigation Compass Builder
//!
//! Renders the interactive 3D Orientation Gizmo (Compass) in the top-right corner of the Viewport.
//! 3D axis connecting lines are rendered directly via [`DrawCommandList`] GPU SDF quads,
//! while the 6 interactive knobs are tagged with semantic IDs [`TAG_COMPASS_POS_X`]..=[`TAG_COMPASS_NEG_Z`].

use super::types::{TAG_COMPASS_CANVAS, ViewportHudParams};
use ae_editor::scene_gizmo::SceneNavigationGizmo;
use ae_renderer::camera::Camera;
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;

/// Diameter of the 3D scene compass canvas in logical pixels.
pub const COMPASS_SIZE: f32 = 72.0;

/// Radius of the compass axis projection sphere.
pub const COMPASS_RADIUS: f32 = 25.0;

/// Top and right margin offsets from the viewport edge in pixels.
pub const COMPASS_MARGIN: f32 = 8.0;

/// Draws the complete 3D orientation compass (circular backdrop, connecting lines, knobs and dots)
/// directly into an [`ImmediateCanvas`].
///
/// Axis connecting lines and interactive knobs rotate  with the camera orientation
/// at 144 Hz with zero persistent child tree nodes.
///
/// # Arguments
/// * `canvas` - Immediate drawing context.
/// * `camera` - Active viewport camera snapshot.
/// * `rect` - Bounding rectangle of the compass widget in screen pixels.
pub fn draw_compass_canvas(canvas: &mut ImmediateCanvas<'_>, camera: &Camera, rect: Rect) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let center_x = rect.x + rect.width * 0.5;
    let center_y = rect.y + rect.height * 0.5;
    let radius = COMPASS_RADIUS;

    // 1. Draw circular glass background and subtle outer border ring
    let bg_color = Color::rgba(0.04, 0.05, 0.08, 0.65);
    let border_color = Color::rgba(1.0, 1.0, 1.0, 0.08);
    canvas.fill_rect(rect, bg_color, rect.width * 0.5);
    canvas.stroke_rect(rect, border_color, 1.0, rect.width * 0.5);

    // 2. Compute 3D projected axis endpoints (sorted back-to-front by view depth)
    let endpoints =
        SceneNavigationGizmo::compute_axis_endpoints(camera.pitch.0, camera.yaw.0, radius);

    // 3. Render axis connecting lines and endpoints in sorted depth order
    for (dx, dy, label, color_rgb, is_positive) in endpoints {
        let axis_color = Color::rgba(
            color_rgb[0] as f32 / 255.0,
            color_rgb[1] as f32 / 255.0,
            color_rgb[2] as f32 / 255.0,
            if is_positive { 0.95 } else { 0.45 },
        );

        // Draw smooth, continuous solid axis line from center to endpoint
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1.0 {
            let line_thick = if is_positive { 1.8 } else { 1.0 };
            let step_dist = line_thick * 0.75;
            let num_steps = (len / step_dist).ceil().max(1.0) as usize;
            for i in 0..=num_steps {
                let t = i as f32 / num_steps as f32;
                let qx = center_x + dx * t - line_thick * 0.5;
                let qy = center_y + dy * t - line_thick * 0.5;
                canvas.fill_rect(Rect::new(qx, qy, line_thick, line_thick), axis_color, 0.0);
            }
        }

        // Draw endpoint knob or dot
        if is_positive {
            let knob_size = 14.0;
            let knob_x = center_x + dx - knob_size * 0.5;
            let knob_y = center_y + dy - knob_size * 0.5;
            let knob_rect = Rect::new(knob_x, knob_y, knob_size, knob_size);

            // Clean, borderless solid circular knob disk
            canvas.fill_rect(knob_rect, axis_color, knob_size * 0.5);

            // Centered single-character axis indicator label ("X", "Y", "Z")
            let text_pos = Point::new(knob_x + 3.8, knob_y + 1.8);
            canvas.draw_text(text_pos, label, 9.0, Color::WHITE);
        } else {
            let dot_size = 6.0;
            let dot_x = center_x + dx - dot_size * 0.5;
            let dot_y = center_y + dy - dot_size * 0.5;
            let dot_rect = Rect::new(dot_x, dot_y, dot_size, dot_size);

            let dim_color = Color::rgba(
                axis_color.r * 0.45,
                axis_color.g * 0.45,
                axis_color.b * 0.45,
                0.75,
            );
            canvas.fill_rect(dot_rect, dim_color, dot_size * 0.5);
        }
    }
}

/// Builds the 3D Scene Navigation Compass in the top-right corner of the Viewport.
///
/// Anchored via [`Style::position_absolute`], [`Style::right`], and [`Style::top`].
/// Emits an [`ImmediateCanvas`] node tagged with [`TAG_COMPASS_CANVAS`] with zero persistent child tree nodes.
pub fn build_scene_navigation_compass(scope: &mut UiScope<'_>, params: &ViewportHudParams<'_>) {
    let compass_rect = Rect::new(
        params.viewport_rect.right() - COMPASS_SIZE - COMPASS_MARGIN,
        params.viewport_rect.y + COMPASS_MARGIN,
        COMPASS_SIZE,
        COMPASS_SIZE,
    );

    scope.volatile_overlay(
        "SceneNavCompass",
        TAG_COMPASS_CANVAS,
        compass_rect,
        |canvas| {
            draw_compass_canvas(canvas, params.camera, compass_rect);
        },
    );
}

/// Updates the 3D scene compass volatile canvas primitives in-place in the UI tree.
///
/// Resolves the compass bounding rectangle and repaints the glass disc, lines, rotating knobs,
/// and labels without modifying retained tree topology or triggering Taffy layout passes.
///
/// # Arguments
/// * `tree` - Mutable reference to the UI hierarchy.
/// * `camera` - Active camera instance with current pitch and yaw angles.
/// * `viewport_rect` - Screen bounding box of the active viewport.
pub fn update_compass_canvas(tree: &mut UiTree, camera: &Camera, viewport_rect: Rect) -> bool {
    let compass_rect = Rect::new(
        viewport_rect.right() - COMPASS_SIZE - COMPASS_MARGIN,
        viewport_rect.y + COMPASS_MARGIN,
        COMPASS_SIZE,
        COMPASS_SIZE,
    );

    irisui::widgets::canvas::update_canvas_by_tag(tree, TAG_COMPASS_CANVAS, |canvas| {
        draw_compass_canvas(canvas, camera, compass_rect);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_editor::gizmo::{GizmoMode, GizmoSpace};
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};
    use hecs::World;

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
        let snapping = SnapSettings::default();
        let world = World::new();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        build_scene_navigation_compass(&mut UiScope::new(&mut tree, root), &params);

        // Verify that NOT A SINGLE fake "AxisLineDot" node was created
        let mut names = Vec::new();
        for (_, node) in tree.iter() {
            if let Some(ref name) = node.name {
                names.push(name.as_str());
            }
        }

        assert!(
            !names.contains(&"AxisLineDot"),
            "AxisLineDot fake nodes must be 100% eliminated from the UI tree"
        );

        // Verify total node count is exactly 1 (single autonomous ImmediateCanvas host node)
        let compass_node_count = names
            .iter()
            .filter(|n| n.starts_with("Compass") || n.starts_with("SceneNav"))
            .count();
        assert_eq!(
            compass_node_count, 1,
            "Compass hierarchy must strictly contain exactly 1 autonomous ImmediateCanvas node with zero persistent child tree nodes"
        );
    }

    #[test]
    fn test_compass_semantic_tags_assigned() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        build_scene_navigation_compass(&mut UiScope::new(&mut tree, root), &params);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(tags.contains(&TAG_COMPASS_CANVAS));
    }

    #[test]
    fn test_compass_canvas_draws_primitives() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");
        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        build_scene_navigation_compass(&mut UiScope::new(&mut tree, root), &params);

        let compass_id = tree
            .find_node_by_tag(TAG_COMPASS_CANVAS)
            .expect("Compass canvas node should be registered with TAG_COMPASS_CANVAS");

        let primitives = tree
            .canvas_primitives(compass_id)
            .expect("primitives should exist for compass");
        assert!(
            !primitives.is_empty(),
            "ImmediateCanvas must contain glass backdrop, axis lines, knobs, and text primitives"
        );
        // Backdrop rect + border + lines + 3 knobs + 3 texts + 3 dots >= 30 primitives
        assert!(
            primitives.len() >= 30,
            "ImmediateCanvas should contain at least 30 volatile primitives for compass visualization"
        );

        // Test in-place volatile update pass without structural modifications
        let updated = update_compass_canvas(&mut tree, &camera, params.viewport_rect);
        assert!(
            updated,
            "update_compass_canvas must succeed for valid compass tag"
        );
    }
}