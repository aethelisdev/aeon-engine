// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Surface elevation and micron-border design tokens for Iris UI.
//!
//! Provides mathematically harmonious dark-mode elevation layers (Levels 0–4),
//! subtle translucent 1px micron-borders, and high-contrast accent tokens.
//!

use irisui::prelude::Color;

/// Level 0: Ultra-deep obsidian canvas void behind docked workspaces.
pub const ELEVATION_0_CANVAS: Color = Color::rgba(0.028, 0.031, 0.043, 1.0);

/// Level 1: Dock panel and workspace content area background.
pub const ELEVATION_1_PANEL: Color = Color::rgba(0.047, 0.052, 0.068, 1.0);

/// Level 2: Tab strip headers, menubar, and status bar surface.
pub const ELEVATION_2_HEADER: Color = Color::rgba(0.068, 0.076, 0.100, 1.0);

/// Level 3: Inactive tab pills, inner cards, and input backgrounds.
pub const ELEVATION_3_INACTIVE_PILL: Color = Color::rgba(0.078, 0.088, 0.118, 0.85);

/// Hovered tab pill or card background.
pub const ELEVATION_3_HOVERED_PILL: Color = Color::rgba(0.105, 0.120, 0.160, 1.0);

/// Active tab pill background  matching panel level 1.
pub const ELEVATION_3_ACTIVE_PILL: Color = Color::rgba(0.047, 0.052, 0.068, 1.0);

/// Level 4: Floating windows, modals, and dropdown popups.
pub const ELEVATION_4_POPUP: Color = Color::rgba(0.075, 0.086, 0.133, 0.98);

/// Subtle dark slate hairline border for panel frames and division lines.
pub const BORDER_MICRON: Color = Color::rgba(0.18, 0.21, 0.28, 0.85);

/// Subtle dark slate border for elevated popup dialogs and floating cards.
pub const BORDER_ELEVATED: Color = Color::rgba(0.22, 0.25, 0.34, 0.90);

/// Defined dark slate splitter slit color providing clear panel boundaries.
pub const SPLITTER_IDLE: Color = Color::rgba(0.18, 0.21, 0.28, 1.0);

/// Focused or dragged active splitter glow.
pub const SPLITTER_ACTIVE: Color = Color::rgba(0.0, 0.898, 1.0, 0.95);

/// Primary electric cyan accent color for active indicators, highlights, and borders.
pub const ACCENT_CYAN: Color = Color::rgba(0.0, 0.898, 1.0, 1.0);

/// Secondary cyan glow for soft shadows.
pub const ACCENT_CYAN_GLOW: Color = Color::rgba(0.0, 0.898, 1.0, 0.35);

/// Subtle muted text color for inactive headers and hints.
pub const TEXT_MUTED: Color = Color::rgba(0.55, 0.58, 0.68, 1.0);

/// Standard body text color for panel content.
pub const TEXT_REGULAR: Color = Color::rgba(0.85, 0.88, 0.95, 1.0);

/// High-contrast bright text color for active titles and highlights.
pub const TEXT_BRIGHT: Color = Color::rgba(0.98, 0.99, 1.0, 1.0);

/// Returns the authoritative surface elevation color for a specific hierarchical layer index (0..=4).
#[inline]
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
    fn test_elevation_layer_progression() {
        let l0 = elevation_for_level(0);
        let l1 = elevation_for_level(1);
        let l2 = elevation_for_level(2);
        let l3 = elevation_for_level(3);
        let l4 = elevation_for_level(4);

        assert!(l0.r < l1.r);
        assert!(l1.r < l2.r);
        assert_eq!(l1, ELEVATION_3_ACTIVE_PILL);
        assert_eq!(l3, ELEVATION_3_INACTIVE_PILL);
        assert_eq!(l4, ELEVATION_4_POPUP);
    }

    #[test]
    fn test_border_and_splitter_invariants() {
        let micron = BORDER_MICRON;
        let elevated = BORDER_ELEVATED;
        let idle = SPLITTER_IDLE;
        let active = SPLITTER_ACTIVE;

        assert!(micron.a > 0.50 && micron.a <= 0.95);
        assert!(elevated.a >= micron.a);
        assert!(idle.a > 0.50);
        assert!(active.b > 0.90);
    }

    #[test]
    fn test_accent_colors_and_text_contrast() {
        let cyan = ACCENT_CYAN;
        let bright = TEXT_BRIGHT;
        let regular = TEXT_REGULAR;
        let muted = TEXT_MUTED;

        assert!(cyan.g > 0.80 && cyan.b > 0.90);
        assert!(bright.r > regular.r);
        assert!(regular.r > muted.r);
    }
}