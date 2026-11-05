// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hierarchical AABB scissor clipping stack for nested UI container rendering.
//!
//! Provides mathematically verified axis-aligned bounding box (AABB) intersection tracking
//! for nested UI structures (e.g. `DockPanel` -> `ScrollArea` -> `VirtualList`). Ensures that
//! child widgets are strictly clipped within the intersection of all ancestor containers,
//! while allowing elevated layers (modals, dropdown popups, tooltips) to cleanly decouple
//! without clipping leaks.

use iris_core::Rect;

/// Maintains an axis-aligned bounding box (AABB) clipping stack for hierarchical rendering.
///
/// Each nested container with clipping enabled (`clip_children = true`) pushes its layout bounds
/// onto the stack. The active clipping rectangle is computed as the cumulative intersection of the
/// incoming rectangle with the current top of the stack. When exiting a container, [`Self::pop`]
/// restores the previous clipping bounds without re-evaluating ancestor hierarchies.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScissorStack {
    /// Ordered hierarchy of cumulative intersection rectangles from root to leaf.
    stack: Vec<Rect>,
    /// Cached current cumulative clipping rectangle, or `None` if unclipped.
    current: Option<Rect>,
}

impl ScissorStack {
    /// Creates a new, empty scissor stack with no active clipping.
    #[inline]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: None,
        }
    }

    /// Creates a scissor stack with pre-allocated capacity for nested container depths.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
            current: None,
        }
    }

    /// Creates a scissor stack initialized with an optional root clipping rectangle.
    #[inline]
    pub fn with_initial_clip(clip: Option<Rect>) -> Self {
        match clip {
            Some(rect) => Self {
                stack: vec![rect],
                current: Some(rect),
            },
            None => Self::new(),
        }
    }

    /// Pushes a rectangle onto the stack, intersecting it with the current active clip.
    ///
    /// If the stack currently has an active clipping rectangle, the new effective clip is
    /// `current.intersect(rect)`. If the stack is empty, `rect` becomes the active clip.
    /// Returns the newly computed cumulative clipping rectangle.
    pub fn push(&mut self, rect: Rect) -> Option<Rect> {
        let intersected = match self.current {
            Some(curr) => curr.intersect(rect),
            None => rect,
        };
        self.stack.push(intersected);
        self.current = Some(intersected);
        self.current
    }

    /// Pops the topmost clipping rectangle from the stack and restores the parent clip.
    ///
    /// Returns the restored active clipping rectangle, or `None` if the stack is now empty.
    /// If the stack was already empty, this is a safe no-op returning `None`.
    pub fn pop(&mut self) -> Option<Rect> {
        self.stack.pop();
        self.current = self.stack.last().copied();
        self.current
    }

    /// Returns the current active cumulative clipping rectangle, or `None` if unclipped.
    #[inline]
    pub fn current(&self) -> Option<Rect> {
        self.current
    }

    /// Returns `true` if the scissor stack contains no clipping rectangles.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns the current nesting depth of the scissor stack.
    #[inline]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns a slice view of all cumulative clipping rectangles in the stack from root to leaf.
    #[inline]
    pub fn as_slice(&self) -> &[Rect] {
        &self.stack
    }

    /// Clears all clipping rectangles from the stack, resetting active clipping to `None`.
    ///
    /// Retains allocated stack capacity for subsequent frames.
    #[inline]
    pub fn clear(&mut self) {
        self.stack.clear();
        self.current = None;
    }

    /// Returns `true` if an active scissor rectangle exists and has zero or negative area.
    ///
    /// When `true`, any geometry rendered within this clipping scope is completely invisible
    /// and can be safely culled before issuing GPU draw calls.
    #[inline]
    pub fn is_degenerate(&self) -> bool {
        match self.current {
            Some(rect) => rect.width <= 0.0 || rect.height <= 0.0 || rect == Rect::ZERO,
            None => false,
        }
    }

    /// Converts the current logical clipping rectangle into physical integer screen scissor coordinates.
    ///
    /// The coordinates are clamped to `[0, screen_width]` and `[0, screen_height]` to prevent
    /// out-of-bounds rasterizer access or integer underflow. Returns `None` if no clipping is active
    /// (indicating the viewport should cover the full screen).
    ///
    /// Coordinates are formatted as `(x, y, width, height)` in physical device pixels.
    pub fn to_hardware_scissor(
        &self,
        screen_width: u32,
        screen_height: u32,
    ) -> Option<(u32, u32, u32, u32)> {
        let rect = self.current?;

        if rect.width <= 0.0 || rect.height <= 0.0 || rect == Rect::ZERO {
            return Some((0, 0, 0, 0));
        }

        let sx = rect.x.max(0.0).floor() as u32;
        let sy = rect.y.max(0.0).floor() as u32;

        if sx >= screen_width || sy >= screen_height {
            return Some((screen_width, screen_height, 0, 0));
        }

        let right = (rect.x + rect.width).max(0.0).ceil() as u32;
        let bottom = (rect.y + rect.height).max(0.0).ceil() as u32;

        let clamped_right = right.min(screen_width);
        let clamped_bottom = bottom.min(screen_height);

        let sw = clamped_right.saturating_sub(sx);
        let sh = clamped_bottom.saturating_sub(sy);

        Some((sx, sy, sw, sh))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scissor_stack_empty_initialization() {
        let stack = ScissorStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.current(), None);
        assert!(!stack.is_degenerate());
        assert_eq!(stack.to_hardware_scissor(800, 600), None);
    }

    #[test]
    fn test_scissor_stack_with_initial_clip() {
        let clip = Rect::new(10.0, 20.0, 200.0, 100.0);
        let stack = ScissorStack::with_initial_clip(Some(clip));
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.current(), Some(clip));

        let empty = ScissorStack::with_initial_clip(None);
        assert!(empty.is_empty());
        assert_eq!(empty.current(), None);
    }

    #[test]
    fn test_scissor_stack_nested_5_intersections() {
        let mut stack = ScissorStack::new();

        // 1. Root container (e.g. Window: 0, 0, 1000, 800)
        let r1 = Rect::new(0.0, 0.0, 1000.0, 800.0);
        assert_eq!(stack.push(r1), Some(r1));
        assert_eq!(stack.len(), 1);

        // 2. Dock panel (e.g. Panel: 50, 50, 600, 500)
        let r2 = Rect::new(50.0, 50.0, 600.0, 500.0);
        let inter2 = stack.push(r2).unwrap();
        assert_eq!(inter2, Rect::new(50.0, 50.0, 600.0, 500.0));
        assert_eq!(stack.len(), 2);

        // 3. Scroll area (e.g. ScrollArea: 100, 100, 400, 300)
        let r3 = Rect::new(100.0, 100.0, 400.0, 300.0);
        let inter3 = stack.push(r3).unwrap();
        assert_eq!(inter3, Rect::new(100.0, 100.0, 400.0, 300.0));
        assert_eq!(stack.len(), 3);

        // 4. Virtual list (e.g. List: 120, 150, 350, 200)
        let r4 = Rect::new(120.0, 150.0, 350.0, 200.0);
        let inter4 = stack.push(r4).unwrap();
        // Intersection of (100, 100, 400, 300)[right=500, bottom=400] and (120, 150, 350, 200)[right=470, bottom=350]
        // min_x=120, min_y=150, max_x=470, max_y=350 -> width=350, height=200
        assert_eq!(inter4, Rect::new(120.0, 150.0, 350.0, 200.0));
        assert_eq!(stack.len(), 4);

        // 5. Nested item slot (e.g. ItemSlot: 150, 180, 200, 100)
        let r5 = Rect::new(150.0, 180.0, 200.0, 100.0);
        let inter5 = stack.push(r5).unwrap();
        assert_eq!(inter5, Rect::new(150.0, 180.0, 200.0, 100.0));
        assert_eq!(stack.len(), 5);

        // Pop 5 -> restores 4
        assert_eq!(stack.pop(), Some(inter4));
        assert_eq!(stack.len(), 4);

        // Pop 4 -> restores 3
        assert_eq!(stack.pop(), Some(inter3));
        assert_eq!(stack.len(), 3);

        // Pop 3 -> restores 2
        assert_eq!(stack.pop(), Some(inter2));
        assert_eq!(stack.len(), 2);

        // Pop 2 -> restores 1
        assert_eq!(stack.pop(), Some(r1));
        assert_eq!(stack.len(), 1);

        // Pop 1 -> stack empty
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
        assert_eq!(stack.current(), None);
    }

    #[test]
    fn test_scissor_stack_disjoint_rects_produce_degenerate() {
        let mut stack = ScissorStack::new();

        // Push rect at (0, 0, 100, 100)
        stack.push(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(!stack.is_degenerate());

        // Push non-overlapping rect at (200, 200, 100, 100)
        let intersected = stack.push(Rect::new(200.0, 200.0, 100.0, 100.0));
        assert_eq!(intersected, Some(Rect::ZERO));
        assert!(stack.is_degenerate());
        assert_eq!(stack.to_hardware_scissor(800, 600), Some((0, 0, 0, 0)));

        // Pop restores valid parent clip
        assert_eq!(stack.pop(), Some(Rect::new(0.0, 0.0, 100.0, 100.0)));
        assert!(!stack.is_degenerate());
    }

    #[test]
    fn test_scissor_stack_sibling_branches_restore_cleanly() {
        let mut stack = ScissorStack::new();
        let parent = Rect::new(0.0, 0.0, 400.0, 400.0);
        stack.push(parent);

        // Branch A
        let child_a = Rect::new(10.0, 10.0, 100.0, 100.0);
        stack.push(child_a);
        assert_eq!(stack.current(), Some(child_a));
        stack.pop();
        assert_eq!(stack.current(), Some(parent));

        // Branch B
        let child_b = Rect::new(150.0, 150.0, 200.0, 200.0);
        stack.push(child_b);
        assert_eq!(stack.current(), Some(child_b));
        stack.pop();
        assert_eq!(stack.current(), Some(parent));
    }

    #[test]
    fn test_scissor_stack_hardware_scissor_clamping() {
        let mut stack = ScissorStack::new();

        // Normal rect
        stack.push(Rect::new(10.2, 20.7, 100.4, 50.1));
        // sx = 10, sy = 20, right = 111, bottom = 71 -> sw = 101, sh = 51
        assert_eq!(stack.to_hardware_scissor(800, 600), Some((10, 20, 101, 51)));

        // Negative coordinates clamped to 0
        stack.clear();
        stack.push(Rect::new(-50.0, -20.0, 100.0, 80.0));
        // sx = 0, sy = 0, right = 50, bottom = 60 -> sw = 50, sh = 60
        assert_eq!(stack.to_hardware_scissor(800, 600), Some((0, 0, 50, 60)));

        // Rect extending beyond screen boundaries
        stack.clear();
        stack.push(Rect::new(750.0, 550.0, 200.0, 200.0));
        // sx = 750, sy = 550, screen = 800, 600 -> sw = 50, sh = 50
        assert_eq!(
            stack.to_hardware_scissor(800, 600),
            Some((750, 550, 50, 50))
        );

        // Rect entirely outside screen bounds
        stack.clear();
        stack.push(Rect::new(900.0, 700.0, 100.0, 100.0));
        assert_eq!(stack.to_hardware_scissor(800, 600), Some((800, 600, 0, 0)));
    }

    #[test]
    fn test_scissor_stack_clear_and_capacity() {
        let mut stack = ScissorStack::with_capacity(16);
        assert!(stack.is_empty());
        stack.push(Rect::new(0.0, 0.0, 100.0, 100.0));
        stack.push(Rect::new(20.0, 20.0, 50.0, 50.0));
        assert_eq!(stack.len(), 2);

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.current(), None);
    }
}