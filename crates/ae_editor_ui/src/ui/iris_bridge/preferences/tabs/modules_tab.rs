// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # System Modules Preferences Tab
//!
//! Renders hardware-accelerated configuration cards for toggling core engine systems
//! (Physics, Audio, 3D Render, 2D Render) declaratively via [`UiScope`] and [`ActionBus`].

use super::super::types::PreferencesParams;
use crate::ui::types::EngineUiAction;
use ae_core::modules::EngineModule;
use irisui::prelude::*;
use irisui::widgets::declarative::scope::WrappedLabelDescriptor;
use irisui::workbench::ActionBus;

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
pub fn build_modules_tab(
    scope: &mut UiScope<'_>,
    params: &PreferencesParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.section_heading(
        "🧩  System Modules",
        "Enable or disable core systems to optimize performance or isolate systems. Disabled modules consume zero background CPU/GPU cycles.",
    );

    for card_data in &MODULES {
        scope.with_id(card_data.name, |mod_scope| {
            mod_scope.container_named(
                "ModuleCard",
                Style::new()
                    .flex_col()
                    .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                    .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                    .border_radius(6.0)
                    .padding_insets(Insets::new(14.0, 16.0, 18.0, 16.0))
                    .margin_insets(Insets::new(0.0, 0.0, 14.0, 0.0)),
                |card| {
                    card.container_named(
                        "ModuleHeaderRow",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .height(26.0)
                            .margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
                        |row| {
                            row.empty_box_passive_named(
                                "ModuleDot",
                                Style::new()
                                    .width(10.0)
                                    .height(10.0)
                                    .background(
                                        if params.enabled_modules.contains(&card_data.module) {
                                            card_data.color
                                        } else {
                                            Color::rgba(0.24, 0.26, 0.32, 1.0)
                                        },
                                    )
                                    .border_radius(5.0)
                                    .margin_insets(Insets::new(0.0, 12.0, 0.0, 0.0)),
                            );

                            row.label_styled_passive(
                                "ModuleName",
                                card_data.name,
                                13.5,
                                Color::rgba(1.0, 1.0, 1.0, 1.0),
                                TextAlign::Left,
                                Style::new().margin_insets(Insets::new(0.0, 14.0, 0.0, 0.0)),
                            );

                            row.label_styled_passive(
                                "ModuleDetail",
                                card_data.detail,
                                10.0,
                                Color::rgba(0.50, 0.54, 0.65, 1.0),
                                TextAlign::Left,
                                Style::new().flex_grow(1.0),
                            );

                            row.label_styled_passive(
                                "ModuleStatus",
                                if params.enabled_modules.contains(&card_data.module) {
                                    "ENABLED"
                                } else {
                                    "DISABLED"
                                },
                                10.5,
                                if params.enabled_modules.contains(&card_data.module) {
                                    Color::rgba(0.39, 0.86, 0.39, 1.0)
                                } else {
                                    Color::rgba(0.86, 0.39, 0.39, 1.0)
                                },
                                TextAlign::Right,
                                Style::new().margin_insets(Insets::new(0.0, 12.0, 0.0, 0.0)),
                            );

                            row.checkbox_val(
                                "",
                                params.enabled_modules.contains(&card_data.module),
                            )
                            .on_click(|| {
                                action_bus.emit(EngineUiAction::ToggleModule(card_data.module));
                            });
                        },
                    );

                    card.label_styled_passive_wrapped(
                        "ModuleDesc",
                        card_data.desc,
                        WrappedLabelDescriptor::new(
                            11.0,
                            Color::rgba(0.65, 0.68, 0.76, 1.0),
                            Style::new().margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
                        ),
                    );
                },
            );
        });
    }
}