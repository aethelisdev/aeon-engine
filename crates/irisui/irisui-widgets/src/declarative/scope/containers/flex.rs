// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Flexbox Layout Primitives
//!
//! Provides idiomatic row and column grouping primitives on [`UiScope`]: standard flex rows,
//! parameterized spaced rows, flex columns, spaced columns, space-between headers, and centered boxes.

use super::super::core::UiScope;
use super::generic::InnerResponse;
use irisui_core::{AlignItems, Color, Insets, JustifyContent, Style};

impl<'a> UiScope<'a> {
    /// Emits a horizontal flexbox container (`flex_direction: Row`) with standard centered alignment and 6.0px gap.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting horizontal items.
    pub fn row<R, F>(&mut self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.row_spaced(6.0, f)
    }

    /// Emits a horizontal flexbox container (`flex_direction: Row`) with centered vertical alignment and custom gap.
    ///
    /// Eliminates the imperative `let left_style = Style::new().flex_row()...` boilerplate across panels.
    ///
    /// # Arguments
    /// * `gap` - Spacing gap between horizontal children in physical pixels.
    /// * `f` - Child scope closure emitting horizontal items.
    pub fn row_spaced<R, F>(&mut self, gap: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(gap);
        self.container(style, f)
    }

    /// Emits a horizontal flexbox container with `justify_content: SpaceBetween` and vertically centered items.
    ///
    /// Ideal for header strips, titlebars, and toolbars containing left-aligned titles and right-aligned action buttons.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting spaced-apart items.
    pub fn space_between<R, F>(&mut self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween);
        self.container(style, f)
    }

    /// Emits a vertical flexbox container (`flex_direction: Column`) with stretch alignment and 4.0px gap.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting vertical items.
    pub fn column<R, F>(&mut self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.column_spaced(4.0, f)
    }

    /// Emits a vertical flexbox container (`flex_direction: Column`) with stretch alignment and custom gap.
    ///
    /// Eliminates intermediate `let style = Style::new().flex_col()...` boilerplate across panels.
    ///
    /// # Arguments
    /// * `gap` - Spacing gap between vertical children in physical pixels.
    /// * `f` - Child scope closure emitting vertical items.
    pub fn column_spaced<R, F>(&mut self, gap: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let style = Style::new()
            .flex_col()
            .align_items(AlignItems::Stretch)
            .gap(gap);
        self.container(style, f)
    }

    /// Emits a flexbox container perfectly centered along both horizontal and vertical axes.
    ///
    /// Ideal for modal dialog centers, placeholder empty state cards, and icon badges.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting centered items.
    pub fn centered<R, F>(&mut self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let style = Style::new()
            .flex_col()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.container(style, f)
    }

    /// Emits a flex-growing column container vertically centered with stretched horizontal children.
    ///
    /// Ideal for empty states, missing-selection notices, and placeholder viewports that must fill
    /// all available panel height while stretching enclosed cards or banners across the full width.
    ///
    /// # Arguments
    /// * `padding` - Uniform outer padding inset applied around all edges in logical pixels.
    /// * `f` - Child scope closure emitting vertically centered content.
    pub fn centered_stretch<R, F>(&mut self, padding: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .align_items(AlignItems::Stretch)
            .justify_content(JustifyContent::Center)
            .padding_insets(Insets::new(padding, padding, padding, padding));
        self.container(style, f)
    }

    /// Emits a horizontal toolbar container with a fixed height and centered flex alignment.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug name for the toolbar node.
    /// * `height` - Fixed toolbar height in physical pixels.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the toolbar scope.
    pub fn toolbar<R, F>(
        &mut self,
        name: &'static str,
        height: f32,
        bg: Color,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(height)
            .background(bg)
            .gap(6.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08));
        self.container_named(name, style, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_flex_row_and_column_spaced() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let (row_resp, col_resp, row_id, col_id) = {
            let mut scope = UiScope::new(&mut tree, root);

            let row_resp = scope.row_spaced(14.0, |row| {
                row.label("A");
                row.label("B");
                100
            });

            let col_resp = scope.column_spaced(20.0, |col| {
                col.label("1");
                col.label("2");
                "col_inner"
            });

            (
                row_resp,
                col_resp,
                row_resp.response.id,
                col_resp.response.id,
            )
        };

        assert_eq!(row_resp.inner, 100);
        let row_node = tree.get(row_id).expect("Row node exists");
        assert_eq!(row_node.style.gap, 14.0);
        assert_eq!(row_node.children.len(), 2);

        assert_eq!(col_resp.inner, "col_inner");
        let col_node = tree.get(col_id).expect("Column node exists");
        assert_eq!(col_node.style.gap, 20.0);
        assert_eq!(col_node.children.len(), 2);
    }

    #[test]
    fn test_space_between_and_centered_shortcuts() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let (sb_resp, ctr_resp, fill_resp, sb_id, ctr_id, fill_id) = {
            let mut scope = UiScope::new(&mut tree, root);

            let sb_resp = scope.space_between(|row| {
                row.label("Left");
                row.label("Right");
                true
            });

            let ctr_resp = scope.centered(|col| {
                col.label("Center content");
                999
            });

            let fill_resp = scope.centered_stretch(12.0, |col| {
                col.label("Stretched empty state");
                42
            });

            (
                sb_resp,
                ctr_resp,
                fill_resp,
                sb_resp.response.id,
                ctr_resp.response.id,
                fill_resp.response.id,
            )
        };

        assert!(sb_resp.inner);
        let sb_node = tree.get(sb_id).expect("Space between node exists");
        assert_eq!(
            sb_node.style.justify_content,
            irisui_core::JustifyContent::SpaceBetween
        );
        assert_eq!(sb_node.style.align_items, irisui_core::AlignItems::Center);

        assert_eq!(ctr_resp.inner, 999);
        let ctr_node = tree.get(ctr_id).expect("Centered node exists");
        assert_eq!(
            ctr_node.style.justify_content,
            irisui_core::JustifyContent::Center
        );
        assert_eq!(ctr_node.style.align_items, irisui_core::AlignItems::Center);

        assert_eq!(fill_resp.inner, 42);
        let fill_node = tree.get(fill_id).expect("Centered stretch node exists");
        assert_eq!(fill_node.style.flex_grow, 1.0);
        assert_eq!(
            fill_node.style.align_items,
            irisui_core::AlignItems::Stretch
        );
        assert_eq!(
            fill_node.style.justify_content,
            irisui_core::JustifyContent::Center
        );
        assert_eq!(fill_node.style.padding.left, 12.0);
    }
}