// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Model Interactive Wireframe Orbit Viewport Builder
//!
//! Renders an interactive 3D bounding box wireframe canvas with projected 3D coordinates,
//! orbital yaw and pitch rotation, zoom distance scaling, and 3D coordinate axes
//! using pure declarative [`UiScope`] and zero-allocation GPU command list quads.
//!

use crate::ui::iris_bridge::assets::types::AssetPreviewModalState;
use irisui::prelude::*;

/// Renders the 3D model inspection preview with interactive wireframe orbit canvas using declarative [`UiScope`].
///
/// TODO(architecture): Replace CPU 2D projection software-rendering with an
/// offscreen WGPU render-to-texture preview pipeline from `ae_renderer`.
pub(crate) fn render_model_preview_content(ui: &mut UiScope<'_>, _modal: &AssetPreviewModalState) {
    // 1. Controls Bar: Instruction Hint + Spawn Action Button
    ui.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .height(28.0),
        |row| {
            row.label(
                "Left Drag: 3D Orbit | Wheel: Zoom Distance",
                11.0,
                Color::rgba(0.55, 0.60, 0.72, 1.0),
                TextAlign::Left,
            );
            row.modal_confirm_button("Spawn into Scene", 140.0);
        },
    );

    // 2. Interactive 3D Wireframe Viewport Canvas Container (Height: 280 px)
    ui.canvas(
        Style::new()
            .flex_col()
            .height(280.0)
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60))
            .clip_children(true),
        WidgetRole::OscilloscopeCanvas,
        crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_ORBIT,
        Some(WidgetCursor::Grab),
    );

    // 3. Model Metrics Summary Row
    ui.label(
        "Format: 3D glTF / GLB • Mesh Pipeline: Indexed PBR • Bounding Box: Normalized [-1.0 .. 1.0]",
        10.5,
        Color::rgba(0.60, 0.65, 0.75, 1.0),
        TextAlign::Left,
    );
}

/// Directly appends the 12 wireframe bounding box edges and 3 coordinate axis quads to the draw command list.
pub fn append_wireframe_quads(
    command_list: &mut DrawCommandList,
    canvas_rect: Rect,
    modal: &AssetPreviewModalState,
) {
    if canvas_rect.width <= 0.0 || canvas_rect.height <= 0.0 {
        return;
    }

    let cx = canvas_rect.x + canvas_rect.width * 0.5;
    let cy = canvas_rect.y + canvas_rect.height * 0.5;
    let scale = (85.0 / modal.zoom_distance).clamp(25.0, 180.0);

    // 8 Box corners in local 3D unit coordinates: `[-1..1, -1..1, -1..1]`
    let corners_3d = [
        (-1.0, -1.0, -1.0),
        (1.0, -1.0, -1.0),
        (1.0, 1.0, -1.0),
        (-1.0, 1.0, -1.0),
        (-1.0, -1.0, 1.0),
        (1.0, -1.0, 1.0),
        (1.0, 1.0, 1.0),
        (-1.0, 1.0, 1.0),
    ];

    let center = [cx, cy];
    let mut proj_points = [(0.0_f32, 0.0_f32); 8];
    for (i, &(x, y, z)) in corners_3d.iter().enumerate() {
        proj_points[i] =
            project_3d_point([x, y, z], modal.orbit_yaw, modal.orbit_pitch, center, scale);
    }

    // 12 Wireframe Box Edges
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // Bottom ring
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // Top ring
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // Vertical pillars
    ];

    if modal.show_wireframe {
        let edge_style = Style::new()
            .background(Color::rgba(0.0, 0.85, 1.0, 0.45))
            .border_radius(0.75);

        for &(p0_idx, p1_idx) in &edges {
            let (x0, y0) = proj_points[p0_idx];
            let (x1, y1) = proj_points[p1_idx];

            let min_x = x0.min(x1);
            let min_y = y0.min(y1);
            let w = (x1 - x0).abs().max(1.5);
            let h = (y1 - y0).abs().max(1.5);

            command_list.push_quad(QuadInstance::from_style(
                Rect::new(min_x, min_y, w, h),
                &edge_style,
                Some(canvas_rect),
            ));
        }
    }

    // Center Coordinate Axes Indicator
    let (ox, oy) = project_3d_point(
        [0.0, 0.0, 0.0],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );
    let (xx, xy) = project_3d_point(
        [0.6, 0.0, 0.0],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );
    let (yx, yy) = project_3d_point(
        [0.0, 0.6, 0.0],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );
    let (zx, zy) = project_3d_point(
        [0.0, 0.0, 0.6],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );

    // X Axis (Red)
    let x_style = Style::new().background(Color::rgba(1.0, 0.30, 0.30, 0.90));
    command_list.push_quad(QuadInstance::from_style(
        Rect::new(
            ox.min(xx),
            oy.min(xy),
            (xx - ox).abs().max(2.0),
            (xy - oy).abs().max(2.0),
        ),
        &x_style,
        Some(canvas_rect),
    ));

    // Y Axis (Green)
    let y_style = Style::new().background(Color::rgba(0.30, 1.0, 0.30, 0.90));
    command_list.push_quad(QuadInstance::from_style(
        Rect::new(
            ox.min(yx),
            oy.min(yy),
            (yx - ox).abs().max(2.0),
            (yy - oy).abs().max(2.0),
        ),
        &y_style,
        Some(canvas_rect),
    ));

    // Z Axis (Blue)
    let z_style = Style::new().background(Color::rgba(0.30, 0.60, 1.0, 0.90));
    command_list.push_quad(QuadInstance::from_style(
        Rect::new(
            ox.min(zx),
            oy.min(zy),
            (zx - ox).abs().max(2.0),
            (zy - oy).abs().max(2.0),
        ),
        &z_style,
        Some(canvas_rect),
    ));
}

