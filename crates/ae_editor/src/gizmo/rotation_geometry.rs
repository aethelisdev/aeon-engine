// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
//! Procedural geometry generation for dynamic, view-aligned rotation gizmo handles.
//!
//! Provides:
//! - Camera-facing 180-degree front semi-torus arcs during idle viewport navigation.
//! - Complete 360-degree closed torus ring for the active handle during interactive rotation drag.
//! - Camera-aligned outer screen rotation ring for view-plane angular manipulation.
//! - Persistent visibility across all view angles including edge-on 90-degree alignment.
//!

use cgmath::{InnerSpace, Rotation as _, Vector3};

use super::core::{ActiveAxis, GizmoSystem};
use super::render::GizmoVertex;

/// Parameters describing procedural construction of a rotation torus ring.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TorusRingDescriptor {
    /// Radius from the gizmo center to the center of the ring tube.
    pub major_radius: f32,
    /// Cross-sectional radius of the ring tube.
    pub minor_radius: f32,
    /// Number of circumferential segments around the major ring.
    pub segments: usize,
    /// Number of segments around the minor tube cross-section.
    pub cross_segments: usize,
    /// RGB color of the ring handle.
    pub base_color: [f32; 3],
    /// Rotation plane axis (`ActiveAxis::X`, `Y`, or `Z`).
    pub axis: ActiveAxis,
    /// True if rendering the full 360-degree circle (during drag), false for 180-degree front arc.
    pub is_full_360: bool,
    /// Normalized vector pointing from gizmo origin toward the camera in local space.
    pub view_dir_local: Vector3<f32>,
}

impl GizmoSystem {
    /// Generates a torus ring handle: either a 180-degree front-facing arc or a full 360-degree circle during drag.
    /// When idle, generates a clean 180-degree arc facing the camera that never vanishes even when viewed edge-on.
    /// When dragging, generates a complete 360-degree circle for the active handle being rotated.
    /// ### Arguments
    /// * `params` - Geometry parameters and orientation descriptor.
    pub(crate) fn build_torus_ring(params: &TorusRingDescriptor) -> Vec<GizmoVertex> {
        let normal = match params.axis {
            ActiveAxis::X => Vector3::unit_x(),
            ActiveAxis::Y => Vector3::unit_y(),
            ActiveAxis::Z => Vector3::unit_z(),
            _ => Vector3::unit_z(),
        };

        let color = [
            params.base_color[0],
            params.base_color[1],
            params.base_color[2],
            1.0,
        ];

        // Project view direction onto the circle plane to find the center of the front 180-degree arc
        let proj = params.view_dir_local - normal * params.view_dir_local.dot(normal);
        let u = if proj.magnitude2() > 0.0001 {
            proj.normalize()
        } else {
            match params.axis {
                ActiveAxis::X => Vector3::unit_y(),
                ActiveAxis::Y => Vector3::unit_z(),
                ActiveAxis::Z => Vector3::unit_x(),
                _ => Vector3::unit_x(),
            }
        };
        let v = normal.cross(u).normalize();

        let mut vertices = Vec::with_capacity(params.segments * params.cross_segments * 6);

        let (arc_start, arc_span) = if params.is_full_360 {
            (0.0, std::f32::consts::TAU)
        } else {
            (-std::f32::consts::FRAC_PI_2, std::f32::consts::PI)
        };

        for i in 0..params.segments {
            let a1 = arc_start + (i as f32) / (params.segments as f32) * arc_span;
            let a2 = arc_start + ((i + 1) as f32) / (params.segments as f32) * arc_span;

            let (sin1, cos1) = a1.sin_cos();
            let (sin2, cos2) = a2.sin_cos();

            let spine1 = (u * cos1 + v * sin1) * params.major_radius;
            let spine2 = (u * cos2 + v * sin2) * params.major_radius;

            let b1_1 = (u * cos1 + v * sin1).normalize();
            let b1_2 = (u * cos2 + v * sin2).normalize();
            let b2 = normal;

            for j in 0..params.cross_segments {
                let ca1 = (j as f32) / (params.cross_segments as f32) * std::f32::consts::TAU;
                let ca2 = ((j + 1) as f32) / (params.cross_segments as f32) * std::f32::consts::TAU;

                let (csin1, ccos1) = ca1.sin_cos();
                let (csin2, ccos2) = ca2.sin_cos();

                let p1 = spine1 + (b1_1 * ccos1 + b2 * csin1) * params.minor_radius;
                let p2 = spine2 + (b1_2 * ccos1 + b2 * csin1) * params.minor_radius;
                let p3 = spine2 + (b1_2 * ccos2 + b2 * csin2) * params.minor_radius;
                let p4 = spine1 + (b1_1 * ccos2 + b2 * csin2) * params.minor_radius;

                let v_fn = |p: Vector3<f32>| GizmoVertex {
                    position: [p.x, p.y, p.z],
                    color,
                    uv: [0.0, 0.0],
                };
                vertices.extend_from_slice(&[
                    v_fn(p1),
                    v_fn(p2),
                    v_fn(p3),
                    v_fn(p1),
                    v_fn(p3),
                    v_fn(p4),
                ]);
            }
        }

        vertices
    }

