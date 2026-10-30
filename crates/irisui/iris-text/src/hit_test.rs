// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Hit-Testing Engine (`iris-text::hit_test`)
//!
//! Subpixel glyph boundary resolver for interactive text editing controls.
//! Supports both direct typography engine evaluation and zero-allocation $O(\log N)$
//! binary search over retained/cached layout metrics.

use crate::system::TextSystem;

/// Resolves the nearest character index from an ordered slice of cumulative character advances.
///
/// Designed for retained UI architectures: queries precomputed character widths cached during
/// layout/rendering with zero font engine overhead, zero `RefCell` borrowing, and zero heap allocation.
///
/// Uses binary search ($O(\log N)$) combined with midpoint rounding: if `local_x` falls in the
/// left half of a character, the caret is placed before it; if in the right half, after it.
///
/// # Arguments
/// * `glyph_advances` - Monotonically increasing slice of cumulative right-edge X positions.
/// * `local_x` - Horizontal click position relative to text origin.
///
/// # Returns
/// Nearest character boundary index ranging from `0` to `glyph_advances.len()`.
pub fn hit_test_from_cached_advances(glyph_advances: &[f32], local_x: f32) -> usize {
    if glyph_advances.is_empty() || local_x <= 0.0 {
        return 0;
    }
    let total_w = *glyph_advances.last().unwrap_or(&0.0);
    if local_x >= total_w {
        return glyph_advances.len();
    }

    match glyph_advances.binary_search_by(|adv| {
        adv.partial_cmp(&local_x)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        Ok(exact_idx) => exact_idx + 1,
        Err(idx) => {
            let prev_x = if idx == 0 {
                0.0
            } else {
                glyph_advances[idx - 1]
            };
            let next_x = glyph_advances[idx];
            let mid = (prev_x + next_x) * 0.5;
            if local_x < mid { idx } else { idx + 1 }
        }
    }
}

/// Resolves the nearest UTF-8 byte offset from an ordered slice of glyph clusters `(start_byte_idx, right_x)`.
///
/// Provides $O(\log N)$ hit-testing over cached multi-byte UTF-8 glyph clusters (such as non-ASCII
/// characters or emojis) without calling the font shaping engine.
///
/// # Arguments
/// * `clusters` - Slice of `(start_byte_idx, right_x)` tuples ordered by right_x.
/// * `total_bytes` - Total byte length of the source string (`text.len()`).
/// * `local_x` - Horizontal pointer position relative to text origin.
///
/// # Returns
/// Valid UTF-8 byte offset in `0..=total_bytes`.
pub fn hit_test_from_cached_clusters(
    clusters: &[(usize, f32)],
    total_bytes: usize,
    local_x: f32,
) -> usize {
    if clusters.is_empty() || local_x <= 0.0 {
        return 0;
    }
    let total_w = clusters.last().map(|c| c.1).unwrap_or(0.0);
    if local_x >= total_w {
        return total_bytes;
    }

    match clusters.binary_search_by(|c| {
        c.1.partial_cmp(&local_x)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        Ok(idx) => {
            // Exactly on the boundary of clusters[idx] -> start of next or total_bytes
            if idx + 1 < clusters.len() {
                clusters[idx + 1].0
            } else {
                total_bytes
            }
        }
        Err(idx) => {
            let prev_x = if idx == 0 { 0.0 } else { clusters[idx - 1].1 };
            let (curr_start, curr_right) = clusters[idx];
            let mid = (prev_x + curr_right) * 0.5;
            if local_x < mid {
                curr_start
            } else if idx + 1 < clusters.len() {
                clusters[idx + 1].0
            } else {
                total_bytes
            }
        }
    }
}

/// Resolves the nearest UTF-8 character boundary index for a given horizontal pixel offset
/// directly using an explicit `&mut TextSystem` reference (zero hidden global state).
///
/// Evaluates shaped glyph geometries from `cosmic-text` and calculates mid-glyph rounding.
///
/// # Arguments
/// * `system` - Explicit mutable reference to the typography engine.
/// * `text` - Source text string being hit-tested.
/// * `font_size` - Physical font size in pixels.
/// * `local_x` - Horizontal click position relative to the text content start.
///
/// # Returns
/// Valid UTF-8 byte offset within `text` (ranging from `0` to `text.len()`).
pub fn index_for_position_in_system(
    system: &mut TextSystem,
    text: &str,
    font_size: f32,
    local_x: f32,
) -> usize {
    if text.is_empty() || local_x <= 0.0 {
        return 0;
    }

    let params = crate::system::TextShapeParams {
        font_size,
        line_height: font_size * 1.2,
        bounds_width: f32::MAX,
        bounds_height: font_size * 2.0,
        align: iris_core::TextAlign::Left,
        wrap: iris_core::TextWrap::None,
    };

    let buffer = system.shape_text(text, params);
    let Some(run) = buffer.layout_runs().next() else {
        return 0;
    };

    if local_x >= run.line_w {
        return text.len();
    }

    let mut last_end = 0;
    for glyph in run.glyphs.iter() {
        let glyph_start_x = glyph.x;
        let glyph_width = glyph.w;
        let mid_x = glyph_start_x + glyph_width * 0.5;

        if local_x < mid_x {
            return glyph.start;
        }

        last_end = glyph.end;
        if local_x <= glyph_start_x + glyph_width {
            return glyph.end;
        }
    }

    last_end.min(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_test_from_cached_advances_binary_search() {
        // 4 characters: 'A' (10px), 'B' (10px), 'C' (10px), 'D' (10px)
        let advances = [10.0, 20.0, 30.0, 40.0];

        // Before or at start
        assert_eq!(hit_test_from_cached_advances(&advances, -5.0), 0);
        assert_eq!(hit_test_from_cached_advances(&advances, 0.0), 0);

        // Within 'A' (0..10), midpoint is 5.0
        assert_eq!(hit_test_from_cached_advances(&advances, 3.0), 0);
        assert_eq!(hit_test_from_cached_advances(&advances, 5.0), 1);
        assert_eq!(hit_test_from_cached_advances(&advances, 7.0), 1);

        // Within 'B' (10..20), midpoint is 15.0
        assert_eq!(hit_test_from_cached_advances(&advances, 12.0), 1);
        assert_eq!(hit_test_from_cached_advances(&advances, 17.0), 2);

        // Within 'D' (30..40), midpoint is 35.0
        assert_eq!(hit_test_from_cached_advances(&advances, 32.0), 3);
        assert_eq!(hit_test_from_cached_advances(&advances, 38.0), 4);

        // Beyond total width
        assert_eq!(hit_test_from_cached_advances(&advances, 40.0), 4);
        assert_eq!(hit_test_from_cached_advances(&advances, 100.0), 4);
    }

    #[test]
    fn test_hit_test_from_cached_clusters_utf8() {
        // "şö" -> 'ş' (2 bytes, 12.0px), 'ö' (2 bytes, 24.0px)
        let clusters = [(0, 12.0), (2, 24.0)];
        let total_bytes = 4;

        assert_eq!(
            hit_test_from_cached_clusters(&clusters, total_bytes, 0.0),
            0
        );
        // Left half of 'ş' (midpoint 6.0)
        assert_eq!(
            hit_test_from_cached_clusters(&clusters, total_bytes, 4.0),
            0
        );
        // Right half of 'ş'
        assert_eq!(
            hit_test_from_cached_clusters(&clusters, total_bytes, 8.0),
            2
        );
        // Right half of 'ö'
        assert_eq!(
            hit_test_from_cached_clusters(&clusters, total_bytes, 20.0),
            4
        );
        // Beyond
        assert_eq!(
            hit_test_from_cached_clusters(&clusters, total_bytes, 50.0),
            4
        );
    }

    #[test]
    fn test_hit_test_ascii_basic_in_system() {
        let mut system = TextSystem::new();
        let text = "Hello";
        let font_size = 14.0;

        assert_eq!(
            index_for_position_in_system(&mut system, text, font_size, -10.0),
            0
        );
        assert_eq!(
            index_for_position_in_system(&mut system, text, font_size, 0.0),
            0
        );
        assert_eq!(
            index_for_position_in_system(&mut system, text, font_size, 5000.0),
            text.len()
        );
    }

    #[test]
    fn test_hit_test_turkish_characters_in_system() {
        let mut system = TextSystem::new();
        let text = "şöğüç";
        let font_size = 14.0;

        assert_eq!(
            index_for_position_in_system(&mut system, text, font_size, 0.0),
            0
        );
        let total_adv = system.measure_advance(text, font_size);
        assert!(total_adv > 0.0);

        assert_eq!(
            index_for_position_in_system(&mut system, text, font_size, total_adv + 20.0),
            text.len()
        );

        for step in 0..20 {
            let x = total_adv * (step as f32 / 20.0);
            let idx = index_for_position_in_system(&mut system, text, font_size, x);
            assert!(
                text.is_char_boundary(idx),
                "Hit-tested index {idx} must be a valid UTF-8 character boundary"
            );
        }
    }
}