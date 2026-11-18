// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI (`irisui`)
//!
//! Umbrella facade crate uniting the modular Iris UI framework crates:
//! - `irisui-core`: Generational arena, nodes, styles, dirty tracking, and geometry primitives.
//! - `irisui-layout`: Pure, high-performance flexbox and absolute layout solver.
//! - `irisui-text`: Glyphon-based GPU subpixel font rendering and text measurement engine.
//! - `irisui-wgpu`: High-performance batched instanced quad renderer and custom external texture pipelines.
//! - `irisui-widgets`: Industrial, hardware-accelerated game engine UI widget kit.
//! - `irisui-dock`: Generational binary split-tree docking and floating window multi-viewport engine.
//! - `irisui-winit`: Winit platform adapter, event translation, HighDPI scaling, and cursor mapping.
//! - `irisui-workbench`: Desktop IDE shell, declarative panel management, and lifecycle synchronization.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use irisui_core as core;
pub use irisui_dock as dock;
pub use irisui_layout as layout;
pub use irisui_text as text;
pub use irisui_wgpu as wgpu_backend;
pub use irisui_widgets as widgets;
pub use irisui_winit as winit;
pub use irisui_workbench as workbench;

pub use irisui_wgpu::{
    IrisSurfaceFrameFlags, IrisSurfaceHost, IrisSurfaceRenderParams, SurfaceRefreshMode,
};
pub use irisui_widgets::{SubtreeSession, rebuild_leaf_subtree};

/// Fluent, comprehensive umbrella prelude re-exporting all standard Iris UI primitives from subcrates.
///
/// Designed to be imported as `use irisui::prelude::*;` for , idiomatic engine UI development.
pub mod prelude {
    pub use irisui_core::prelude::*;
    pub use irisui_dock::prelude::*;
    pub use irisui_layout::prelude::*;
    pub use irisui_text::prelude::*;
    pub use irisui_wgpu::prelude::*;
    pub use irisui_widgets::prelude::*;
    pub use irisui_workbench::prelude::*;
}