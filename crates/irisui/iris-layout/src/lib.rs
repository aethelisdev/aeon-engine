// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Layout Engine (`iris-layout`)
//!
//! Adapts the Taffy Flexbox & CSS Grid layout engine to the Retained-Mode `UiTree`,
//! providing dirty-state caching for zero-overhead layout recalculations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod engine;
pub mod error;

#[cfg(test)]
mod tests;

pub use engine::LayoutEngine;
pub use error::LayoutError;

/// Layout engine prelude re-exporting the layout manager and error types.
///
/// Designed to be imported via wildcard `use iris_layout::prelude::*;` or through the `irisui` facade.
pub mod prelude {
    pub use crate::{LayoutEngine, LayoutError};
}