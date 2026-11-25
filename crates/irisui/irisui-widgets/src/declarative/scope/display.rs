// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Display, Typography & Visual Decorator Primitives
//!
//! Provides text labels, headings, texture atlas icons, carets, and canvas swatches on [`crate::UiScope`]:
//! - Typography, 1-argument labels, headings, and badges in [`labels`].
//! - Key-value rows, stack formatted numbers, and telemetry badges in [`key_value`].
//! - Texture atlas icons and hover reactivity in [`icons`].
//! - External textures, scrollbars, and color swatches in [`canvas`].

pub mod badge_pill;
pub mod canvas;
pub mod icons;
pub mod key_value;
pub mod labels;
pub mod live;
pub mod log_row;
pub mod section_heading;
pub mod telemetry;

pub use badge_pill::BadgePillSpec;
pub use canvas::{ImmediateCanvasSpec, VerticalScrollbarSpec};
pub use key_value::{
    NumberLabelDescriptor, format_file_size_stack, format_number_stack,
    format_prefix_number_suffix_stack,
};
pub use labels::{LabeledStatusDescriptor, WrappedLabelDescriptor};
pub use live::LiveTextProps;
pub use log_row::{LogRowBadge, LogRowDescriptor};