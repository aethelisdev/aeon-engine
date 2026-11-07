// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Color Palette & Swatches Grid
//!
//! Provides the [`UiScope::prop_color_palette`] and [`UiScope::prop_color_palette_with_config`]
//! declarative layout methods for interactive color palette inspection, swatch picking,
//! in-place swatch accumulation (`+`), bulk clearing (`🗑`), and swatch removal.

use iris_core::geometry::Insets;
use iris_core::{AlignItems, Color, FlexWrap, Style, TextAlign};

use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::Button;
use crate::declarative::widget::color_swatch::ColorSwatch;

/// Configuration parameters for rendering a declarative color palette.
#[derive(Debug, Clone, Copy)]
pub struct PropertyPaletteConfig<'a> {
    /// Human-readable section label displayed in the action row.
    pub label: &'a str,
    /// Currently active color to be captured and added when clicking `+`.
    pub current_color: Color,
    /// Fixed default color swatches displayed at the head of the grid.
    pub default_swatches: &'a [Color],
    /// Maximum allowed count of custom saved swatches.
    pub max_custom: usize,
    /// Width and height dimension of individual color swatches in physical pixels.
    pub swatch_size: f32,
    /// Gap between adjacent swatches in physical pixels.
    pub swatch_gap: f32,
    /// Optional explicit semantic hit-testing tag for the `+` (add) button.
    pub add_tag: Option<u64>,
    /// Optional explicit semantic hit-testing tag for the `🗑` (clear) button.
    pub clear_tag: Option<u64>,
    /// Optional base tag value for generating sequential semantic tags for swatches.
    pub swatch_tag_base: Option<u64>,
}

impl<'a> PropertyPaletteConfig<'a> {
    /// Constructs a standard configuration with default sizing and limits.
    #[inline]
    #[must_use]
    pub const fn new(label: &'a str, current_color: Color, default_swatches: &'a [Color]) -> Self {
        Self {
            label,
            current_color,
            default_swatches,
            max_custom: 28,
            swatch_size: 16.0,
            swatch_gap: 6.0,
            add_tag: None,
            clear_tag: None,
            swatch_tag_base: None,
        }
    }

    /// Attaches explicit semantic hit-testing tags for automation testing.
    ///
    /// Ignores non-semantic `0` tags to permit automatic tag derivation.
    #[inline]
    #[must_use]
    pub const fn with_tags(mut self, add_tag: u64, clear_tag: u64, swatch_tag_base: u64) -> Self {
        self.add_tag = if add_tag != 0 { Some(add_tag) } else { None };
        self.clear_tag = if clear_tag != 0 {
            Some(clear_tag)
        } else {
            None
        };
        self.swatch_tag_base = if swatch_tag_base != 0 {
            Some(swatch_tag_base)
        } else {
            None
        };
        self
    }

    /// Sets the maximum number of custom swatches permitted in the palette.
    #[inline]
    #[must_use]
    pub const fn max_custom(mut self, max: usize) -> Self {
        self.max_custom = max;
        self
    }

    /// Sets the physical pixel dimensions of individual swatches.
    #[inline]
    #[must_use]
    pub const fn swatch_size(mut self, size: f32) -> Self {
        self.swatch_size = size;
        self
    }
}

/// Comprehensive interaction response returned after evaluating a declarative color palette.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PropertyPaletteResponse {
    /// Color selected by clicking an existing swatch on this frame (if any).
    pub selected_color: Option<Color>,
    /// Whether the active color was added to custom swatches on this frame.
    pub added: bool,
    /// Whether custom swatches were cleared on this frame.
    pub cleared: bool,
    /// Zero-based index of custom swatch removed via secondary click (if any).
    pub removed_index: Option<usize>,
}

impl PropertyPaletteResponse {
    /// Constructs a new [`PropertyPaletteResponse`].
    #[inline]
    #[must_use]
    pub const fn new(
        selected_color: Option<Color>,
        added: bool,
        cleared: bool,
        removed_index: Option<usize>,
    ) -> Self {
        Self {
            selected_color,
            added,
            cleared,
            removed_index,
        }
    }

