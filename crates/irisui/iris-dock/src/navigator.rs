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
use iris_core::style::Style;
use iris_core::tree::UiTree;

/// Normalized relative positions for the 4 dashed line segments inside split buttons.
const DASHED_LINE_ALPHAS: [f32; 8] = [
    0.0625, 0.1875, 0.3125, 0.4375, 0.5625, 0.6875, 0.8125, 0.9375,
];

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
/// - `tree`: Target UI tree receiving the generated widgets.
/// - `parent_id`: Parent widget node to which anchor button subtrees will be appended.
/// - `geometry`: Precomputed navigator geometry.
/// - `active_zone`: Currently hovered drop zone, if any.
/// - `style`: Styling tokens governing colors, corner radii, and dimensions.
pub fn build_dock_navigator_nodes(
    tree: &mut UiTree,
    parent_id: WidgetId,
    geometry: &DockNavigatorGeometry,
    active_zone: Option<DropZone>,
    style: &DockNavigatorStyle,
) {
    if geometry.host_content_rect.width < style.button_size
        || geometry.host_content_rect.height < style.button_size
    {
        return;
    }

    let zones = [
        DropZone::Center,
        DropZone::Left,
        DropZone::Right,
        DropZone::Top,
        DropZone::Bottom,
    ];

    for zone in zones {
        let is_active = active_zone == Some(zone);
        let button_rect = geometry.get_button_rect(zone);

        if button_rect.width <= 0.0 || button_rect.height <= 0.0 {
            continue;
        }

        let bg_color = if is_active {
            style.hover_bg
        } else {
            style.idle_bg
        };
        let border_color = if is_active {
            style.hover_border
        } else {
            style.idle_border
        };
        let border_width = if is_active { 1.5 } else { 1.0 };

        // 1. Outer button card container
        let button_id = tree.create_node();
        if let Some(node) = tree.get_mut(button_id) {
            node.set_style(
                Style::new()
                    .background(bg_color)
                    .border(border_width, border_color)
                    .border_radius(style.corner_radius),
            );
            node.computed_rect = button_rect;
        }
        let _ = tree.add_child(parent_id, button_id);

        // 2. Inner mini blueprint window frame (shrunk inset)
        let inset = (button_rect.width * 0.10).max(2.5);
        let inner_rect = Rect::new(
            button_rect.x + inset,
            button_rect.y + inset,
            (button_rect.width - inset * 2.0).max(1.0),
            (button_rect.height - inset * 2.0).max(1.0),
        );

        let inner_window_id = tree.create_node();
        if let Some(node) = tree.get_mut(inner_window_id) {
            node.set_style(
                Style::new()
                    .background(Color::TRANSPARENT)
                    .border(0.75, style.divider_color)
                    .border_radius(1.0),
            );
            node.computed_rect = inner_rect;
        }
        let _ = tree.add_child(button_id, inner_window_id);

        // 3. Mini window top header bar rim
        let rim_height = (inner_rect.height * 0.12).max(3.0);
        let rim_rect = Rect::new(inner_rect.x, inner_rect.y, inner_rect.width, rim_height);

        let rim_id = tree.create_node();
        if let Some(node) = tree.get_mut(rim_id) {
            node.set_style(
                Style::new()
                    .background(style.header_color)
                    .border_radius(0.5),
            );
            node.computed_rect = rim_rect;
        }
        let _ = tree.add_child(button_id, rim_id);

        // 4. Interior body geometry below rim
        let body_y = inner_rect.y + rim_height;
        let body_h = (inner_rect.bottom() - body_y).max(1.0);
        let body_x = inner_rect.x;
        let body_w = inner_rect.width;

        // 5. Active partition fill when hovered
        if is_active {
            let active_fill_rect = match zone {
                DropZone::Center => Rect::new(body_x, body_y, body_w, body_h),
                DropZone::Left | DropZone::ScreenLeft => {
                    Rect::new(body_x, body_y, (body_w * 0.5).max(1.0), body_h)
                }
                DropZone::Right | DropZone::ScreenRight => {
                    let half_w = body_w * 0.5;
                    Rect::new(body_x + half_w, body_y, half_w.max(1.0), body_h)
                }
                DropZone::Top | DropZone::ScreenTop => {
                    Rect::new(body_x, body_y, body_w, (body_h * 0.5).max(1.0))
                }
                DropZone::Bottom | DropZone::ScreenBottom => {
                    let half_h = body_h * 0.5;
                    Rect::new(body_x, body_y + half_h, body_w, half_h.max(1.0))
                }
            };

            let fill_id = tree.create_node();
            if let Some(node) = tree.get_mut(fill_id) {
                node.set_style(Style::new().background(style.active_partition_fill));
                node.computed_rect = active_fill_rect;
            }
            let _ = tree.add_child(button_id, fill_id);
        }

        // 6. Dashed division lines for the 4 directional buttons (2-way partition preview)
        match zone {
            DropZone::Center => {
                // Center has no divider line, representing whole pane append
            }
            DropZone::Left | DropZone::Right | DropZone::ScreenLeft | DropZone::ScreenRight => {
                // Vertical dashed line dividing into left and right halves
                let mid_x = (body_x + body_w * 0.5 - 0.5).round();
                for dash_pair in DASHED_LINE_ALPHAS.chunks(2) {
                    let t0 = dash_pair[0];
                    let t1 = dash_pair[1];
                    let seg_y = body_y + body_h * t0;
                    let seg_h = (body_h * (t1 - t0)).max(1.0);

                    let dash_id = tree.create_node();
                    if let Some(node) = tree.get_mut(dash_id) {
                        node.set_style(Style::new().background(style.divider_color));
                        node.computed_rect = Rect::new(mid_x, seg_y, 1.0, seg_h);
                    }
                    let _ = tree.add_child(button_id, dash_id);
                }
            }
            DropZone::Top | DropZone::Bottom | DropZone::ScreenTop | DropZone::ScreenBottom => {
                // Horizontal dashed line dividing into top and bottom halves
                let mid_y = (body_y + body_h * 0.5 - 0.5).round();
                for dash_pair in DASHED_LINE_ALPHAS.chunks(2) {
                    let t0 = dash_pair[0];
                    let t1 = dash_pair[1];
                    let seg_x = body_x + body_w * t0;
                    let seg_w = (body_w * (t1 - t0)).max(1.0);

                    let dash_id = tree.create_node();
                    if let Some(node) = tree.get_mut(dash_id) {
                        node.set_style(Style::new().background(style.divider_color));
                        node.computed_rect = Rect::new(seg_x, mid_y, seg_w, 1.0);
                    }
                    let _ = tree.add_child(button_id, dash_id);
                }
            }
        }
    }
}

