// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! UI action processor and engine mutation dispatcher subsystem.
//!

pub mod components;
pub mod dispatcher;
pub mod scene_io;
pub mod spawning;
pub mod system;
pub mod transform;

pub use dispatcher::*;