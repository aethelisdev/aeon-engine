// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Virtualized List Declarative Widget Module
//!
//! Hosts the mathematical slice windowing engine and declarative widget API
//! for rendering arbitrarily large collections with constant $O(1)$ memory.

pub mod math;

pub use math::{
    DEFAULT_OVERSCAN, VirtualSlice, VirtualSliceParams, compute_slice, compute_virtual_slice,
};
