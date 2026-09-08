// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame Rendering Pipeline Sub-module.
//!
//! Coordinates the frame rendering lifecycle: viewport resizing, shadow cascades,
//! main forward geometry passes, post-processing bloom, selection outlines, and Iris UI.
//!

pub mod forward;
pub mod outline;
pub mod pipeline;

pub use forward::*;
pub use pipeline::*;