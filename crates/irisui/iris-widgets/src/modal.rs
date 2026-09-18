// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generic modal dialogue builder, scrim compositor, and layout framing system.
//!
//! Provides [`ModalDialogBuilder`] for constructing centered or positioned modal dialog
//! cards with backdrop scrims, title bars, '✖' close buttons, and standardized Confirm/Cancel action buttons.

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Styling and color metrics configuration for modal dialogue boxes.
#[derive(Debug, Clone)]
pub struct ModalDialogStyle {
    /// Background fill color of the main dialog card.
    pub background: Color,
    /// Border stroke thickness in logical pixels.
    pub border_width: f32,
    /// Border stroke color.
    pub border_color: Color,
    /// Outer corner radius of the dialog card.
    pub border_radius: f32,
    /// Drop shadow vertical offset.
    pub box_shadow_offset_y: f32,
    /// Drop shadow blur radius.
    pub box_shadow_blur: f32,
    /// Drop shadow color.
    pub box_shadow_color: Color,
    /// Color of the semi-transparent screen backdrop scrim blocker.
    pub scrim_color: Color,
    /// Height of the top header bar in logical pixels.
    pub header_height: f32,
    /// Background color of the header bar.
    pub header_background: Color,
    /// Border color separating the header bar.
    pub header_border_color: Color,
    /// Title text label color.
    pub title_color: Color,
    /// Idle color of the header '✖' close button glyph.
    pub close_color_idle: Color,
    /// Hovered color of the header '✖' close button glyph.
    pub close_color_hover: Color,
    /// Idle background fill color of the primary Confirm button.
    pub confirm_btn_bg_idle: Color,
    /// Hovered background fill color of the primary Confirm button.
    pub confirm_btn_bg_hover: Color,
    /// Text color of the primary Confirm button.
    pub confirm_btn_text: Color,
    /// Idle background fill color of the secondary Cancel button.
    pub cancel_btn_bg_idle: Color,
    /// Hovered background fill color of the secondary Cancel button.
    pub cancel_btn_bg_hover: Color,
    /// Text color of the secondary Cancel button.
    pub cancel_btn_text: Color,
}

impl Default for ModalDialogStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.08, 0.08, 0.10, 0.98),
            border_width: 1.0,
            border_color: Color::rgba(0.20, 0.22, 0.28, 1.0),
            border_radius: 8.0,
            box_shadow_offset_y: 8.0,
            box_shadow_blur: 24.0,
            box_shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.70),
            scrim_color: Color::rgba(0.0, 0.0, 0.0, 0.55),
            header_height: 32.0,
            header_background: Color::rgba(0.06, 0.06, 0.08, 1.0),
            header_border_color: Color::rgba(0.18, 0.20, 0.26, 1.0),
            title_color: Color::rgba(0.88, 0.88, 0.92, 1.0),
            close_color_idle: Color::rgba(0.60, 0.60, 0.65, 1.0),
            close_color_hover: Color::rgba(1.0, 0.35, 0.35, 1.0),
            confirm_btn_bg_idle: Color::rgba(0.14, 0.40, 0.58, 1.0),
            confirm_btn_bg_hover: Color::rgba(0.18, 0.50, 0.72, 1.0),
            confirm_btn_text: Color::WHITE,
            cancel_btn_bg_idle: Color::rgba(0.14, 0.15, 0.18, 1.0),
            cancel_btn_bg_hover: Color::rgba(0.20, 0.22, 0.28, 1.0),
            cancel_btn_text: Color::rgba(0.75, 0.78, 0.84, 1.0),
        }
    }
}