    /// Generates the outer camera-facing screen-space rotation ring (Feature B).
    /// The ring is constructed along the camera's right and up vectors, ensuring it remains
    /// a circular handle from any perspective or orthographic view angle.
    /// ### Arguments
    /// * `major_radius` - Major radius of the outer screen ring.
    /// * `minor_radius` - Tube thickness of the screen ring.
    /// * `segments` - Circumferential subdivisions.
    /// * `cross_segments` - Cross-sectional subdivisions around the tube.
    pub(crate) fn build_screen_rotation_ring(
        &self,
        major_radius: f32,
        minor_radius: f32,
        segments: usize,
        cross_segments: usize,
    ) -> Vec<GizmoVertex> {
        let mut right = self.cam_right.get();
        let mut up = self.cam_up.get();

        if self.space == super::space::GizmoSpace::Local {
            let inv_rot = self.entity_rotation.conjugate();
            right = inv_rot.rotate_vector(right);
            up = inv_rot.rotate_vector(up);
        }

        let right = if right.magnitude2() > 0.0001 {
            right.normalize()
        } else {
            Vector3::unit_x()
        };
        let up = if up.magnitude2() > 0.0001 {
            up.normalize()
        } else {
            Vector3::unit_y()
        };
        let normal = right.cross(up).normalize();

        let color = if self.is_dragging && self.active_axis == ActiveAxis::Screen {
            [1.0, 0.9, 0.2, 0.95] // High-visibility yellow when actively rotating
        } else if self.hovered_axis == ActiveAxis::Screen {
            [1.0, 1.0, 1.0, 0.9] // White when hovered
        } else {
            [0.75, 0.75, 0.8, 0.55] // Semi-transparent clean outer guide
        };

        let mut vertices = Vec::with_capacity(segments * cross_segments * 6);

        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;

            let (sin1, cos1) = a1.sin_cos();
            let (sin2, cos2) = a2.sin_cos();

            let spine1 = (right * cos1 + up * sin1) * major_radius;
            let spine2 = (right * cos2 + up * sin2) * major_radius;

            let b1_1 = (right * cos1 + up * sin1).normalize();
            let b1_2 = (right * cos2 + up * sin2).normalize();
            let b2 = normal;

            for j in 0..cross_segments {
                let ca1 = (j as f32) / (cross_segments as f32) * std::f32::consts::TAU;
                let ca2 = ((j + 1) as f32) / (cross_segments as f32) * std::f32::consts::TAU;

                let (csin1, ccos1) = ca1.sin_cos();
                let (csin2, ccos2) = ca2.sin_cos();

                let p1 = spine1 + (b1_1 * ccos1 + b2 * csin1) * minor_radius;
                let p2 = spine2 + (b1_2 * ccos1 + b2 * csin1) * minor_radius;
                let p3 = spine2 + (b1_2 * ccos2 + b2 * csin2) * minor_radius;
                let p4 = spine1 + (b1_1 * ccos2 + b2 * csin2) * minor_radius;

                let v_fn = |p: Vector3<f32>| GizmoVertex {
                    position: [p.x, p.y, p.z],
                    color,
                    uv: [0.0, 0.0],
                };
                vertices.extend_from_slice(&[
                    v_fn(p1),
                    v_fn(p2),
                    v_fn(p3),
                    v_fn(p1),
                    v_fn(p3),
                    v_fn(p4),
                ]);
            }
        }

