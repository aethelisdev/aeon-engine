// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Information Preferences Tabs
//!
//! Renders hardware-accelerated configuration and information views for
//! Navigation (Tab 3), Keymap (Tab 4), System (Tab 5), Add-ons (Tab 6), Input (Tab 7),
//! and Experimental (Tab 8) declaratively using [`UiScope`].

use super::super::components::pref_heading;
use super::super::types::PreferencesParams;
use irisui::prelude::*;

/// Builds a generic information preferences tab with heading, separator, and informative cards.
pub fn build_info_tab(scope: &mut UiScope<'_>, tab_index: u8, _params: &PreferencesParams<'_>) {
    let (heading, subtitle, cards): (&str, &str, Vec<(&str, &str, &str)>) = match tab_index {
        3 => (
            "Navigation",
            "Orbit and viewport navigation settings.",
            vec![
                (
                    "🧭 Viewport Orbit Mode",
                    "Turntable (Z-up aligned)",
                    "Provides stable orbital rotations without roll deviation.",
                ),
                (
                    "🔍 Zoom Behavior",
                    "Smooth Exponential Zoom",
                    "Scroll wheel accelerates zoom smoothly based on distance to target.",
                ),
                (
                    "🚀 Camera Fly Speed",
                    "Base Speed: 1.0x (Boost: 3.0x via Shift)",
                    "Hold Right Mouse Button and use WASD to fly through the scene.",
                ),
            ],
        ),
        4 => (
            "Keymap",
            "Manage editor keyboard shortcuts.",
            vec![
                (
                    "🎮 Viewport Navigation",
                    "Right Mouse + WASD: Fly Cam | Alt + Left Mouse: Orbit | Middle Mouse: Pan | Scroll: Zoom",
                    "Desktop standard mouse navigation controls.",
                ),
                (
                    "📐 Transform Gizmos",
                    "W: Translate (Move) | E: Rotate | R: Scale | Q: World/Local Coordinate Space",
                    "Quick hotkeys for switching object manipulation tools.",
                ),
                (
                    "✨ Scene Operations",
                    "Ctrl+Z: Undo | Ctrl+Y: Redo | F: Focus Selected | Delete: Delete Entity",
                    "Essential scene editing shortcuts.",
                ),
            ],
        ),
        5 => (
            "System",
            "Hardware, device adapter, and memory diagnostics.",
            vec![
                (
                    "🖥 GPU Backend",
                    "Hardware-Accelerated WGPU 30.0.1 (Vulkan / DX12 / Metal)",
                    "Direct hardware access with modern low-overhead graphics pipeline.",
                ),
                (
                    "⚡ Multi-Threading",
                    "Rayon Parallel Task Scheduler",
                    "Physics, animation, and asset streaming run across all CPU cores.",
                ),
                (
                    "🛡 Memory Safety",
                    "100% Safe Rust Engine Core (Zero Unsafe Blocks)",
                    "Engine architecture guarantees zero memory corruption or data races.",
                ),
            ],
        ),
        6 => (
            "Add-ons",
            "Engine plugins, extensions, and asset importers.",
            vec![
                (
                    "📦 glTF 2.0 / GLB Importer",
                    "Integrated & Active (Built-in)",
                    "Full support for PBR materials, skinning, and embedded buffers.",
                ),
                (
                    "🎨 Iris UI Framework",
                    "Hardware SDF GPU Retained UI (100% Declarative UiScope)",
                    "Zero CPU overhead dynamic vector rendering pipeline.",
                ),
                (
                    "🔊 Kira Audio Engine",
                    "Procedural Dynamic Audio Synthesizer",
                    "Spatial 3D audio, streaming soundscapes, and low-latency DSP.",
                ),
            ],
        ),
        7 => (
            "Input",
            "Mouse, keyboard, and controller configuration.",
            vec![
                (
                    "🖱 Mouse Sensitivity",
                    "Look Sensitivity: 1.0 | Orbit Sensitivity: 1.0",
                    "Controls linear mouse cursor translation multiplier in 3D viewport.",
                ),
                (
                    "🔄 Invert Y-Axis",
                    "Disabled (Standard Pitch)",
                    "Camera pitch tilts naturally with mouse motion.",
                ),
            ],
        ),
        _ => (
            "Experimental",
            "Preview  engine features under active development.",
            vec![
                (
                    "⚡ GPU Occlusion Culling",
                    "Active (Hardware Hi-Z Query)",
                    "Discards hidden meshes before rasterization pass.",
                ),
                (
                    "🌟 Compute Shader Skinning",
                    "Active (Compute Dispatch)",
                    "Transforms skinned skeletal vertices directly in GPU memory.",
                ),
            ],
        ),
    };

    pref_heading(scope, heading, subtitle);

    for (card_title, primary_text, desc_text) in cards {
        scope.container_named(
            "InfoCard",
            Style::new()
                .flex_col()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0)
                .padding_insets(Insets::new(10.0, 14.0, 10.0, 14.0))
                .margin_insets(Insets::new(0.0, 0.0, 12.0, 0.0)),
            |card| {
                card.label_styled_passive(
                    "InfoTitle",
                    card_title,
                    12.5,
                    Color::rgba(1.0, 1.0, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                );
                card.label_styled_passive(
                    "InfoPrimary",
                    primary_text,
                    11.5,
                    Color::rgba(0.0, 0.85, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                );
                card.label_styled_passive(
                    "InfoDesc",
                    desc_text,
                    11.0,
                    Color::rgba(0.65, 0.68, 0.76, 1.0),
                    TextAlign::Left,
                    Style::new(),
                );
            },
        );
    }
}