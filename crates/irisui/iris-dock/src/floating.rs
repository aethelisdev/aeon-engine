// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window model enabling detachable, free-floating panel surfaces.
//!
//! Each [`FloatingWindow`] hosts an independent [`DockTree`] instance, allowing
//! detached tabs to be repositioned, resized, split into sub-panes, or docked back.

use crate::tree::DockTree;
use iris_core::{Point, Rect};

/// Independent floating panel surface detached from the main dock hierarchy.
#[derive(Debug, Clone)]
pub struct FloatingWindow<T> {
    /// Unique identifier for this floating window.
    pub id: u64,
    /// Human-readable title displayed on the floating window title bar.
    pub title: String,
    /// Absolute bounding rectangle in screen/viewport logical coordinates.
    pub rect: Rect,
    /// Isolated docking tree hosted within this floating window.
    pub tree: DockTree<T>,
    /// Whether this window currently has keyboard/mouse focus.
    pub is_focused: bool,
    /// Whether this window is collapsed or minimized to title bar.
    pub is_minimized: bool,
}

impl<T> FloatingWindow<T> {
    /// Creates a new floating window hosting the specified initial tabs in a single root leaf.
    pub fn new(id: u64, title: impl Into<String>, rect: Rect, initial_tabs: Vec<T>) -> Self {
        let mut tree = DockTree::new();
        if !initial_tabs.is_empty() {
            let root = tree.create_leaf(initial_tabs);
            tree.set_root(root);
        }

        Self {
            id,
            title: title.into(),
            rect,
            tree,
            is_focused: true,
            is_minimized: false,
        }
    }

    /// Creates a new floating window hosting an existing pre-configured [`DockTree`].
    pub fn from_tree(id: u64, title: impl Into<String>, rect: Rect, tree: DockTree<T>) -> Self {
        Self {
            id,
            title: title.into(),
            rect,
            tree,
            is_focused: true,
            is_minimized: false,
        }
    }

    /// Translates the floating window position by the specified drag delta.
    pub fn drag_title(&mut self, delta: Point) {
        self.rect.x += delta.x;
        self.rect.y += delta.y;
    }

    /// Resizes the floating window dimensions, enforcing the specified minimum size.
    pub fn resize(&mut self, delta_w: f32, delta_h: f32, min_size: f32) {
        self.rect.width = (self.rect.width + delta_w).max(min_size);
        self.rect.height = (self.rect.height + delta_h).max(min_size);
    }

    /// Returns the bounding rectangle of the floating window's top title bar.
    pub fn title_bar_rect(&self, title_bar_height: f32) -> Rect {
        Rect::new(self.rect.x, self.rect.y, self.rect.width, title_bar_height)
    }

    /// Returns the interior content area below the title bar available for docking layout.
    pub fn content_rect(&self, title_bar_height: f32) -> Rect {
        let content_h = (self.rect.height - title_bar_height).max(0.0);
        Rect::new(
            self.rect.x,
            self.rect.y + title_bar_height,
            self.rect.width,
            content_h,
        )
    }

    /// Checks if a screen cursor coordinate falls inside this floating window.
    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains_point(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_window_creation_and_geometry() {
        let mut window = FloatingWindow::new(
            1,
            "Inspector Window",
            Rect::new(100.0, 150.0, 400.0, 300.0),
            vec!["InspectorTab"],
        );

        assert_eq!(window.id, 1);
        assert_eq!(window.title, "Inspector Window");
        assert!(window.contains_point(Point::new(150.0, 200.0)));
        assert!(!window.contains_point(Point::new(50.0, 50.0)));

        // Title bar & Content rect
        let title_rect = window.title_bar_rect(28.0);
        assert_eq!(title_rect, Rect::new(100.0, 150.0, 400.0, 28.0));
        let content_rect = window.content_rect(28.0);
        assert_eq!(content_rect, Rect::new(100.0, 178.0, 400.0, 272.0));

        // Drag title
        window.drag_title(Point::new(50.0, -20.0));
        assert_eq!(window.rect.x, 150.0);
        assert_eq!(window.rect.y, 130.0);

        // Resize with clamp
        window.resize(50.0, -500.0, 100.0);
        assert_eq!(window.rect.width, 450.0);
        assert_eq!(window.rect.height, 100.0); // clamped to min_size
    }
}