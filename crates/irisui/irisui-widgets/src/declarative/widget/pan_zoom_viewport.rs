// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Pan & Zoom Interactive Canvas Viewport (`PanZoomViewport`)
//!
//! Provides a reusable, motor-independent 2D viewport container supporting aspect-fitted
//! virtual coordinate mapping, interactive wheel zooming, middle-mouse dragging pan,
//! single-canvas SDF grid drawing, draggable element boxes, and anchor/pin guide overlays.

pub mod math;
pub mod scope;
pub mod spec;
#[cfg(test)]
mod tests;
pub mod viewport;

pub use math::{PanZoomTransform, snap_scalar};
pub use scope::{PanZoomBoxResponse, PanZoomBoxSpec, PanZoomScope, PinGuideSpec};
pub use spec::PanZoomSpec;
pub use viewport::PanZoomViewportResponse;