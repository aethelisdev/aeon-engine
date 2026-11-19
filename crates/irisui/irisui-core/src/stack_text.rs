// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Fixed-Capacity Stack UTF-8 Text Buffer
//!
//! Provides [`StackString`], a fixed-size stack-allocated UTF-8 text buffer implementing
//! [`std::fmt::Write`] without dynamic heap allocation. Intended for high-frequency UI
//! telemetry, counters, and live HUD readouts in hot render/update loops.

use std::fmt::{self, Debug, Display, Write};
use std::ops::Deref;

/// Stack-allocated UTF-8 text buffer with fixed byte capacity `N` and zero heap allocation.
///
/// Implements [`std::fmt::Write`] and [`std::ops::Deref<Target = str>`]. Multi-byte UTF-8 input
/// that would exceed `N` bytes is truncated cleanly on a valid scalar codepoint boundary without
/// panicking or allocating.
#[derive(Clone, Copy)]
pub struct StackString<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for StackString<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> StackString<N> {
    /// Creates an empty stack-allocated text buffer with `N` bytes of inline capacity.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    /// Formats `args` into a fresh [`StackString`] without heap allocation.
    ///
    /// Truncates safely on a UTF-8 codepoint boundary if the formatted result exceeds `N` bytes.
    #[inline]
    #[must_use]
    pub fn from_args(args: fmt::Arguments<'_>) -> Self {
        let mut out = Self::new();
        let _ = out.write_fmt(args);
        out
    }

    /// Returns the valid UTF-8 string slice currently stored in this buffer.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    /// Returns the current initialized byte length in `[0, N]`.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the buffer contains zero bytes.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the fixed compile-time byte capacity `N`.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Resets the buffer length to zero without releasing or reallocating memory.
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends UTF-8 slice `s` up to the remaining capacity, stopping on codepoint boundaries.
    ///
    /// Returns `true` when the entire slice fit without truncation, or `false` if truncated.
    #[inline]
    pub fn push_str(&mut self, s: &str) -> bool {
        let remaining = N.saturating_sub(self.len);
        let copy_len = if s.len() <= remaining {
            s.len()
        } else {
            let mut end = remaining;
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            end
        };
        if copy_len > 0 {
            self.buf[self.len..self.len + copy_len].copy_from_slice(&s.as_bytes()[..copy_len]);
            self.len += copy_len;
        }
        copy_len == s.len()
    }
}

impl<const N: usize> Write for StackString<N> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = self.push_str(s);
        Ok(())
    }
}

impl<const N: usize> Deref for StackString<N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for StackString<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> PartialEq for StackString<N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for StackString<N> {}

impl<const N: usize> Debug for StackString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> Display for StackString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_string_formats_and_resets_without_allocation() {
        let mut s = StackString::<32>::new();
        assert!(s.is_empty());
        assert_eq!(s.capacity(), 32);

        write!(s, "{:.0} FPS ({:.2}ms)", 144.0_f32, 6.94_f32).expect("write succeeds");
        assert_eq!(s.as_str(), "144 FPS (6.94ms)");
        assert_eq!(s.len(), "144 FPS (6.94ms)".len());

        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.as_str(), "");
    }

    #[test]
    fn test_stack_string_truncates_on_utf8_boundary_without_panic() {
        let mut s = StackString::<5>::new();
        // '✓' is 3 bytes + " A" (2 bytes) = 5 bytes; 'B' is truncated safely.
        assert!(!s.push_str("✓ AB"));
        assert_eq!(s.as_str(), "✓ A");

        let mut split = StackString::<5>::new();
        // Second '✓' (bytes 3..6) exceeds capacity 5 -> stops cleanly on char boundary.
        assert!(!split.push_str("✓✓"));
        assert_eq!(split.as_str(), "✓");

        let multi = StackString::<4>::from_args(format_args!("⚠️⚠️"));
        assert!(std::str::from_utf8(multi.as_bytes()).is_ok());
        assert!(multi.len() <= 4);

        let zero_cap = StackString::<0>::from_args(format_args!("abc"));
        assert_eq!(zero_cap.as_str(), "");
        assert!(zero_cap.is_empty());
    }
}