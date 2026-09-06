// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Layout computation and hit-testing for individual tab buttons, close icons, and overflow chevrons.
//!
//! Calculates exact bounding boxes for tab buttons, right-aligned close `x` targets,
//! an optional `+` tab addition button, and an overflow `▾` chevron when tabs exceed pane width.

use crate::tab_viewer::TabViewer;
use iris_core::{Point, Rect};

/// Geometric layout info computed for an individual tab button on a leaf's tab bar.
#[derive(Debug, Clone, PartialEq)]
pub struct TabLayoutInfo {
    /// Zero-based index of this tab within its host leaf.
    pub index: usize,
    /// Bounding rectangle of the entire tab button.
    pub rect: Rect,
    /// Hit rectangle of the tab's close button (`x`), if closeable.
    pub close_btn_rect: Option<Rect>,
    /// Display title rendered on the tab.
    pub title: String,
    /// Whether this tab is currently the active and visible tab.
    pub is_active: bool,
    /// Whether this tab has unsaved modifications and displays a dirty marker (`•`).
    pub is_modified: bool,
    /// Optional tooltip displayed when hovering over the tab header.
    pub tooltip: Option<String>,
    /// Whether this tab can be closed by the user.
    pub is_closeable: bool,
    /// Whether this tab can be dragged out of its leaf.
    pub is_draggable: bool,
}

/// Geometric layout info computed for a leaf's entire top tab strip.
#[derive(Debug, Clone, PartialEq)]
pub struct TabBarLayoutInfo {
    /// Bounding rectangle of the entire tab bar strip.
    pub rect: Rect,
    /// Layout rectangles computed for each tab hosted within the leaf.
    pub tabs: Vec<TabLayoutInfo>,
    /// Hit rectangle for the add tab button (`+`), if enabled.
    pub add_btn_rect: Option<Rect>,
    /// Hit rectangle for the overflow chevron button (`▾`), if tabs exceed available width.
    pub overflow_chevron_rect: Option<Rect>,
    /// Indices of tabs that overflowed past the visible width of the tab bar.
    pub overflow_tabs: Vec<usize>,
    /// Horizontal scroll offset applied to the tabs.
    pub scroll_offset: f32,
}

/// Computes pixel-precise layout boundaries for all tabs, close buttons, and auxiliary controls.
pub fn compute_tab_bar_layout<Tab, V: TabViewer<Tab>>(
    tab_bar_rect: Rect,
    tabs: &[Tab],
    active_tab: usize,
    viewer: &V,
    scroll_offset: f32,
    show_add_button: bool,
) -> TabBarLayoutInfo {
    if tab_bar_rect.width <= 0.0 || tab_bar_rect.height <= 0.0 || tabs.is_empty() {
        return TabBarLayoutInfo {
            rect: tab_bar_rect,
            tabs: Vec::new(),
            add_btn_rect: None,
            overflow_chevron_rect: None,
            overflow_tabs: Vec::new(),
            scroll_offset,
        };
    }

    let tab_height = tab_bar_rect.height;
    let button_size = 24.0_f32.min(tab_height);
    let mut reserved_right = 0.0_f32;

    // Estimate total tab width needed without scrolling
    let mut raw_tab_widths = Vec::with_capacity(tabs.len());
    let mut total_tabs_width = 0.0_f32;

    for tab in tabs {
        let title = viewer.title(tab);
        let is_closeable = viewer.closeable(tab);
        let text_width = (title.len() as f32) * 7.5;
        let padding = 20.0_f32;
        let close_width = if is_closeable { 18.0 } else { 0.0 };
        let tab_w = (text_width + padding + close_width).clamp(56.0, 180.0);
        raw_tab_widths.push((title, is_closeable, tab_w));
        total_tabs_width += tab_w;
    }

    if show_add_button {
        reserved_right += button_size;
    }

    let has_overflow = total_tabs_width > (tab_bar_rect.width - reserved_right).max(0.0);
    let overflow_chevron_rect = if has_overflow {
        reserved_right += button_size;
        Some(Rect::new(
            tab_bar_rect.x + tab_bar_rect.width - reserved_right,
            tab_bar_rect.y + (tab_height - button_size) * 0.5,
            button_size,
            button_size,
        ))
    } else {
        None
    };

    let add_btn_rect = if show_add_button {
        let add_x = if has_overflow {
            tab_bar_rect.x + tab_bar_rect.width - button_size
        } else {
            (tab_bar_rect.x + total_tabs_width - scroll_offset)
                .min(tab_bar_rect.x + tab_bar_rect.width - button_size)
        };
        Some(Rect::new(
            add_x,
            tab_bar_rect.y + (tab_height - button_size) * 0.5,
            button_size,
            button_size,
        ))
    } else {
        None
    };

    let visible_max_x = tab_bar_rect.x + tab_bar_rect.width - reserved_right;
    let mut current_x = tab_bar_rect.x - scroll_offset;
    let mut computed_tabs = Vec::with_capacity(tabs.len());
    let mut overflow_tabs = Vec::new();

    for (idx, (tab, (title, is_closeable, tab_w))) in tabs.iter().zip(raw_tab_widths).enumerate() {
        let tab_rect = Rect::new(current_x, tab_bar_rect.y, tab_w, tab_height);

        // Check overflow
        if current_x + tab_w > visible_max_x || current_x < tab_bar_rect.x {
            overflow_tabs.push(idx);
        }

        let close_btn_rect = if is_closeable {
            let close_size = 14.0_f32.min(tab_height * 0.6);
            Some(Rect::new(
                current_x + tab_w - close_size - 6.0,
                tab_bar_rect.y + (tab_height - close_size) * 0.5,
                close_size,
                close_size,
            ))
        } else {
            None
        };

        computed_tabs.push(TabLayoutInfo {
            index: idx,
            rect: tab_rect,
            close_btn_rect,
            title,
            is_active: idx == active_tab,
            is_modified: viewer.is_modified(tab),
            tooltip: viewer.tooltip(tab),
            is_closeable,
            is_draggable: viewer.is_draggable(tab),
        });

        current_x += tab_w;
    }

    TabBarLayoutInfo {
        rect: tab_bar_rect,
        tabs: computed_tabs,
        add_btn_rect,
        overflow_chevron_rect,
        overflow_tabs,
        scroll_offset,
    }
}

