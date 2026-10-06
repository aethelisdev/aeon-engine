// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Interaction Types & Hashing Primitives (`iris-widgets::declarative::types`)
//!
//! Exposes widget interaction responses and persistent tag hashing utilities for declarative UI scopes.
//!

use iris_core::WidgetId;

/// Encapsulates the interaction result of an emitted widget, allowing immediate
/// consumption of user interactions in modern declarative style (`if ui.button("Save").clicked()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WidgetResponse {
    /// The unique allocated identifier of the emitted widget node in the UI arena.
    pub id: WidgetId,
    /// Whether the widget was clicked by the user during this interaction pass.
    pub clicked: bool,
    /// Whether the pointer cursor is currently hovering over this widget.
    pub hovered: bool,
    /// Whether the bound property value was modified during this interaction pass.
    pub changed: bool,
}

impl WidgetResponse {
    /// Constructs a new widget response with explicit state flags.
    #[inline]
    pub const fn new(id: WidgetId, clicked: bool, hovered: bool, changed: bool) -> Self {
        Self {
            id,
            clicked,
            hovered,
            changed,
        }
    }

    /// Returns `true` if the widget was clicked during this frame.
    #[inline]
    pub const fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` if the widget is currently hovered.
    #[inline]
    pub const fn hovered(&self) -> bool {
        self.hovered
    }

    /// Returns `true` if the underlying value was changed during this frame.
    #[inline]
    pub const fn changed(&self) -> bool {
        self.changed
    }
}

/// Computes a deterministic 64-bit FNV-1a hash of a label string for persistent declarative node tagging.
///
/// Equivalent to calling [`hash_label_with_seed(0, label)`].
#[inline]
pub fn hash_label(label: &str) -> u64 {
    hash_label_with_seed(0, label)
}

/// Computes a deterministic 64-bit FNV-1a hash of a label string seeded with an ancestor or scope identifier.
///
/// When `seed == 0`, produces the canonical FNV-1a hash. Non-zero seeds permute
/// the initial offset basis, mathematically isolating identical labels across distinct scopes or entities.
#[inline]
pub fn hash_label_with_seed(seed: u64, label: &str) -> u64 {
    let mut hash: u64 = if seed == 0 {
        0xcbf2_9ce4_8422_2325
    } else {
        (0xcbf2_9ce4_8422_2325 ^ seed).wrapping_mul(0x0100_0000_01b3)
    };
    for &b in label.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Combines parent and child scope seeds deterministically using FNV-1a mixing.
///
/// Ensures hierarchical scopes like `Entity -> Component -> Property` produce uniquely
/// isolated seed paths without heap allocation.
#[inline]
pub fn combine_seeds(parent_seed: u64, child_seed: u64) -> u64 {
    if parent_seed == 0 {
        child_seed
    } else {
        let mut hash = parent_seed;
        for b in child_seed.to_le_bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash
    }
}

/// Splits a widget label around the optional `##` identifier separator without heap allocation.
///
/// Returns `(visible_label, full_id_source)`.
/// If `##` is present (e.g. `"Offset##collider"`), `visible_label` is `"Offset"` and `full_id_source` is `"Offset##collider"`.
/// If `##` is not present, both elements reference the identical input slice.
#[inline]
pub fn split_label_id(label: &str) -> (&str, &str) {
    if let Some((visible, _)) = label.split_once("##") {
        (visible, label)
    } else {
        (label, label)
    }
}

/// Trait for types that can serve as a seed identifier for declarative sub-scopes.
///
/// Converts numeric identifiers or string keys into a 64-bit seed without heap allocation.
pub trait ScopeId {
    /// Converts this identifier into a 64-bit seed representation.
    fn into_seed(self) -> u64;
}

impl ScopeId for u64 {
    #[inline]
    fn into_seed(self) -> u64 {
        self
    }
}

impl ScopeId for usize {
    #[inline]
    fn into_seed(self) -> u64 {
        self as u64
    }
}

impl ScopeId for u32 {
    #[inline]
    fn into_seed(self) -> u64 {
        self as u64
    }
}

impl ScopeId for i32 {
    #[inline]
    fn into_seed(self) -> u64 {
        self as u64
    }
}

impl ScopeId for &str {
    #[inline]
    fn into_seed(self) -> u64 {
        hash_label(self)
    }
}

impl ScopeId for &String {
    #[inline]
    fn into_seed(self) -> u64 {
        hash_label(self.as_str())
    }
}