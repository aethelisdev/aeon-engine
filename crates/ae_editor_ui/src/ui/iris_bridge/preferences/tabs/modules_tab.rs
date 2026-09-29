// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # System Modules Preferences Tab
//!
//! Renders hardware-accelerated configuration cards for toggling core engine systems
//! (Physics, Audio, 3D Render Viewport pass) with live status indicators and zero background CPU/GPU overhead.

use super::super::components::pref_heading;
use super::super::types::{PreferencesParams, PreferencesToggleId, encode_toggle_tag};
use ae_core::modules::EngineModule;
use irisui::prelude::*;

/// System module descriptor with UI presentation metadata.
struct ModuleCardData {
    module: EngineModule,
    name: &'static str,
    desc: &'static str,
    detail: &'static str,
    color: Color,
}

const MODULES: [ModuleCardData; 4] = [
    ModuleCardData {
        module: EngineModule::Physics,
        name: "Physics",
        desc: "Runs position/velocity integration, collisions, and character controller updates. Disabling halts all physical simulations and saves CPU cycles.",
        detail: "⚙ FixedUpdate Loop",
        color: Color::rgba(0.92, 0.45, 0.23, 1.0),
    },
    ModuleCardData {
        module: EngineModule::Audio,
        name: "Audio",
        desc: "Processes sound playback and environmental effects. Disabling stops all audio processing.",
        detail: "🔊 Audio Pipeline",
        color: Color::rgba(0.23, 0.65, 0.92, 1.0),
    },
    ModuleCardData {
        module: EngineModule::Render,
        name: "3D Render",
        desc: "Renders 3D geometry, shadows, skybox, and post-processing. Disable to bypass the 3D render pipeline.",
        detail: "👁 3D Viewport Pass",
        color: Color::rgba(0.45, 0.92, 0.23, 1.0),
    },
    ModuleCardData {
        module: EngineModule::Render2D,
        name: "2D Render",
        desc: "Renders 2D sprites, sprite batching, tiles, and sorting layers. Disable to bypass the 2D sprite pipeline.",
        detail: "🖼 2D Sprite Pass",
        color: Color::rgba(0.18, 0.80, 0.44, 1.0),
    },
];

/// Builds the System Modules preferences tab content declaratively using [`UiScope`].
pub fn build_modules_tab(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "🧩  System Modules",
        "Enable or disable core systems to optimize performance or isolate systems. Disabled modules consume zero background CPU/GPU cycles.",
    );

    for card_data in &MODULES {
        let is_enabled = params.enabled_modules.contains(&card_data.module);
        let toggle_tag = encode_toggle_tag(PreferencesToggleId::Module(card_data.module));
        let is_hovered = params.hovered_tag == Some(toggle_tag);

        scope.container_named(
            "ModuleCard",
            Style::new()
                .flex_col()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0)
                .padding_insets(Insets::new(14.0, 16.0, 18.0, 16.0))
                .margin_insets(Insets::new(0.0, 0.0, 14.0, 0.0)),
            |card| {
                // Header row: Dot + Name + Detail badge + Status + Checkbox
                card.container_named(
                    "ModuleHeaderRow",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .height(26.0)
                        .margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
                    |row| {
                        // Colored Status Dot
                        let dot_col = if is_enabled {
                            card_data.color
                        } else {
                            Color::rgba(0.24, 0.26, 0.32, 1.0)
                        };
                        row.empty_box_passive_named(
                            "ModuleDot",
                            Style::new()
                                .width(10.0)
                                .height(10.0)
                                .background(dot_col)
                                .border_radius(5.0)
                                .margin_insets(Insets::new(0.0, 12.0, 0.0, 0.0)),
                        );

                        // Module Name
                        row.label_styled_passive(
                            "ModuleName",
                            card_data.name,
                            13.5,
                            Color::rgba(1.0, 1.0, 1.0, 1.0),
                            TextAlign::Left,
                            Style::new().margin_insets(Insets::new(0.0, 14.0, 0.0, 0.0)),
                        );

                        // Detail Badge
                        row.label_styled_passive(
                            "ModuleDetail",
                            card_data.detail,
                            10.0,
                            Color::rgba(0.50, 0.54, 0.65, 1.0),
                            TextAlign::Left,
                            Style::new().flex_grow(1.0),
                        );

                        // Status Text ("ENABLED" / "DISABLED")
                        let (status_text, status_col) = if is_enabled {
                            ("ENABLED", Color::rgba(0.39, 0.86, 0.39, 1.0))
                        } else {
                            ("DISABLED", Color::rgba(0.86, 0.39, 0.39, 1.0))
                        };
                        row.label_styled_passive(
                            "ModuleStatus",
                            status_text,
                            10.5,
                            status_col,
                            TextAlign::Right,
                            Style::new().margin_insets(Insets::new(0.0, 12.0, 0.0, 0.0)),
                        );

                        // Checkbox Trigger Box
                        let cb_bg = if is_enabled {
                            Color::rgba(0.0, 0.70, 0.85, 1.0)
                        } else if is_hovered {
                            Color::rgba(0.18, 0.20, 0.28, 1.0)
                        } else {
                            Color::rgba(0.11, 0.12, 0.16, 1.0)
                        };

                        row.container_tagged(
                            "ModuleCb",
                            Style::new()
                                .width(16.0)
                                .height(16.0)
                                .background(cb_bg)
                                .border(1.0, Color::rgba(0.25, 0.30, 0.42, 1.0))
                                .border_radius(3.0)
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::Center),
                            WidgetRole::Checkbox,
                            toggle_tag,
                            |cb| {
                                if is_enabled {
                                    cb.label_styled_passive(
                                        "ModuleCheckMark",
                                        "✓",
                                        11.0,
                                        Color::rgba(0.05, 0.06, 0.08, 1.0),
                                        TextAlign::Center,
                                        Style::new(),
                                    );
                                }
                            },
                        );
                    },
                );

                // Description with explicit wrapping width
                let desc_id = card.label_styled_passive(
                    "ModuleDesc",
                    card_data.desc,
                    11.0,
                    Color::rgba(0.65, 0.68, 0.76, 1.0),
                    TextAlign::Left,
                    Style::new()
                        .width(530.0)
                        .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
                );
                if let Some(node) = card.tree_mut().get_mut(desc_id) {
                    node.set_text_wrap(TextWrap::Word);
                }
            },
        );
    }
}