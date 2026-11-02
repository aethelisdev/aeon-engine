// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mathematical Visible Slice Engine for Virtualized Lists
//!
//! Provides an exact $O(1)$ zero-allocation windowing engine for arbitrarily large datasets (100k+ rows).
//! Calculates the visible half-open index range (`first_index..last_index`), dynamic top/bottom
//! spacer geometries to emulate full scroll container volume, and total virtual height.

/// Default overscan buffer margin (2 items above and 2 items below visible bounds).
///
/// Prevents blank frame pop-in and white flashes during high-velocity trackpad and mousewheel fling gestures.
pub const DEFAULT_OVERSCAN: usize = 2;

/// Input parameters configuring the $O(1)$ virtual list slice calculation.
///
/// Encapsulates dataset size, row height, viewport dimensions, vertical scroll position,
/// and bidirectional overscan buffer margins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualSliceParams {
    /// Total count of items in the underlying dataset ($N$).
    pub item_count: usize,
    /// Fixed vertical stride or height allocated to each item in logical pixels ($H$).
    pub item_height: f32,
    /// Height of the visible clipping viewport container in logical pixels ($V$).
    pub viewport_height: f32,
    /// Current vertical scroll position in logical pixels ($S_y$).
    pub scroll_y: f32,
    /// Number of overscan buffer items retained beyond top and bottom viewport edges.
    pub overscan: usize,
}

impl Default for VirtualSliceParams {
    /// Creates a default parameter configuration with 0 items, 24px item height, 0px viewport, and default overscan.
    #[inline]
    fn default() -> Self {
        Self {
            item_count: 0,
            item_height: 24.0,
            viewport_height: 0.0,
            scroll_y: 0.0,
            overscan: DEFAULT_OVERSCAN,
        }
    }
}

impl VirtualSliceParams {
    /// Creates a new parameter block with the default bidirectional overscan buffer (2 items).
    ///
    /// # Arguments
    ///
    /// * `item_count` - Total number of elements in the collection ($N$).
    /// * `item_height` - Uniform height per item in logical pixels ($H$).
    /// * `viewport_height` - Vertical height of the visible scrolling frame ($V$).
    /// * `scroll_y` - Current vertical scroll offset ($S_y$).
    #[inline]
    #[must_use]
    pub const fn new(
        item_count: usize,
        item_height: f32,
        viewport_height: f32,
        scroll_y: f32,
    ) -> Self {
        Self {
            item_count,
            item_height,
            viewport_height,
            scroll_y,
            overscan: DEFAULT_OVERSCAN,
        }
    }

    /// Overrides the number of overscan buffer items placed above and below the visible window.
    ///
    /// Setting this to `0` renders only the strictly visible rows, while larger values (e.g. 5)
    /// provide a wider pre-rendered margin at the cost of rendering a few additional DOM nodes.
    #[inline]
    #[must_use]
    pub const fn with_overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }
}

/// Description of an active virtualized window of visible items within a collection.
///
/// Contains the visible half-open index range (`first_index..last_index`), top and bottom
/// spacer heights simulating the full scroll height without allocating inactive DOM nodes,
/// and the aggregate virtual content height.
///
/// # Mathematical Invariant
///
/// For any valid slice:
///
/// $$H_{\text{top}} + (\text{last\_index} - \text{first\_index}) \times H + H_{\text{bottom}} \equiv H_{\text{total}}$$
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualSlice {
    /// Zero-based index of the first item intersecting the visible viewport or top overscan (inclusive).
    pub first_index: usize,
    /// Zero-based ending index of items intersecting the visible viewport or bottom overscan (exclusive).
    pub last_index: usize,
    /// Height in logical pixels of the top spacer element preceding visible items ($H_{\text{top}}$).
    pub top_spacer: f32,
    /// Height in logical pixels of the bottom spacer element succeeding visible items ($H_{\text{bottom}}$).
    pub bottom_spacer: f32,
    /// Total virtual content height in logical pixels ($N \times H$).
    pub total_height: f32,
}

