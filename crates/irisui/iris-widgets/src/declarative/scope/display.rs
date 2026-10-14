// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Display, Typography & Visual Decorator Primitives
//!
//! Provides text labels, headings, texture atlas icons, carets, and canvas swatches on [`UiScope`]:
//! - Typography, 1-argument labels, headings, and badges in [`labels`].
//! - Texture atlas icons and hover reactivity in [`icons`].
//! - External textures, scrollbars, and color swatches in [`canvas`].
//!

pub mod canvas;
pub mod icons;
pub mod labels;

pub use labels::WrappedLabelDescriptor;