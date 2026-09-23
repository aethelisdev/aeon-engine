// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Scene Navigation Compass Builder
//!
//! Renders the interactive 3D Orientation Gizmo (Compass) in the top-right corner of the Viewport.
//! 3D axis connecting lines are rendered directly via [`DrawCommandList`] GPU SDF quads,
//! while the 6 interactive knobs are tagged with semantic IDs [`TAG_COMPASS_POS_X`]..=[`TAG_COMPASS_NEG_Z`].

use super::types::{
    TAG_COMPASS_CANVAS, TAG_COMPASS_NEG_X, TAG_COMPASS_NEG_Y, TAG_COMPASS_NEG_Z, TAG_COMPASS_POS_X,
    TAG_COMPASS_POS_Y, TAG_COMPASS_POS_Z, ViewportHudParams,
};
use ae_editor::scene_gizmo::SceneNavigationGizmo;
use irisui::prelude::*;

/// Appends GPU SDF quads directly into the hardware command list for the 3D scene compass.
///
/// Draws 3D projected axis connecting lines (X, Y, Z, -X, -Y, -Z) from the center directly
/// into the GPU command batch with zero intermediate layout nodes in the UI tree.
pub fn append_compass_quads(command_list: &mut DrawCommandList, rect: Rect, pitch: f32, yaw: f32) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let center_x = rect.x + rect.width * 0.5;
    let center_y = rect.y + rect.height * 0.5;
    let radius = 28.0;

    // Compute projected axis endpoints
    let endpoints = SceneNavigationGizmo::compute_axis_endpoints(pitch, yaw, radius);

    // Draw 6 Axis Connecting Lines as SDF quads along their direction vectors
    for (dx, dy, _label, color_rgb, is_positive) in endpoints {
        let line_thick = if is_positive { 2.0 } else { 1.2 };
        let axis_color = Color::rgba(
            color_rgb[0] as f32 / 255.0,
            color_rgb[1] as f32 / 255.0,
            color_rgb[2] as f32 / 255.0,
            if is_positive { 0.90 } else { 0.40 },
        );

        // Subdivide line segment into small quads along (dx, dy)
        let num_quads = 8;
        for i in 1..num_quads {
            let t = i as f32 / (num_quads as f32);
            let qx = center_x + dx * t - line_thick * 0.5;
            let qy = center_y + dy * t - line_thick * 0.5;

            command_list.push_quad(QuadInstance {
                rect: [qx, qy, line_thick, line_thick],
                color: axis_color.to_linear().to_array(),
                corner_radii: [line_thick * 0.5; 4],
                clip_rect: [rect.x, rect.y, rect.right(), rect.bottom()],
                ..Default::default()
            });
        }
    }
}

/// Builds the 3D Scene Navigation Compass in the top-right corner of the Viewport.
///
/// Anchored via [`Style::position_absolute`], [`Style::right`], and [`Style::top`].
/// Emits a single canvas node for custom hardware line rendering, and 6 interactive
/// knob nodes tagged with semantic IDs [`TAG_COMPASS_POS_X`]..=[`TAG_COMPASS_NEG_Z`].
pub fn build_scene_navigation_compass(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
) {
    let radius = 28.0;
    let total_radius = radius + 8.0; // 36px radius = 72px diameter
    let compass_size = total_radius * 2.0;

    // 1. Compass Canvas Host: Absolutely anchored at top-right with circular glass styling
    let canvas_style = Style::new()
        .position_absolute()
        .right(8.0)
        .top(8.0)
        .width(compass_size)
        .height(compass_size)
        .background(Color::rgba(0.0, 0.0, 0.0, 0.51))
        .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.12))
        .border_radius(total_radius)
        .box_shadow(0.0, 4.0, 12.0, Color::rgba(0.0, 0.0, 0.0, 0.40));

    // 2. Compute projected axis endpoints for interactive knobs
    let endpoints = SceneNavigationGizmo::compute_axis_endpoints(
        params.camera.pitch.0,
        params.camera.yaw.0,
        radius,
    );

    let mut scope = UiScope::new(tree, parent_id);
    scope.container_tagged(
        "SceneNavCompass",
        canvas_style,
        WidgetRole::OscilloscopeCanvas,
        TAG_COMPASS_CANVAS,
        |compass_scope| {
            for (dx, dy, label, color_rgb, is_positive) in endpoints {
                let axis_color = Color::rgba(
                    color_rgb[0] as f32 / 255.0,
                    color_rgb[1] as f32 / 255.0,
                    color_rgb[2] as f32 / 255.0,
                    1.0,
                );

                if is_positive {
                    let knob_size = 13.0;
                    let rel_knob_x = total_radius + dx - knob_size * 0.5;
                    let rel_knob_y = total_radius + dy - knob_size * 0.5;
                    let tag = match label {
                        "X" => TAG_COMPASS_POS_X,
                        "Y" => TAG_COMPASS_POS_Y,
                        "Z" => TAG_COMPASS_POS_Z,
                        _ => 0,
                    };
                    compass_scope
                        .compass_knob(label, tag, rel_knob_x, rel_knob_y, knob_size, axis_color);
                } else {
                    let dot_size = 7.0;
                    let rel_dot_x = total_radius + dx - dot_size * 0.5;
                    let rel_dot_y = total_radius + dy - dot_size * 0.5;
                    let tag = match label {
                        "-X" => TAG_COMPASS_NEG_X,
                        "-Y" => TAG_COMPASS_NEG_Y,
                        "-Z" => TAG_COMPASS_NEG_Z,
                        _ => 0,
                    };
                    let dim_color = Color::rgba(
                        axis_color.r * 0.45,
                        axis_color.g * 0.45,
                        axis_color.b * 0.45,
                        0.75,
                    );
                    compass_scope.compass_dot(tag, rel_dot_x, rel_dot_y, dot_size, dim_color);
                }
            }
        },
    );
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

        build_scene_navigation_compass(&mut tree, root, &params);

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

        // Verify total node count is small and clean (1 canvas + 3 knobs + 3 texts + 3 dots = 10)
        let compass_node_count = names
            .iter()
            .filter(|n| n.starts_with("Compass") || n.starts_with("SceneNav"))
            .count();
        assert_eq!(
            compass_node_count, 10,
            "Compass hierarchy must strictly contain 1 canvas host, 3 positive knobs, 3 text nodes, and 3 negative dots"
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

        build_scene_navigation_compass(&mut tree, root, &params);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(tags.contains(&TAG_COMPASS_CANVAS));
        assert!(tags.contains(&TAG_COMPASS_POS_X));
        assert!(tags.contains(&TAG_COMPASS_POS_Y));
        assert!(tags.contains(&TAG_COMPASS_POS_Z));
        assert!(tags.contains(&TAG_COMPASS_NEG_X));
        assert!(tags.contains(&TAG_COMPASS_NEG_Y));
        assert!(tags.contains(&TAG_COMPASS_NEG_Z));
    }

    #[test]
    fn test_append_compass_quads_generates_gpu_draw_calls() {
        let mut command_list = DrawCommandList::new();
        let rect = Rect::new(720.0, 8.0, 72.0, 72.0);

        append_compass_quads(&mut command_list, rect, 0.2, -0.4);

        // 6 axes * 7 line segment quads = 42 quads in command list
        assert!(
            command_list.quads.len() >= 40,
            "Hardware command list must contain projected axis line quads"
        );
    }
}