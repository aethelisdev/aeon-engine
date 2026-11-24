// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU draw call breakdown and VRAM memory allocation diagnostics.

/// Detailed breakdown of GPU draw calls, compute dispatches, and culling statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DrawCallBreakdown {
    /// Total aggregate draw calls dispatched to the GPU command encoder.
    pub total_draw_calls: u32,
    /// Primitive batched draw calls.
    pub batched_draw_calls: u32,
    /// Hardware instanced draw calls for repeated 3D meshes and entities.
    pub instanced_draw_calls: u32,
    /// Dispatched compute shader passes.
    pub dispatched_compute: u32,
    /// Meshes excluded from rendering by frustum culling or visibility masks.
    pub culled_meshes: u32,
    /// Total rendered triangle count for the active frame.
    pub triangles: u64,
    /// Total rendered vertex count for the active frame.
    pub vertices: u64,
}

impl DrawCallBreakdown {
    /// Returns the hardware instancing efficiency ratio (percentage of instanced draw calls).
    #[must_use]
    pub fn instancing_ratio_percent(&self) -> f32 {
        if self.total_draw_calls == 0 {
            0.0
        } else {
            (self.instanced_draw_calls as f32 / self.total_draw_calls as f32) * 100.0
        }
    }
}

/// Detailed breakdown of allocated Video RAM (VRAM) consumption across graphics subsystems.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VramStats {
    /// VRAM allocated for loaded 2D/3D textures.
    pub texture_vram_mb: f32,
    /// VRAM allocated for static and dynamic vertex/index buffers.
    pub mesh_index_vram_mb: f32,
    /// VRAM allocated for uniform buffers, instance storage, shadow maps, and render targets.
    pub dynamic_uniform_vram_mb: f32,
    /// Total combined VRAM allocated by the rendering engine.
    pub total_vram_mb: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_call_instancing_ratio_zero_division_guard() {
        let empty = DrawCallBreakdown::default();
        assert_eq!(empty.instancing_ratio_percent(), 0.0);

        let mixed = DrawCallBreakdown {
            total_draw_calls: 20,
            instanced_draw_calls: 15,
            ..Default::default()
        };
        assert!((mixed.instancing_ratio_percent() - 75.0).abs() < 1e-4);
    }
}