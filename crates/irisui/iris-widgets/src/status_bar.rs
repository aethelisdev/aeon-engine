// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bottom status and diagnostics utility bar widget builder.

use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, UiTree, WidgetId};

/// Builder for bottom application status bar and diagnostics HUD.
pub struct StatusBarBuilder<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
    left_group: WidgetId,
    right_group: WidgetId,
}

impl<'a> StatusBarBuilder<'a> {
    /// Creates a new `StatusBarBuilder` positioned along the bottom of the window.
    pub fn new(tree: &'a mut UiTree, width: f32, height: f32) -> Self {
        let node_id = tree.create_node();
        let left_group = tree.create_node();
        let right_group = tree.create_node();

        if let Some(node) = tree.get_mut(node_id) {
            node.set_name("BottomStatusBar");
            node.set_style(
                Style::new()
                    .flex_row()
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Center)
                    .width(width)
                    .height(height)
                    .padding_insets(Insets::new(0.0, 10.0, 0.0, 10.0))
                    .background(Color::hex("#0f0f14"))
                    .border(1.0, Color::hex("#282a34")),
            );
        }

        if let Some(left) = tree.get_mut(left_group) {
            left.set_name("StatusBarLeftGroup");
            left.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(10.0),
            );
        }

        if let Some(right) = tree.get_mut(right_group) {
            right.set_name("StatusBarRightGroup");
            right.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(8.0),
            );
        }

        let _ = tree.add_child(node_id, left_group);
        let _ = tree.add_child(node_id, right_group);

        Self {
            tree,
            node_id,
            left_group,
            right_group,
        }
    }

    /// Adds a status text message or indicator to the left side of the bar.
    pub fn add_status_indicator(&mut self, text: &str, color: Color) -> WidgetId {
        let id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(id) {
            node.set_name("StatusBarIndicator");
            node.set_text(text);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Left;
            node.text_color = color;
        }
        let _ = self.tree.add_child(self.left_group, id);
        id
    }

    /// Adds a pill-shaped diagnostic metric badge to the right side of the bar.
    pub fn add_diagnostics_pill(
        &mut self,
        text: &str,
        text_color: Color,
        bg_color: Color,
    ) -> WidgetId {
        let pill_id = self.tree.create_node();
        let text_id = self.tree.create_node();

        if let Some(node) = self.tree.get_mut(text_id) {
            node.set_name("DiagnosticsPillText");
            node.set_text(text);
            node.font_size = 10.5;
            node.line_height = 13.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
        }

        if let Some(node) = self.tree.get_mut(pill_id) {
            node.set_name("DiagnosticsPill");
            node.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .padding_insets(Insets::new(1.0, 6.0, 1.0, 6.0))
                    .background(bg_color)
                    .border_radius(3.0),
            );
        }

        let _ = self.tree.add_child(pill_id, text_id);
        let _ = self.tree.add_child(self.right_group, pill_id);
        pill_id
    }

    /// Adds a plain label to the right side of the status bar.
    pub fn add_right_label(&mut self, text: &str, color: Color) -> WidgetId {
        let id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(id) {
            node.set_name("StatusBarRightLabel");
            node.set_text(text);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Right;
            node.text_color = color;
        }
        let _ = self.tree.add_child(self.right_group, id);
        id
    }

    /// Consumes the builder and returns the root `WidgetId` of the status bar.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}