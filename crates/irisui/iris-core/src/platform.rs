// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cross-platform clipboard provider abstraction, URL launcher, and shared platform utilities.
//!
//! Provides the [`ClipboardProvider`] trait for interacting with the platform
//! clipboard and safe cross-platform URL launcher utilities adhering strictly
//! to zero-unsafe abstractions and thread safety.

pub mod clipboard;
pub mod url;

pub use clipboard::{ClipboardProvider, InMemoryClipboard};
pub use url::open_url;