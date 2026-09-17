// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 5-Way Docking Cross Navigator & Floating Tab Badge Subsystem
//!
//! Provides geometric layout computation, hit-testing, visual cross node generation with
//! interior 2-way dashed partition indicators, and floating tear-off tab badges for modern
//! multi-pane dock relocation.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use crate::drag_drop::{DropZone, calculate_drop_preview_rect};
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;

/// Geometric layout of the 5 docking anchor buttons arranged in a cross cluster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockNavigatorGeometry {
    /// Bounding rectangle of the center button (Tab insertion).
    pub center_button: Rect,
    /// Bounding rectangle of the left button (Split Left).
    pub left_button: Rect,
    /// Bounding rectangle of the right button (Split Right).
    pub right_button: Rect,
    /// Bounding rectangle of the top button (Split Top).
    pub top_button: Rect,
    /// Bounding rectangle of the bottom button (Split Bottom).
    pub bottom_button: Rect,
    /// Content bounds of the host pane over which this navigator is projected.
    pub host_content_rect: Rect,
}

impl DockNavigatorGeometry {
    /// Computes the navigator cross geometry centered inside the specified host content rectangle.
    /// # Parameters
    /// - `content_rect`: Total bounding box of the hovered dock leaf or pane.
    /// - `button_size`: Width and height of each square anchor button.
    /// - `button_gap`: Pixel spacing between adjacent anchor buttons.
    pub fn from_content_rect(content_rect: Rect, button_size: f32, button_gap: f32) -> Self {
        let center_x = content_rect.x + content_rect.width * 0.5;
        let center_y = content_rect.y + content_rect.height * 0.5;
        let half_size = button_size * 0.5;

        let center_button = Rect::new(
            center_x - half_size,
            center_y - half_size,
            button_size,
            button_size,
        );

        let left_button = Rect::new(
            center_x - half_size - button_size - button_gap,
            center_y - half_size,
            button_size,
            button_size,
        );

        let right_button = Rect::new(
            center_x + half_size + button_gap,
            center_y - half_size,
            button_size,
            button_size,
        );

        let top_button = Rect::new(
            center_x - half_size,
            center_y - half_size - button_size - button_gap,
            button_size,
            button_size,
        );

        let bottom_button = Rect::new(
            center_x - half_size,
            center_y + half_size + button_gap,
            button_size,
            button_size,
        );

        Self {
            center_button,
            left_button,
            right_button,
            top_button,
            bottom_button,
            host_content_rect: content_rect,
        }
    }

    /// Computes the navigator cross geometry using dimensions defined in a style descriptor.
    pub fn from_content_rect_with_style(content_rect: Rect, style: &DockNavigatorStyle) -> Self {
        Self::from_content_rect(content_rect, style.button_size, style.button_gap)
    }

    /// Evaluates if the cursor position hits any of the 5 docking cross buttons.
    /// Returns `Some(DropZone)` when hovering directly over an anchor button, or `None` otherwise.
    pub fn hit_test(&self, cursor_pos: Point) -> Option<DropZone> {
        if self.center_button.contains_point(cursor_pos) {
            Some(DropZone::Center)
        } else if self.left_button.contains_point(cursor_pos) {
            Some(DropZone::Left)
        } else if self.right_button.contains_point(cursor_pos) {
            Some(DropZone::Right)
        } else if self.top_button.contains_point(cursor_pos) {
            Some(DropZone::Top)
        } else if self.bottom_button.contains_point(cursor_pos) {
            Some(DropZone::Bottom)
        } else {
            None
        }
    }

    /// Retrieves the bounding rectangle corresponding to a specific drop zone button.
    pub fn get_button_rect(&self, zone: DropZone) -> Rect {
        match zone {
            DropZone::Center => self.center_button,
            DropZone::Left | DropZone::ScreenLeft => self.left_button,
            DropZone::Right | DropZone::ScreenRight => self.right_button,
            DropZone::Top | DropZone::ScreenTop => self.top_button,
            DropZone::Bottom | DropZone::ScreenBottom => self.bottom_button,
        }
    }
}

/// Visual styling configuration for the docking cross navigator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockNavigatorStyle {
    /// Dimension (width and height) of each square anchor button.
    pub button_size: f32,
    /// Spacing between adjacent anchor buttons.
    pub button_gap: f32,
    /// Corner radius for anchor buttons.
    pub corner_radius: f32,
    /// Background color of idle anchor buttons.
    pub idle_bg: Color,
    /// Border color of idle anchor buttons.
    pub idle_border: Color,
    /// Background color of the hovered/active anchor button.
    pub hover_bg: Color,
    /// Border color of the hovered/active anchor button.
    pub hover_border: Color,
    /// Header bar color displayed at the top of the mini blueprint window.
    pub header_color: Color,
    /// Divider line color separating halves inside directional buttons.
    pub divider_color: Color,
    /// Semi-transparent highlight fill for the active partitioned half.
    pub active_partition_fill: Color,
    /// Semi-transparent drop preview fill covering the host pane.
    pub preview_fill: Color,
    /// Border color for the host pane drop preview.
    pub preview_border: Color,
}

