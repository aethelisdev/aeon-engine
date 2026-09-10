// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use cgmath::Matrix4;
use glam::Vec2;

use crate::camera::Camera2D;
use crate::components::{SortMode, SpriteRenderer, SpriteSortKey};
use crate::mode::ActiveDimensionMode;

#[test]
fn test_sprite_renderer_defaults_and_uv_flips() {
    let mut sprite = SpriteRenderer::new(None);
    assert_eq!(sprite.uv_rect, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(sprite.effective_uvs(), [0.0, 0.0, 1.0, 1.0]);

    sprite.flip_x = true;
    assert_eq!(sprite.effective_uvs(), [1.0, 0.0, 0.0, 1.0]);

    sprite.flip_y = true;
    assert_eq!(sprite.effective_uvs(), [1.0, 1.0, 0.0, 0.0]);

    sprite.flip_x = false;
    assert_eq!(sprite.effective_uvs(), [0.0, 1.0, 1.0, 0.0]);
}

#[test]
fn test_sprite_sort_key_ordering() {
    let bg_key = SpriteSortKey::from_layer_order_texture(-10, 0, 1);
    let mid_key_0 = SpriteSortKey::from_layer_order_texture(0, 0, 1);
    let mid_key_1 = SpriteSortKey::from_layer_order_texture(0, 1, 1);
    let fg_key = SpriteSortKey::from_layer_order_texture(10, 0, 1);

    assert!(bg_key < mid_key_0);
    assert!(mid_key_0 < mid_key_1);
    assert!(mid_key_1 < fg_key);
}

#[test]
fn test_sprite_y_sorting() {
    let tree_top = SpriteSortKey::from_y_sort(0, 10.0, 1);
    let tree_bottom = SpriteSortKey::from_y_sort(0, -5.0, 1);

    assert!(tree_top < tree_bottom);
}

#[test]
fn test_camera_2d_orthographic_matrix_validity() {
    let cam = Camera2D::new(10.0).with_zoom(2.0);
    let matrix = cam.build_view_projection(Vec2::new(5.0, 5.0), 16.0 / 9.0);

    assert_ne!(matrix, Matrix4::from_scale(0.0));
    assert!(matrix.x.x.is_finite());
    assert!(matrix.w.w.is_finite());
}

#[test]
fn test_camera_2d_target_follow_deadzone() {
    let mut cam = Camera2D::new(10.0);
    cam.deadzone = [2.0, 2.0];
    cam.smooth_speed = 0.0;

    let current = Vec2::new(0.0, 0.0);

    let inside_target = Vec2::new(1.0, 1.5);
    let follow_inside = cam.update_follow(current, inside_target, 0.016);
    assert_eq!(follow_inside, current);

    let outside_target = Vec2::new(5.0, 0.0);
    let follow_outside = cam.update_follow(current, outside_target, 0.016);
    assert_eq!(follow_outside, Vec2::new(3.0, 0.0));
}

#[test]
fn test_active_dimension_mode() {
    let mode_2d = ActiveDimensionMode::Mode2D;
    assert!(mode_2d.is_2d());
    assert!(!mode_2d.is_3d());

    let mode_3d = ActiveDimensionMode::Mode3D;
    assert!(mode_3d.is_3d());
    assert!(!mode_3d.is_2d());

    assert_eq!(SortMode::default(), SortMode::LayerAndOrder);
}