// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::prelude::*;

#[test]
fn test_umbrella_facade_reexports() {
    let _cam = Camera2D::new(12.0);
    let _sprite = SpriteRenderer::new(None);
    let _anim = SpriteAnimation::from_grid(2, 2, 4, 8.0);
    let _rb = RigidBody2D::new(BodyType2D::Dynamic);
    let _col = Collider2D::cuboid(1.0, 1.0);
    let _mode = ActiveDimensionMode::Mode2D;
    assert!(_mode.is_2d());
}