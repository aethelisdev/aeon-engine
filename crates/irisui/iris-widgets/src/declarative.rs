// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout Primitives & Context Scope (`UiScope`)
//!
//! Provides a modern, declarative hierarchical UI builder interface resembling
//! modern UI frameworks. Panels use declarative scopes (`row`, `column`, `card`,
//! `button`, `drag_value`) instead of manually tracking pixel coordinates, parallel
//! target lists, or constructing monolithic boilerplate builders.

use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, UiTree, WidgetCursor, WidgetId,
    WidgetRole,
};

/// Declarative UI hierarchical builder scope.
///
/// Encapsulates a reference to the generational [`UiTree`] arena and the current
/// parent container [`WidgetId`]. All widgets created within this scope are automatically
/// appended as children to the active parent with appropriate flexbox layout properties.
pub struct UiScope<'a> {
    tree: &'a mut UiTree,
    parent: WidgetId,
}

impl<'a> UiScope<'a> {
    /// Creates a new declarative scope attached to the given parent node.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node arena.
    /// * `parent` - Target parent widget node receiving emitted children.
    pub fn new(tree: &'a mut UiTree, parent: WidgetId) -> Self {
        Self { tree, parent }
    }

    /// Returns the current parent container's [`WidgetId`].
    #[inline]
    pub fn parent(&self) -> WidgetId {
        self.parent
    }

    /// Returns a mutable reference to the underlying [`UiTree`] arena.
    #[inline]
    pub fn tree_mut(&mut self) -> &mut UiTree {
        self.tree
    }

    /// Emits a generic styled container and executes a nested child builder closure.
    ///
    /// # Arguments
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container<F>(&mut self, style: Style, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = UiScope::new(self.tree, node_id);
        f(&mut child_scope);
        node_id
    }

    /// Emits a horizontal flexbox container (`flex_direction: Row`) with standard item centering.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting horizontal items.
    pub fn row<F>(&mut self, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(6.0);
        self.container(style, f)
    }

    /// Emits a vertical flexbox container (`flex_direction: Column`) with stretch alignment.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting vertically stacked items.
    pub fn column<F>(&mut self, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_col()
            .align_items(AlignItems::Stretch)
            .gap(4.0);
        self.container(style, f)
    }

    /// Emits a collapsible-styled property card container with a stylized header and body area.
    ///
    /// # Arguments
    /// * `title` - Card section header text displayed at the top.
    /// * `f` - Child builder closure receiving the body scope of the card.
    pub fn card<F>(&mut self, title: &str, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.12, 0.12, 0.15, 0.95))
            .border(1.0, Color::rgba(0.22, 0.22, 0.28, 0.8))
            .border_radius(4.0)
            .padding(6.0)
            .gap(4.0);

        self.container(card_style, |card_scope| {
            // Header bar
            let header_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0));

            card_scope.container(header_style, |header_scope| {
                header_scope.text_colored(title, Color::rgba(0.9, 0.92, 0.96, 1.0));
            });

            // Card body column
            card_scope.column(f);
        })
    }

    /// Emits a static text label node with default secondary text coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn text(&mut self, text: impl Into<String>) -> WidgetId {
        self.text_colored(text, Color::rgba(0.78, 0.80, 0.85, 1.0))
    }

    /// Emits a static text label node with explicit text coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `color` - RGBA foreground color.
    pub fn text_colored(&mut self, text: impl Into<String>, color: Color) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.text_color = color;
            node.font_size = 12.0;
            node.line_height = 16.0;
            node.text_align = TextAlign::Left;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits an interactive push button with semantic tag identifier.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button(&mut self, label: impl Into<String>, tag: u64) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.tag = tag;
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::rgba(0.9, 0.92, 0.96, 1.0);
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(4.0, 8.0, 4.0, 8.0))
                    .background(Color::rgba(0.18, 0.19, 0.24, 0.95))
                    .border(1.0, Color::rgba(0.28, 0.29, 0.36, 0.8))
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a numeric drag-value pill component for vector properties.
    ///
    /// # Arguments
    /// * `tag` - Semantic identifier to map hover, drag, and numeric scroll events.
    /// * `prefix` - Short axis or property prefix (e.g. `"X: "`, `"Y: "`, `"Z: "`).
    /// * `value` - Floating-point scalar value to display.
    pub fn drag_value(&mut self, tag: u64, prefix: &str, value: f32) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.cursor = Some(WidgetCursor::EwResize);
            node.tag = tag;
            node.set_text(format!("{}{:.2}", prefix, value));
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::rgba(0.85, 0.88, 0.94, 1.0);
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(3.0, 6.0, 3.0, 6.0))
                    .background(Color::rgba(0.14, 0.15, 0.18, 0.95))
                    .border(1.0, Color::rgba(0.24, 0.25, 0.30, 0.8))
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a horizontal divider separator line.
    pub fn separator(&mut self) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Separator;
            node.set_style(
                Style::new()
                    .height(1.0)
                    .background(Color::rgba(0.24, 0.25, 0.30, 0.6)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_scope_hierarchy() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root should be created");

        let mut scope = UiScope::new(&mut tree, root);
        let card_id = scope.card("Transform", |card| {
            card.row(|row| {
                row.text("Position");
                row.drag_value(101, "X: ", 0.0);
                row.drag_value(102, "Y: ", 1.0);
                row.drag_value(103, "Z: ", 2.0);
            });
            card.button("Reset", 200);
        });

        assert!(tree.get(card_id).is_some());
        let root_node = tree.get(root).expect("Root node exists");
        assert_eq!(root_node.children.len(), 1);
        assert_eq!(root_node.children[0], card_id);

        let card_node = tree.get(card_id).expect("Card node exists");
        // Card has header container and body container
        assert_eq!(card_node.children.len(), 2);
    }
}