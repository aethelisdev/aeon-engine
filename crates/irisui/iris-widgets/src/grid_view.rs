// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Responsive Grid View Layout & Virtualization
//!
//! Provides mathematically verified, responsive wrapping grid layout calculations and
//! viewport scissor culling for card grids, asset browsers, and inventory slots.
//!
//! Handles column computation, row distribution, total scroll height estimation,
//! and virtualized visible cell filtering.

use iris_core::geometry::Rect;

/// Configuration and layout calculator for responsive, wrapping grid views.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResponsiveGrid {
    /// Width and height of an individual grid cell in logical pixels.
    pub item_size: (f32, f32),
    /// Horizontal and vertical spacing between adjacent grid cells.
    pub spacing: (f32, f32),
    /// Outer horizontal and vertical padding around the grid content.
    pub padding: (f32, f32),
}

impl ResponsiveGrid {
    /// Creates a new responsive grid layout with specified item dimensions.
    #[inline]
    pub fn new(item_width: f32, item_height: f32) -> Self {
        Self {
            item_size: (item_width.max(1.0), item_height.max(1.0)),
            spacing: (8.0, 8.0),
            padding: (8.0, 8.0),
        }
    }

    /// Sets the horizontal and vertical spacing between cells in logical pixels.
    #[inline]
    pub fn spacing(mut self, spacing_x: f32, spacing_y: f32) -> Self {
        self.spacing = (spacing_x.max(0.0), spacing_y.max(0.0));
        self
    }

    /// Sets the outer horizontal and vertical padding around the grid in logical pixels.
    #[inline]
    pub fn padding(mut self, padding_x: f32, padding_y: f32) -> Self {
        self.padding = (padding_x.max(0.0), padding_y.max(0.0));
        self
    }

    /// Computes the number of columns that fit within the given available container width.
    #[inline]
    pub fn compute_columns(&self, available_width: f32) -> usize {
        let content_width = (available_width - 2.0 * self.padding.0).max(self.item_size.0);
        let stride = self.item_size.0 + self.spacing.0;
        if stride <= 0.0 {
            return 1;
        }
        let cols = ((content_width + self.spacing.0) / stride).floor();
        (cols as usize).max(1)
    }

    /// Computes the total required scrollable content height for a given number of items.
    #[inline]
    pub fn compute_content_height(&self, total_items: usize, available_width: f32) -> f32 {
        if total_items == 0 {
            return 0.0;
        }
        let cols = self.compute_columns(available_width);
        let rows = total_items.div_ceil(cols);
        let row_stride = self.item_size.1 + self.spacing.1;
        2.0 * self.padding.1 + (rows as f32 * row_stride) - self.spacing.1
    }

    /// Computes the visible cells and their layout bounding boxes within a viewport rectangle.
    ///
    /// Applies viewport culling to skip all rows completely outside the visible viewport area,
    /// returning only visible items with their global 0-based indices and `Rect` positions.
    pub fn compute_visible_cells(
        &self,
        vp_rect: Rect,
        total_items: usize,
        scroll_y: f32,
    ) -> Vec<(usize, Rect)> {
        if total_items == 0 {
            return Vec::new();
        }

        let cols = self.compute_columns(vp_rect.width);
        let start_x = vp_rect.x + self.padding.0;
        let start_y = vp_rect.y + self.padding.1 - scroll_y;
        let row_stride = self.item_size.1 + self.spacing.1;
        let col_stride = self.item_size.0 + self.spacing.0;

        let total_rows = total_items.div_ceil(cols);
        let mut visible = Vec::with_capacity(cols * 8);

        for row_idx in 0..total_rows {
            let row_y = start_y + (row_idx as f32 * row_stride);

            // Viewport Scissor Cull: skip rows completely outside visible screen
            if row_y + self.item_size.1 <= vp_rect.y {
                continue;
            }
            if row_y >= vp_rect.bottom() {
                break;
            }

            let start_item_idx = row_idx * cols;
            let end_item_idx = (start_item_idx + cols).min(total_items);

            for (col_idx, item_idx) in (start_item_idx..end_item_idx).enumerate() {
                let card_x = start_x + (col_idx as f32 * col_stride);
                let cell_rect = Rect::new(card_x, row_y, self.item_size.0, self.item_size.1);
                visible.push((item_idx, cell_rect));
            }
        }

        visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_responsive_grid_column_calculation() {
        let grid = ResponsiveGrid::new(100.0, 100.0)
            .spacing(10.0, 10.0)
            .padding(10.0, 10.0);

        // 340.0 width: padding=20.0, available=320.0.
        // stride = 110.0. (320 + 10) / 110 = 3.0 -> 3 cols
        assert_eq!(grid.compute_columns(340.0), 3);

        // narrow width should always yield at least 1 col
        assert_eq!(grid.compute_columns(50.0), 1);
    }

    #[test]
    fn test_responsive_grid_viewport_culling() {
        let grid = ResponsiveGrid::new(100.0, 100.0)
            .spacing(10.0, 10.0)
            .padding(10.0, 10.0);

        let vp_rect = Rect::new(0.0, 0.0, 340.0, 200.0);
        // Total 30 items -> 10 rows.
        // With scroll_y = 0.0:
        // row 0: y=10, height=100 (visible)
        // row 1: y=120, height=100 (visible)
        // row 2: y=230 > 200 (culled)
        let visible = grid.compute_visible_cells(vp_rect, 30, 0.0);
        assert_eq!(visible.len(), 6); // 2 rows * 3 cols = 6 items
        assert_eq!(visible[0].0, 0);
        assert_eq!(visible[5].0, 5);

        // With scroll_y = 110.0 (scrolled down 1 row):
        // row 0: y=-100 < 0 (culled)
        // row 1: y=10 (visible)
        // row 2: y=120 (visible)
        let visible_scrolled = grid.compute_visible_cells(vp_rect, 30, 110.0);
        assert_eq!(visible_scrolled.len(), 6);
        assert_eq!(visible_scrolled[0].0, 3);
        assert_eq!(visible_scrolled[5].0, 8);
    }
}