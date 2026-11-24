// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU uniform constants retained inside the renderer crate.
//!
//! Scene lighting, cascade shadow, and procedural sky uniform layouts live in `ae_light`.

/// Default depth stencil format across the rendering pipeline.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;