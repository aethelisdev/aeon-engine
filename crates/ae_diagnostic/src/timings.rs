// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! CPU synchronization stage timings and render pass command recording telemetry.

/// High-resolution CPU thread execution and synchronization stage timings (in milliseconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CpuSyncTimings {
    /// ECS system updates, entity hierarchy transformations, and gameplay logic.
    pub main_logic_ms: f32,
    /// Physics simulation stepping, kinematic velocities, and spatial broadphase queries.
    pub physics_ms: f32,
    /// Render preparation, instance buffer staging, and command encoder recording.
    pub render_prep_ms: f32,
    /// Wall-clock time spent waiting on GPU presentation.
    pub wait_for_gpu_ms: f32,
    /// Editor UI layout calculation, widget tessellation, and event handling.
    pub ui_editor_ms: f32,
    /// Total combined CPU frame execution duration.
    pub total_cpu_ms: f32,
}

impl CpuSyncTimings {
    /// Determines whether the frame bottleneck is currently CPU-bound or GPU-bound.
    #[must_use]
    pub fn is_cpu_bound(&self, total_record_ms: f32) -> bool {
        if !total_record_ms.is_finite() {
            return false;
        }
        let active_cpu = self.main_logic_ms + self.physics_ms + self.render_prep_ms;
        active_cpu > total_record_ms && self.wait_for_gpu_ms < 2.0
    }
}

/// Render pass command recording and dispatch durations on the CPU (in milliseconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RenderPassRecordingTimings {
    /// Cascaded directional shadow map pass CPU command recording duration in milliseconds.
    pub shadow_pass_record_ms: f32,
    /// Main forward opaque, transparent, and overlay scene geometry CPU recording duration.
    pub main_opaque_pass_record_ms: f32,
    /// Post-processing bloom and selection outline pass CPU recording duration.
    pub post_process_pass_record_ms: f32,
    /// Iris UI composite and draw command recording duration in milliseconds.
    pub ui_pass_record_ms: f32,
    /// Total aggregated CPU command buffer recording and pass orchestration duration.
    pub total_record_ms: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cpu_bound_detection_and_nan_guard() {
        let cpu_heavy = CpuSyncTimings {
            main_logic_ms: 4.0,
            physics_ms: 3.0,
            render_prep_ms: 2.0,
            wait_for_gpu_ms: 0.5,
            ..Default::default()
        };
        assert!(cpu_heavy.is_cpu_bound(5.0));
        assert!(!cpu_heavy.is_cpu_bound(f32::NAN));

        let gpu_stalled = CpuSyncTimings {
            wait_for_gpu_ms: 6.0,
            ..cpu_heavy
        };
        assert!(!gpu_stalled.is_cpu_bound(5.0));
    }
}