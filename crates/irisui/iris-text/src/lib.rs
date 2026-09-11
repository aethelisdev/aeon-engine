// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Typography Engine (`iris-text`)
//!
//! Subpixel GPU text rendering, font shaping, and glyph caching engine for Iris UI.
//! Powered by `cosmic-text` and `glyphon`.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod renderer;
pub mod section;
pub mod system;

pub use renderer::{TextPrepareParams, TextRenderer};
pub use section::TextSection;
pub use system::TextSystem;

#[cfg(test)]
mod tests;