/// Helper for projecting 3D point through yaw and pitch angles into 2D viewport coordinates.
fn project_3d_point(
    pt: [f32; 3],
    yaw: f32,
    pitch: f32,
    center: [f32; 2],
    scale: f32,
) -> (f32, f32) {
    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let x1 = pt[0] * cos_y - pt[2] * sin_y;
    let z1 = pt[0] * sin_y + pt[2] * cos_y;

    let cos_p = pitch.cos();
    let sin_p = pitch.sin();
    let y2 = pt[1] * cos_p - z1 * sin_p;
    let z2 = pt[1] * sin_p + z1 * cos_p;

    let proj_factor = scale / (1.0 + z2 * 0.18).max(0.2);
    (center[0] + x1 * proj_factor, center[1] - y2 * proj_factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_3d_point_invariants() {
        let center = [200.0, 150.0];
        let scale = 50.0;

        // Origin projects exactly to center
        let (ox, oy) = project_3d_point([0.0, 0.0, 0.0], 0.0, 0.0, center, scale);
        assert!((ox - center[0]).abs() < f32::EPSILON);
        assert!((oy - center[1]).abs() < f32::EPSILON);

        // Symmetric X displacement
        let (pos_x, _) = project_3d_point([1.0, 0.0, 0.0], 0.0, 0.0, center, scale);
        let (neg_x, _) = project_3d_point([-1.0, 0.0, 0.0], 0.0, 0.0, center, scale);
        assert!((pos_x - center[0] - (center[0] - neg_x)).abs() < 1e-4);

        // No NaN or Inf on extreme values
        let (ex, ey) = project_3d_point(
            [10.0, -10.0, 50.0],
            std::f32::consts::PI,
            -1.5,
            center,
            100.0,
        );
        assert!(ex.is_finite());
        assert!(ey.is_finite());
    }

    #[test]
    fn test_append_wireframe_quads_direct_command_list() {
        let canvas = Rect::new(50.0, 50.0, 300.0, 200.0);
        let mut modal = AssetPreviewModalState::default();

        // 1. With wireframe enabled: 12 edges + 3 axes = 15 quads
        let mut list_with_wireframe = DrawCommandList::new();
        append_wireframe_quads(&mut list_with_wireframe, canvas, &modal);
        assert_eq!(list_with_wireframe.quads.len(), 15);

        for quad in &list_with_wireframe.quads {
            assert!(quad.rect[0].is_finite());
            assert!(quad.rect[1].is_finite());
            assert!(quad.rect[2] >= 1.5);
            assert!(quad.rect[3] >= 1.5);
            assert_eq!(
                quad.clip_rect,
                [canvas.x, canvas.y, canvas.right(), canvas.bottom()]
            );
        }

        // 2. With wireframe disabled: only 3 coordinate axes quads
        modal.show_wireframe = false;
        let mut list_without_wireframe = DrawCommandList::new();
        append_wireframe_quads(&mut list_without_wireframe, canvas, &modal);
        assert_eq!(list_without_wireframe.quads.len(), 3);

        for quad in &list_without_wireframe.quads {
            assert!(quad.rect[0].is_finite());
            assert!(quad.rect[1].is_finite());
            assert_eq!(
                quad.clip_rect,
                [canvas.x, canvas.y, canvas.right(), canvas.bottom()]
            );
        }
    }
}