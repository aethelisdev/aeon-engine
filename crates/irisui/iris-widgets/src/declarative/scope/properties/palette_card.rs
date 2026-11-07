// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Color Palette Card Primitives
//!
//! Provides [`UiScope::color_palette_card`] and [`ColorPaletteCardConfig`]
//! for rendering an integrated card containing an interactive color swatch,
//! live hex input, and quick-select palette grid in a single declarative call.

use iris_core::Color;

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::properties::swatch_row::PropertyColorSwatchRowConfig;

/// Configuration options for rendering a declarative color palette card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPaletteCardConfig<'a> {
    /// Human-readable title displayed in the card header.
    pub title: &'a str,
    /// Semantic node name assigned to the container card.
    pub card_name: &'a str,
    /// Currently active color rendered inside the swatch and used as default addition.
    pub current_color: Color,
    /// Hex-formatted string displayed inside the text input box.
    pub hex: &'a str,
    /// Slice of default colors presented at the head of the palette.
    pub default_palette: &'a [Color],
    /// Optional semantic tag assigned to the color swatch button.
    pub swatch_tag: Option<u64>,
    /// Optional semantic tag assigned to the hex input box.
    pub hex_tag: Option<u64>,
}

impl<'a> ColorPaletteCardConfig<'a> {
    /// Constructs a new [`ColorPaletteCardConfig`] with standard defaults.
    #[inline]
    #[must_use]
    pub const fn new(
        card_name: &'a str,
        title: &'a str,
        current_color: Color,
        hex: &'a str,
        default_palette: &'a [Color],
    ) -> Self {
        Self {
            card_name,
            title,
            current_color,
            hex,
            default_palette,
            swatch_tag: None,
            hex_tag: None,
        }
    }

    /// Overrides the persistent hit-testing semantic tags for swatch and hex boxes.
    #[inline]
    #[must_use]
    pub const fn with_tags(mut self, swatch_tag: u64, hex_tag: u64) -> Self {
        self.swatch_tag = Some(swatch_tag);
        self.hex_tag = Some(hex_tag);
        self
    }
}

impl<'a> UiScope<'a> {
    /// Renders an autonomous declarative color palette card containing a preview swatch row
    /// and a quick-select swatch grid, returning `Some(Color)` if a new color was committed or selected.
    ///
    /// Completely eliminates closure nesting staircases and intermediate `let` variables in consumer UI code.
    pub fn color_palette_card(
        &mut self,
        config: ColorPaletteCardConfig<'_>,
        saved_swatches: &mut Vec<[f32; 4]>,
        is_color_picker_open: &mut bool,
    ) -> Option<Color> {
        let row_config = match (config.swatch_tag, config.hex_tag) {
            (Some(st), Some(ht)) => {
                PropertyColorSwatchRowConfig::new("Object Color:", config.current_color, config.hex)
                    .with_tags(st, ht)
            }
            _ => {
                PropertyColorSwatchRowConfig::new("Object Color:", config.current_color, config.hex)
            }
        };

        self.card_named(config.card_name, config.title, |card| {
            card.prop_color_swatch_row_with_config(row_config, is_color_picker_open)
                .then_palette_map(
                    card,
                    "Add to Palette:",
                    config.default_palette,
                    saved_swatches,
                    |c| c,
                )
        })
        .inner
    }
}