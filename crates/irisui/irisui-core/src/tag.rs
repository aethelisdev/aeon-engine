// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Stable Semantic UI Interaction Tags (`UiTag`)
//!
//! Provides type-safe, compile-time hashable 64-bit identifiers for interactive widgets,
//! modals, menus, and viewports, eliminating ad-hoc hexadecimal constants.

use std::fmt;

const FNV1A_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_PRIME: u64 = 0x0100_0000_01b3;

/// Type-safe, compile-time constructible 64-bit identifier for UI interaction routing and state lookup.
///
/// Wraps a raw `u64` tag derived deterministically from hierarchical string keys via FNV-1a
/// hashing, ensuring zero heap allocations and collision-free scope isolation.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UiTag(pub u64);

impl UiTag {
    /// Sentinel empty tag representing an untagged node (`0`).
    pub const NONE: Self = Self(0);

    /// Wraps an existing raw 64-bit tag value.
    #[inline]
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw 64-bit integer value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Returns `true` when this tag is the unset zero sentinel [`UiTag::NONE`].
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Computes a deterministic 64-bit tag from a static semantic key at compile time or runtime.
    ///
    /// Uses standard 64-bit FNV-1a hashing matching `irisui_widgets::declarative::hash_label`.
    #[inline]
    #[must_use]
    pub const fn from_str(key: &str) -> Self {
        Self(fnv1a_bytes(FNV1A_OFFSET_BASIS, key.as_bytes()))
    }

    /// Derives a collision-isolated child tag scoped under `self`.
    ///
    /// When `self` is [`UiTag::NONE`], falls back to [`UiTag::from_str`].
    /// Matches `hash_label_with_seed(self.0, key)`.
    #[inline]
    #[must_use]
    pub const fn child(self, key: &str) -> Self {
        let seed_basis = if self.0 == 0 {
            FNV1A_OFFSET_BASIS
        } else {
            (FNV1A_OFFSET_BASIS ^ self.0).wrapping_mul(FNV1A_PRIME)
        };
        Self(fnv1a_bytes(seed_basis, key.as_bytes()))
    }

    /// Derives an indexed child tag scoped under `self` (e.g. for list rows or toolbar slots).
    #[inline]
    #[must_use]
    pub const fn indexed(self, index: u64) -> Self {
        let bytes = index.to_le_bytes();
        let basis = if self.0 == 0 {
            FNV1A_OFFSET_BASIS
        } else {
            (FNV1A_OFFSET_BASIS ^ self.0).wrapping_mul(FNV1A_PRIME)
        };
        Self(fnv1a_bytes(basis, &bytes))
    }
}

#[inline]
const fn fnv1a_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        i += 1;
    }
    hash
}

impl From<UiTag> for u64 {
    #[inline]
    fn from(tag: UiTag) -> Self {
        tag.0
    }
}

impl From<u64> for UiTag {
    #[inline]
    fn from(raw: u64) -> Self {
        Self(raw)
    }
}

impl From<&str> for UiTag {
    #[inline]
    fn from(key: &str) -> Self {
        Self::from_str(key)
    }
}

impl fmt::Display for UiTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UiTag(0x{:016x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABOUT_MODAL: UiTag = UiTag::from_str("modal.about");
    const EXPORT_MODAL: UiTag = UiTag::from_str("modal.export_project");

    #[test]
    fn test_ui_tag_const_determinism_and_uniqueness() {
        assert!(!ABOUT_MODAL.is_empty());
        assert!(UiTag::NONE.is_empty());
        assert_eq!(ABOUT_MODAL, UiTag::from_str("modal.about"));
        assert_ne!(ABOUT_MODAL, EXPORT_MODAL);
        assert_eq!(u64::from(ABOUT_MODAL), ABOUT_MODAL.raw());
        assert_eq!(UiTag::from("modal.about"), ABOUT_MODAL);
    }

    #[test]
    fn test_ui_tag_child_and_indexed_isolation() {
        let view_menu = UiTag::from_str("View");
        let window_menu = UiTag::from_str("Window");
        assert_ne!(
            view_menu.child("Hierarchy"),
            window_menu.child("Hierarchy"),
            "Identical child labels under distinct parent tags must never collide"
        );
        assert_ne!(view_menu.indexed(0), view_menu.indexed(1));
        assert_eq!(UiTag::NONE.child("File"), UiTag::from_str("File"));
    }
}