        vertices
    }

    /// Generates complete dynamic view-aligned rotation ring geometry.
    /// Behavior:
    /// - During drag: Active handle becomes a full 360-degree circle so the user sees the complete trajectory.
    /// - When idle: Generates clean 180-degree front arcs for X, Y, Z (never vanishing) plus outer screen ring.
    /// ### Arguments
    /// * `radius` - Major radius of the orthogonal rotation rings.
    pub(crate) fn build_dynamic_rotation_vertices(&self, radius: f32) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let thickness = radius * 0.04;

        let cam_fwd = self.cam_forward.get();
        let view_dir_world = if cam_fwd.magnitude2() > 0.0001 {
            -cam_fwd.normalize()
        } else {
            Vector3::unit_z()
        };

        let view_dir_local = if self.space == super::space::GizmoSpace::Local {
            self.entity_rotation
                .conjugate()
                .rotate_vector(view_dir_world)
                .normalize()
        } else {
            view_dir_world
        };

        let red = if self.is_dragging && self.active_axis == ActiveAxis::X {
            [1.0, 0.25, 0.25] // Vibrant axis red when actively rotating X
        } else if self.hovered_axis == ActiveAxis::X {
            [1.0, 0.85, 0.2] // Golden yellow hover indicator
        } else {
            [0.9, 0.2, 0.2] // Base axis red
        };
        let green = if self.is_dragging && self.active_axis == ActiveAxis::Y {
            [0.25, 1.0, 0.25] // Vibrant axis green when actively rotating Y
        } else if self.hovered_axis == ActiveAxis::Y {
            [1.0, 0.85, 0.2] // Golden yellow hover indicator
        } else {
            [0.2, 0.85, 0.2] // Base axis green
        };
        let blue = if self.is_dragging && self.active_axis == ActiveAxis::Z {
            [0.25, 0.5, 1.0] // Vibrant axis blue when actively rotating Z
        } else if self.hovered_axis == ActiveAxis::Z {
            [1.0, 0.85, 0.2] // Golden yellow hover indicator
        } else {
            [0.2, 0.45, 1.0] // Base axis blue
        };

        if self.is_dragging && self.active_axis != ActiveAxis::None {
            // When actively rotating: show the full 360-degree circle for the active handle!
            match self.active_axis {
                ActiveAxis::X => {
                    vertices.extend(Self::build_torus_ring(&TorusRingDescriptor {
                        major_radius: radius,
                        minor_radius: thickness,
                        segments: 64,
                        cross_segments: 8,
                        base_color: red,
                        axis: ActiveAxis::X,
                        is_full_360: true,
                        view_dir_local,
                    }));
                }
                ActiveAxis::Y => {
                    vertices.extend(Self::build_torus_ring(&TorusRingDescriptor {
                        major_radius: radius,
                        minor_radius: thickness,
                        segments: 64,
                        cross_segments: 8,
                        base_color: green,
                        axis: ActiveAxis::Y,
                        is_full_360: true,
                        view_dir_local,
                    }));
                }
                ActiveAxis::Z => {
                    vertices.extend(Self::build_torus_ring(&TorusRingDescriptor {
                        major_radius: radius,
                        minor_radius: thickness,
                        segments: 64,
                        cross_segments: 8,
                        base_color: blue,
                        axis: ActiveAxis::Z,
                        is_full_360: true,
                        view_dir_local,
                    }));
                }
                ActiveAxis::Screen => {
                    vertices.extend(self.build_screen_rotation_ring(
                        radius * 1.15,
                        thickness * 0.6,
                        64,
                        8,
                    ));
                }
                _ => {}
            }
        } else {
            // When idle: show 180-degree front arcs for all axes (never vanishing) + outer screen ring
            vertices.extend(Self::build_torus_ring(&TorusRingDescriptor {
                major_radius: radius,
                minor_radius: thickness,
                segments: 32,
                cross_segments: 8,
                base_color: red,
                axis: ActiveAxis::X,
                is_full_360: false,
                view_dir_local,
            }));
            vertices.extend(Self::build_torus_ring(&TorusRingDescriptor {
                major_radius: radius,
                minor_radius: thickness,
                segments: 32,
                cross_segments: 8,
                base_color: green,
                axis: ActiveAxis::Y,
                is_full_360: false,
                view_dir_local,
            }));
            vertices.extend(Self::build_torus_ring(&TorusRingDescriptor {
                major_radius: radius,
                minor_radius: thickness,
                segments: 32,
                cross_segments: 8,
                base_color: blue,
                axis: ActiveAxis::Z,
                is_full_360: false,
                view_dir_local,
            }));
            vertices.extend(self.build_screen_rotation_ring(radius * 1.15, thickness * 0.6, 48, 8));
        }

        vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torus_ring_arc_generation() {
        let view_dir = Vector3::new(0.0, 0.0, 1.0);
        let verts = GizmoSystem::build_torus_ring(&TorusRingDescriptor {
            major_radius: 1.0,
            minor_radius: 0.04,
            segments: 16,
            cross_segments: 4,
            base_color: [1.0, 0.0, 0.0],
            axis: ActiveAxis::Z,
            is_full_360: false,
            view_dir_local: view_dir,
        });
        assert!(!verts.is_empty());
        assert_eq!(verts.len(), 16 * 4 * 6);
        for v in &verts {
            assert!(v.position[0].is_finite());
            assert!(v.position[1].is_finite());
            assert!(v.position[2].is_finite());
            assert!(v.color[3] > 0.5);
        }
    }

    #[test]
    fn test_torus_ring_full_360_generation() {
        let view_dir = Vector3::new(0.0, 1.0, 0.0);
        let verts = GizmoSystem::build_torus_ring(&TorusRingDescriptor {
            major_radius: 1.0,
            minor_radius: 0.04,
            segments: 32,
            cross_segments: 4,
            base_color: [0.0, 1.0, 0.0],
            axis: ActiveAxis::Y,
            is_full_360: true,
            view_dir_local: view_dir,
        });
        assert!(!verts.is_empty());
        assert_eq!(verts.len(), 32 * 4 * 6);
        for v in &verts {
            assert!(v.position[0].is_finite());
            assert!(v.position[1].is_finite());
            assert!(v.position[2].is_finite());
            assert!(v.color[3] > 0.5);
        }
    }

    #[test]
    fn test_edge_on_persistent_visibility() {
        // Looking along Y axis directly edge-on to the X-axis ring (normal = X, plane = YZ)
        // normal = [1, 0, 0], view_dir = [0, 1, 0] => dot = 0.0
        let view_dir = Vector3::new(0.0, 1.0, 0.0);
        let verts = GizmoSystem::build_torus_ring(&TorusRingDescriptor {
            major_radius: 1.0,
            minor_radius: 0.04,
            segments: 16,
            cross_segments: 4,
            base_color: [1.0, 0.0, 0.0],
            axis: ActiveAxis::X,
            is_full_360: false,
            view_dir_local: view_dir,
        });
        // Rings must remain persistently visible even when viewed edge-on (never suppressed)
        assert!(!verts.is_empty());
        assert_eq!(verts.len(), 16 * 4 * 6);
    }
}