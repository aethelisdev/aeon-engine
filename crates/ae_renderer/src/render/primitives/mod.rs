// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Primitives Sub-module — Manages static mesh vertex data, procedural mesh generators, and GeometrySystem GPU buffers.
//!

pub mod data;
pub mod generators;
pub mod system;

pub use data::*;
pub use generators::*;
pub use system::*;