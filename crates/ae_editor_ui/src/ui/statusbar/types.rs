// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Status Bar Types and Sizing Constants
//!
//! Provides layout sizing parameters, diagnostic metrics payload structures,
//! and display configurations for the bottom application status bar.
//!

use irisui::prelude::Color;

/// Canonical height of the bottom application status bar in logical pixels.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// State parameters required to construct the bottom status bar matching editor specifications.
#[derive(Debug, Clone)]
pub struct StatusBarParams<'a> {
    /// Screen surface width in logical display pixels.
    pub screen_width: f32,
    /// Screen surface height in logical display pixels.
    pub screen_height: f32,
    /// Active temporary status message spans with their associated colors.
    pub status_spans: Option<&'a [(String, Color)]>,
    /// Optional version text override. When None, defaults to `Aeon Engine v{CARGO_PKG_VERSION}`.
    pub version_text: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_params_construction() {
        let spans = vec![("Ready".to_string(), Color::WHITE)];
        let params = StatusBarParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            status_spans: Some(&spans),
            version_text: Some("Aeon Engine v0.9.0"),
        };

        assert_eq!(params.screen_width, 1920.0);
        assert_eq!(params.screen_height, 1080.0);
        assert_eq!(STATUS_BAR_HEIGHT, 22.0);
        assert_eq!(params.version_text, Some("Aeon Engine v0.9.0"));
    }
}