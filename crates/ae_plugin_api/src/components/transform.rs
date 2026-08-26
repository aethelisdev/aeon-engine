// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Transform and spatial hierarchy ECS components.
//!

use serde::{Deserialize, Serialize};

/// 3D world-space position component for ECS entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    /// Creates a new `Position` with the given coordinates.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns the origin position `(0.0, 0.0, 0.0)`.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}

/// Quaternion-based rotation component for orientation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Rotation {
    /// Returns the identity quaternion (no rotation: `w=1, x=y=z=0`).
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::identity()
    }
}

/// Non-uniform scale component for entity transform.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Scale {
    /// Creates a new `Scale` with the given x, y, z scale factors.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns unit scale `(1.0, 1.0, 1.0)`.
    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::one()
    }
}

/// Parent entity reference for hierarchical transforms and scene graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parent(pub hecs::Entity);

impl Default for Parent {
    fn default() -> Self {
        Self(hecs::Entity::DANGLING)
    }
}

/// Child entity list for hierarchical transforms and scene graphs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Children(pub Vec<hecs::Entity>);

/// Cached world-space transform matrix for hierarchical transforms, picking, and rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub cgmath::Matrix4<f32>);

/// Marker component flagging entities whose transform changed this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformDirty;