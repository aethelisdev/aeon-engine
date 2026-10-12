// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Interaction Types & Hashing Primitives (`iris-widgets::declarative::types`)
//!
//! Exposes widget interaction responses and persistent tag hashing utilities for declarative UI scopes.
//!

pub use iris_core::response::{ResponseFlags, WidgetResponse};

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