// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Titled Surface Card Containers
//!
//! Provides lightweight, spacious bordered card groups with a standalone heading
//! label above free-form column content (ideal for preference sheets, shortcut
//! lists, and grouped settings without icon/action headers).

use super::super::core::UiScope;
use super::generic::InnerResponse;
use irisui_core::{Insets, Style, TextAlign};

impl<'a> UiScope<'a> {
    /// Emits a themed bordered surface card with a bold left-aligned heading and child column body.
    ///
    /// Uses generic `"SurfaceCard"` and `"SurfaceCardTitle"` node identifiers. For explicit
    /// debug or test node naming, use [`Self::titled_surface_card_named`].
    ///
    /// # Arguments
    /// * `title` - Card heading text displayed at the top of the surface.
    /// * `f` - Child closure emitting content inside the bordered card surface.
    pub fn titled_surface_card<R, F>(&mut self, title: &str, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.titled_surface_card_named("SurfaceCard", "SurfaceCardTitle", title, f)
    }

    /// Emits a named themed surface card with a passive heading label and bottom card spacing.
    ///
    /// # Arguments
    /// * `card_name` - Container node identifier assigned to the outer card frame.
    /// * `title_name` - Text node identifier assigned to the card title label.
    /// * `title` - Card heading text displayed at the top of the surface.
    /// * `f` - Child closure emitting content inside the bordered card surface.
    pub fn titled_surface_card_named<R, F>(
        &mut self,
        card_name: impl Into<String>,
        title_name: impl Into<String>,
        title: &str,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let theme = self.tree.theme();
        let card_style = Style::new()
            .flex_col()
            .background(theme.card_bg)
            .border(1.0, theme.card_border)
            .border_radius(6.0)
            .padding_insets(Insets::new(12.0, 14.0, 10.0, 14.0))
            .margin_insets(Insets::new(0.0, 0.0, 12.0, 0.0));

        self.container_named(card_name, card_style, |card| {
            card.label_styled_passive(
                title_name,
                title,
                12.5,
                theme.text_primary,
                TextAlign::Left,
                Style::new().margin_insets(Insets::new(0.0, 0.0, 8.0, 0.0)),
            );
            f(card)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_titled_surface_card_structure_and_spacing() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let resp = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.titled_surface_card_named("GroupCard", "GroupTitle", "Navigation", |card| {
                card.label("Shortcut entry");
                7
            })
        };

        assert_eq!(resp.inner, 7);
        let card_node = tree.get(resp.response.id).expect("Card node exists");
        assert_eq!(card_node.name.as_deref(), Some("GroupCard"));
        assert_eq!(card_node.style.margin.bottom, 12.0);
        assert_eq!(card_node.children.len(), 2);

        let title_node = tree.get(card_node.children[0]).expect("Title node exists");
        assert_eq!(title_node.name.as_deref(), Some("GroupTitle"));
        assert_eq!(title_node.text.as_deref(), Some("Navigation"));
        assert_eq!(title_node.font_size, 12.5);
    }
}