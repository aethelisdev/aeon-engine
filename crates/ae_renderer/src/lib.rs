// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod asset;
pub mod camera;
pub mod graphics_settings;
pub mod math;
/// AE Renderer — Graphics Engine and Rendering Pipeline.
/// Handles WGPU-based 3D PBR rendering loop, shadow maps (CSM), MSAA,
/// bloom post-processing, and asset loading/VRAM management.
pub mod render;