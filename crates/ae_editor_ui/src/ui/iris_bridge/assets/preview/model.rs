// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Model Interactive Wireframe Orbit Viewport Builder
//!
//! Renders an interactive 3D bounding box wireframe canvas with projected 3D coordinates,
//! orbital yaw and pitch rotation, zoom distance scaling, and 3D coordinate axes
//! using declarative [`UiScope::immediate_canvas_with`] and `CanvasOrigin::NodeCenter`
//! primitives without render-time custom-draw hooks.

use crate::ui::iris_bridge::assets::types::AssetPreviewModalState;
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;

const CORNERS_3D: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
];

const WIREFRAME_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

const COORDINATE_AXES: [([f32; 3], Color); 3] = [
    ([0.6, 0.0, 0.0], Color::rgba(1.0, 0.30, 0.30, 0.90)),
    ([0.0, 0.6, 0.0], Color::rgba(0.30, 1.0, 0.30, 0.90)),
    ([0.0, 0.0, 0.6], Color::rgba(0.30, 0.60, 1.0, 0.90)),
];

/// Renders the 3D model inspection preview with interactive wireframe orbit canvas using declarative [`UiScope`].
pub(crate) fn render_model_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    // 1. Controls Bar: Instruction Hint + Spawn Action Button
    ui.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .height(28.0),
        |row| {
            row.colored_label(
                Color::rgba(0.55, 0.60, 0.72, 1.0),
                "Left Drag: 3D Orbit | Wheel: Zoom Distance",
            );
            row.modal_confirm_button("Spawn into Scene", 140.0);
        },
    );

    // 2. Interactive 3D Wireframe Viewport Canvas Container (Height: 280 px)
    ui.immediate_canvas_with(
        ImmediateCanvasSpec::new(
            "AssetPreviewOrbitCanvas",
            crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_ORBIT,
            Style::new()
                .flex_col()
                .height(280.0)
                .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
                .border_radius(6.0)
                .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60))
                .clip_children(true),
        )
        .origin(CanvasOrigin::NodeCenter)
        .cursor(WidgetCursor::Grab),
        |canvas| draw_wireframe_orbit_primitives(canvas, modal),
    );

    // 3. Model Metrics Summary Row
    ui.colored_label(
        Color::rgba(0.60, 0.65, 0.75, 1.0),
        "Format: 3D glTF / GLB • Mesh Pipeline: Indexed PBR • Bounding Box: Normalized [-1.0 .. 1.0]",
    );
}

#[inline]
fn orbit_scale(zoom_distance: f32) -> f32 {
    (85.0 / zoom_distance).clamp(25.0, 180.0)
}

#[inline]
fn line_segment_rect((x0, y0): (f32, f32), (x1, y1): (f32, f32), min_thickness: f32) -> Rect {
    Rect::new(
        x0.min(x1),
        y0.min(y1),
        (x1 - x0).abs().max(min_thickness),
        (y1 - y0).abs().max(min_thickness),
    )
}

/// Emits node-centered 12 wireframe bounding box edges and 3 coordinate axes into `canvas`.
fn draw_wireframe_orbit_primitives(
    canvas: &mut ImmediateCanvas<'_>,
    modal: &AssetPreviewModalState,
) {
    if modal.show_wireframe {
        for (p0_idx, p1_idx) in WIREFRAME_EDGES {
            canvas.fill_rect(
                line_segment_rect(
                    project_3d_point(
                        CORNERS_3D[p0_idx],
                        modal.orbit_yaw,
                        modal.orbit_pitch,
                        [0.0, 0.0],
                        orbit_scale(modal.zoom_distance),
                    ),
                    project_3d_point(
                        CORNERS_3D[p1_idx],
                        modal.orbit_yaw,
                        modal.orbit_pitch,
                        [0.0, 0.0],
                        orbit_scale(modal.zoom_distance),
                    ),
                    1.5,
                ),
                Color::rgba(0.0, 0.85, 1.0, 0.45),
                0.75,
            );
        }
    }

    for (tip, color) in COORDINATE_AXES {
        canvas.fill_rect(
            line_segment_rect(
                project_3d_point(
                    [0.0, 0.0, 0.0],
                    modal.orbit_yaw,
                    modal.orbit_pitch,
                    [0.0, 0.0],
                    orbit_scale(modal.zoom_distance),
                ),
                project_3d_point(
                    tip,
                    modal.orbit_yaw,
                    modal.orbit_pitch,
                    [0.0, 0.0],
                    orbit_scale(modal.zoom_distance),
                ),
                2.0,
            ),
            color,
            0.0,
        );
    }
}

