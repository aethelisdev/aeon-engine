// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor window event routing, pointer capture arbitration, and viewport interaction handlers.

pub mod arbiter;
pub mod handlers;
#[cfg(test)]
pub mod tests;

pub use arbiter::{InputArbiter, PointerCapture};