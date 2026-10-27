// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Window, Input, and Platform Event Arbitration Subsystem.
//!
//! Coordinates event translation, UI and 3D viewport input arbitration,
//! cursor lifecycle states, and hardware device interactions.

pub mod arbiter;
pub mod handlers;

#[cfg(test)]
mod tests;

pub use arbiter::{InputArbiter, PointerCapture};