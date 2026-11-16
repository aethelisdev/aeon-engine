// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cross-platform system clipboard integration for Iris UI.
//!
//! Provides the [`SystemClipboard`] driver implementing [`ClipboardProvider`],
//! supporting native desktop clipboards (X11, Wayland, Windows, macOS) via safe bindings
//! with automatic headless in-memory fallback.

pub mod system;

pub use system::SystemClipboard;