/// Output frame describing allocated widget IDs and interactive hit targets of a built modal dialog.
#[derive(Debug, Clone)]
pub struct ModalDialogFrame {
    /// Allocated widget ID of the full-screen backdrop scrim blocker (if enabled).
    pub scrim_id: Option<WidgetId>,
    /// Allocated widget ID of the main dialog card container.
    pub card_id: WidgetId,
    /// Allocated widget ID of the header bar.
    pub header_id: WidgetId,
    /// Allocated widget ID of the body content container node where callers can attach custom widgets.
    pub content_id: WidgetId,
    /// Bounding rectangle of the dialog card in screen coordinates.
    pub dialog_rect: Rect,
    /// Hit target of the top-right header '✖' close icon (if enabled).
    pub header_close_rect: Option<Rect>,
    /// Hit target of the primary confirm button (if configured).
    pub confirm_btn_rect: Option<Rect>,
    /// Hit target of the secondary cancel button (if configured).
    pub cancel_btn_rect: Option<Rect>,
    /// Usable content rectangle inside the dialog card below the header and above action buttons.
    pub content_rect: Rect,
}

impl ModalDialogFrame {
    /// Checks if a cursor point falls on the dialog card boundary.
    #[inline]
    pub fn is_point_over_card(&self, point: Point) -> bool {
        self.dialog_rect.contains_point(point)
    }

    /// Checks if a cursor point falls on the header '✖' close button.
    #[inline]
    pub fn is_point_over_header_close(&self, point: Point) -> bool {
        self.header_close_rect
            .is_some_and(|r| r.contains_point(point))
    }

    /// Checks if a cursor point falls on the confirm action button.
    #[inline]
    pub fn is_point_over_confirm(&self, point: Point) -> bool {
        self.confirm_btn_rect
            .is_some_and(|r| r.contains_point(point))
    }

    /// Checks if a cursor point falls on the cancel action button.
    #[inline]
    pub fn is_point_over_cancel(&self, point: Point) -> bool {
        self.cancel_btn_rect
            .is_some_and(|r| r.contains_point(point))
    }
}

/// Fluent builder for constructing standardized modal dialog interfaces.
pub struct ModalDialogBuilder {
    title: String,
    width: f32,
    height: f32,
    position: Option<Point>,
    screen_size: Option<(f32, f32)>,
    has_scrim: bool,
    has_close_btn: bool,
    header_icon_uv: Option<[f32; 4]>,
    header_icon_tint: Option<Color>,
    confirm_btn: Option<(String, Option<Color>)>,
    confirm_btn_width: f32,
    cancel_btn: Option<String>,
    cancel_btn_width: f32,
    cursor_pos: Point,
    style: ModalDialogStyle,
}

