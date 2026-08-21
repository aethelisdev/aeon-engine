// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! AE Plugin API — Shared Types, Components, Events, and Interfaces.
//!

pub mod abi;
pub mod components;
pub mod context;
pub mod events;
pub mod resources;

// Re-export everything to maintain a clean, flat public API surface
pub use abi::*;
pub use components::*;
pub use context::*;
pub use events::*;
pub use resources::*;