impl VirtualSlice {
    /// Creates an empty slice with zero visible items and zero heights.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            first_index: 0,
            last_index: 0,
            top_spacer: 0.0,
            bottom_spacer: 0.0,
            total_height: 0.0,
        }
    }

    /// Number of items within the visible slice (`last_index - first_index`).
    #[inline]
    #[must_use]
    pub const fn visible_count(&self) -> usize {
        self.last_index.saturating_sub(self.first_index)
    }

    /// Returns `true` if the slice contains zero visible items.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.first_index >= self.last_index
    }

    /// Returns the half-open index range of visible items (`first_index..last_index`).
    #[inline]
    #[must_use]
    pub const fn range(&self) -> std::ops::Range<usize> {
        self.first_index..self.last_index
    }

    /// Computes the aggregate rendered height of visible items.
    #[inline]
    #[must_use]
    pub fn rendered_height(&self, item_height: f32) -> f32 {
        self.visible_count() as f32 * item_height
    }

    /// Returns whether the specified item index is contained within the visible slice.
    #[inline]
    #[must_use]
    pub const fn contains(&self, index: usize) -> bool {
        index >= self.first_index && index < self.last_index
    }
}

/// Calculates the visible slice and spacer geometries in $O(1)$ constant time.
///
/// # Mathematical Model
///
/// Given $N$ total items, uniform item height $H$, viewport height $V$, and scroll position $S_y$:
///
/// $$\text{first\_idx} = \max\left(0, \left\lfloor \frac{S_y}{H} \right\rfloor - \text{overscan}\right)$$
/// $$\text{last\_idx} = \min\left(N, \left\lceil \frac{S_y + V}{H} \right\rceil + \text{overscan}\right)$$
/// $$H_{\text{top}} = \text{first\_idx} \times H$$
/// $$H_{\text{bottom}} = (N - \text{last\_idx}) \times H$$
/// $$H_{\text{total}} = N \times H$$
///
/// # Invariants & Guarantees
///
/// - **Zero Allocations**: Pure arithmetic calculation with zero heap allocations in per-frame execution.
/// - **$O(1)$ Scaling**: Time complexity is strictly constant irrespective of whether $N = 10$ or $N = 10.000.000$.
/// - **Exact Spatial Partitioning**: The sum of top spacer, rendered items, and bottom spacer always equals $H_{\text{total}}$.
/// - **Robust Boundary Safety**: Gracefully handles `NaN`, infinite values, negative overscroll (elastic rubber-banding),
///   and deep overscroll past the bottom edge.
#[must_use]
pub fn compute_virtual_slice(params: VirtualSliceParams) -> VirtualSlice {
    if params.item_count == 0 || !params.item_height.is_finite() || params.item_height <= 0.0 {
        return VirtualSlice::empty();
    }

    let item_count = params.item_count;
    let item_height = params.item_height;
    let h = item_height as f64;
    let total_height = (item_count as f64 * h) as f32;

    let v = if params.viewport_height.is_finite() && params.viewport_height > 0.0 {
        params.viewport_height as f64
    } else {
        0.0
    };

    let sy = if params.scroll_y.is_finite() {
        params.scroll_y as f64
    } else {
        0.0
    };

    let overscan = params.overscan as i64;
    let n = item_count as i64;

    // first_idx = max(0, floor(Sy / H) - overscan)
    let raw_first = (sy / h).floor() as i64;
    let first_idx_i64 = (raw_first - overscan).clamp(0, n);
    let first_index = first_idx_i64 as usize;

    // last_idx = min(N, ceil((Sy + V) / H) + overscan)
    let raw_last = ((sy + v) / h).ceil() as i64;
    let last_idx_i64 = (raw_last + overscan).clamp(0, n);
    let last_index = (last_idx_i64 as usize).max(first_index);

    let top_spacer = (first_index as f64 * h) as f32;
    let bottom_spacer = ((item_count - last_index) as f64 * h) as f32;

    VirtualSlice {
        first_index,
        last_index,
        top_spacer,
        bottom_spacer,
        total_height,
    }
}

