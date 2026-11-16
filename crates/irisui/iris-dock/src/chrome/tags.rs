// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent 64-bit semantic tag namespaces and encoders for dock chrome widgets.
//!
//! Assigns collision-free high-nibble tag prefixes for interactive dock tab headers (`0xD0C1`),
//! overflow chevrons (`0xD0C2`), and close buttons (`0xD0C4`).

/// Base 64-bit semantic tag prefix for dock tab header widgets (`0xD0C1_*` namespace).
pub const TAG_DOCK_TAB_BASE: u64 = 0xD0C1_0000_0000_0000;
/// Base 64-bit semantic tag prefix for dock tab overflow chevron widgets (`0xD0C2_*` namespace).
pub const TAG_DOCK_TAB_CHEVRON_BASE: u64 = 0xD0C2_0000_0000_0000;
/// Base 64-bit semantic tag prefix for dock tab close button widgets (`0xD0C4_*` namespace).
pub const TAG_DOCK_TAB_CLOSE_BASE: u64 = 0xD0C4_0000_0000_0000;

/// Encodes a leaf FFI key and tab index into a persistent dock tab hover/hit tag.
#[inline]
#[must_use]
pub fn encode_dock_tab_tag(leaf_ffi: u64, tab_idx: usize) -> u64 {
    TAG_DOCK_TAB_BASE | ((leaf_ffi & 0xFFFF_FFFF) << 16) | ((tab_idx as u64) & 0xFFFF)
}

/// Encodes a leaf FFI key into a persistent dock tab overflow chevron tag.
#[inline]
#[must_use]
pub fn encode_dock_tab_chevron_tag(leaf_ffi: u64) -> u64 {
    TAG_DOCK_TAB_CHEVRON_BASE | (leaf_ffi & 0xFFFF_FFFF_FFFF)
}

/// Encodes a leaf FFI key and tab index into a persistent dock tab close button tag.
#[inline]
#[must_use]
pub fn encode_dock_tab_close_tag(leaf_ffi: u64, tab_idx: usize) -> u64 {
    TAG_DOCK_TAB_CLOSE_BASE | ((leaf_ffi & 0xFFFF_FFFF) << 16) | ((tab_idx as u64) & 0xFFFF)
}

/// Returns `true` if `tag` belongs to an `IrisDockTab` header node.
#[inline]
#[must_use]
pub fn is_dock_tab_tag(tag: u64) -> bool {
    (tag & 0xFFFF_0000_0000_0000) == TAG_DOCK_TAB_BASE
}

/// Returns `true` if `tag` belongs to an `IrisDockTabChevron` overflow button.
#[inline]
#[must_use]
pub fn is_dock_tab_chevron_tag(tag: u64) -> bool {
    (tag & 0xFFFF_0000_0000_0000) == TAG_DOCK_TAB_CHEVRON_BASE
}

/// Returns `true` if `tag` belongs to an `IrisDockTabCloseBtn` close button.
#[inline]
#[must_use]
pub fn is_dock_tab_close_tag(tag: u64) -> bool {
    (tag & 0xFFFF_0000_0000_0000) == TAG_DOCK_TAB_CLOSE_BASE
}

/// Returns `true` if `tag` identifies any interactive dock strip chrome (tab, chevron, or close button)
/// so workbench event routing preserves `DockArea` click/drag handling alongside tree hover state.
#[inline]
#[must_use]
pub fn is_dock_chrome_tag(tag: u64) -> bool {
    is_dock_tab_tag(tag) || is_dock_tab_chevron_tag(tag) || is_dock_tab_close_tag(tag)
}