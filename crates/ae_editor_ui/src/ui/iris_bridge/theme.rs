// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Surface elevation, design tokens, and workspace theme presets for Iris UI.
//!
//! Enforces a strict 3-tier elevation hierarchy:
//! - Level 0 (Canvas Void & Splitter Gaps): Deepest background tone.
//! - Level 1 (Panel Bodies): Base dark anthracite for inspector, hierarchy, etc.
//! - Level 2 (Tab Strip & Inactive Tabs): Receding surface matching or darker than panel body.
//! - Level 3 (Active Tab & Focused Pane): Lightest surface layer anchored with a 2px accent underline.

use irisui::prelude::{Color, Theme};

/// Helper converting 8-bit RGBA integer channels to a compile-time constant [`Color`].
#[inline]
#[must_use]
pub const fn hex_color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

/// Predefined color elevation presets for Aeon Studio editor workspaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    /// High-contrast near-black workspace emphasizing electric cyan accents.
    #[default]
    ObsidianDeep,
    /// Low-fatigue slate grey workspace modeled after industry-standard digital content creation tools.
    ColdSlate,
}

impl ThemePreset {
    /// Resolves this preset into its concrete strongly-typed semantic tokens.
    #[inline]
    #[must_use]
    pub fn tokens(self) -> EditorThemeTokens {
        match self {
            Self::ObsidianDeep => EditorThemeTokens::obsidian_deep(),
            Self::ColdSlate => EditorThemeTokens::cold_slate(),
        }
    }
}

/// Strongly-typed semantic design tokens for Aeon Studio editor workspaces.
///
/// Implements the hierarchical elevation model:
/// - Background canvas / gaps: deepest tone.
/// - Panel bodies: dark anthracite baseline.
/// - Inactive tabs / headers: integrated receding tone.
/// - Active tab / focus: elevated light tone with 2px accent underline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorThemeTokens {
    /// Deepest canvas void behind docked panes and panel gaps.
    pub canvas_bg: Color,
    /// Base background color for docked panels (Inspector, Hierarchy, etc.).
    pub panel_bg: Color,
    /// Elevated background color for inner card containers (Inspector, empty notices, etc.).
    pub card_bg: Color,
    /// Subtle outline border color for inner card containers.
    pub card_border: Color,
    /// Header background color for inner card container title strips.
    pub card_header_bg: Color,
    /// Receding background color for tab strip headers and inactive tabs.
    pub tab_bar_bg: Color,
    /// Inactive tab pill background color.
    pub tab_idle_bg: Color,
    /// Elevated background color for the currently active tab.
    pub tab_active_bg: Color,
    /// Hovered tab pill background color.
    pub tab_hover_bg: Color,
    /// Idle divider / splitter line stroke color.
    pub splitter_idle: Color,
    /// Dragged or focused active splitter glow color.
    pub splitter_active: Color,
    /// 1px hairline border color between adjacent surfaces.
    pub border_micron: Color,
    /// Elevated border color for dialog popups and floating cards.
    pub border_elevated: Color,
    /// Primary 2px active tab accent line and focal indicator color (`#00D4B2`).
    pub accent: Color,
    /// Secondary accent glow for subtle shadows.
    pub accent_glow: Color,
    /// High-contrast bright text color for active headings and labels (`#ECEFF4` / `#E4E7EB`).
    pub text_active: Color,
    /// Standard body text color for panel content.
    pub text_regular: Color,
    /// Dimmed text color for inactive tabs and hints (`#70757F` / `#636B78`).
    pub text_muted: Color,
    /// Floating window, modal, and dropdown popup background.
    pub popup_bg: Color,
}

impl Default for EditorThemeTokens {
    #[inline]
    fn default() -> Self {
        Self::obsidian_deep()
    }
}