/// Projects a local 3D point through yaw/pitch rotation around `center`.
fn project_3d_point(
    pt: [f32; 3],
    yaw: f32,
    pitch: f32,
    center: [f32; 2],
    scale: f32,
) -> (f32, f32) {
    project_rotated(
        pt[0] * yaw.cos() - pt[2] * yaw.sin(),
        pt[1] * pitch.cos() - (pt[0] * yaw.sin() + pt[2] * yaw.cos()) * pitch.sin(),
        pt[1] * pitch.sin() + (pt[0] * yaw.sin() + pt[2] * yaw.cos()) * pitch.cos(),
        center,
        scale,
    )
}

#[inline]
fn project_rotated(x1: f32, y2: f32, z2: f32, center: [f32; 2], scale: f32) -> (f32, f32) {
    (
        center[0] + x1 * (scale / (1.0 + z2 * 0.18).max(0.2)),
        center[1] - y2 * (scale / (1.0 + z2 * 0.18).max(0.2)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_3d_point_invariants() {
        let center = [200.0, 150.0];
        let scale = 50.0;

        let (ox, oy) = project_3d_point([0.0, 0.0, 0.0], 0.0, 0.0, center, scale);
        assert!((ox - center[0]).abs() < f32::EPSILON);
        assert!((oy - center[1]).abs() < f32::EPSILON);

        let (pos_x, _) = project_3d_point([1.0, 0.0, 0.0], 0.0, 0.0, center, scale);
        let (neg_x, _) = project_3d_point([-1.0, 0.0, 0.0], 0.0, 0.0, center, scale);
        assert!((pos_x - center[0] - (center[0] - neg_x)).abs() < 1e-4);

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
    fn test_render_model_preview_compiles_wireframe_and_axes_on_first_frame() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let mut modal = AssetPreviewModalState::default();

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.container(
                Style::new().flex_col().size(Size::new(400.0, 360.0)),
                |col| render_model_preview_content(col, &modal),
            );
        }
        let mut layout = LayoutEngine::new();
        let _ = layout.compute_layout(&mut tree, Size::new(400.0, 360.0));

        let orbit_id = tree
            .find_node_by_tag(crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_ORBIT)
            .expect("orbit canvas node");
        let orbit_rect = tree.get(orbit_id).expect("orbit node").computed_rect;
        assert!((orbit_rect.width - 400.0).abs() < f32::EPSILON);
        assert!((orbit_rect.height - 280.0).abs() < f32::EPSILON);

        let mut opts = irisui::wgpu_backend::TreeCompilerOptions::new();
        let cmd_with_wireframe =
            irisui::wgpu_backend::compile_tree_draw_commands(&tree, orbit_id, &mut opts);
        assert_eq!(cmd_with_wireframe.quads.len(), 16);
        let center_x = orbit_rect.x + orbit_rect.width * 0.5;
        let center_y = orbit_rect.y + orbit_rect.height * 0.5;
        for quad in &cmd_with_wireframe.quads[1..] {
            assert!(quad.rect[0] > orbit_rect.x && quad.rect[0] < orbit_rect.x + orbit_rect.width);
            assert!(quad.rect[1] > orbit_rect.y && quad.rect[1] < orbit_rect.y + orbit_rect.height);
            assert!((quad.rect[0] - center_x).abs() < 150.0);
            assert!((quad.rect[1] - center_y).abs() < 150.0);
        }

        modal.show_wireframe = false;
        let mut tree2 = UiTree::new();
        let root2 = tree2.create_root().expect("root2");
        {
            let mut scope2 = UiScope::new(&mut tree2, root2);
            scope2.container(
                Style::new().flex_col().size(Size::new(400.0, 360.0)),
                |col| render_model_preview_content(col, &modal),
            );
        }
        let _ = layout.compute_layout(&mut tree2, Size::new(400.0, 360.0));
        let orbit_id2 = tree2
            .find_node_by_tag(crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_ORBIT)
            .expect("orbit canvas node 2");
        let cmd_without_wireframe =
            irisui::wgpu_backend::compile_tree_draw_commands(&tree2, orbit_id2, &mut opts);
        assert_eq!(cmd_without_wireframe.quads.len(), 4);
    }
}