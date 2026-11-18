// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Zero-Allocation String-Keyed Typography Cache
//!
//! Stores shaped `cosmic_text::Buffer` instances and layout metrics in contiguous arena vectors
//! indexed by `Copy` lookup keys `(fnv1a_hash, byte_len, metrics...)`. Cache hits perform zero
//! heap string allocations and zero `Buffer` clones, while hash collisions resolve cleanly through
//! an intrusive linked index chain inside the entry arena.

use std::collections::HashMap;
use std::hash::Hash;

/// Deterministic 64-bit FNV-1a hash over UTF-8 string bytes for zero-alloc cache probing.
#[inline]
#[must_use]
pub(crate) fn hash_str_fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in text.as_bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Arena slot pairing an owned heap string with its cached value and optional collision chain link.
pub(crate) struct CacheEntry<T> {
    text: Box<str>,
    pub(crate) value: T,
    next_collision: Option<usize>,
}

/// Contiguous arena cache keyed by a `Copy` hash/metric tuple without allocating `String` on lookups.
pub(crate) struct StringKeyedArenaCache<K, T> {
    index: HashMap<K, usize>,
    entries: Vec<CacheEntry<T>>,
}

impl<K: Eq + Hash + Copy, T> StringKeyedArenaCache<K, T> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.index.clear();
        self.entries.clear();
    }

    #[inline]
    pub(crate) fn entries(&self) -> &[CacheEntry<T>] {
        &self.entries
    }

    #[inline]
    pub(crate) fn get_value(&self, idx: usize) -> Option<&T> {
        self.entries.get(idx).map(|e| &e.value)
    }

    /// Probes the cache for `(key, text)` with zero heap allocation.
    #[inline]
    pub(crate) fn get_indexed(&self, key: &K, text: &str) -> Option<(usize, &T)> {
        let mut cursor = self.index.get(key).copied();
        while let Some(idx) = cursor {
            let entry = &self.entries[idx];
            if entry.text.as_ref() == text {
                return Some((idx, &entry.value));
            }
            cursor = entry.next_collision;
        }
        None
    }

    /// Inserts a newly computed entry without invalidating existing indices in this frame.
    pub(crate) fn insert_preserving_indices(&mut self, key: K, text: &str, value: T) -> usize {
        let new_idx = self.entries.len();
        let prev_head = self.index.insert(key, new_idx);
        self.entries.push(CacheEntry {
            text: Box::from(text),
            value,
            next_collision: prev_head,
        });
        new_idx
    }

    /// Clears the arena if `max_entries` is exceeded before inserting a new entry.
    pub(crate) fn insert_bounded(
        &mut self,
        key: K,
        text: &str,
        value: T,
        max_entries: usize,
    ) -> usize {
        if self.entries.len() > max_entries {
            self.clear();
        }
        self.insert_preserving_indices(key, text, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_keyed_arena_cache_handles_collisions_and_zero_alloc_hits() {
        let mut cache: StringKeyedArenaCache<(u64, usize), u32> =
            StringKeyedArenaCache::with_capacity(8);

        // Force identical lookup key to verify intrusive collision chaining
        let forced_key = (0x1234_u64, 5_usize);
        let idx_a = cache.insert_bounded(forced_key, "alpha", 10, 64);
        let idx_b = cache.insert_bounded(forced_key, "bravo", 20, 64);

        assert_eq!(cache.get_indexed(&forced_key, "alpha"), Some((idx_a, &10)));
        assert_eq!(cache.get_indexed(&forced_key, "bravo"), Some((idx_b, &20)));
        assert_eq!(cache.get_indexed(&forced_key, "charl"), None);
    }
}