impl EditorThemeTokens {
    /// Palette 1: "Obsidian Deep"
    ///
    /// Near-black high contrast workspace emphasizing Aeon's electric cyan accent line.
    ///
    /// - Canvas / Gaps: `#08080A`
    /// - Panel Bodies: `#111215`
    /// - Cards: `#16181D` (border `#22242A`)
    /// - Card Headers / Toggles: `#242831` (elevated slate grey)
    /// - Active Tab: `#1A1C20` + `#00D4B2` 2px line
    /// - Inactive Tab / Header: `#0D0E10`
    /// - Splitter / 1px Border: `#22242A`
    /// - Active Text: `#ECEFF4`
    /// - Inactive Text: `#70757F`
    #[must_use]
    pub const fn obsidian_deep() -> Self {
        Self {
            canvas_bg: hex_color(8, 8, 10, 255),
            panel_bg: hex_color(17, 18, 21, 255),
            card_bg: hex_color(22, 24, 29, 255),
            card_border: hex_color(34, 36, 42, 255),
            card_header_bg: hex_color(36, 40, 49, 255),
            tab_bar_bg: hex_color(13, 14, 16, 255),
            tab_idle_bg: hex_color(13, 14, 16, 255),
            tab_active_bg: hex_color(26, 28, 32, 255),
            tab_hover_bg: hex_color(22, 24, 29, 255),
            splitter_idle: hex_color(34, 36, 42, 255),
            splitter_active: hex_color(0, 212, 178, 255),
            border_micron: hex_color(34, 36, 42, 255),
            border_elevated: hex_color(42, 45, 54, 255),
            accent: hex_color(0, 212, 178, 255),
            accent_glow: Color::rgba(0.0, 0.831, 0.698, 0.35),
            text_active: hex_color(236, 239, 244, 255),
            text_regular: hex_color(216, 222, 233, 255),
            text_muted: hex_color(112, 117, 127, 255),
            popup_bg: hex_color(19, 20, 24, 250),
        }
    }

    /// Palette 2: "Cold Slate"
    ///
    /// Subtle cold-tinted slate grey palette optimized for long work sessions.
    ///
    /// - Canvas / Gaps: `#0D0F12`
    /// - Panel Bodies: `#15181E`
    /// - Cards: `#1B1E26` (border `#282D37`)
    /// - Card Headers / Toggles: `#2A303C` (elevated cool slate grey)
    /// - Active Tab: `#1F232B` + `#00D4B2` 2px line
    /// - Inactive Tab / Header: `#111317`
    /// - Splitter / Border: `#282D37`
    /// - Active Text: `#E4E7EB`
    /// - Inactive Text: `#636B78`
    #[must_use]
    pub const fn cold_slate() -> Self {
        Self {
            canvas_bg: hex_color(13, 15, 18, 255),
            panel_bg: hex_color(21, 24, 30, 255),
            card_bg: hex_color(27, 30, 38, 255),
            card_border: hex_color(40, 45, 55, 255),
            card_header_bg: hex_color(42, 48, 60, 255),
            tab_bar_bg: hex_color(17, 19, 23, 255),
            tab_idle_bg: hex_color(17, 19, 23, 255),
            tab_active_bg: hex_color(31, 35, 43, 255),
            tab_hover_bg: hex_color(27, 30, 38, 255),
            splitter_idle: hex_color(40, 45, 55, 255),
            splitter_active: hex_color(0, 212, 178, 255),
            border_micron: hex_color(40, 45, 55, 255),
            border_elevated: hex_color(50, 56, 68, 255),
            accent: hex_color(0, 212, 178, 255),
            accent_glow: Color::rgba(0.0, 0.831, 0.698, 0.35),
            text_active: hex_color(228, 231, 235, 255),
            text_regular: hex_color(200, 205, 215, 255),
            text_muted: hex_color(99, 107, 120, 255),
            popup_bg: hex_color(25, 29, 37, 250),
        }
    }

