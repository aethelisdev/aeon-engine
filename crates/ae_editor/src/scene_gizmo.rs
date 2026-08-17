// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use cgmath::{Point3, Rad};
use std::f32::consts::FRAC_PI_2;

/// Defines predefined camera orientation snap views for the 3D Scene Navigation Gizmo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneViewSnap {
    Top,
    Bottom,
    Front,
    Back,
    Right,
    Left,
    Perspective,
}

impl SceneViewSnap {
    /// Computes target camera pitch, yaw, and position offset for the given view snap.
    /// # Parameters
    /// - `target`: The 3D orbit target point of the camera (`Point3<f32>`).
    /// - `distance`: Distance from target point.
    pub fn compute_transform(
        &self,
        target: Point3<f32>,
        distance: f32,
    ) -> (Rad<f32>, Rad<f32>, Point3<f32>) {
        let d = distance.max(1.0);
        match self {
            SceneViewSnap::Top => (
                Rad(-FRAC_PI_2 + 0.001),
                Rad(0.0),
                Point3::new(target.x, target.y + d, target.z),
            ),
            SceneViewSnap::Bottom => (
                Rad(FRAC_PI_2 - 0.001),
                Rad(0.0),
                Point3::new(target.x, target.y - d, target.z),
            ),
            SceneViewSnap::Front => (
                Rad(0.0),
                Rad(FRAC_PI_2),
                Point3::new(target.x, target.y, target.z - d),
            ),
            SceneViewSnap::Back => (
                Rad(0.0),
                Rad(-FRAC_PI_2),
                Point3::new(target.x, target.y, target.z + d),
            ),
            SceneViewSnap::Right => (
                Rad(0.0),
                Rad(0.0),
                Point3::new(target.x + d, target.y, target.z),
            ),
            SceneViewSnap::Left => (
                Rad(0.0),
                Rad(std::f32::consts::PI),
                Point3::new(target.x - d, target.y, target.z),
            ),
            SceneViewSnap::Perspective => (
                Rad(-0.4),
                Rad(0.6),
                Point3::new(target.x + d * 0.7, target.y + d * 0.5, target.z + d * 0.7),
            ),
        }
    }
}

/// 3D Scene Viewport Navigation Gizmo orientation axes calculations.
/// Projects world axes (X-Red, Y-Green, Z-Blue) into 2D viewport coordinates
/// and provides click-hit testing for snapping camera views.
pub struct SceneNavigationGizmo;

impl SceneNavigationGizmo {
    /// Computes 2D viewport screen positions for 3D orientation axis endpoints sorted from back to front.
    /// # Parameters
    /// - `pitch`: Active camera pitch angle (`f32` radians).
    /// - `yaw`: Active camera yaw angle (`f32` radians).
    /// - `radius`: Visual pixel radius of the gizmo compass ring.
    /// Returns an array of `(dx, dy, label, color_rgb, is_positive)` sorted back-to-front by view depth.
    pub fn compute_axis_endpoints(
        pitch: f32,
        yaw: f32,
        radius: f32,
    ) -> [(f32, f32, &'static str, [u8; 3], bool); 6] {
        let (sin_p, cos_p) = pitch.sin_cos();
        let (sin_y, cos_y) = yaw.sin_cos();

        // 3D camera view matrix direction vectors
        // Right vector (X)
        let rx = cos_y;
        let ry = 0.0;
        let rz = -sin_y;

        // Up vector (Y)
        let ux = -sin_p * sin_y;
        let uy = cos_p;
        let uz = -sin_p * cos_y;

        // Forward vector (Z)
        let fx = cos_p * sin_y;
        let fy = sin_p;
        let fz = cos_p * cos_y;

        // Project world axes (X, Y, Z, -X, -Y, -Z) onto camera right (screen X) and up (screen Y)
        let x_screen_x = rx;
        let x_screen_y = ux;
        let x_depth = fx;

        let y_screen_x = ry;
        let y_screen_y = uy;
        let y_depth = fy;

        let z_screen_x = rz;
        let z_screen_y = uz;
        let z_depth = fz;

        let mut axes = [
            (
                x_screen_x * radius,
                -x_screen_y * radius,
                "X",
                [235, 70, 70],
                true,
                x_depth,
            ), // +X (Red)
            (
                y_screen_x * radius,
                -y_screen_y * radius,
                "Y",
                [70, 215, 70],
                true,
                y_depth,
            ), // +Y (Green)
            (
                z_screen_x * radius,
                -z_screen_y * radius,
                "Z",
                [70, 140, 245],
                true,
                z_depth,
            ), // +Z (Blue)
            (
                -x_screen_x * radius,
                x_screen_y * radius,
                "-X",
                [160, 70, 70],
                false,
                -x_depth,
            ), // -X (Dark Red)
            (
                -y_screen_x * radius,
                y_screen_y * radius,
                "-Y",
                [70, 160, 70],
                false,
                -y_depth,
            ), // -Y (Dark Green)
            (
                -z_screen_x * radius,
                z_screen_y * radius,
                "-Z",
                [70, 90, 180],
                false,
                -z_depth,
            ), // -Z (Dark Blue)
        ];

        // Sort by depth back-to-front (lowest depth first, highest depth last)
        axes.sort_by(|a, b| a.5.total_cmp(&b.5));

        axes.map(|(dx, dy, label, color, is_pos, _depth)| (dx, dy, label, color, is_pos))
    }
}