/// Builds a semi-transparent drop zone preview rectangle inside the target pane.
pub fn build_drop_preview_node(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
    zone: DropZone,
    style: &DockNavigatorStyle,
) -> WidgetId {
    let preview_rect = calculate_drop_preview_rect(content_rect, zone);
    let preview_id = tree.create_node();

    if let Some(node) = tree.get_mut(preview_id) {
        node.set_style(
            Style::new()
                .background(style.preview_fill)
                .border(1.5, style.preview_border)
                .border_radius(4.0),
        );
        node.computed_rect = preview_rect;
    }
    let _ = tree.add_child(parent_id, preview_id);
    preview_id
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
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: FloatingTabBadgeParams<'_>,
) -> WidgetId {
    let badge_x = params.cursor_pos.x + 12.0;
    let badge_y = params.cursor_pos.y + 12.0;
    let badge_height = 24.0;

    let char_count = params.title.chars().count();
    let text_approx_w = (char_count as f32) * 7.5 + if params.icon.is_some() { 20.0 } else { 0.0 };
    let badge_width = (text_approx_w + 34.0).max(80.0);

    let badge_rect = Rect::new(badge_x, badge_y, badge_width, badge_height);

    // 1. Badge container pill
    let badge_id = tree.create_node();
    if let Some(node) = tree.get_mut(badge_id) {
        node.set_style(
            Style::new()
                .background(Color::from_u8(16, 20, 28, 235))
                .border(1.0, Color::from_u8(0, 229, 255, 200))
                .border_radius(4.0),
        );
        node.computed_rect = badge_rect;
    }
    let _ = tree.add_child(parent_id, badge_id);

    // 2. Icon + Title label node
    let label_id = tree.create_node();
    if let Some(node) = tree.get_mut(label_id) {
        let display_text = if let Some(icon_str) = params.icon {
            format!("{icon_str} {}", params.title)
        } else {
            params.title.to_string()
        };
        node.set_style(Style::new());
        node.computed_rect = Rect::new(
            badge_x + 8.0,
            badge_y + 4.0,
            badge_width - 24.0,
            badge_height - 8.0,
        );
        node.text = Some(display_text);
        node.font_size = 11.5;
        node.text_color = Color::WHITE;
    }
    let _ = tree.add_child(badge_id, label_id);

    // 3. Trailing close '✕' glyph
    let close_id = tree.create_node();
    if let Some(node) = tree.get_mut(close_id) {
        node.set_style(Style::new());
        node.computed_rect = Rect::new(
            badge_x + badge_width - 18.0,
            badge_y + 4.0,
            14.0,
            badge_height - 8.0,
        );
        node.text = Some("✕".to_string());
        node.font_size = 10.0;
        node.text_color = Color::from_u8(160, 175, 195, 255);
    }
    let _ = tree.add_child(badge_id, close_id);

    badge_id
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

    #[test]
    fn test_dock_navigator_nodes_construction() {
        let mut tree = UiTree::new();
        let root_id = tree.create_node();
        let _ = tree.set_root(root_id);

        let content_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        let style = DockNavigatorStyle::default();
        let geometry = DockNavigatorGeometry::from_content_rect_with_style(content_rect, &style);

        build_dock_navigator_nodes(&mut tree, root_id, &geometry, Some(DropZone::Left), &style);

        let root_node = tree.get(root_id).expect("Root node exists");
        // Root must have 5 child buttons
        assert_eq!(root_node.children.len(), 5);

        // Verify Left button active styling
        let left_button_id = root_node.children[1];
        let left_button = tree.get(left_button_id).expect("Left button exists");
        assert_eq!(left_button.style.background_color, style.hover_bg);

        // Left button must contain inner window, header rim, active fill, and 4 dashed segments
        assert_eq!(left_button.children.len(), 7);
    }

    #[test]
    fn test_degenerate_content_rect_safety() {
        let mut tree = UiTree::new();
        let root_id = tree.create_node();
        let _ = tree.set_root(root_id);

        let degenerate_rect = Rect::new(0.0, 0.0, 20.0, 20.0);
        let style = DockNavigatorStyle::default();
        let geometry = DockNavigatorGeometry::from_content_rect_with_style(degenerate_rect, &style);

        build_dock_navigator_nodes(&mut tree, root_id, &geometry, None, &style);

        let root_node = tree.get(root_id).expect("Root node exists");
        assert_eq!(root_node.children.len(), 0);

        assert_eq!(
            hit_test_navigator(degenerate_rect, Point::new(10.0, 10.0), &style),
            None
        );
    }

    #[test]
    fn test_floating_tab_badge_construction() {
        let mut tree = UiTree::new();
        let root_id = tree.create_node();
        let _ = tree.set_root(root_id);

        let badge_id = build_floating_tab_badge(
            &mut tree,
            root_id,
            FloatingTabBadgeParams {
                cursor_pos: Point::new(150.0, 200.0),
                title: "Hierarchy",
                icon: Some("📦"),
            },
        );

        let badge_node = tree.get(badge_id).expect("Badge node exists");
        assert_eq!(badge_node.computed_rect.x, 150.0 + 12.0);
        assert_eq!(badge_node.computed_rect.y, 200.0 + 12.0);
        assert_eq!(badge_node.children.len(), 2); // label + close
    }
}