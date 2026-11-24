// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Engine simulation timing, fixed-timestep accumulation, and frame pacing primitives.

pub mod clock;
pub mod pacing;

pub use clock::Time;
pub use pacing::{FramePacer, compute_frame_pacing_sleep};