impl ModalDialogBuilder {
    /// Initializes a new modal builder with the specified title label.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 480.0,
            height: 250.0,
            position: None,
            screen_size: None,
            has_scrim: true,
            has_close_btn: true,
            header_icon_uv: None,
            header_icon_tint: None,
            confirm_btn: None,
            confirm_btn_width: 120.0,
            cancel_btn: None,
            cancel_btn_width: 80.0,
            cursor_pos: Point::new(-1.0, -1.0),
            style: ModalDialogStyle::default(),
        }
    }

    /// Configures an optional GPU SDF texture array icon drawn in the header bar.
    /// Automatically offsets the title text rightwards by 20px so it never overlaps the icon.
    pub fn header_icon(mut self, uv: [f32; 4], tint: Color) -> Self {
        self.header_icon_uv = Some(uv);
        self.header_icon_tint = Some(tint);
        self
    }

    /// Sets explicit dimensions for the modal dialog card.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Explicitly anchors the modal card top-left position.
    pub fn position(mut self, pos: Point) -> Self {
        self.position = Some(pos);
        self
    }

    /// Centers the modal card on the screen based on the viewport dimensions.
    pub fn center_on_screen(mut self, screen_width: f32, screen_height: f32) -> Self {
        self.screen_size = Some((screen_width, screen_height));
        self
    }

    /// Toggles whether a full-screen semi-transparent backdrop scrim blocker is spawned.
    /// Defaults to `true`.
    pub fn scrim(mut self, enabled: bool) -> Self {
        self.has_scrim = enabled;
        self
    }

    /// Toggles the top-right header '✖' close button.
    /// Defaults to `true`.
    pub fn close_button(mut self, enabled: bool) -> Self {
        self.has_close_btn = enabled;
        self
    }

    /// Configures the primary action button (e.g., 'Delete', 'Apply Rename', 'Confirm').
    pub fn confirm_button(mut self, label: impl Into<String>, custom_bg: Option<Color>) -> Self {
        self.confirm_btn = Some((label.into(), custom_bg));
        self
    }

    /// Sets the width of the primary confirm button in logical pixels.
    pub fn confirm_button_width(mut self, width: f32) -> Self {
        self.confirm_btn_width = width;
        self
    }

    /// Configures the secondary dismiss button (e.g., 'Cancel').
    pub fn cancel_button(mut self, label: impl Into<String>) -> Self {
        self.cancel_btn = Some(label.into());
        self
    }

    /// Sets the width of the secondary cancel button in logical pixels.
    pub fn cancel_button_width(mut self, width: f32) -> Self {
        self.cancel_btn_width = width;
        self
    }

    /// Provides cursor position for evaluating hover states during node construction.
    pub fn cursor_pos(mut self, pos: Point) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Applies custom theme and color metrics.
    pub fn style(mut self, style: ModalDialogStyle) -> Self {
        self.style = style;
        self
    }

    /// Builds the complete modal widget hierarchy into the specified [`UiTree`].
    /// Returns a [`ModalDialogFrame`] with allocated IDs and bounding rectangles.
    pub fn build(self, tree: &mut UiTree) -> ModalDialogFrame {
        let (screen_w, screen_h) = self.screen_size.unwrap_or((1920.0, 1080.0));
        let (left, top) = if let Some(pos) = self.position {
            (pos.x, pos.y)
        } else {
            let cx = ((screen_w - self.width) * 0.5).max(0.0).round();
            let cy = ((screen_h - self.height) * 0.5).max(28.0).round();
            (cx, cy)
        };

        let dialog_rect = Rect::new(left, top, self.width, self.height);

        // 1. Semi-Transparent Scrim Overlay (Full Screen)
        let scrim_id = if self.has_scrim {
            let (screen_w, screen_h) = self
                .screen_size
                .unwrap_or((self.width + 400.0, self.height + 400.0));
            let node_id = tree.create_node();
            if let Some(node) = tree.get_mut(node_id) {
                node.set_name("ModalScrim");
                node.set_role(WidgetRole::Default);
                node.layer = UiLayer::Modal;
                node.computed_rect = Rect::new(0.0, 0.0, screen_w, screen_h);
                node.style = Style::new().background(self.style.scrim_color);
            }
            if let Some(root) = tree.root() {
                let _ = tree.add_child(root, node_id);
            }
            Some(node_id)
        } else {
            None
        };

        // 2. Main Dialog Card Container
        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ModalDialogCard");
            node.set_role(WidgetRole::ModalWindow);
            node.computed_rect = dialog_rect;
            node.style = Style::new()
                .background(self.style.background)
                .border(self.style.border_width, self.style.border_color)
                .border_radius(self.style.border_radius)
                .box_shadow(
                    0.0,
                    self.style.box_shadow_offset_y,
                    self.style.box_shadow_blur,
                    self.style.box_shadow_color,
                );
        }

        if let Some(scrim) = scrim_id {
            let _ = tree.add_child(scrim, card_id);
        } else if let Some(root) = tree.root() {
            let _ = tree.add_child(root, card_id);
        }

        // 3. Header Bar
        let header_id = tree.create_node();
        if let Some(node) = tree.get_mut(header_id) {
            node.set_name("ModalHeader");
            node.computed_rect = Rect::new(left, top, self.width, self.style.header_height);
            node.style = Style::new()
                .background(self.style.header_background)
                .border(self.style.border_width, self.style.header_border_color)
                .border_radius(self.style.border_radius);
        }
        let _ = tree.add_child(card_id, header_id);

        // Header Icon (if configured)
        let title_start_x = if let Some(uv) = self.header_icon_uv {
            let icon_node = tree.create_node();
            if let Some(node) = tree.get_mut(icon_node) {
                node.set_name("ModalHeaderIcon");
                node.set_texture_uv(uv);
                if let Some(tint) = self.header_icon_tint {
                    node.set_texture_tint(tint);
                }
                node.computed_rect = Rect::new(left + 14.0, top + 9.0, 14.0, 14.0);
            }
            let _ = tree.add_child(header_id, icon_node);
            left + 34.0
        } else {
            left + 14.0
        };

        // Header Title Label
        let title_label = tree.create_node();
        if let Some(node) = tree.get_mut(title_label) {
            node.set_name("ModalHeaderTitle");
            node.set_text(self.title);
            node.font_size = 12.5;
            node.line_height = 16.0;
            node.text_color = self.style.title_color;
            node.computed_rect = Rect::new(
                title_start_x,
                top + 7.0,
                (self.width - (title_start_x - left) - 46.0).max(20.0),
                18.0,
            );
            node.text_align = TextAlign::Left;
        }
        let _ = tree.add_child(header_id, title_label);

        // Header Close (✖) Button
        let header_close_rect = if self.has_close_btn {
            let close_rect = Rect::new(left + self.width - 32.0, top + 5.0, 24.0, 22.0);
            let is_close_hovered = close_rect.contains_point(self.cursor_pos);
            let close_node = tree.create_node();
            if let Some(node) = tree.get_mut(close_node) {
                node.set_name("ModalCloseBtn");
                node.set_text("✖");
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_color = if is_close_hovered {
                    self.style.close_color_hover
                } else {
                    self.style.close_color_idle
                };
                node.computed_rect = close_rect;
                node.text_align = TextAlign::Center;
            }
            let _ = tree.add_child(header_id, close_node);
            Some(close_rect)
        } else {
            None
        };

        // 4. Content Area Container
        let has_bottom_actions = self.confirm_btn.is_some() || self.cancel_btn.is_some();
        let bottom_margin = if has_bottom_actions { 52.0 } else { 12.0 };
        let content_rect = Rect::new(
            left + 16.0,
            top + self.style.header_height + 12.0,
            (self.width - 32.0).max(0.0),
            (self.height - self.style.header_height - 12.0 - bottom_margin).max(0.0),
        );

        let content_id = tree.create_node();
        if let Some(node) = tree.get_mut(content_id) {
            node.set_name("ModalContentContainer");
            node.computed_rect = content_rect;
        }
        let _ = tree.add_child(card_id, content_id);

        // 5. Action Buttons (Confirm & Cancel)
        let btn_y = top + self.height - 42.0;
        let btn_h = 28.0;

        let confirm_btn_rect = if let Some((ref label, ref custom_bg)) = self.confirm_btn {
            let btn_w = self.confirm_btn_width;
            let btn_x = left + self.width - 16.0 - btn_w;
            let rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
            let is_hovered = rect.contains_point(self.cursor_pos);

            let bg = custom_bg.unwrap_or(if is_hovered {
                self.style.confirm_btn_bg_hover
            } else {
                self.style.confirm_btn_bg_idle
            });

            let btn_node = tree.create_node();
            if let Some(node) = tree.get_mut(btn_node) {
                node.set_name("ModalConfirmBtn");
                node.computed_rect = rect;
                node.style = Style::new()
                    .background(bg)
                    .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.15))
                    .border_radius(4.0);
            }
            let _ = tree.add_child(card_id, btn_node);

            let label_node = tree.create_node();
            if let Some(node) = tree.get_mut(label_node) {
                node.set_name("ModalConfirmLabel");
                node.set_text(label);
                node.font_size = 11.5;
                node.line_height = 14.0;
                node.text_color = self.style.confirm_btn_text;
                node.computed_rect = Rect::new(btn_x, btn_y + 7.0, btn_w, 14.0);
                node.text_align = TextAlign::Center;
            }
            let _ = tree.add_child(btn_node, label_node);

            Some(rect)
        } else {
            None
        };

        let cancel_btn_rect = if let Some(ref label) = self.cancel_btn {
            let btn_w = self.cancel_btn_width;
            let btn_x = if let Some(c_rect) = confirm_btn_rect {
                c_rect.x - 12.0 - btn_w
            } else {
                left + self.width - 16.0 - btn_w
            };
            let rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
            let is_hovered = rect.contains_point(self.cursor_pos);

            let bg = if is_hovered {
                self.style.cancel_btn_bg_hover
            } else {
                self.style.cancel_btn_bg_idle
            };

            let btn_node = tree.create_node();
            if let Some(node) = tree.get_mut(btn_node) {
                node.set_name("ModalCancelBtn");
                node.computed_rect = rect;
                node.style = Style::new()
                    .background(bg)
                    .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.10))
                    .border_radius(4.0);
            }
            let _ = tree.add_child(card_id, btn_node);

            let label_node = tree.create_node();
            if let Some(node) = tree.get_mut(label_node) {
                node.set_name("ModalCancelLabel");
                node.set_text(label);
                node.font_size = 11.5;
                node.line_height = 14.0;
                node.text_color = self.style.cancel_btn_text;
                node.computed_rect = Rect::new(btn_x, btn_y + 7.0, btn_w, 14.0);
                node.text_align = TextAlign::Center;
            }
            let _ = tree.add_child(btn_node, label_node);

            Some(rect)
        } else {
            None
        };

        ModalDialogFrame {
            scrim_id,
            card_id,
            header_id,
            content_id,
            dialog_rect,
            header_close_rect,
            confirm_btn_rect,
            cancel_btn_rect,
            content_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_dialog_builder_defaults() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        let frame = ModalDialogBuilder::new("Test Modal")
            .size(400.0, 300.0)
            .center_on_screen(1920.0, 1080.0)
            .confirm_button("Confirm", None)
            .cancel_button("Cancel")
            .build(&mut tree);

        assert!(frame.scrim_id.is_some());
        assert_eq!(frame.dialog_rect.width, 400.0);
        assert_eq!(frame.dialog_rect.height, 300.0);
        assert!(frame.header_close_rect.is_some());
        assert!(frame.confirm_btn_rect.is_some());
        assert!(frame.cancel_btn_rect.is_some());

        // Hit testing helpers
        assert!(frame.is_point_over_card(Point::new(
            frame.dialog_rect.x + 50.0,
            frame.dialog_rect.y + 50.0
        )));
        assert!(!frame.is_point_over_card(Point::new(10.0, 10.0)));

        let c_rect = frame.confirm_btn_rect.unwrap();
        assert!(frame.is_point_over_confirm(Point::new(c_rect.x + 5.0, c_rect.y + 5.0)));

        let can_rect = frame.cancel_btn_rect.unwrap();
        assert!(frame.is_point_over_cancel(Point::new(can_rect.x + 5.0, can_rect.y + 5.0)));
    }

    #[test]
    fn test_modal_without_scrim_or_buttons() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        let frame = ModalDialogBuilder::new("Alert")
            .size(300.0, 150.0)
            .position(Point::new(100.0, 100.0))
            .scrim(false)
            .close_button(false)
            .build(&mut tree);

        assert!(frame.scrim_id.is_none());
        assert!(frame.header_close_rect.is_none());
        assert!(frame.confirm_btn_rect.is_none());
        assert!(frame.cancel_btn_rect.is_none());
        assert_eq!(frame.dialog_rect, Rect::new(100.0, 100.0, 300.0, 150.0));
    }
}