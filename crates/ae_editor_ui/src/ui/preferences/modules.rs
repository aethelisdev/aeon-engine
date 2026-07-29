// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::super::EngineUiAction;
use ae_core::modules::EngineModule;

/// Renders the System Modules settings tab.
pub fn draw(
    ui: &mut egui::Ui,
    enabled_modules: &std::collections::HashSet<EngineModule>,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    ui.heading("🧩 System Modules");
    ui.separator();
    ui.add_space(10.0);
    ui.label("Enable or disable core systems to optimize performance or isolate systems. Disabled modules consume zero background CPU/GPU cycles.");
    ui.add_space(15.0);

    let modules = [
        (
            EngineModule::Physics,
            "Physics (Fizik)",
            "Runs position/velocity integration, collisions, and character controller updates. Disabling halts all physical simulations and saves CPU cycles.",
            "⚙ FixedUpdate Loop",
            egui::Color32::from_rgb(235, 115, 60), // Orange
        ),
        (
            EngineModule::Audio,
            "Audio (Ses)",
            "Processes sound playback and environmental effects. Disabling stops all audio processing.",
            "🔊 Audio Pipeline",
            egui::Color32::from_rgb(60, 165, 235), // Blue
        ),
        (
            EngineModule::Render,
            "Render (Render)",
            "Renders 3D geometry, shadows, skybox, and post-processing. Disable to bypass the render pipeline.",
            "👁 3D Viewport Pass",
            egui::Color32::from_rgb(115, 235, 60), // Green
        ),
    ];

    for &(module, name, desc, detail, color) in &modules {
        let is_enabled = enabled_modules.contains(&module);

        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    // Indicator dot
                    let (rect, _) =
                        ui.allocate_at_least(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    let dot_color = if is_enabled {
                        color
                    } else {
                        egui::Color32::from_rgb(60, 60, 65)
                    };
                    ui.painter().circle_filled(rect.center(), 5.0, dot_color);

                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new(name)
                            .strong()
                            .size(14.0)
                            .color(egui::Color32::WHITE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let text = if is_enabled { "ENABLED" } else { "DISABLED" };
                        let text_color = if is_enabled {
                            egui::Color32::from_rgb(100, 220, 100)
                        } else {
                            egui::Color32::from_rgb(220, 100, 100)
                        };

                        let mut toggle_val = is_enabled;
                        if ui.checkbox(&mut toggle_val, "").clicked() {
                            ui_actions.push(EngineUiAction::ToggleModule(module));
                        }

                        ui.label(
                            egui::RichText::new(text)
                                .strong()
                                .size(11.0)
                                .color(text_color),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(detail)
                                .small()
                                .color(egui::Color32::from_rgb(140, 140, 150)),
                        );
                    });
                });

                ui.add_space(6.0);
                ui.label(egui::RichText::new(desc).color(egui::Color32::from_rgb(170, 170, 180)));
            });
        });
        ui.add_space(10.0);
    }
}