// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Domain-based Editor UI Panels Module.
//!
//! Organizes all specialized editor panels into decoupled feature modules:
//! - [`hierarchy`]: Scene Hierarchy / Outliner entity tree and spawn tools.
//! - [`stats`]: Real-time Performance profiler, FPS, and GPU memory metrics.
//! - [`inspector`]: Entity Component Inspector and property drawers.
//! - [`material`]: Material & Submesh Editor for PBR textures and transparency.
//! - [`assets`]: Asset Browser for 3D meshes and 2D textures.
//! - [`console`]: Zero-allocation Developer Console and log viewer.
//! - [`timeline`]: Animation Timeline Studio and transport sequencer.
//!

pub mod assets;
pub mod console;
pub mod hierarchy;
pub mod inspector;
pub mod material;
pub mod stats;
pub mod timeline;

pub use hierarchy::{HierarchyCache, HierarchyRow};