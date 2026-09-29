// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Option Tables
//!
//! Option constants and descriptors for the Graphics settings tab.

use ae_renderer::graphics_settings::{FpsLimit, PcfQuality, ShadowResolution, SkyQuality};

/// Pre-configured shadow resolution options.
pub const SHADOW_RES_OPTIONS: [ShadowResolution; 4] = [
    ShadowResolution::Low,
    ShadowResolution::Medium,
    ShadowResolution::High,
    ShadowResolution::Ultra,
];

/// Pre-configured shadow cascade count options.
pub const CASCADE_OPTIONS: [(u32, &str); 2] = [
    (3, "3 Cascades (Default)"),
    (4, "4 Cascades (High Fidelity)"),
];

/// Pre-configured PCF shadow filter options.
pub const PCF_OPTIONS: [PcfQuality; 3] = [PcfQuality::Off, PcfQuality::Soft, PcfQuality::UltraSoft];

/// Pre-configured framerate limiter options.
pub const FPS_OPTIONS: [FpsLimit; 3] = [FpsLimit::Limit60, FpsLimit::Limit120, FpsLimit::Uncapped];

/// Pre-configured hardware MSAA sample options.
pub const MSAA_OPTIONS: [(u32, &str); 3] = [(1, "Off (1x)"), (2, "2x"), (4, "4x (Default)")];

/// Pre-configured atmospheric sky quality options.
pub const SKY_OPTIONS: [SkyQuality; 3] = [SkyQuality::Low, SkyQuality::Medium, SkyQuality::High];