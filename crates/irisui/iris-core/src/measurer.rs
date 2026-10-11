// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Measurement & Dynamic Typography Abstraction
//!
//! Provides the [`TextMeasurer`] trait decoupled from specific font shaping engines,
//! allowing UI widgets and layout solvers to query dynamic font metrics and horizontal advance.
//!

/// Trait for querying dynamic glyph advances and text dimensions at runtime.
///
/// Implemented by typography engines such as `iris-text::TextSystem`.
pub trait TextMeasurer {
    /// Measures the exact horizontal advance width of a string in physical pixels
    /// at the given font size.
    ///
    /// # Arguments
    /// * `text` - Text string to shape and measure.
    /// * `font_size` - Font size in physical pixels.
    fn measure_text_advance(&mut self, text: &str, font_size: f32) -> f32;
}