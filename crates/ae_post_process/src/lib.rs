// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU post-processing passes, intermediate render target management, bloom/tonemapping,
//! and screen-space selection silhouette outlining for Aeon Engine.

pub mod bloom;
pub mod outline;
pub mod system;
pub mod targets;

pub use outline::{OutlineMaskPassRecorder, OutlineUniforms, SelectionOutlinePass};
pub use system::PostProcessSystem;
pub use targets::{
    POST_PROCESS_DEPTH_FORMAT, estimate_post_process_vram_bytes, sanitize_bloom_intensity,
    sanitize_msaa_samples,
};