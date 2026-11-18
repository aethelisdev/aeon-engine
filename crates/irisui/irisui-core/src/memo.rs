// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Reactive Keyed Memoization Cache
//!
//! Provides generic key-tracked value memoization for retained UI state and filtered
//! collection projections, avoiding manual per-field dirty tracking flags and allocations.

/// Generic reactive cache storing the most recently evaluated key and computed value.
///
/// Predicate-based lookups (`is_dirty_with`, `sync_with`) compare against borrowed frame
/// inputs in place so cache hits incur zero heap allocations.
#[derive(Debug, Clone)]
pub struct MemoCache<K, V> {
    key: Option<K>,
    value: V,
}

impl<K, V: Default> Default for MemoCache<K, V> {
    fn default() -> Self {
        Self::new(V::default())
    }
}

impl<K, V> MemoCache<K, V> {
    /// Creates an uninitialized memo cache seeded with `initial_value`.
    #[inline]
    pub const fn new(initial_value: V) -> Self {
        Self {
            key: None,
            value: initial_value,
        }
    }

    /// Returns `true` if no key is stored yet or `matches_key` returns `false` for the stored key.
    #[inline]
    pub fn is_dirty_with(&self, matches_key: impl FnOnce(&K) -> bool) -> bool {
        self.key.as_ref().is_none_or(|k| !matches_key(k))
    }

    /// Updates `(key, value)` only when [`Self::is_dirty_with`] returns `true`, returning `&value`.
    pub fn sync_with(
        &mut self,
        matches_key: impl FnOnce(&K) -> bool,
        make_key: impl FnOnce() -> K,
        compute: impl FnOnce() -> V,
    ) -> &V {
        if self.is_dirty_with(matches_key) {
            self.value = compute();
            self.key = Some(make_key());
        }
        &self.value
    }

    /// Returns `true` if the stored key differs from `key` or is uninitialized.
    #[inline]
    pub fn is_dirty(&self, key: &K) -> bool
    where
        K: PartialEq,
    {
        self.key.as_ref() != Some(key)
    }

    /// Recomputes `value` when `key` differs from the stored key, returning `&value`.
    pub fn sync(&mut self, key: K, compute: impl FnOnce() -> V) -> &V
    where
        K: PartialEq,
    {
        if self.is_dirty(&key) {
            self.value = compute();
            self.key = Some(key);
        }
        &self.value
    }

    /// Returns a shared reference to the most recently cached value.
    #[inline]
    pub fn get(&self) -> &V {
        &self.value
    }

    /// Returns a shared reference to the stored key snapshot, if initialized.
    #[inline]
    pub fn key(&self) -> Option<&K> {
        self.key.as_ref()
    }

    /// Clears the stored key so the next sync forces recomputation.
    #[inline]
    pub fn invalidate(&mut self) {
        self.key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::MemoCache;

    #[test]
    fn test_memo_cache_sync_and_invalidate() {
        let mut cache: MemoCache<(String, usize), Vec<usize>> = MemoCache::default();
        assert!(cache.is_dirty_with(|_| true));
        assert!(cache.key().is_none());

        let mut recomputes = 0_usize;
        {
            let first = cache.sync_with(
                |(q, c)| q == "mesh" && *c == 3,
                || ("mesh".to_string(), 3),
                || {
                    recomputes += 1;
                    vec![1, 2, 3]
                },
            );
            assert_eq!(first, &[1, 2, 3]);
            assert_eq!(recomputes, 1);
        }

        // Cache hit must not invoke compute or make_key.
        assert!(!cache.is_dirty_with(|(q, c)| q == "mesh" && *c == 3));
        let second = cache.sync_with(
            |(q, c)| q == "mesh" && *c == 3,
            || panic!("make_key must not run on cache hit"),
            || panic!("compute must not run on cache hit"),
        );
        assert_eq!(second, &[1, 2, 3]);

        // Invalidate forces recomputation on next sync.
        cache.invalidate();
        assert!(cache.is_dirty_with(|(q, c)| q == "mesh" && *c == 3));
        let third = cache.sync(("tex".to_string(), 1), || {
            recomputes += 1;
            vec![42]
        });
        assert_eq!(third, &[42]);
        assert_eq!(recomputes, 2);
    }
}