/// Convenience function to compute a virtual slice directly from raw scalar parameters.
///
/// Delegates directly to [`compute_virtual_slice`].
#[inline]
#[must_use]
pub fn compute_slice(
    item_count: usize,
    item_height: f32,
    viewport_height: f32,
    scroll_y: f32,
    overscan: usize,
) -> VirtualSlice {
    compute_virtual_slice(VirtualSliceParams {
        item_count,
        item_height,
        viewport_height,
        scroll_y,
        overscan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies default parameters and fluent builder configuration.
    #[test]
    fn test_virtual_slice_default_params() {
        let params = VirtualSliceParams::default();
        assert_eq!(params.item_count, 0);
        assert_eq!(params.item_height, 24.0);
        assert_eq!(params.viewport_height, 0.0);
        assert_eq!(params.scroll_y, 0.0);
        assert_eq!(params.overscan, DEFAULT_OVERSCAN);

        let custom = VirtualSliceParams::new(500, 30.0, 400.0, 100.0).with_overscan(4);
        assert_eq!(custom.item_count, 500);
        assert_eq!(custom.item_height, 30.0);
        assert_eq!(custom.viewport_height, 400.0);
        assert_eq!(custom.scroll_y, 100.0);
        assert_eq!(custom.overscan, 4);
    }

    /// Verifies Blueprint requirement: $N=100.000, H=24.0, V=600.0$ at $S_y=0.0$ (top of list).
    #[test]
    fn test_virtual_slice_n100k_top() {
        let params = VirtualSliceParams::new(100_000, 24.0, 600.0, 0.0);
        let slice = compute_virtual_slice(params);

        // At Sy=0: floor(0/24) - 2 = -2 -> clamped to 0
        assert_eq!(slice.first_index, 0);
        // ceil(600/24) + 2 = 25 + 2 = 27
        assert_eq!(slice.last_index, 27);
        assert_eq!(slice.visible_count(), 27);
        assert_eq!(slice.top_spacer, 0.0);
        assert_eq!(slice.bottom_spacer, (100_000 - 27) as f32 * 24.0);
        assert_eq!(slice.total_height, 2_400_000.0);

        // Partition invariant check: top_spacer + rendered_height + bottom_spacer == total_height
        let rendered = slice.rendered_height(24.0);
        assert_eq!(
            slice.top_spacer + rendered + slice.bottom_spacer,
            slice.total_height
        );
    }

    /// Verifies Blueprint requirement: $N=100.000, H=24.0, V=600.0$ at $S_y=1200.0$ (middle of list).
    #[test]
    fn test_virtual_slice_n100k_middle() {
        let params = VirtualSliceParams::new(100_000, 24.0, 600.0, 1200.0);
        let slice = compute_virtual_slice(params);

        // Sy/H = 1200/24 = 50. first_idx = 50 - 2 = 48
        assert_eq!(slice.first_index, 48);
        // (Sy + V)/H = (1200 + 600)/24 = 1800/24 = 75. last_idx = 75 + 2 = 77
        assert_eq!(slice.last_index, 77);
        assert_eq!(slice.visible_count(), 29);
        assert_eq!(slice.top_spacer, 48.0 * 24.0);
        assert_eq!(slice.bottom_spacer, (100_000 - 77) as f32 * 24.0);
        assert_eq!(slice.total_height, 2_400_000.0);

        let rendered = slice.rendered_height(24.0);
        assert_eq!(
            slice.top_spacer + rendered + slice.bottom_spacer,
            slice.total_height
        );
        assert!(slice.contains(48));
        assert!(slice.contains(76));
        assert!(!slice.contains(47));
        assert!(!slice.contains(77));
    }

    /// Verifies Blueprint requirement: $N=100.000, H=24.0, V=600.0$ at $S_y=2.399.000.0$ (near bottom edge).
    #[test]
    fn test_virtual_slice_n100k_near_bottom() {
        let params = VirtualSliceParams::new(100_000, 24.0, 600.0, 2_399_000.0);
        let slice = compute_virtual_slice(params);

        // Sy/H = 2_399_000 / 24 = 99958.333... -> floor is 99958. first_idx = 99958 - 2 = 99956
        assert_eq!(slice.first_index, 99_956);
        // (Sy + V)/H = 2_399_600 / 24 = 99983.333... -> ceil is 99984. last_idx = 99984 + 2 = 99986
        assert_eq!(slice.last_index, 99_986);
        assert_eq!(slice.visible_count(), 30);
        assert_eq!(slice.top_spacer, 99_956.0 * 24.0);
        assert_eq!(slice.bottom_spacer, (100_000 - 99_986) as f32 * 24.0);
        assert_eq!(slice.total_height, 2_400_000.0);

        let rendered = slice.rendered_height(24.0);
        assert_eq!(
            slice.top_spacer + rendered + slice.bottom_spacer,
            slice.total_height
        );
    }

    /// Verifies exact bottom boundary where $S_y + V == H_{\text{total}}$.
    #[test]
    fn test_virtual_slice_exact_bottom_boundary() {
        // Max scroll = 2_400_000 - 600 = 2_399_400.0
        let params = VirtualSliceParams::new(100_000, 24.0, 600.0, 2_399_400.0);
        let slice = compute_virtual_slice(params);

        // Sy/H = 2_399_400 / 24 = 99975. first_idx = 99975 - 2 = 99973
        assert_eq!(slice.first_index, 99_973);
        // (Sy + V)/H = 2_400_000 / 24 = 100000. last_idx = min(100000, 100000 + 2) = 100000
        assert_eq!(slice.last_index, 100_000);
        assert_eq!(slice.visible_count(), 27);
        assert_eq!(slice.bottom_spacer, 0.0);
        assert_eq!(slice.total_height, 2_400_000.0);

        let rendered = slice.rendered_height(24.0);
        assert_eq!(
            slice.top_spacer + rendered + slice.bottom_spacer,
            slice.total_height
        );
    }

    /// Verifies empty collection ($N=0$) safety and invariants.
    #[test]
    fn test_virtual_slice_empty_collection() {
        let params = VirtualSliceParams::new(0, 24.0, 600.0, 100.0);
        let slice = compute_virtual_slice(params);

        assert_eq!(slice, VirtualSlice::empty());
        assert_eq!(slice.visible_count(), 0);
        assert!(slice.is_empty());
        assert_eq!(slice.range(), 0..0);
        assert_eq!(slice.top_spacer, 0.0);
        assert_eq!(slice.bottom_spacer, 0.0);
        assert_eq!(slice.total_height, 0.0);
    }

    /// Verifies degenerate inputs: zero or negative item heights and non-finite values.
    #[test]
    fn test_virtual_slice_degenerate_and_nan() {
        // Zero item height
        let slice_zero_h = compute_slice(100, 0.0, 600.0, 0.0, 2);
        assert_eq!(slice_zero_h, VirtualSlice::empty());

        // Negative item height
        let slice_neg_h = compute_slice(100, -20.0, 600.0, 0.0, 2);
        assert_eq!(slice_neg_h, VirtualSlice::empty());

        // NaN item height
        let slice_nan_h = compute_slice(100, f32::NAN, 600.0, 0.0, 2);
        assert_eq!(slice_nan_h, VirtualSlice::empty());

        // NaN scroll_y gracefully treated as 0.0
        let slice_nan_sy = compute_slice(100, 20.0, 200.0, f32::NAN, 2);
        assert_eq!(slice_nan_sy.first_index, 0);
        assert_eq!(slice_nan_sy.total_height, 2000.0);

        // NaN viewport_height treated as 0.0
        let slice_nan_v = compute_slice(100, 20.0, f32::NAN, 50.0, 2);
        assert_eq!(slice_nan_v.total_height, 2000.0);
    }

    /// Verifies negative overscroll (elastic rubber-banding above the top of the collection).
    #[test]
    fn test_virtual_slice_negative_overscroll_rubberband() {
        let params = VirtualSliceParams::new(1000, 20.0, 400.0, -250.0);
        let slice = compute_virtual_slice(params);

        assert_eq!(slice.first_index, 0);
        assert_eq!(slice.top_spacer, 0.0);
        // Sy + V = -250 + 400 = 150. ceil(150 / 20) = 8. last_idx = 8 + 2 = 10
        assert_eq!(slice.last_index, 10);
        assert_eq!(slice.visible_count(), 10);

        let rendered = slice.rendered_height(20.0);
        assert_eq!(
            slice.top_spacer + rendered + slice.bottom_spacer,
            slice.total_height
        );
    }

    /// Verifies extreme overscroll past the bottom edge.
    #[test]
    fn test_virtual_slice_extreme_bottom_overscroll() {
        let params = VirtualSliceParams::new(1000, 20.0, 400.0, 50_000.0);
        let slice = compute_virtual_slice(params);

        // Both indices clamped to N (1000)
        assert_eq!(slice.first_index, 1000);
        assert_eq!(slice.last_index, 1000);
        assert_eq!(slice.visible_count(), 0);
        assert_eq!(slice.top_spacer, 20_000.0);
        assert_eq!(slice.bottom_spacer, 0.0);
        assert_eq!(slice.total_height, 20_000.0);
        assert_eq!(
            slice.top_spacer + slice.rendered_height(20.0) + slice.bottom_spacer,
            slice.total_height
        );
    }

    /// Verifies overscan parameter tuning (0 vs 2 vs 5).
    #[test]
    fn test_virtual_slice_overscan_tuning() {
        let item_count = 100;
        let item_height = 20.0;
        let viewport_h = 100.0; // 5 items exactly fit viewport
        let scroll_y = 40.0; // Starts at item 2

        // Overscan = 0
        let s0 = compute_slice(item_count, item_height, viewport_h, scroll_y, 0);
        assert_eq!(s0.first_index, 2);
        assert_eq!(s0.last_index, 7); // 2 + 5 = 7
        assert_eq!(s0.visible_count(), 5);

        // Overscan = 2
        let s2 = compute_slice(item_count, item_height, viewport_h, scroll_y, 2);
        assert_eq!(s2.first_index, 0); // 2 - 2 = 0
        assert_eq!(s2.last_index, 9); // 7 + 2 = 9
        assert_eq!(s2.visible_count(), 9);

        // Overscan = 5
        let s5 = compute_slice(item_count, item_height, viewport_h, scroll_y, 5);
        assert_eq!(s5.first_index, 0); // clamp(2 - 5) = 0
        assert_eq!(s5.last_index, 12); // 7 + 5 = 12
        assert_eq!(s5.visible_count(), 12);
    }

    /// Verifies fractional heights and fractional scroll offsets.
    #[test]
    fn test_virtual_slice_fractional_metrics() {
        let params = VirtualSliceParams::new(500, 19.5, 301.7, 143.25);
        let slice = compute_virtual_slice(params);

        // 143.25 / 19.5 = 7.346... -> floor is 7. first_idx = 7 - 2 = 5
        assert_eq!(slice.first_index, 5);
        // (143.25 + 301.7) / 19.5 = 444.95 / 19.5 = 22.817... -> ceil is 23. last_idx = 23 + 2 = 25
        assert_eq!(slice.last_index, 25);
        assert_eq!(slice.visible_count(), 20);

        let rendered = slice.rendered_height(19.5);
        let sum = slice.top_spacer + rendered + slice.bottom_spacer;
        assert!((sum - slice.total_height).abs() < 1e-3);
    }

    /// Verifies $O(1)$ constant time scaling and numerical stability with 10.000.000 items.
    #[test]
    fn test_virtual_slice_o1_scaling_stress() {
        let params = VirtualSliceParams::new(10_000_000, 24.0, 800.0, 120_000_000.0);
        let start = std::time::Instant::now();
        let slice = compute_virtual_slice(params);
        let elapsed = start.elapsed();

        // Must execute in sub-microsecond range (< 50 microseconds even on debug build)
        assert!(elapsed.as_micros() < 500);

        // Total content height: 10_000_000 * 24 = 240_000_000.0
        assert_eq!(slice.total_height, 240_000_000.0);
        assert_eq!(slice.visible_count(), 38); // 800/24 = 33.33 -> 34 + 4 = 38
        assert_eq!(slice.first_index, 4_999_998); // 120_000_000 / 24 = 5_000_000 - 2 = 4_999_998
        assert_eq!(slice.last_index, 5_000_036); // 5_000_000 + 34 + 2 = 5_000_036

        let rendered = slice.rendered_height(24.0);
        let sum = slice.top_spacer as f64 + rendered as f64 + slice.bottom_spacer as f64;
        assert!((sum - slice.total_height as f64).abs() < 1.0);
    }
}