// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Domain-based Editor UI Panels Module.
//!
//! Provides the Asset Browser subsystem:
//! - [`assets`]: Asset Browser for 3D meshes, 2D textures, and file operations.
//!

pub mod assets;

pub use assets::types::AssetBrowserState;