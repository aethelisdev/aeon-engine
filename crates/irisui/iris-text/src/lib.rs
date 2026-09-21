// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Typography Engine (`iris-text`)
//!
//! Subpixel GPU text rendering, font shaping, and glyph caching engine for Iris UI.
//! Powered by `cosmic-text` and `glyphon`.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod collector;
pub mod renderer;
pub mod section;
pub mod system;

pub use collector::{
    TextCollectionOptions, collect_text_sections, collect_text_sections_with_options,
};
pub use renderer::TextRenderer;
pub use section::TextSection;
pub use system::{TextShapeParams, TextSystem};

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Color, Rect, TextAlign};

    #[test]
    fn test_text_measurement() {
        let mut system = TextSystem::new();
        let size = system.measure_text("Hello Iris UI", 16.0, 20.0, None);

        assert!(size.width > 0.0);
        assert!(size.height >= 20.0);
    }

    #[test]
    fn test_text_section_builder() {
        let section = TextSection::new("Button Label", Rect::new(0.0, 0.0, 100.0, 40.0))
            .with_font_size(14.0, 18.0)
            .with_color(Color::RED)
            .with_align(TextAlign::Center);

        assert_eq!(section.text, "Button Label");
        assert_eq!(section.font_size, 14.0);
        assert_eq!(section.color, Color::RED);
        assert_eq!(section.align, TextAlign::Center);
    }

    #[test]
    fn test_text_wrapping_modes() {
        use iris_core::TextWrap;

        let mut system = TextSystem::new();
        let long_sentence = "Place 3D models, textures, shaders, or scenes into this folder.";

        // In a narrow box of 120px with sufficient height (40px) and TextWrap::Auto, it should wrap into multiple lines
        let wrapped_buf = system.shape_text(
            long_sentence,
            TextShapeParams {
                font_size: 12.0,
                line_height: 16.0,
                bounds_width: 120.0,
                bounds_height: 60.0,
                align: TextAlign::Left,
                wrap: TextWrap::Auto,
            },
        );
        let wrapped_lines = wrapped_buf.layout_runs().count();
        assert!(
            wrapped_lines > 1,
            "Expected wrapped lines > 1, got {wrapped_lines}"
        );

        // With TextWrap::None, it must strictly produce exactly 1 line
        let single_buf = system.shape_text(
            long_sentence,
            TextShapeParams {
                font_size: 12.0,
                line_height: 16.0,
                bounds_width: 120.0,
                bounds_height: 60.0,
                align: TextAlign::Left,
                wrap: TextWrap::None,
            },
        );
        let single_lines = single_buf.layout_runs().count();
        assert_eq!(
            single_lines, 1,
            "Expected exactly 1 line for TextWrap::None, got {single_lines}"
        );
    }

    #[test]
    fn test_measure_tree_populates_content_size() {
        use iris_core::UiTree;

        let mut tree = UiTree::new();
        let root = tree.create_node();
        let child1 = tree.create_node();
        let child2 = tree.create_node();

        if let Some(node) = tree.get_mut(child1) {
            node.set_text("File");
            node.font_size = 12.0;
            node.line_height = 14.0;
        }

        if let Some(node) = tree.get_mut(child2) {
            node.set_text("Very Long Action Button Label");
            node.font_size = 14.0;
            node.line_height = 18.0;
        }

        let _ = tree.add_child(root, child1);
        let _ = tree.add_child(root, child2);

        let mut system = TextSystem::new();
        system.measure_tree(&mut tree, root);

        let size1 = tree.get(child1).unwrap().content_size;
        let size2 = tree.get(child2).unwrap().content_size;

        assert!(size1.width > 0.0);
        assert!(size1.height >= 14.0);
        assert!(size2.width > size1.width);
        assert!(size2.height >= 18.0);
    }
}