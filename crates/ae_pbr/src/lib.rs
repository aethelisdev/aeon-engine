// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU 3D PBR, cascaded shadow mapping, procedural sky, infinite ground grid, and sprite
//! pipelines alongside WebGPU scene uniform managers and vertex/instance buffer layouts.

pub mod grid;
pub mod manager;
pub mod pbr;
pub mod shadow_pipeline;
pub mod shadow_system;
pub mod sky;
pub mod sprite;
pub mod uniforms;
pub mod vertex;

pub use manager::{PipelineConfigParams, PipelineManager};
pub use shadow_system::{ShadowCascadePass, ShadowSystem};
pub use uniforms::SceneUniforms;
pub use vertex::{
    INSTANCE_SIZE, Instance, PBR_DEPTH_FORMAT, RenderInstanceItem, sprite_vertex_buffer_layout,
    vertex_buffer_layout,
};