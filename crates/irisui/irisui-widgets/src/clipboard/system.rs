// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native operating system clipboard driver implementing [`ClipboardProvider`].
//!
//! Provides thread-safe, 100% safe Rust clipboard integration using `arboard` when enabled on
//! desktop platforms (`clipboard-os` feature and non-Android targets), with automatic headless
//! in-memory fallback when disabled or running on mobile targets.

use std::sync::Mutex;

#[cfg(all(feature = "clipboard-os", not(target_os = "android")))]
use arboard::Clipboard;
use irisui_core::platform::{ClipboardProvider, InMemoryClipboard};

#[cfg(all(feature = "clipboard-os", not(target_os = "android")))]
type NativeClipboardHandle = Option<Clipboard>;

#[cfg(not(all(feature = "clipboard-os", not(target_os = "android"))))]
type NativeClipboardHandle = Option<()>;

fn try_open_native_clipboard() -> NativeClipboardHandle {
    #[cfg(all(feature = "clipboard-os", not(target_os = "android")))]
    {
        Clipboard::new().ok()
    }
    #[cfg(not(all(feature = "clipboard-os", not(target_os = "android"))))]
    {
        None
    }
}

/// Operating system clipboard driver for Iris UI applications.
///
/// Encapsulates platform pasteboard access (X11, Wayland, Windows, macOS) behind
/// a thread-safe `Mutex`. When OS clipboard initialization is unavailable or disabled on mobile,
/// falls back cleanly to an [`InMemoryClipboard`].
pub struct SystemClipboard {
    os_clipboard: Mutex<NativeClipboardHandle>,
    fallback: InMemoryClipboard,
}

impl SystemClipboard {
    /// Creates a new system clipboard instance, attempting to connect to the OS clipboard.
    #[must_use]
    pub fn new() -> Self {
        Self {
            os_clipboard: Mutex::new(try_open_native_clipboard()),
            fallback: InMemoryClipboard::new(),
        }
    }

    /// Creates a system clipboard initialized with specified initial text in the fallback buffer.
    #[must_use]
    pub fn with_initial_text(initial_text: impl Into<String>) -> Self {
        Self {
            os_clipboard: Mutex::new(try_open_native_clipboard()),
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
        #[cfg(all(feature = "clipboard-os", not(target_os = "android")))]
        if let Ok(mut guard) = self.os_clipboard.lock()
            && let Some(clipboard) = guard.as_mut()
            && let Ok(text) = clipboard.get_text()
        {
            return Some(text);
        }
        self.fallback.get_text()
    }

    fn set_text(&self, text: &str) -> bool {
        #[cfg(all(feature = "clipboard-os", not(target_os = "android")))]
        if let Ok(mut guard) = self.os_clipboard.lock()
            && let Some(clipboard) = guard.as_mut()
            && clipboard.set_text(text).is_ok()
        {
            let _ = self.fallback.set_text(text);
            return true;
        }
        self.fallback.set_text(text)
    }

    fn clear(&self) -> bool {
        #[cfg(all(feature = "clipboard-os", not(target_os = "android")))]
        if let Ok(mut guard) = self.os_clipboard.lock()
            && let Some(clipboard) = guard.as_mut()
        {
            let _ = clipboard.clear();
            let _ = self.fallback.clear();
            return true;
        }
        self.fallback.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clipboard_creation_and_lifecycle() {
        let clipboard = SystemClipboard::headless_fallback();
        let test_message = "Aeon Engine - OS Clipboard Integration Test";

        assert!(clipboard.set_text(test_message));
        assert_eq!(clipboard.get_text(), Some(test_message.to_owned()));

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
        let _ = clipboard.is_os_native_available();
    }
}