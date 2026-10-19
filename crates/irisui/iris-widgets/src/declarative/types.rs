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

/// Formats a floating-point value cleanly without deceptive trailing zeros (e.g. `-115.8` instead of `-115.800`).
///
/// Follows industry standards ( / DCC tools):
/// - Integral values always retain a single trailing decimal: `0.0`, `1.0`, `50.0`, `-115.0`.
/// - Fractional values display up to `max_decimals` precision with redundant trailing zeros stripped:
///   `-115.800` -> `-115.8`, `-0.500` -> `-0.5`, `0.250` -> `0.25`.
#[inline]
pub fn format_compact_float(val: f32, max_decimals: usize) -> String {
    if !val.is_finite() {
        return format!("{}", val);
    }
    let formatted = format!("{:.precision$}", val, precision = max_decimals);
    if let Some(dot_idx) = formatted.find('.') {
        let trimmed = formatted.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{}0", trimmed)
        } else if trimmed.len() <= dot_idx + 1 {
            format!("{}0", &formatted[..=dot_idx])
        } else {
            trimmed.to_string()
        }
    } else {
        format!("{}.0", formatted)
    }
}

/// Evaluates a simple mathematical numeric expression string (e.g. `5-10`, `100/4`, `(2+3)*5`).
///
/// Follows standard DCC /  inspection ergonomics:
/// - Supports basic arithmetic operators: `+`, `-`, `*`, `/`.
/// - Supports unary minus/plus: `-5`, `+10`, `5 + -2`.
/// - Supports grouping parentheses: `(10 + 5) * 2`.
/// - Guarantees zero-division and non-finite protection: returns `None` on division by zero or invalid syntax.
pub fn eval_numeric_expression(expr: &str) -> Option<f32> {
    let clean = expr.trim();
    if clean.is_empty() {
        return None;
    }
    if let Ok(num) = clean.parse::<f32>() {
        return if num.is_finite() { Some(num) } else { None };
    }

    struct Parser<'a> {
        chars: std::iter::Peekable<std::str::Chars<'a>>,
    }

    impl<'a> Parser<'a> {
        fn skip_whitespace(&mut self) {
            while let Some(&c) = self.chars.peek() {
                if c.is_whitespace() {
                    self.chars.next();
                } else {
                    break;
                }
            }
        }

        fn parse_expr(&mut self) -> Option<f32> {
            let mut left = self.parse_term()?;
            loop {
                self.skip_whitespace();
                match self.chars.peek() {
                    Some('+') => {
                        self.chars.next();
                        let right = self.parse_term()?;
                        left += right;
                    }
                    Some('-') => {
                        self.chars.next();
                        let right = self.parse_term()?;
                        left -= right;
                    }
                    _ => break,
                }
            }
            if left.is_finite() { Some(left) } else { None }
        }

        fn parse_term(&mut self) -> Option<f32> {
            let mut left = self.parse_factor()?;
            loop {
                self.skip_whitespace();
                match self.chars.peek() {
                    Some('*') => {
                        self.chars.next();
                        let right = self.parse_factor()?;
                        left *= right;
                    }
                    Some('/') => {
                        self.chars.next();
                        let right = self.parse_factor()?;
                        if right.abs() < 1e-12 {
                            return None;
                        }
                        left /= right;
                    }
                    _ => break,
                }
            }
            if left.is_finite() { Some(left) } else { None }
        }

        fn parse_factor(&mut self) -> Option<f32> {
            self.skip_whitespace();
            let mut sign = 1.0;
            while let Some(&c) = self.chars.peek() {
                if c == '+' {
                    self.chars.next();
                    self.skip_whitespace();
                } else if c == '-' {
                    self.chars.next();
                    sign = -sign;
                    self.skip_whitespace();
                } else {
                    break;
                }
            }

            let val = match self.chars.peek() {
                Some('(') => {
                    self.chars.next();
                    let v = self.parse_expr()?;
                    self.skip_whitespace();
                    if self.chars.next() != Some(')') {
                        return None;
                    }
                    v
                }
                Some(&c) if c.is_ascii_digit() || c == '.' => {
                    let mut s = String::new();
                    let mut has_dot = false;
                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_digit() {
                            s.push(c);
                            self.chars.next();
                        } else if c == '.' && !has_dot {
                            has_dot = true;
                            s.push(c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    s.parse::<f32>().ok()?
                }
                _ => return None,
            };

            let res = sign * val;
            if res.is_finite() { Some(res) } else { None }
        }
    }

    let mut parser = Parser {
        chars: clean.chars().peekable(),
    };
    let result = parser.parse_expr()?;
    parser.skip_whitespace();
    if parser.chars.next().is_none() {
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_numeric_expression() {
        assert_eq!(eval_numeric_expression("5-10"), Some(-5.0));
        assert_eq!(eval_numeric_expression("10+5"), Some(15.0));
        assert_eq!(eval_numeric_expression("100/4"), Some(25.0));
        assert_eq!(eval_numeric_expression("2 + 3 * 4"), Some(14.0));
        assert_eq!(eval_numeric_expression("(2 + 3) * 4"), Some(20.0));
        assert_eq!(eval_numeric_expression("-115.8"), Some(-115.8));
        assert_eq!(eval_numeric_expression("5 / 0"), None);
        assert_eq!(eval_numeric_expression("44444444"), Some(44444444.0));
    }
}