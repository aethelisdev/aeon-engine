// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Extension Hooks
//!
//! Provides generic, engine-independent extension contracts for integrating
//! hardware-accelerated 3D viewports, type-safe drag-and-drop payloads,
//! and custom volatile overlay rendering pipelines into workbench panels.

pub mod drag_drop;
pub mod external_texture;
pub mod overlay;
pub mod ui_ext;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_overlay;

pub use drag_drop::{
    DragDropController, DragDropPayload, DragDropPreviewStyle, TypedDragDropPayload,
};
pub use external_texture::{AspectRatio, ExternalTextureView};
pub use overlay::{
    ClosureOverlayHook, CustomOverlayHook, OverlayActiveFn, OverlayAnchor, OverlayDismissFn,
    OverlayEventFn, OverlayHook, OverlayHost, OverlayId, OverlayRenderFn,
};
pub use ui_ext::WorkbenchUiScopeExt;