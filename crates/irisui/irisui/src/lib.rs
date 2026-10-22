// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI (`irisui`)
//!
//! Umbrella facade crate uniting the modular Iris UI framework crates:
//! - `iris-core`: Generational arena, nodes, styles, dirty tracking, and geometry primitives.
//! - `iris-layout`: Pure, high-performance flexbox and absolute layout solver.
//! - `iris-text`: Glyphon-based GPU subpixel font rendering and text measurement engine.
//! - `iris-wgpu`: High-performance batched instanced quad renderer and custom external texture pipelines.
//! - `iris-widgets`: Industrial, hardware-accelerated game engine UI widget kit.
//! - `iris-dock`: Generational binary split-tree docking and floating window multi-viewport engine.
//! - `iris-winit`: Winit platform adapter, event translation, HighDPI scaling, and cursor mapping.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use iris_core as core;
pub use iris_dock as dock;
pub use iris_layout as layout;
pub use iris_text as text;
pub use iris_wgpu as wgpu_backend;
pub use iris_widgets as widgets;
pub use iris_winit as winit;

pub use iris_widgets::{SubtreeSession, rebuild_leaf_subtree};

/// Fluent, comprehensive umbrella prelude re-exporting all standard Iris UI primitives from subcrates.
///
/// Designed to be imported as `use irisui::prelude::*;` for , idiomatic engine UI development.
pub mod prelude {
    pub use iris_core::prelude::*;
    pub use iris_dock::prelude::*;
    pub use iris_layout::prelude::*;
    pub use iris_text::prelude::*;
    pub use iris_wgpu::prelude::*;
    pub use iris_widgets::prelude::*;
}