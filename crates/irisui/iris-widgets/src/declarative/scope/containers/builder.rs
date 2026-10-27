// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Fluent Container & Layout Builders
//!
//! Provides zero-`let` method-chained container builders on [`UiScope`]: [`RowBuilder`],
//! [`ColumnBuilder`], and [`ContainerBuilder`]. Eliminates intermediate `let ..style = Style::new()`
//! assignments across consumer codebases.

use super::super::core::UiScope;
use super::generic::InnerResponse;
use iris_core::{AlignItems, Color, CornerRadii, Insets, JustifyContent, Style, WidgetRole};

/// Fluent builder for horizontal flexbox row containers.
#[must_use = "builders do nothing until .show() is called"]
pub struct RowBuilder<'a, 'b> {
    scope: &'b mut UiScope<'a>,
    name: Option<String>,
    style: Style,
    tag: u64,
    role: WidgetRole,
}

impl<'a, 'b> RowBuilder<'a, 'b> {
    /// Creates a new row builder with standard horizontal flex defaults.
    pub fn new(scope: &'b mut UiScope<'a>) -> Self {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(6.0);
        Self {
            scope,
            name: None,
            style,
            tag: 0,
            role: WidgetRole::Default,
        }
    }

    /// Sets a descriptive node name for tree inspection and debugging.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the horizontal spacing gap between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.style = self.style.gap(gap);
        self
    }

    /// Sets uniform padding across all four sides.
    pub fn padding(mut self, padding: f32) -> Self {
        self.style = self.style.padding(padding);
        self
    }

    /// Sets directional padding insets.
    pub fn padding_insets(mut self, insets: Insets) -> Self {
        self.style = self.style.padding_insets(insets);
        self
    }

    /// Sets directional margin insets.
    pub fn margin_insets(mut self, insets: Insets) -> Self {
        self.style = self.style.margin_insets(insets);
        self
    }

    /// Sets the fixed container height in physical pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.style = self.style.height(height);
        self
    }

    /// Sets the fixed container width in physical pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.style = self.style.width(width);
        self
    }

    /// Sets the flex-grow factor.
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.style = self.style.flex_grow(grow);
        self
    }

    /// Sets cross-axis alignment (`align-items`).
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.style = self.style.align_items(align);
        self
    }

    /// Sets main-axis distribution (`justify-content`).
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.style = self.style.justify_content(justify);
        self
    }

    /// Shorthand setting `justify-content: SpaceBetween`.
    pub fn space_between(mut self) -> Self {
        self.style = self.style.justify_content(JustifyContent::SpaceBetween);
        self
    }

    /// Shorthand setting centered alignment along both axes (`align-items: Center`, `justify-content: Center`).
    pub fn centered(mut self) -> Self {
        self.style = self
            .style
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self
    }

    /// Sets background fill color.
    pub fn background(mut self, color: Color) -> Self {
        self.style = self.style.background(color);
        self
    }

    /// Sets background fill color from an active theme token.
    pub fn background_themed(mut self, f: impl FnOnce(&iris_core::Theme) -> Color) -> Self {
        let theme = self.scope.theme();
        self.style = self.style.background(f(&theme));
        self
    }

    /// Sets border width and color.
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style = self.style.border(width, color);
        self
    }

    /// Sets border stroke width and color from an active theme token.
    pub fn border_themed(mut self, width: f32, f: impl FnOnce(&iris_core::Theme) -> Color) -> Self {
        let theme = self.scope.theme();
        self.style = self.style.border(width, f(&theme));
        self
    }

    /// Sets uniform border corner radius.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.style = self.style.border_radius(radius);
        self
    }

    /// Sets discrete corner radii.
    pub fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.style = self.style.corner_radii(radii);
        self
    }

    /// Configures child clipping (overflow hidden).
    pub fn clip_children(mut self, clip: bool) -> Self {
        self.style = self.style.clip_children(clip);
        self
    }

    /// Assigns a persistent 64-bit semantic tag and accessibility role.
    pub fn tagged(mut self, tag: u64, role: WidgetRole) -> Self {
        self.tag = tag;
        self.role = role;
        self
    }

    /// Builds the row container and executes the child builder closure within the new scope, returning [`InnerResponse`].
    pub fn show<R, F>(self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if self.tag != 0 {
            let name = self.name.unwrap_or_else(|| "Row".to_string());
            self.scope
                .container_tagged(name, self.style, self.role, self.tag, f)
        } else if let Some(name) = self.name {
            self.scope.container_named(name, self.style, f)
        } else {
            self.scope.container(self.style, f)
        }
    }
}

