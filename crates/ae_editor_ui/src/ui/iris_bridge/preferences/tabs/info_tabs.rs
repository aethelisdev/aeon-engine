// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Information Preferences Tabs
//!
//! Renders hardware-accelerated configuration and information views for
//! Navigation (Tab 3), Keymap (Tab 4), System (Tab 5), Add-ons (Tab 6), Input (Tab 7), and Experimental (Tab 8).

use super::super::types::{PreferencesParams, PreferencesTargets};
use irisui::prelude::*;

/// Builds a generic information preferences tab with heading, separator, and informative cards.
pub fn build_info_tab(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
    tab_index: u8,
    params: &PreferencesParams<'_>,
    _targets: &mut PreferencesTargets,
) -> f32 {
    let mut virtual_y = 16.0;
    let scroll_y = params.scroll_offset_y;
    let content_w = content_rect.width - 32.0;
    let base_x = content_rect.x + 16.0;

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
                    "🎨 Iris UI Extension Engine",
                    "Native SDF Typography & Layout Pipeline",
                    "Modern game UI designer and runtime rendering system.",
                ),
                (
                    "🔌 C API Plugin Host",
                    "Ready (Dynamic Library Loader)",
                    "Extensible native modules with zero rebuild requirement.",
                ),
            ],
        ),
        7 => (
            "Input",
            "Gamepad, keyboard, and mouse binding preferences.",
            vec![
                (
                    "🖱 Mouse Smoothing",
                    "Raw Sub-Pixel Delta Input",
                    "Hardware mouse polling without artificial acceleration or lag.",
                ),
                (
                    "🎮 Gamepad Support",
                    "XInput / DualSense Standard",
                    "Automatic controller detection and hot-plugging.",
                ),
                (
                    "⌨ IME Composition",
                    "Native Operating System IME Pipeline",
                    "Full support for multi-byte international text input.",
                ),
            ],
        ),
        8 => (
            "Experimental",
            "Preview upcoming engine and rendering features.",
            vec![
                (
                    "⚡ Meshlet GPU Culling",
                    "Experimental (Compute Shader Driven)",
                    "Nanite-style virtualized geometry and sub-mesh occlusion culling.",
                ),
                (
                    "🌊 Real-Time Water Simulation",
                    "In Development (Gerstner Waves & FFT)",
                    "Dynamic buoyancy, foam generation, and underwater caustics.",
                ),
                (
                    "🌐 Multi-User Live Sync",
                    "Prototype (CRDT Networking)",
                    "Real-time collaborative level editing over local network.",
                ),
            ],
        ),
        _ => ("Settings", "Additional preferences.", Vec::new()),
    };

    // 1. Heading
    let heading_id = tree.create_node();
    if let Some(node) = tree.get_mut(heading_id) {
        node.set_name("InfoHeading");
        node.set_text(heading);
        node.font_size = 17.0;
        node.line_height = 22.0;
        node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            22.0,
        );
    }
    let _ = tree.add_child(parent_id, heading_id);
    virtual_y += 24.0;

    // 2. Subtitle
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("InfoSubtitle");
        node.set_text(subtitle);
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.65, 0.68, 0.76, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, sub_id);
    virtual_y += 24.0;

    // 3. Separator
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("InfoSep");
        node.style = Style::new().background(Color::rgba(0.20, 0.22, 0.30, 0.70));
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            1.0,
        );
    }
    let _ = tree.add_child(parent_id, sep_id);
    virtual_y += 16.0;

    // 4. Cards
    for (card_title, primary_val, card_desc) in cards {
        let card_h = 76.0;
        let card_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            card_h,
        );

        let card_node = tree.create_node();
        if let Some(node) = tree.get_mut(card_node) {
            node.set_name("InfoCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_node);

        let t_node = tree.create_node();
        if let Some(node) = tree.get_mut(t_node) {
            node.set_name("InfoCardTitle");
            node.set_text(card_title);
            node.font_size = 13.0;
            node.line_height = 18.0;
            node.text_color = Color::rgba(0.0, 0.90, 1.0, 1.0);
            node.computed_rect = Rect::new(
                base_x + 14.0,
                content_rect.y + virtual_y - scroll_y + 10.0,
                content_w - 28.0,
                18.0,
            );
        }
        let _ = tree.add_child(card_node, t_node);

        let val_node = tree.create_node();
        if let Some(node) = tree.get_mut(val_node) {
            node.set_name("InfoCardVal");
            node.set_text(primary_val);
            node.font_size = 11.5;
            node.line_height = 16.0;
            node.text_color = Color::rgba(0.92, 0.94, 0.98, 1.0);
            node.computed_rect = Rect::new(
                base_x + 14.0,
                content_rect.y + virtual_y - scroll_y + 30.0,
                content_w - 28.0,
                16.0,
            );
        }
        let _ = tree.add_child(card_node, val_node);

        let d_node = tree.create_node();
        if let Some(node) = tree.get_mut(d_node) {
            node.set_name("InfoCardDesc");
            node.set_text(card_desc);
            node.font_size = 10.5;
            node.line_height = 14.0;
            node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
            node.computed_rect = Rect::new(
                base_x + 14.0,
                content_rect.y + virtual_y - scroll_y + 50.0,
                content_w - 28.0,
                14.0,
            );
        }
        let _ = tree.add_child(card_node, d_node);

        virtual_y += card_h + 12.0;
    }

    virtual_y += 10.0;
    virtual_y
}