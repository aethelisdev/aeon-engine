// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Responsive Declarative Section Heading
//!
//! Renders a panel or form section header with a prominent title, responsive
//! full-width wrapping subtitle, themed hairline divider, and vertical spacer on [`UiScope`].

use super::WrappedLabelDescriptor;
use crate::declarative::scope::core::UiScope;
use irisui_core::{Insets, Style, TextAlign};

impl<'a> UiScope<'a> {
    /// Emits a responsive section header with title, word-wrapping subtitle, divider, and bottom spacing.
    ///
    /// Automatically stretches across the parent container width rather than relying on hardcoded pixel widths.
    ///
    /// # Arguments
    /// * `title` - Primary section title rendered at `17.0pt`.
    /// * `subtitle` - Descriptive subtitle wrapped across available container width at `11.5pt`.
    pub fn section_heading(&mut self, title: &str, subtitle: &str) {
        self.container_named("SectionHeading", Style::new().flex_col(), |col| {
            let text_primary = col.theme().text_primary;
            let text_secondary = col.theme().text_secondary;
            let divider_color = col.theme().divider;

            col.label_styled_passive(
                "SectionHeadingTitle",
                title,
                17.0,
                text_primary,
                TextAlign::Left,
                Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
            );
            col.label_styled_passive_wrapped(
                "SectionHeadingSubtitle",
                subtitle,
                WrappedLabelDescriptor::new(
                    11.5,
                    text_secondary,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
                ),
            );
            col.divider(divider_color);
            col.empty_box_passive_named("SectionHeadingSpacer", Style::new().height(8.0));
        });
    }
}

#[cfg(test)]
mod tests {
    use irisui_core::{Rect, Style, UiTree};

    use crate::declarative::scope::UiScope;

    #[test]
    fn test_section_heading_stretches_to_parent_width_without_hardcoded_limit() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.container_named("NarrowHost", Style::new().flex_col().width(280.0), |col| {
                col.section_heading(
                    "General Settings",
                    "Configure workspace scale, palette contrast, and interactive camera behaviors across viewports.",
                );
            });
            scope.finish_layout(Rect::new(0.0, 0.0, 280.0, 400.0));
        }

        let subtitle_id = tree
            .find_node_by_name("SectionHeadingSubtitle")
            .expect("subtitle node must exist");
        let subtitle_rect = tree.get(subtitle_id).expect("node").computed_rect;
        assert!(
            (subtitle_rect.width - 280.0).abs() <= 1.0,
            "Subtitle must stretch responsively to parent width (280.0), got {}",
            subtitle_rect.width
        );
        assert!(
            subtitle_rect.height > 20.0,
            "Long subtitle must wrap across multiple lines in narrow container, got {}",
            subtitle_rect.height
        );
    }
}