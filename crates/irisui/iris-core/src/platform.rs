// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cross-platform clipboard provider abstraction and shared memory utilities.
//!
//! Provides the [`ClipboardProvider`] trait for interacting with the platform
//! clipboard (such as system pasteboard, X11/Wayland selections, or in-memory test mocks)
//! adhering strictly to zero-unsafe abstractions and thread safety.

pub mod clipboard;

pub use clipboard::{ClipboardProvider, InMemoryClipboard};