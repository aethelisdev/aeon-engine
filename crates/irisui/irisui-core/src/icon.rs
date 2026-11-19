// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unified Declarative UI Icon Representation (`UiIcon`)
//!
//! Unifies text/unicode glyphs and GPU texture atlas UV rectangles under a single
//! `Copy` algebraic enum so tabs, menus, cards, and widgets share one icon contract.

/// Unified icon representation supporting either a static text/unicode glyph or a GPU texture atlas quad.
///
/// Replaces split `icon(&str)` and `atlas_icon([f32; 4])` methods across dock tabs, panel metadata,
/// menu items, and declarative widgets.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UiIcon {
    /// Static unicode character or vector icon glyph identifier (e.g. `"⚙"`, `"🔍"`).
    Glyph(&'static str),
    /// Normalized GPU texture atlas UV rectangle `[u_min, v_min, u_max, v_max]`.
    Atlas([f32; 4]),
}

impl UiIcon {
    /// Constructs a unicode/glyph icon variant.
    #[inline]
    #[must_use]
    pub const fn glyph(glyph: &'static str) -> Self {
        Self::Glyph(glyph)
    }

    /// Constructs a GPU texture atlas UV icon variant.
    #[inline]
    #[must_use]
    pub const fn atlas(uv: [f32; 4]) -> Self {
        Self::Atlas(uv)
    }

    /// Returns the static glyph slice when this icon is [`UiIcon::Glyph`].
    #[inline]
    #[must_use]
    pub const fn as_glyph(self) -> Option<&'static str> {
        match self {
            Self::Glyph(glyph) => Some(glyph),
            Self::Atlas(_) => None,
        }
    }

    /// Returns the GPU texture atlas UV rectangle when this icon is [`UiIcon::Atlas`].
    #[inline]
    #[must_use]
    pub const fn as_atlas(self) -> Option<[f32; 4]> {
        match self {
            Self::Glyph(_) => None,
            Self::Atlas(uv) => Some(uv),
        }
    }

    /// Returns `true` when this icon uses GPU texture atlas UV coordinates.
    #[inline]
    #[must_use]
    pub const fn is_atlas(self) -> bool {
        matches!(self, Self::Atlas(_))
    }
}

impl From<&'static str> for UiIcon {
    #[inline]
    fn from(glyph: &'static str) -> Self {
        Self::Glyph(glyph)
    }
}

impl From<[f32; 4]> for UiIcon {
    #[inline]
    fn from(uv: [f32; 4]) -> Self {
        Self::Atlas(uv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_icon_variants_and_accessors() {
        let g = UiIcon::glyph("⚙");
        let a = UiIcon::atlas([0.0, 0.25, 0.25, 0.5]);

        assert_eq!(g.as_glyph(), Some("⚙"));
        assert_eq!(g.as_atlas(), None);
        assert!(!g.is_atlas());

        assert_eq!(a.as_glyph(), None);
        assert_eq!(a.as_atlas(), Some([0.0, 0.25, 0.25, 0.5]));
        assert!(a.is_atlas());

        assert_eq!(UiIcon::from("📁"), UiIcon::Glyph("📁"));
        assert_eq!(
            UiIcon::from([1.0, 0.0, 0.5, 0.5]),
            UiIcon::Atlas([1.0, 0.0, 0.5, 0.5])
        );
    }
}