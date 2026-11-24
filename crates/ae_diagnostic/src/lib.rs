// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Engine diagnostics, zero-allocation frame pacing buffers, CPU/GPU telemetry, and profiler.

pub mod frame_pacing;
pub mod gpu_stats;
pub mod profiler;
pub mod timings;

pub use frame_pacing::{FramePacingStats, FrameRingBuffer};
pub use gpu_stats::{DrawCallBreakdown, VramStats};
pub use profiler::Profiler;
pub use timings::{CpuSyncTimings, RenderPassRecordingTimings};