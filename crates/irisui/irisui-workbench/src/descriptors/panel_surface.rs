// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Surface Styling & Container Wrapping
//!
//! Provides [`PanelSurfaceStyle`] alongside [`PanelBackground`] and [`PanelBorder`] so
//! workbench panels declare their root container name, semantic tag, flex direction,
//! background fill, border stroke, and scroll policy directly in metadata instead of
//! wrapping every panel function with boilerplate `scope.panel(...)` containers.

use irisui_core::{Color, FlexDirection};
use irisui_widgets::UiScope;
use irisui_widgets::declarative::scope::InnerResponse;

/// Background fill policy for a workbench panel root container.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PanelBackground {
    /// Resolves to the active [`irisui_core::Theme::panel_bg`] color token.
    #[default]
    ThemePanelBg,
    /// Resolves to the active [`irisui_core::Theme::card_bg`] color token.
    ThemeCardBg,
    /// Uses an explicit RGBA color independent of theme tokens.
    Color(Color),
}

/// Border stroke policy for a workbench panel root container.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PanelBorder {
    /// Emits a `1.0` px uniform border tinted by [`irisui_core::Theme::panel_border`].
    #[default]
    ThemeDefault,
    /// Emits an explicit uniform border width tinted by [`irisui_core::Theme::panel_border`].
    ThemeWidth(f32),
    /// Emits an explicit uniform border width and color.
    Custom(f32, Color),
    /// Omits the outer panel border stroke.
    None,
}

/// Declarative root container descriptor automatically wrapping a workbench panel's content.
///
/// Attached via [`crate::descriptors::PanelMetadata::with_surface`] so workbench leaf draw
/// targets instantiate the root container, assign debug names and interaction tags, apply
/// themed or custom surfaces, and keep individual panel modules focused solely on inner widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelSurfaceStyle {
    /// Debug node identifier assigned to the generated panel root node.
    pub root_name: &'static str,
    /// Optional 64-bit semantic interaction tag assigned to the root node (`0` when untagged).
    pub root_tag: u64,
    /// Flexbox axis applied to direct children inside the panel root container.
    pub direction: FlexDirection,
    /// Background fill rule applied to the root container.
    pub background: PanelBackground,
    /// Border stroke rule applied to the root container.
    pub border: PanelBorder,
    /// Enables autonomous vertical scrolling when child content overflows panel height.
    pub auto_scroll: bool,
}

impl PanelSurfaceStyle {
    /// Creates a column-oriented [`PanelSurfaceStyle`] using theme defaults and no semantic tag.
    #[inline]
    #[must_use]
    pub const fn new(root_name: &'static str) -> Self {
        Self {
            root_name,
            root_tag: 0,
            direction: FlexDirection::Column,
            background: PanelBackground::ThemePanelBg,
            border: PanelBorder::ThemeDefault,
            auto_scroll: false,
        }
    }

    /// Assigns a 64-bit semantic interaction tag to the generated panel root node.
    #[inline]
    #[must_use]
    pub const fn with_tag(mut self, tag: u64) -> Self {
        self.root_tag = tag;
        self
    }

    /// Configures the flex axis applied to direct children inside the panel container.
    #[inline]
    #[must_use]
    pub const fn with_direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Configures the panel container to arrange direct children horizontally (`FlexDirection::Row`).
    #[inline]
    #[must_use]
    pub const fn row(mut self) -> Self {
        self.direction = FlexDirection::Row;
        self
    }

    /// Configures the background fill policy for the panel root container.
    #[inline]
    #[must_use]
    pub const fn with_background(mut self, background: PanelBackground) -> Self {
        self.background = background;
        self
    }

    /// Configures the border stroke policy for the panel root container.
    #[inline]
    #[must_use]
    pub const fn with_border(mut self, border: PanelBorder) -> Self {
        self.border = border;
        self
    }

    /// Configures whether the panel root container attaches an autonomous vertical scrollbar.
    #[inline]
    #[must_use]
    pub const fn with_auto_scroll(mut self, enabled: bool) -> Self {
        self.auto_scroll = enabled;
        self
    }

    /// Materializes the configured panel root container inside `scope` and executes `render`.
    ///
    /// # Arguments
    /// * `scope` - Parent declarative UI scope owning the panel slot or dock leaf.
    /// * `render` - Inner panel view builder closure receiving the styled panel scope.
    pub fn show<R>(
        &self,
        scope: &mut UiScope<'_>,
        render: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> InnerResponse<R> {
        let mut builder = scope
            .panel(self.root_name)
            .tagged(self.root_tag)
            .direction(self.direction)
            .scrollable(self.auto_scroll);

        builder = match self.background {
            PanelBackground::ThemePanelBg => builder,
            PanelBackground::ThemeCardBg => builder.background_themed(|t| t.card_bg),
            PanelBackground::Color(color) => builder.background(color),
        };

        builder = match self.border {
            PanelBorder::ThemeDefault => builder,
            PanelBorder::ThemeWidth(width) => builder.border_themed(width, |t| t.panel_border),
            PanelBorder::Custom(width, color) => builder.border(width, color),
            PanelBorder::None => builder.without_border(),
        };

        builder.show(render)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::{Border, UiTree};

    #[test]
    fn test_panel_surface_style_applies_row_custom_bg_and_no_border() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let custom_bg = Color::rgba(0.11, 0.12, 0.13, 1.0);
        let style = PanelSurfaceStyle::new("SurfaceTestRoot")
            .with_tag(0xDEAD_BEEF)
            .row()
            .with_background(PanelBackground::Color(custom_bg))
            .with_border(PanelBorder::None);

        let node_id = {
            let mut scope = UiScope::new(&mut tree, root);
            style.show(&mut scope, |_| ()).response.id
        };

        let node = tree.get(node_id).expect("Panel root exists");
        assert_eq!(node.name.as_deref(), Some("SurfaceTestRoot"));
        assert_eq!(node.tag, 0xDEAD_BEEF);
        assert_eq!(node.style.flex_direction, FlexDirection::Row);
        assert_eq!(node.style.background_color, custom_bg);
        assert_eq!(node.style.border.width, irisui_core::Insets::all(0.0));
    }

    #[test]
    fn test_panel_surface_style_applies_custom_border_and_card_bg() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let border_color = Color::rgba(0.2, 0.3, 0.4, 1.0);
        let style = PanelSurfaceStyle::new("CardSurfaceRoot")
            .with_background(PanelBackground::ThemeCardBg)
            .with_border(PanelBorder::Custom(2.5, border_color));

        let node_id = {
            let mut scope = UiScope::new(&mut tree, root);
            style.show(&mut scope, |_| ()).response.id
        };

        let node = tree.get(node_id).expect("Panel root exists");
        assert_eq!(node.style.background_color, tree.theme().card_bg);
        assert_eq!(node.style.border, Border::uniform(2.5, border_color));
    }
}