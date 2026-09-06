// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Multi-viewport and detached floating window docking management.
//!
//! Enables tabs to be dragged out of the main window into independent operating system
//! floating windows (multi-monitor workflow) and docked back .

use crate::tree::{DockNodeId, DockTree};
use iris_core::{Point, Rect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an OS-level detached floating viewport window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FloatingWindowId(pub u64);

/// Representation of a detached floating window containing its own binary dock tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingDockWindow<T> {
    /// Unique identifier of this window.
    pub id: FloatingWindowId,
    /// Window title bar text.
    pub title: String,
    /// Screen-space desktop coordinate position `(x, y)` in physical pixels.
    pub position: (i32, i32),
    /// Framebuffer dimensions `(width, height)` in physical pixels.
    pub size: (u32, u32),
    /// Embedded dock tree hosting panels inside this floating window.
    pub tree: DockTree<T>,
}

/// Central multi-viewport manager coordinating the main application dock tree and all floating OS windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiViewportManager<T> {
    /// Primary docking tree residing inside the main application window.
    pub main_tree: DockTree<T>,
    /// Active detached secondary floating windows mapped by their ID.
    pub floating_windows: HashMap<FloatingWindowId, FloatingDockWindow<T>>,
    /// Next window ID sequence generator.
    next_id: u64,
}

impl<T: Clone + PartialEq> Default for MultiViewportManager<T> {
    fn default() -> Self {
        Self {
            main_tree: DockTree::new(),
            floating_windows: HashMap::new(),
            next_id: 1,
        }
    }
}

impl<T: Clone + PartialEq> MultiViewportManager<T> {
    /// Creates a new, empty multi-viewport manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Detaches a tab or group of tabs from any dock tree into a newly created floating OS window.
    pub fn detach_to_floating_window(
        &mut self,
        title: impl Into<String>,
        position: (i32, i32),
        size: (u32, u32),
        tabs: Vec<T>,
    ) -> FloatingWindowId {
        let window_id = FloatingWindowId(self.next_id);
        self.next_id += 1;

        let mut window_tree = DockTree::new();
        let leaf = window_tree.create_leaf(tabs);
        window_tree.set_root(leaf);

        let window = FloatingDockWindow {
            id: window_id,
            title: title.into(),
            position,
            size,
            tree: window_tree,
        };

        self.floating_windows.insert(window_id, window);
        window_id
    }

    /// Closes a floating window and re-inserts its tabs into the main application tree at a target drop zone.
    pub fn dock_floating_window_back(
        &mut self,
        window_id: FloatingWindowId,
        target_leaf: DockNodeId,
        drop_zone: crate::drag_drop::DropZone,
    ) -> Result<(), crate::tree::DockError> {
        let window = self
            .floating_windows
            .remove(&window_id)
            .ok_or(crate::tree::DockError::NodeNotFound)?;

        let mut tabs_to_dock = window.tree.all_tabs();

        if tabs_to_dock.is_empty() {
            return Ok(());
        }

        match drop_zone {
            crate::drag_drop::DropZone::Center => {
                for tab in tabs_to_dock {
                    self.main_tree.add_tab(target_leaf, tab)?;
                }
            }
            crate::drag_drop::DropZone::Left
            | crate::drag_drop::DropZone::Right
            | crate::drag_drop::DropZone::Top
            | crate::drag_drop::DropZone::Bottom => {
                let first_tab = tabs_to_dock.remove(0);
                let new_leaf = self.main_tree.dock_tab(target_leaf, drop_zone, first_tab)?;
                for tab in tabs_to_dock {
                    self.main_tree.add_tab(new_leaf, tab)?;
                }
            }
            crate::drag_drop::DropZone::ScreenLeft
            | crate::drag_drop::DropZone::ScreenRight
            | crate::drag_drop::DropZone::ScreenTop
            | crate::drag_drop::DropZone::ScreenBottom => {
                let first_tab = tabs_to_dock.remove(0);
                let new_leaf = self.main_tree.dock_root(drop_zone, first_tab)?;
                for tab in tabs_to_dock {
                    self.main_tree.add_tab(new_leaf, tab)?;
                }
            }
        }

        Ok(())
    }

    /// Determines whether a global screen position is outside the main window bounds.
    pub fn is_outside_main_window(screen_pos: Point, main_window_rect: Rect) -> bool {
        !main_window_rect.contains_point(screen_pos)
    }
}