    /// Invokes `f(color)` if a swatch color was selected during this frame.
    #[inline]
    pub fn on_select<F: FnOnce(Color)>(self, f: F) -> Self {
        if let Some(col) = self.selected_color {
            f(col);
        }
        self
    }

    /// Invokes `f()` if the active color was added to custom swatches during this frame.
    #[inline]
    pub fn on_add<F: FnOnce()>(self, f: F) -> Self {
        if self.added {
            f();
        }
        self
    }

    /// Invokes `f()` if custom swatches were cleared during this frame.
    #[inline]
    pub fn on_clear<F: FnOnce()>(self, f: F) -> Self {
        if self.cleared {
            f();
        }
        self
    }

    /// Invokes `f(index)` if a custom swatch was removed during this frame.
    #[inline]
    pub fn on_remove<F: FnOnce(usize)>(self, f: F) -> Self {
        if let Some(idx) = self.removed_index {
            f(idx);
        }
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative color palette section with standard sizing.
    ///
    /// Directly modifies `custom_swatches` in-place when `+` (add), `🗑` (clear),
    /// or right-click swatch removal is triggered.
    pub fn prop_color_palette(
        &mut self,
        label: &str,
        current_color: Color,
        default_swatches: &[Color],
        custom_swatches: &mut Vec<[f32; 4]>,
    ) -> PropertyPaletteResponse {
        self.prop_color_palette_with_config(
            PropertyPaletteConfig::new(label, current_color, default_swatches),
            custom_swatches,
        )
    }

    /// Emits a declarative color palette section using the provided [`PropertyPaletteConfig`].
    ///
    /// Directly modifies `custom_swatches` in-place when `+` (add), `🗑` (clear),
    /// or right-click swatch removal is triggered.
    pub fn prop_color_palette_with_config(
        &mut self,
        config: PropertyPaletteConfig<'_>,
        custom_swatches: &mut Vec<[f32; 4]>,
    ) -> PropertyPaletteResponse {
        let (added, cleared) = self.render_palette_action_bar(&config, custom_swatches);
        let (selected_color, removed_index) =
            self.render_palette_swatches_grid(&config, custom_swatches);

        PropertyPaletteResponse::new(selected_color, added, cleared, removed_index)
    }

    fn render_palette_action_bar(
        &mut self,
        config: &PropertyPaletteConfig<'_>,
        custom_swatches: &mut Vec<[f32; 4]>,
    ) -> (bool, bool) {
        let mut added = false;
        let mut cleared = false;

        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(22.0)
            .gap(6.0);

        self.container_named("PaletteActionBar", row_style, |row| {
            row.label_styled_passive(
                "PaletteActionLabel",
                config.label,
                11.0,
                row.theme().text_muted,
                TextAlign::Left,
                Style::new().width(88.0).height(22.0),
            );

            let mut add_btn = Button::new("+##add_palette")
                .name("AddPaletteButton")
                .size(18.0, 18.0)
                .padding(Insets::ZERO);
            if let Some(tag) = config.add_tag {
                add_btn = add_btn.tag(tag);
            }
            if row.add(add_btn).clicked() {
                let arr = config.current_color.to_array();
                if !custom_swatches.contains(&arr) && custom_swatches.len() < config.max_custom {
                    custom_swatches.push(arr);
                    added = true;
                }
                row.tree.event_state.clear_frame_clicks();
            }

            let mut clear_btn = Button::new("🗑##clear_palette")
                .name("ClearPaletteButton")
                .size(18.0, 18.0)
                .padding(Insets::ZERO);
            if let Some(tag) = config.clear_tag {
                clear_btn = clear_btn.tag(tag);
            }
            if row.add(clear_btn).clicked() && !custom_swatches.is_empty() {
                custom_swatches.clear();
                cleared = true;
                row.tree.event_state.clear_frame_clicks();
            }
        });

        (added, cleared)
    }

    fn render_palette_swatches_grid(
        &mut self,
        config: &PropertyPaletteConfig<'_>,
        custom_swatches: &mut Vec<[f32; 4]>,
    ) -> (Option<Color>, Option<usize>) {
        let mut selected_color = None;
        let mut to_remove = None;

        let grid_style = Style::new()
            .flex_row()
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::Center)
            .gap(config.swatch_gap);

        self.container_named("PaletteSwatchesGrid", grid_style, |wrap| {
            for (idx, &col) in config.default_swatches.iter().enumerate() {
                let mut swatch = ColorSwatch::new(col)
                    .size(config.swatch_size, config.swatch_size)
                    .corner_radius(3.0);
                if let Some(base) = config.swatch_tag_base {
                    swatch = swatch.tag(base + idx as u64);
                } else {
                    swatch = swatch.tag(crate::declarative::types::combine_seeds(
                        wrap.seed, idx as u64,
                    ));
                }
                if wrap.add(swatch).clicked() {
                    selected_color = Some(col);
                }
            }

            let offset = config.default_swatches.len();
            for (custom_idx, arr) in custom_swatches.iter().enumerate() {
                let col = Color::from(*arr);
                let mut swatch = ColorSwatch::new(col)
                    .size(config.swatch_size, config.swatch_size)
                    .corner_radius(3.0);
                if let Some(base) = config.swatch_tag_base {
                    swatch = swatch.tag(base + (offset + custom_idx) as u64);
                } else {
                    swatch = swatch.tag(crate::declarative::types::combine_seeds(
                        wrap.seed,
                        (offset + custom_idx) as u64,
                    ));
                }
                let resp = wrap.add(swatch);
                if resp.secondary_clicked() {
                    to_remove = Some(custom_idx);
                } else if resp.clicked() {
                    selected_color = Some(col);
                }
            }
        });

        let mut removed_index = None;
        if let Some(idx) = to_remove
            && idx < custom_swatches.len()
        {
            custom_swatches.remove(idx);
            removed_index = Some(idx);
        }

        (selected_color, removed_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::WidgetId;
    use iris_core::tree::UiTree;

    #[test]
    fn test_property_palette_config_builder() {
        let defaults = [Color::RED, Color::GREEN];
        let cfg = PropertyPaletteConfig::new("TestPalette", Color::BLUE, &defaults)
            .with_tags(101, 102, 200)
            .max_custom(10)
            .swatch_size(20.0);

        assert_eq!(cfg.label, "TestPalette");
        assert_eq!(cfg.current_color, Color::BLUE);
        assert_eq!(cfg.max_custom, 10);
        assert_eq!(cfg.swatch_size, 20.0);
        assert_eq!(cfg.add_tag, Some(101));
        assert_eq!(cfg.clear_tag, Some(102));
        assert_eq!(cfg.swatch_tag_base, Some(200));
    }

    #[test]
    fn test_property_palette_response_handlers() {
        let mut selected = None;
        let mut add_called = false;
        let mut clear_called = false;
        let mut removed = None;

        let resp = PropertyPaletteResponse::new(Some(Color::YELLOW), true, true, Some(3))
            .on_select(|c| selected = Some(c))
            .on_add(|| add_called = true)
            .on_clear(|| clear_called = true)
            .on_remove(|idx| removed = Some(idx));

        assert_eq!(selected, Some(Color::YELLOW));
        assert!(add_called);
        assert!(clear_called);
        assert_eq!(removed, Some(3));
        assert!(resp.added);
        assert!(resp.cleared);
    }

    #[test]
    fn test_prop_color_palette_renders_and_tags_nodes() {
        let mut tree = UiTree::new();
        let defaults = [Color::WHITE, Color::BLACK];
        let mut custom = vec![[1.0, 0.0, 0.0, 1.0]];

        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        let cfg = PropertyPaletteConfig::new("Palette:", Color::GREEN, &defaults)
            .with_tags(5001, 5002, 6000);

        let resp = scope.prop_color_palette_with_config(cfg, &mut custom);
        assert!(!resp.added);
        assert!(!resp.cleared);
        assert_eq!(resp.selected_color, None);

        // Verify tagged nodes exist in tree
        assert!(tree.iter().any(|(_, n)| n.tag == 5001));
        assert!(tree.iter().any(|(_, n)| n.tag == 5002));
        assert!(tree.iter().any(|(_, n)| n.tag == 6000));
        assert!(tree.iter().any(|(_, n)| n.tag == 6001));
        assert!(tree.iter().any(|(_, n)| n.tag == 6002));
    }
}