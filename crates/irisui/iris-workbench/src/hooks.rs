// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Extension Hooks
//!
//! Provides generic, engine-independent extension contracts for integrating
//! hardware-accelerated 3D viewports, type-safe drag-and-drop payloads,
//! and custom volatile overlay rendering pipelines into workbench panels.

pub mod drag_drop;
pub mod overlay;
pub mod texture;

#[cfg(test)]
mod tests;

pub use drag_drop::{DragDropPayload, TypedDragDropPayload};
pub use overlay::CustomOverlayHook;
pub use texture::ExternalTextureHook;