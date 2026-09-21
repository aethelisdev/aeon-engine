// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Overlay Action Dispatchers
//!
//! Handles action events triggered by Iris UI overlays including Preferences,
//! Viewport HUD, Scene Hierarchy, and Performance Stats panels.

pub mod assets;
pub mod preferences;
pub mod viewports;

pub use preferences::PreferencesActionContext;