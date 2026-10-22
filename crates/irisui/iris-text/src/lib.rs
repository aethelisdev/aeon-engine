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
pub use iris_core::TextMeasurer;
pub use renderer::TextRenderer;
pub use section::TextSection;
pub use system::{TextShapeParams, TextSystem, measure_text_advance, set_default_text_family};

/// Typography engine prelude re-exporting text systems, sections, and measurement helpers.
///
/// Designed to be imported via wildcard `use iris_text::prelude::*;` or through the `irisui` facade.
pub mod prelude {
    pub use crate::{
        TextCollectionOptions, TextMeasurer, TextRenderer, TextSection, TextShapeParams,
        TextSystem, collect_text_sections, collect_text_sections_with_options,
        measure_text_advance, set_default_text_family,
    };
}

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
    fn test_custom_font_family() {
        let mut system = TextSystem::new();
        assert_eq!(system.default_family(), "sans-serif");

        system.set_default_family("Noto Sans");
        assert_eq!(system.default_family(), "Noto Sans");

        let adv = system.measure_advance("Test String", 14.0);
        assert!(adv > 0.0);

        system.set_default_family("sans-serif");
        assert_eq!(system.default_family(), "sans-serif");
        let adv_fallback = system.measure_advance("Test String", 14.0);
        assert!(adv_fallback > 0.0);
    }

    #[test]
    fn test_text_advance_dynamic() {
        let adv_11 = measure_text_advance("Hello World", 11.0);
        let adv_22 = measure_text_advance("Hello World", 22.0);

        assert!(adv_11 > 0.0);
        assert!(adv_22 > adv_11);
        assert!((adv_22 - adv_11 * 2.0).abs() < 5.0);

        let empty = measure_text_advance("", 11.0);
        assert_eq!(empty, 0.0);
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
}