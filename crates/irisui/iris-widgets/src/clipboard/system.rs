// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native operating system clipboard driver implementing [`ClipboardProvider`].
//!
//! Provides thread-safe, 100% safe Rust clipboard integration using `arboard`
//! with automatic headless in-memory fallback for environments without a display server.

use std::sync::Mutex;

use arboard::Clipboard;
use iris_core::platform::{ClipboardProvider, InMemoryClipboard};

/// Production-grade operating system clipboard driver for Iris UI desktop applications.
///
/// Encapsulates platform pasteboard access (X11, Wayland, Windows, macOS) behind
/// a thread-safe `Mutex`. In headless environments, CI pipelines, or when OS clipboard
/// initialization fails, gracefully falls back to an [`InMemoryClipboard`] to guarantee
/// unbroken runtime continuity without panicking.
pub struct SystemClipboard {
    os_clipboard: Mutex<Option<Clipboard>>,
    fallback: InMemoryClipboard,
}

impl SystemClipboard {
    /// Creates a new system clipboard instance, attempting to connect to the OS clipboard.
    ///
    /// If platform clipboard initialization fails (e.g., headless display server,
    /// missing X11/Wayland environment, or restricted sandbox), an internal fallback
    /// is used transparently.
    #[must_use]
    pub fn new() -> Self {
        let os_clipboard = Clipboard::new().ok();
        Self {
            os_clipboard: Mutex::new(os_clipboard),
            fallback: InMemoryClipboard::new(),
        }
    }

    /// Creates a system clipboard initialized with specified initial text in the fallback buffer.
    #[must_use]
    pub fn with_initial_text(initial_text: impl Into<String>) -> Self {
        let os_clipboard = Clipboard::new().ok();
        Self {
            os_clipboard: Mutex::new(os_clipboard),
            fallback: InMemoryClipboard::with_text(initial_text),
        }
    }

    /// Returns `true` if the underlying OS clipboard was successfully initialized.
    #[must_use]
    pub fn is_os_native_available(&self) -> bool {
        let Ok(guard) = self.os_clipboard.lock() else {
            return false;
        };
        guard.is_some()
    }

    /// Internal constructor allowing explicit headless fallback mode for deterministic unit testing.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn headless_fallback() -> Self {
        Self {
            os_clipboard: Mutex::new(None),
            fallback: InMemoryClipboard::new(),
        }
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardProvider for SystemClipboard {
    fn get_text(&self) -> Option<String> {
        let Ok(mut guard) = self.os_clipboard.lock() else {
            return self.fallback.get_text();
        };

        if let Some(clipboard) = guard.as_mut() {
            match clipboard.get_text() {
                Ok(text) => Some(text),
                Err(_) => self.fallback.get_text(),
            }
        } else {
            self.fallback.get_text()
        }
    }

    fn set_text(&self, text: &str) -> bool {
        let Ok(mut guard) = self.os_clipboard.lock() else {
            return self.fallback.set_text(text);
        };

        if let Some(clipboard) = guard.as_mut() {
            if clipboard.set_text(text).is_ok() {
                // Keep the in-memory fallback synchronized as well
                let _ = self.fallback.set_text(text);
                true
            } else {
                self.fallback.set_text(text)
            }
        } else {
            self.fallback.set_text(text)
        }
    }

    fn clear(&self) -> bool {
        let Ok(mut guard) = self.os_clipboard.lock() else {
            return self.fallback.clear();
        };

        if let Some(clipboard) = guard.as_mut() {
            let _ = clipboard.clear();
            let _ = self.fallback.clear();
            true
        } else {
            self.fallback.clear()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clipboard_creation_and_lifecycle() {
        let clipboard = SystemClipboard::headless_fallback();
        let test_message = "Aeon Engine - OS Clipboard Integration Test";

        // Writing must succeed either natively or via in-memory fallback
        assert!(clipboard.set_text(test_message));
        assert_eq!(clipboard.get_text(), Some(test_message.to_owned()));

        // Clear must succeed and reflect in subsequent reads
        assert!(clipboard.clear());
        assert_eq!(clipboard.get_text(), None);
    }

    #[test]
    fn test_system_clipboard_unicode_and_newlines() {
        let clipboard = SystemClipboard::headless_fallback();
        let unicode_text = "Örnek Türkçe Metin: İŞĞÜÇÖ ıişğüçö \n İkinci satır \t Tab ve 🦀 Yengeç";

        assert!(clipboard.set_text(unicode_text));
        assert_eq!(clipboard.get_text(), Some(unicode_text.to_owned()));
    }

    #[test]
    fn test_system_clipboard_fallback_cohesion() {
        // Test fallback path directly when native clipboard is None
        let clipboard = SystemClipboard::headless_fallback();

        assert!(!clipboard.is_os_native_available());
        assert_eq!(clipboard.get_text(), None);

        assert!(clipboard.set_text("Fallback Data"));
        assert_eq!(clipboard.get_text(), Some("Fallback Data".to_owned()));

        assert!(clipboard.clear());
        assert_eq!(clipboard.get_text(), None);
    }

    #[test]
    fn test_system_clipboard_default_and_native_instantiation() {
        let clipboard = SystemClipboard::default();
        // Verifies graceful initialization without panic in either headless or display environment
        let _ = clipboard.is_os_native_available();
    }
}