/// Calculates the target tab insertion index when dragging a tab over an existing tab bar.
/// Compares cursor X position against tab midpoints to determine if the dropped tab
/// should be placed before or after each item in the strip.
pub fn calculate_tab_reorder_index(
    tab_bar_layout: &TabBarLayoutInfo,
    cursor_pos: Point,
) -> Option<usize> {
    if !tab_bar_layout.rect.contains_point(cursor_pos) || tab_bar_layout.tabs.is_empty() {
        return None;
    }

    for (idx, tab) in tab_bar_layout.tabs.iter().enumerate() {
        let midpoint_x = tab.rect.x + tab.rect.width * 0.5;
        if cursor_pos.x < midpoint_x {
            return Some(idx);
        }
    }

    Some(tab_bar_layout.tabs.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab_viewer::SimpleTabViewer;

    #[test]
    fn test_compute_tab_bar_layout_and_close_buttons() {
        let tabs = vec!["Hierarchy", "Scene", "Console"];
        let bar_rect = Rect::new(0.0, 0.0, 500.0, 28.0);
        let viewer = SimpleTabViewer;

        let layout = compute_tab_bar_layout(bar_rect, &tabs, 1, &viewer, 0.0, true);

        assert_eq!(layout.tabs.len(), 3);
        assert!(!layout.tabs[0].is_active);
        assert!(layout.tabs[1].is_active);
        assert_eq!(layout.tabs[1].title, "Scene");

        // Every closeable tab has a close button rect inside tab.rect
        for tab in &layout.tabs {
            assert!(tab.is_closeable);
            let close_rect = tab.close_btn_rect.expect("Close button present");
            assert!(close_rect.x >= tab.rect.x);
            assert!(close_rect.x + close_rect.width <= tab.rect.x + tab.rect.width);
        }

        assert!(layout.add_btn_rect.is_some());
        assert!(layout.overflow_chevron_rect.is_none());
        assert!(layout.overflow_tabs.is_empty());
    }

    #[test]
    fn test_tab_bar_overflow_detection() {
        let tabs = vec![
            "Tab Number One",
            "Tab Number Two",
            "Tab Number Three",
            "Tab Number Four",
            "Tab Number Five",
        ];
        // Very narrow bar
        let bar_rect = Rect::new(0.0, 0.0, 200.0, 26.0);
        let viewer = SimpleTabViewer;

        let layout = compute_tab_bar_layout(bar_rect, &tabs, 0, &viewer, 0.0, true);

        assert!(layout.overflow_chevron_rect.is_some());
        assert!(!layout.overflow_tabs.is_empty());
    }

    #[test]
    fn test_calculate_tab_reorder_index() {
        let tabs = vec!["A", "B", "C"];
        let bar_rect = Rect::new(0.0, 0.0, 300.0, 26.0);
        let viewer = SimpleTabViewer;
        let layout = compute_tab_bar_layout(bar_rect, &tabs, 0, &viewer, 0.0, false);

        // Before first tab
        assert_eq!(
            calculate_tab_reorder_index(&layout, Point::new(10.0, 10.0)),
            Some(0)
        );

        // Past last tab
        assert_eq!(
            calculate_tab_reorder_index(&layout, Point::new(280.0, 10.0)),
            Some(3)
        );

        // Outside bar rect
        assert_eq!(
            calculate_tab_reorder_index(&layout, Point::new(10.0, 50.0)),
            None
        );
    }
}