/// Fluent builder for vertical flexbox column containers.
#[must_use = "builders do nothing until .show() is called"]
pub struct ColumnBuilder<'a, 'b> {
    scope: &'b mut UiScope<'a>,
    name: Option<String>,
    style: Style,
    tag: u64,
    role: WidgetRole,
}

impl<'a, 'b> ColumnBuilder<'a, 'b> {
    /// Creates a new column builder with standard vertical flex defaults.
    pub fn new(scope: &'b mut UiScope<'a>) -> Self {
        let style = Style::new()
            .flex_col()
            .align_items(AlignItems::Stretch)
            .gap(4.0);
        Self {
            scope,
            name: None,
            style,
            tag: 0,
            role: WidgetRole::Default,
        }
    }

    /// Sets a descriptive node name for tree inspection and debugging.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the vertical spacing gap between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.style = self.style.gap(gap);
        self
    }

    /// Sets uniform padding across all four sides.
    pub fn padding(mut self, padding: f32) -> Self {
        self.style = self.style.padding(padding);
        self
    }

    /// Sets directional padding insets.
    pub fn padding_insets(mut self, insets: Insets) -> Self {
        self.style = self.style.padding_insets(insets);
        self
    }

    /// Sets directional margin insets.
    pub fn margin_insets(mut self, insets: Insets) -> Self {
        self.style = self.style.margin_insets(insets);
        self
    }

    /// Sets the fixed container height in physical pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.style = self.style.height(height);
        self
    }

    /// Sets the fixed container width in physical pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.style = self.style.width(width);
        self
    }

    /// Sets the flex-grow factor.
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.style = self.style.flex_grow(grow);
        self
    }

    /// Sets cross-axis alignment (`align-items`).
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.style = self.style.align_items(align);
        self
    }

    /// Sets main-axis distribution (`justify-content`).
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.style = self.style.justify_content(justify);
        self
    }

    /// Shorthand setting centered alignment along both axes.
    pub fn centered(mut self) -> Self {
        self.style = self
            .style
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self
    }

    /// Shorthand setting `justify-content: SpaceBetween`.
    pub fn space_between(mut self) -> Self {
        self.style = self.style.justify_content(JustifyContent::SpaceBetween);
        self
    }

    /// Sets background fill color.
    pub fn background(mut self, color: Color) -> Self {
        self.style = self.style.background(color);
        self
    }

    /// Sets background fill color from an active theme token.
    pub fn background_themed(mut self, f: impl FnOnce(&iris_core::Theme) -> Color) -> Self {
        let theme = self.scope.theme();
        self.style = self.style.background(f(&theme));
        self
    }

    /// Sets border width and color.
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style = self.style.border(width, color);
        self
    }

    /// Sets border stroke width and color from an active theme token.
    pub fn border_themed(mut self, width: f32, f: impl FnOnce(&iris_core::Theme) -> Color) -> Self {
        let theme = self.scope.theme();
        self.style = self.style.border(width, f(&theme));
        self
    }

    /// Sets uniform border corner radius.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.style = self.style.border_radius(radius);
        self
    }

    /// Sets discrete corner radii.
    pub fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.style = self.style.corner_radii(radii);
        self
    }

    /// Configures child clipping (overflow hidden).
    pub fn clip_children(mut self, clip: bool) -> Self {
        self.style = self.style.clip_children(clip);
        self
    }

    /// Sets vertical scroll offset in physical pixels.
    pub fn scroll_offset_y(mut self, offset: f32) -> Self {
        self.style = self.style.scroll_offset_y(offset);
        self
    }

    /// Assigns a persistent 64-bit semantic tag and accessibility role.
    pub fn tagged(mut self, tag: u64, role: WidgetRole) -> Self {
        self.tag = tag;
        self.role = role;
        self
    }

    /// Builds the column container and executes the child builder closure within the new scope, returning [`InnerResponse`].
    pub fn show<R, F>(self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if self.tag != 0 {
            let name = self.name.unwrap_or_else(|| "Column".to_string());
            self.scope
                .container_tagged(name, self.style, self.role, self.tag, f)
        } else if let Some(name) = self.name {
            self.scope.container_named(name, self.style, f)
        } else {
            self.scope.container(self.style, f)
        }
    }
}

