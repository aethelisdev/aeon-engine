// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Plain-old-data mesh and skinning vertex representations.
//!
//! These structs carry zero graphics API dependencies so asset cookers,
//! physics exporters, and scene loaders can share exact byte layouts without
//! linking a GPU backend.

/// PBR 3D mesh vertex containing object-space position, RGB vertex color,
/// surface normal, and primary UV coordinates (44 bytes, C-layout POD).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Object-space vertex position `[x, y, z]`.
    pub position: [f32; 3],
    /// Linear RGB vertex color multiplier `[r, g, b]`.
    pub color: [f32; 3],
    /// Object-space surface normal `[nx, ny, nz]`.
    pub normal: [f32; 3],
    /// Primary texture coordinates `[u, v]`.
    pub uv: [f32; 2],
}

/// 3D billboard or quad vertex containing position, UV coordinates, and normal
/// (32 bytes, C-layout POD).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    /// Local vertex position `[x, y, z]`.
    pub position: [f32; 3],
    /// Texture coordinates `[u, v]`.
    pub uv: [f32; 2],
    /// Surface normal `[nx, ny, nz]`.
    pub normal: [f32; 3],
}

/// Skeletal bind-pose and joint influence payload paired per vertex with [`Vertex`].
///
/// Stores the unmodified bind-pose position and normal alongside up to four
/// influencing joint indices and normalized blend weights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkinVertex {
    /// Unskinned bind-pose vertex position `[x, y, z]`.
    pub bind_position: [f32; 3],
    /// Unskinned bind-pose surface normal `[nx, ny, nz]`.
    pub bind_normal: [f32; 3],
    /// Up to four skeleton joint indices affecting this vertex.
    pub joint_indices: [u32; 4],
    /// Blend weights corresponding to `joint_indices`.
    pub joint_weights: [f32; 4],
}