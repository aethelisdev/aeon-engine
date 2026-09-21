// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Scene Navigation Compass Builder
//!
//! Renders the interactive 3D Orientation Gizmo (Compass) in the top-right corner of the Viewport.
//! Connects the center to axis knobs with 3D projection lines, and snaps camera on click.

use super::types::{ViewportHudAction, ViewportHudParams, ViewportHudTargets};
use ae_editor::scene_gizmo::{SceneNavigationGizmo, SceneViewSnap};
use irisui::prelude::*;

/// Builds the 3D Scene Navigation Compass in the top-right corner of the Viewport.
pub fn build_scene_navigation_compass(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let compass_center_x = params.viewport_rect.x + params.viewport_rect.width - 44.0;
    let compass_center_y = params.viewport_rect.y + 44.0;
    let radius = 28.0;
    let total_radius = radius + 8.0; // 36px radius = 72px diameter

    let base_rect = Rect::new(
        compass_center_x - total_radius,
        compass_center_y - total_radius,
        total_radius * 2.0,
        total_radius * 2.0,
    );

    // Compass Circular Glass Backdrop
    let compass_id = tree.create_node();
    if let Some(node) = tree.get_mut(compass_id) {
        node.set_name("SceneNavCompass");
        node.computed_rect = base_rect;
        node.style = Style::new()
            .background(Color::rgba(0.0, 0.0, 0.0, 0.51))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.12))
            .border_radius(total_radius)
            .box_shadow(0.0, 4.0, 12.0, Color::rgba(0.0, 0.0, 0.0, 0.40));
    }
    let _ = tree.add_child(parent_id, compass_id);

    // Compute projected axis endpoints
    let endpoints = SceneNavigationGizmo::compute_axis_endpoints(
        params.camera.pitch.0,
        params.camera.yaw.0,
        radius,
    );

    for (dx, dy, label, color_rgb, is_positive) in endpoints {
        let knob_center_x = compass_center_x + dx;
        let knob_center_y = compass_center_y + dy;
        let axis_color = Color::rgba(
            color_rgb[0] as f32 / 255.0,
            color_rgb[1] as f32 / 255.0,
            color_rgb[2] as f32 / 255.0,
            1.0,
        );

        // 1. Draw 3D Axis Connecting Line Segments
        let num_dots = 12;
        let line_thick = if is_positive { 2.0 } else { 1.2 };
        let line_color = if is_positive {
            axis_color
        } else {
            Color::rgba(
                axis_color.r * 0.45,
                axis_color.g * 0.45,
                axis_color.b * 0.45,
                0.45,
            )
        };

        for i in 1..num_dots {
            let t = i as f32 / (num_dots as f32);
            let lx = compass_center_x + dx * t;
            let ly = compass_center_y + dy * t;
            let dot_id = tree.create_node();
            if let Some(node) = tree.get_mut(dot_id) {
                node.set_name("AxisLineDot");
                node.computed_rect = Rect::new(
                    lx - line_thick * 0.5,
                    ly - line_thick * 0.5,
                    line_thick,
                    line_thick,
                );
                node.style = Style::new()
                    .background(line_color)
                    .border_radius(line_thick * 0.5);
            }
            let _ = tree.add_child(compass_id, dot_id);
        }

        // 2. Draw Positive Knobs or Negative Dots
        if is_positive {
            let knob_size = 13.0;
            let knob_rect = Rect::new(
                knob_center_x - knob_size * 0.5,
                knob_center_y - knob_size * 0.5,
                knob_size,
                knob_size,
            );
            let is_hover = knob_rect.contains_point(params.cursor_pos);

            let knob_id = tree.create_node();
            if let Some(node) = tree.get_mut(knob_id) {
                node.set_name("CompassKnob");
                node.computed_rect = knob_rect;
                let bg = if is_hover {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    axis_color
                };
                node.style = Style::new().background(bg).border_radius(knob_size * 0.5);
            }
            let _ = tree.add_child(compass_id, knob_id);

            let txt_id = tree.create_node();
            if let Some(node) = tree.get_mut(txt_id) {
                node.set_name("CompassKnobText");
                node.set_text(label);
                node.font_size = 8.5;
                node.line_height = knob_size;
                node.text_align = TextAlign::Center;
                node.text_color = if is_hover {
                    Color::rgba(0.0, 0.0, 0.0, 1.0)
                } else {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                };
                node.computed_rect = knob_rect;
            }
            let _ = tree.add_child(knob_id, txt_id);

            let snap = match label {
                "X" => SceneViewSnap::Right,
                "Y" => SceneViewSnap::Top,
                "Z" => SceneViewSnap::Front,
                _ => SceneViewSnap::Perspective,
            };
            let (target_pitch, target_yaw, target_pos) =
                snap.compute_transform(params.camera.target, 12.0);
            targets.compass_knobs.push((
                ViewportHudAction::SetCameraTransform {
                    pitch: target_pitch,
                    yaw: target_yaw,
                    position: target_pos,
                    mode: None,
                },
                knob_rect,
            ));
        } else {
            let dot_size = 7.0;
            let dot_rect = Rect::new(
                knob_center_x - dot_size * 0.5,
                knob_center_y - dot_size * 0.5,
                dot_size,
                dot_size,
            );
            let is_hover = dot_rect.contains_point(params.cursor_pos);

            let dot_id = tree.create_node();
            if let Some(node) = tree.get_mut(dot_id) {
                node.set_name("CompassDot");
                node.computed_rect = dot_rect;
                let bg = if is_hover {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::rgba(
                        axis_color.r * 0.45,
                        axis_color.g * 0.45,
                        axis_color.b * 0.45,
                        0.75,
                    )
                };
                node.style = Style::new().background(bg).border_radius(dot_size * 0.5);
            }
            let _ = tree.add_child(compass_id, dot_id);

            let snap = match label {
                "-X" => SceneViewSnap::Left,
                "-Y" => SceneViewSnap::Bottom,
                "-Z" => SceneViewSnap::Back,
                _ => SceneViewSnap::Perspective,
            };
            let (target_pitch, target_yaw, target_pos) =
                snap.compute_transform(params.camera.target, 12.0);
            targets.compass_knobs.push((
                ViewportHudAction::SetCameraTransform {
                    pitch: target_pitch,
                    yaw: target_yaw,
                    position: target_pos,
                    mode: None,
                },
                dot_rect,
            ));
        }
    }
}