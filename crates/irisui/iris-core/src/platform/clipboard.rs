// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Platform clipboard trait and in-memory mock implementation.
//!
//! Adheres strictly to modern Rust standards and 100% memory safety.
//! Provides thread-safe (`Send + Sync`) text read and write operations across desktop platforms.

use std::sync::RwLock;

/// Interface for cross-platform system and mock clipboard providers.
///
/// Implementations must be thread-safe (`Send + Sync`) to allow access from
/// UI event loops, background worker threads, or async runtimes without lock contention.
pub trait ClipboardProvider: Send + Sync {
    /// Retrieves the current text content from the clipboard if available.
    ///
    /// # Failure Modes
    /// Returns `None` if:
    /// - The clipboard is currently empty.
    /// - The clipboard does not contain valid UTF-8 plain text.
    /// - Access to the platform pasteboard failed or timed out.
    #[must_use]
    fn get_text(&self) -> Option<String>;

    /// Sets plain text content onto the platform clipboard.
    ///
    /// # Arguments
    /// * `text` - The UTF-8 string slice to copy to the clipboard.
    ///
    /// # Returns
    /// * `true` if the text was successfully written and registered by the platform clipboard.
    /// * `false` if platform clipboard registration failed or was rejected.
    fn set_text(&self, text: &str) -> bool;

    /// Clears any content stored on the platform clipboard.
    ///
    /// Default implementation sets the clipboard text to an empty string.
    fn clear(&self) -> bool {
        self.set_text("")
    }
}

/// Thread-safe in-memory clipboard implementation for tests and headless environments.
///
/// Employs an internal [`RwLock`] around an `Option<String>` to emulate platform clipboard
/// reads and writes deterministically without external OS dependencies.
#[derive(Debug, Default)]
pub struct InMemoryClipboard {
    storage: RwLock<Option<String>>,
}

impl InMemoryClipboard {
    /// Creates a new, empty in-memory clipboard instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(None),
        }
    }

    /// Creates an in-memory clipboard pre-populated with initial text.
    #[must_use]
    pub fn with_text(initial_text: impl Into<String>) -> Self {
        Self {
            storage: RwLock::new(Some(initial_text.into())),
        }
    }
}

impl ClipboardProvider for InMemoryClipboard {
    fn get_text(&self) -> Option<String> {
        let guard = self.storage.read().ok()?;
        guard.as_ref().cloned()
    }

    fn set_text(&self, text: &str) -> bool {
        let Ok(mut guard) = self.storage.write() else {
            return false;
        };
        *guard = Some(text.to_owned());
        true
    }

    fn clear(&self) -> bool {
        let Ok(mut guard) = self.storage.write() else {
            return false;
        };
        *guard = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_clipboard_initial_state() {
        let clipboard = InMemoryClipboard::new();
        assert_eq!(clipboard.get_text(), None);
    }

    #[test]
    fn test_in_memory_clipboard_set_and_get() {
        let clipboard = InMemoryClipboard::new();
        assert!(clipboard.set_text("Aeon Engine UI"));
        assert_eq!(clipboard.get_text(), Some("Aeon Engine UI".to_owned()));
    }

    #[test]
    fn test_in_memory_clipboard_overwrite() {
        let clipboard = InMemoryClipboard::with_text("Initial Text");
        assert_eq!(clipboard.get_text(), Some("Initial Text".to_owned()));

        assert!(clipboard.set_text("Updated Text"));
        assert_eq!(clipboard.get_text(), Some("Updated Text".to_owned()));
    }

    #[test]
    fn test_in_memory_clipboard_clear() {
        let clipboard = InMemoryClipboard::with_text("Clear me");
        assert_eq!(clipboard.get_text(), Some("Clear me".to_owned()));

        assert!(clipboard.clear());
        assert_eq!(clipboard.get_text(), None);
    }

    #[test]
    fn test_in_memory_clipboard_unicode_and_newlines() {
        let clipboard = InMemoryClipboard::new();
        let unicode_sample = "Türkçe karakterler: ğüşıöç ĞÜŞİÖÇ\nLine 2 \u{1F980} Crab";
        assert!(clipboard.set_text(unicode_sample));
        assert_eq!(clipboard.get_text(), Some(unicode_sample.to_owned()));
    }
}