    /// Converts these editor tokens into an [`irisui::prelude::Theme`] for inner widget styling.
    #[must_use]
    pub fn to_iris_theme(&self) -> Theme {
        Theme {
            panel_bg: self.panel_bg,
            panel_border: self.border_micron,
            card_bg: self.card_bg,
            card_border: self.card_border,
            card_header_bg: self.card_header_bg,
            toolbar_bg: self.tab_bar_bg,
            toolbar_border: self.border_micron,
            modal_scrim: Color::rgba(0.0, 0.0, 0.0, 0.70),
            modal_bg: self.popup_bg,
            modal_border: self.border_elevated,
            button_idle_bg: Color::rgba(0.12, 0.13, 0.16, 0.92),
            button_hover_bg: Color::rgba(0.20, 0.23, 0.30, 0.95),
            button_active_bg: Color::rgba(0.06, 0.46, 0.92, 1.0),
            button_border: Color::rgba(0.24, 0.26, 0.32, 0.85),
            button_hover_border: Color::rgba(0.25, 0.60, 1.0, 0.95),
            text_primary: self.text_active,
            text_secondary: self.text_regular,
            text_muted: self.text_muted,
            text_caret: self.accent,
            accent: self.accent,
            accent_hover: self.accent,
            success: Color::rgba(0.20, 0.80, 0.45, 1.0),
            warning: Color::rgba(0.95, 0.75, 0.20, 1.0),
            danger: Color::rgba(0.95, 0.28, 0.30, 1.0),
            info: Color::rgba(0.25, 0.60, 1.0, 1.0),
            divider: self.splitter_idle,
            scrollbar_track: self.canvas_bg,
            scrollbar_thumb: self.splitter_idle,
            scrollbar_thumb_hover: self.border_elevated,
            corner_radius_sm: 3.0,
            corner_radius_md: 4.0,
            corner_radius_lg: 8.0,
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Static Constants (Obsidian Deep Default Compatibility)
// -------------------------------------------------------------------------------------------------

/// Level 0: Ultra-deep obsidian canvas void behind docked workspaces (`#08080A`).
pub const ELEVATION_0_CANVAS: Color = hex_color(8, 8, 10, 255);

/// Level 1: Dock panel and workspace content area background (`#111215`).
pub const ELEVATION_1_PANEL: Color = hex_color(17, 18, 21, 255);

/// Level 2: Tab strip headers, menubar, and status bar surface (`#0D0E10`).
pub const ELEVATION_2_HEADER: Color = hex_color(13, 14, 16, 255);

/// Level 3: Inactive tab pills, inner cards, and input backgrounds (`#0D0E10`).
pub const ELEVATION_3_INACTIVE_PILL: Color = hex_color(13, 14, 16, 255);

/// Hovered tab pill or card background (`#16181D`).
pub const ELEVATION_3_HOVERED_PILL: Color = hex_color(22, 24, 29, 255);

/// Active tab pill background elevated above panel body (`#1A1C20`).
pub const ELEVATION_3_ACTIVE_PILL: Color = hex_color(26, 28, 32, 255);

/// Level 4: Floating windows, modals, and dropdown popups.
pub const ELEVATION_4_POPUP: Color = hex_color(19, 20, 24, 250);

/// Subtle dark hairline border for panel frames and division lines (`#22242A`).
pub const BORDER_MICRON: Color = hex_color(34, 36, 42, 255);

/// Subtle dark border for elevated popup dialogs and floating cards.
pub const BORDER_ELEVATED: Color = hex_color(42, 45, 54, 255);

/// Defined dark splitter slit color providing clear panel boundaries (`#22242A`).
pub const SPLITTER_IDLE: Color = hex_color(34, 36, 42, 255);

/// Focused or dragged active splitter glow (`#00D4B2`).
pub const SPLITTER_ACTIVE: Color = hex_color(0, 212, 178, 255);

/// Primary electric cyan accent color for active indicators, highlights, and borders (`#00D4B2`).
pub const ACCENT_CYAN: Color = hex_color(0, 212, 178, 255);

/// Secondary cyan glow for soft shadows.
pub const ACCENT_CYAN_GLOW: Color = Color::rgba(0.0, 0.831, 0.698, 0.35);

/// Subtle muted text color for inactive headers and hints (`#70757F`).
pub const TEXT_MUTED: Color = hex_color(112, 117, 127, 255);

/// Standard body text color for panel content.
pub const TEXT_REGULAR: Color = hex_color(216, 222, 233, 255);

/// High-contrast bright text color for active titles and highlights (`#ECEFF4`).
pub const TEXT_BRIGHT: Color = hex_color(236, 239, 244, 255);

/// Returns the authoritative surface elevation color for a specific hierarchical layer index (0..=4).
#[inline]
#[must_use]
pub fn elevation_for_level(level: usize) -> Color {
    match level {
        0 => ELEVATION_0_CANVAS,
        1 => ELEVATION_1_PANEL,
        2 => ELEVATION_2_HEADER,
        3 => ELEVATION_3_INACTIVE_PILL,
        _ => ELEVATION_4_POPUP,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_preset_defaults_and_resolution() {
        assert_eq!(ThemePreset::default(), ThemePreset::ObsidianDeep);
        let default_tokens = ThemePreset::default().tokens();
        assert_eq!(default_tokens, EditorThemeTokens::obsidian_deep());
    }

    #[test]
    fn test_obsidian_deep_elevation_invariants() {
        let t = EditorThemeTokens::obsidian_deep();
        // Verify exact hex mappings
        assert_eq!(t.canvas_bg, hex_color(8, 8, 10, 255));
        assert_eq!(t.panel_bg, hex_color(17, 18, 21, 255));
        assert_eq!(t.card_bg, hex_color(22, 24, 29, 255));
        assert_eq!(t.card_border, hex_color(34, 36, 42, 255));
        assert_eq!(t.card_header_bg, hex_color(36, 40, 49, 255));
        assert_eq!(t.tab_idle_bg, hex_color(13, 14, 16, 255));
        assert_eq!(t.tab_active_bg, hex_color(26, 28, 32, 255));
        assert_eq!(t.splitter_idle, hex_color(34, 36, 42, 255));
        assert_eq!(t.accent, hex_color(0, 212, 178, 255));
        assert_eq!(t.text_active, hex_color(236, 239, 244, 255));
        assert_eq!(t.text_muted, hex_color(112, 117, 127, 255));

        // Elevation luminance invariant: canvas < panel < card < card_header
        assert!(t.canvas_bg.r < t.panel_bg.r);
        assert!(t.panel_bg.r < t.card_bg.r);
        assert!(t.card_bg.r < t.card_header_bg.r);
        assert!(t.card_bg.r < t.tab_active_bg.r);
        // Passive tab is darker than panel body (receding)
        assert!(t.tab_idle_bg.r < t.panel_bg.r);
    }

    #[test]
    fn test_cold_slate_elevation_invariants() {
        let t = EditorThemeTokens::cold_slate();
        // Verify exact hex mappings
        assert_eq!(t.canvas_bg, hex_color(13, 15, 18, 255));
        assert_eq!(t.panel_bg, hex_color(21, 24, 30, 255));
        assert_eq!(t.card_bg, hex_color(27, 30, 38, 255));
        assert_eq!(t.card_border, hex_color(40, 45, 55, 255));
        assert_eq!(t.card_header_bg, hex_color(42, 48, 60, 255));
        assert_eq!(t.tab_idle_bg, hex_color(17, 19, 23, 255));
        assert_eq!(t.tab_active_bg, hex_color(31, 35, 43, 255));
        assert_eq!(t.splitter_idle, hex_color(40, 45, 55, 255));
        assert_eq!(t.accent, hex_color(0, 212, 178, 255));
        assert_eq!(t.text_active, hex_color(228, 231, 235, 255));
        assert_eq!(t.text_muted, hex_color(99, 107, 120, 255));

        // Elevation luminance invariant: canvas < panel < card < card_header
        assert!(t.canvas_bg.r < t.panel_bg.r);
        assert!(t.panel_bg.r < t.card_bg.r);
        assert!(t.card_bg.r < t.card_header_bg.r);
        assert!(t.card_bg.r < t.tab_active_bg.r);
        // Passive tab is darker than panel body (receding)
        assert!(t.tab_idle_bg.r < t.panel_bg.r);
    }

    #[test]
    fn test_iris_theme_generation() {
        let tokens = EditorThemeTokens::obsidian_deep();
        let iris_theme = tokens.to_iris_theme();
        assert_eq!(iris_theme.panel_bg, tokens.panel_bg);
        assert_eq!(iris_theme.card_bg, tokens.card_bg);
        assert_eq!(iris_theme.card_border, tokens.card_border);
        assert_eq!(iris_theme.card_header_bg, tokens.card_header_bg);
        assert_eq!(iris_theme.accent, tokens.accent);
        assert_eq!(iris_theme.text_primary, tokens.text_active);
        assert_eq!(iris_theme.text_muted, tokens.text_muted);
    }
}