impl<'a> UiScope<'a> {
    /// Initiates a fluent, chainable row layout builder for constructing styled horizontal flex rows without `let` statements.
    #[inline]
    pub fn row_with(&mut self) -> RowBuilder<'a, '_> {
        RowBuilder::new(self)
    }

    /// Initiates a fluent, chainable column layout builder for constructing styled vertical flex columns without `let` statements.
    #[inline]
    pub fn column_with(&mut self) -> ColumnBuilder<'a, '_> {
        ColumnBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_row_builder_fluent_chaining() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let (resp, row_id) = {
            let mut scope = UiScope::new(&mut tree, root);

            let resp = scope
                .row_with()
                .name("CustomHeaderRow")
                .gap(8.0)
                .height(32.0)
                .padding(4.0)
                .centered()
                .show(|row| {
                    row.label("Item 1");
                    row.label("Item 2");
                    42
                });
            let rid = resp.response.id;
            (resp, rid)
        };

        assert_eq!(resp.inner, 42);
        let node = tree.get(row_id).expect("Row node exists");
        assert_eq!(node.name.as_deref(), Some("CustomHeaderRow"));
        assert_eq!(node.style.gap, 8.0);
        assert_eq!(node.style.height, Some(32.0));
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn test_column_builder_fluent_chaining() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let (resp, col_id) = {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope
                .column_with()
                .name("SidebarCol")
                .width(200.0)
                .gap(12.0)
                .flex_grow(1.0)
                .space_between()
                .show(|col| {
                    col.label("Header");
                    col.spacer_px(16.0);
                    col.label("Footer");
                    "SidebarContent"
                });
            let col_id = resp.response.id;
            (resp, col_id)
        };

        assert_eq!(resp.inner, "SidebarContent");
        let node = tree.get(col_id).expect("Column node exists");
        assert_eq!(node.name.as_deref(), Some("SidebarCol"));
        assert_eq!(node.style.width, Some(200.0));
        assert_eq!(node.style.gap, 12.0);
        assert_eq!(node.style.flex_grow, 1.0);
        assert_eq!(node.children.len(), 3);
    }

    #[test]
    fn test_builder_themed_styling() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let (row_id, col_id) = {
            let mut scope = UiScope::new(&mut tree, root);

            let row_resp = scope
                .row_with()
                .name("ThemedRow")
                .background_themed(|t| t.toolbar_bg)
                .border_themed(1.0, |t| t.toolbar_border)
                .show(|_| ());

            let col_resp = scope
                .column_with()
                .name("ThemedCol")
                .background_themed(|t| t.card_bg)
                .border_themed(2.0, |t| t.card_border)
                .show(|_| ());

            let r_id = row_resp.response.id;
            let c_id = col_resp.response.id;
            (r_id, c_id)
        };

        let node = tree.get(row_id).expect("Row node exists");
        assert_eq!(node.style.background_color, tree.theme().toolbar_bg);
        assert_eq!(
            node.style.border,
            iris_core::Border::uniform(1.0, tree.theme().toolbar_border)
        );

        let col_node = tree.get(col_id).expect("Col node exists");
        assert_eq!(col_node.style.background_color, tree.theme().card_bg);
        assert_eq!(
            col_node.style.border,
            iris_core::Border::uniform(2.0, tree.theme().card_border)
        );
    }
}