// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent 128-bit GUID generation and virtual-path hashing utilities.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use ae_plugin_api::{AssetGuid, EntityGuid, component_type_hash};

/// Process-wide monotonic counter combined with wall-clock time to avoid identical GUIDs
/// when generating identifiers rapidly across threads.
static GUID_COUNTER: AtomicU64 = AtomicU64::new(1);

const FNV_OFFSET_1: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_OFFSET_2: u64 = 0xaf63_bd4c_8601_b7df;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Stateful, thread-safe 128-bit identifier generator producing non-zero [`EntityGuid`]
/// and [`AssetGuid`] values without external OS entropy syscalls or `unsafe` code.
///
/// Supports both randomized runtime generation ([`GuidGenerator::new`]) and repeatable
/// deterministic sequences ([`GuidGenerator::from_seed`]) for unit tests and procedural builders.
#[derive(Debug)]
pub struct GuidGenerator {
    state_lo: u64,
    state_hi: u64,
}

impl GuidGenerator {
    /// Creates a generator seeded from current nanosecond wall-clock time and a monotonic atomic counter.
    #[must_use]
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x517c_c1b7_2722_0a95_u128, |d| d.as_nanos());
        let counter = GUID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let lo = mix64((nanos as u64) ^ counter.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let hi = mix64(((nanos >> 64) as u64) ^ counter.rotate_left(29) ^ 0xbf58_476d_1ce4_e5b9);
        Self {
            state_lo: lo,
            state_hi: hi,
        }
    }

    /// Creates a deterministic generator from an explicit 64-bit seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state_lo: mix64(seed ^ 0x9e37_79b9_7f4a_7c15),
            state_hi: mix64(seed.wrapping_add(0xbf58_476d_1ce4_e5b9)),
        }
    }

    /// Generates the next non-nil raw 128-bit identifier.
    pub fn next_raw(&mut self) -> u128 {
        self.state_lo = self.state_lo.wrapping_add(0x9e37_79b9_7f4a_7c15);
        self.state_hi = self.state_hi.wrapping_add(0x94d0_49bb_1331_11eb);
        let a = mix64(self.state_lo);
        let b = mix64(self.state_hi ^ a);
        let combined = (u128::from(b) << 64) | u128::from(a);
        if combined == 0 { 1 } else { combined }
    }

    /// Generates the next non-nil [`EntityGuid`].
    pub fn next_entity_guid(&mut self) -> EntityGuid {
        EntityGuid::from_raw(self.next_raw())
    }

    /// Generates the next non-nil [`AssetGuid`].
    pub fn next_asset_guid(&mut self) -> AssetGuid {
        AssetGuid::from_raw(self.next_raw())
    }
}

impl Default for GuidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes a deterministic non-nil [`AssetGuid`] from a normalized virtual asset path.
///
/// Uses dual independent 64-bit FNV-1a passes mixed through SplitMix64 finalizers so
/// identical asset paths always resolve to identical 128-bit identifiers across platforms.
#[must_use]
pub fn asset_guid_from_virtual_path(virtual_path: &str) -> AssetGuid {
    let mut h1 = FNV_OFFSET_1;
    let mut h2 = FNV_OFFSET_2;
    for &byte in virtual_path.as_bytes() {
        let normalized = if byte == b'\\' { b'/' } else { byte };
        h1 ^= u64::from(normalized);
        h1 = h1.wrapping_mul(FNV_PRIME);
        h2 ^= u64::from(normalized).wrapping_add(0x27);
        h2 = h2.wrapping_mul(FNV_PRIME);
    }
    let raw = (u128::from(mix64(h2)) << 64) | u128::from(mix64(h1));
    AssetGuid::from_raw(if raw == 0 { 1 } else { raw })
}

#[inline]
const fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_seeded_guid_generator_determinism_and_uniqueness() {
        let mut g1 = GuidGenerator::from_seed(42);
        let mut g2 = GuidGenerator::from_seed(42);
        let mut seen = HashSet::new();

        for _ in 0..1024 {
            let a = g1.next_entity_guid();
            let b = g2.next_entity_guid();
            assert_eq!(a, b);
            assert!(!a.is_nil());
            assert!(seen.insert(a));
        }
    }

    #[test]
    fn test_asset_guid_from_virtual_path_normalizes_separators() {
        let a = asset_guid_from_virtual_path("models/characters/hero.glb");
        let b = asset_guid_from_virtual_path(r"models\characters\hero.glb");
        assert_eq!(a, b);
        assert!(!a.is_nil());
        assert_ne!(
            a,
            asset_guid_from_virtual_path("models/characters/villain.glb")
        );
    }
}