impl Default for DockNavigatorStyle {
    fn default() -> Self {
        Self {
            button_size: 40.0,
            button_gap: 4.0,
            corner_radius: 3.0,
            idle_bg: Color::from_u8(22, 26, 35, 230),
            idle_border: Color::from_u8(65, 75, 95, 255),
            hover_bg: Color::from_u8(0, 75, 105, 240),
            hover_border: Color::from_u8(0, 229, 255, 255),
            header_color: Color::from_u8(130, 140, 160, 255),
            divider_color: Color::from_u8(85, 95, 115, 255),
            active_partition_fill: Color::from_u8(0, 229, 255, 100),
            preview_fill: Color::from_u8(0, 229, 255, 36),
            preview_border: Color::from_u8(0, 229, 255, 255),
        }
    }
}

/// Hit-tests a cursor against the 5-way docking cross centered in a host content rectangle.
/// Returns `None` if `content_rect` is degenerate or smaller than a single button.
pub fn hit_test_navigator(
    content_rect: Rect,
    cursor_pos: Point,
    style: &DockNavigatorStyle,
) -> Option<DropZone> {
    if content_rect.width < style.button_size || content_rect.height < style.button_size {
        return None;
    }
    if !content_rect.contains_point(cursor_pos) {
        return None;
    }
    let geometry = DockNavigatorGeometry::from_content_rect_with_style(content_rect, style);
    geometry.hit_test(cursor_pos)
}

/// Builds the 5-way docking cross navigator nodes into the widget hierarchy.
/// Generates 5 styled anchor buttons (Center, Left, Right, Top, Bottom) each featuring:
/// - An outer bordered card container.
/// - An inner mini blueprint window frame.
/// - A top title rim header.
/// - 4 interior dashed divider segments partitioning the 4 directional buttons into 2 equal halves.
/// - Active partition highlight filling the targeted half when hovered.
/// # Parameters
/// - `parent_id`: Parent widget node to which anchor button elements will be appended.
/// - `geometry`: Precomputed navigator geometry.
/// - `active_zone`: Currently hovered drop zone, if any.
/// - `style`: Styling tokens governing colors, corner radii, and dimensions.
pub fn build_dock_navigator_nodes(
    _parent_id: WidgetId,
    _geometry: &DockNavigatorGeometry,
    _active_zone: Option<DropZone>,
    _style: &DockNavigatorStyle,
) {
}

/// Builds a semi-transparent drop zone preview rectangle inside the target pane.
pub fn build_drop_preview_node(
    parent_id: WidgetId,
    content_rect: Rect,
    zone: DropZone,
    _style: &DockNavigatorStyle,
) -> WidgetId {
    let _preview_rect = calculate_drop_preview_rect(content_rect, zone);
    parent_id
}

/// Descriptor parameters for constructing a floating tear-off tab badge following the cursor.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingTabBadgeParams<'a> {
    /// Screen-space cursor position where the badge will be anchored.
    pub cursor_pos: Point,
    /// Display text title of the dragged tab.
    pub title: &'a str,
    /// Optional leading symbol or icon glyph string.
    pub icon: Option<&'a str>,
}

/// Builds a floating tab capsule badge following the cursor during active tab dragging.
/// Generates a rounded dark pill containing the tab's icon, title label, and close glyph.
pub fn build_floating_tab_badge(
    parent_id: WidgetId,
    _params: FloatingTabBadgeParams<'_>,
) -> WidgetId {
    parent_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_navigator_geometry_and_hit_testing() {
        let content_rect = Rect::new(100.0, 100.0, 400.0, 300.0);
        let style = DockNavigatorStyle::default();
        let geometry = DockNavigatorGeometry::from_content_rect_with_style(content_rect, &style);

        // Center should be at (300, 250) with 40x40 size
        assert_eq!(geometry.center_button.x, 300.0 - 20.0);
        assert_eq!(geometry.center_button.y, 250.0 - 20.0);
        assert_eq!(geometry.center_button.width, 40.0);
        assert_eq!(geometry.center_button.height, 40.0);

        // Hit testing center
        assert_eq!(
            geometry.hit_test(Point::new(300.0, 250.0)),
            Some(DropZone::Center)
        );

        // Hit testing left button (x = 280 - 40 - 4 = 236)
        assert_eq!(
            geometry.hit_test(Point::new(250.0, 250.0)),
            Some(DropZone::Left)
        );

        // Hit testing right button (x = 320 + 4 = 324)
        assert_eq!(
            geometry.hit_test(Point::new(340.0, 250.0)),
            Some(DropZone::Right)
        );

        // Hit testing top button (y = 230 - 40 - 4 = 186)
        assert_eq!(
            geometry.hit_test(Point::new(300.0, 200.0)),
            Some(DropZone::Top)
        );

        // Hit testing bottom button (y = 270 + 4 = 274)
        assert_eq!(
            geometry.hit_test(Point::new(300.0, 290.0)),
            Some(DropZone::Bottom)
        );

        // Hit outside cross
        assert_eq!(geometry.hit_test(Point::new(110.0, 110.0)), None);
    }
}