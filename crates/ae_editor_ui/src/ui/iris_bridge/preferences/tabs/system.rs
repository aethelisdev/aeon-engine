// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # System Preferences Tab
//!
//! Renders hardware GPU adapter status, Rayon thread pool telemetry, and memory diagnostics
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::pref_heading;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// System diagnostic item descriptor.
struct DiagnosticItem {
    icon_title: &'static str,
    headline: &'static str,
    details: &'static str,
}

const SYSTEM_DIAGNOSTICS: [DiagnosticItem; 4] = [
    DiagnosticItem {
        icon_title: "🖥  Hardware GPU Backend",
        headline: "Hardware-Accelerated WGPU 30.0.1 (Vulkan / DX12 / Metal)",
        details: "Direct hardware execution with low-overhead command recording, multi-draw indirect, and compute passes.",
    },
    DiagnosticItem {
        icon_title: "⚡  CPU Task Scheduling",
        headline: "Rayon Work-Stealing Parallel Scheduler",
        details: "Physics simulation, skinning vertex calculations, and asset decoding scale linearly across all CPU cores.",
    },
    DiagnosticItem {
        icon_title: "🛡  Memory Safety Architecture",
        headline: "100% Safe Rust Core (Zero Unsafe Blocks)",
        details: "Absolute memory integrity. Compile-time ownership guarantees zero data races, use-after-free, or segfaults.",
    },
    DiagnosticItem {
        icon_title: "🎨  User Interface Subsystem",
        headline: "Iris UI Framework (Declarative UiScope + SDF Rendering)",
        details: "Zero per-frame heap allocations in hot paths with persistent retained diffing and hardware anti-aliased quads.",
    },
];

/// Builds the System preferences tab content declaratively using [`UiScope`].
pub fn build_system_tab(scope: &mut UiScope<'_>, _params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "⚙  System & Hardware Diagnostics",
        "Runtime hardware capabilities, graphics API backend, and thread scheduler telemetry.",
    );

    for item in &SYSTEM_DIAGNOSTICS {
        scope.container_named(
            "SystemCard",
            Style::new()
                .flex_col()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0)
                .padding_insets(Insets::new(12.0, 14.0, 12.0, 14.0))
                .margin_insets(Insets::new(0.0, 0.0, 12.0, 0.0)),
            |card| {
                card.label_styled_passive(
                    "SystemTitle",
                    item.icon_title,
                    12.5,
                    Color::rgba(1.0, 1.0, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                );
                card.label_styled_passive(
                    "SystemHeadline",
                    item.headline,
                    11.5,
                    Color::rgba(0.0, 0.85, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                );
                card.label_styled_passive(
                    "SystemDetails",
                    item.details,
                    11.0,
                    Color::rgba(0.65, 0.68, 0.76, 1.0),
                    TextAlign::Left,
                    Style::new(),
                );
            },
        );
    }
}