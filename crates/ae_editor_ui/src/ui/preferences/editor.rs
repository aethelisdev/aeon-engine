// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use ae_editor::snapping::SnapSettings;

/// Renders the Editor tab content in the Preferences window.
/// Controls snapping mode/grid, undo history limit, and live editor updates (hot reload).
pub fn draw(
    ui: &mut egui::Ui,
    snapping_settings: &mut SnapSettings,
    enable_live_updates: &mut bool,
    editor_config: &mut ae_editor::editor_state::EditorConfig,
    status_message: &mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
) {
    ui.label(
        egui::RichText::new("Editor Settings")
            .strong()
            .size(18.0)
            .color(egui::Color32::WHITE),
    );
    ui.separator();
    ui.add_space(12.0);

    // ── Snapping ──
    egui::CollapsingHeader::new("🎯 Snapping")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("snap_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    ui.label("Snap Mode");
                    egui::ComboBox::from_id_salt("snap_mode")
                        .width(200.0)
                        .selected_text(format!("{:?}", snapping_settings.mode))
                        .show_ui(ui, |ui| {
                            use ae_editor::snapping::SnapMode::*;
                            ui.selectable_value(&mut snapping_settings.mode, Off, "Off");
                            ui.selectable_value(&mut snapping_settings.mode, Hold, "Hold (Ctrl)");
                            ui.selectable_value(&mut snapping_settings.mode, Toggle, "Toggle");
                        });
                    ui.end_row();

                    ui.label("Grid Size");
                    ui.add(
                        egui::Slider::new(&mut snapping_settings.grid_size, 0.1..=10.0)
                            .logarithmic(true)
                            .fixed_decimals(2),
                    );
                    ui.end_row();
                });
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── History Settings ──
    egui::CollapsingHeader::new("📝 History Settings")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Undo History Limit:");
                ui.add(egui::Slider::new(&mut editor_config.max_undo_history, 10..=5000));
            });
            ui.label(egui::RichText::new("Maximum number of actions stored in RAM. Lower values prevent memory bloat during extremely long sessions.").size(11.0).color(egui::Color32::from_gray(140)));
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── Physics Settings ──
    egui::CollapsingHeader::new("🎮 Physics Settings")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Fixed Update Frequency:");
                ui.add(egui::Slider::new(&mut editor_config.physics_hz, 30.0..=240.0).suffix(" Hz"));
            });
            ui.label(egui::RichText::new("Physics simulation frequency. Higher values improve simulation accuracy but increase CPU usage.").size(11.0).color(egui::Color32::from_gray(140)));
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── Runtime Settings ──
    egui::CollapsingHeader::new("⚙ Runtime Settings")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            if ui
                .checkbox(
                    enable_live_updates,
                    "Enable Live Editor Updates (Hot Reload)",
                )
                .changed()
            {
                let (suffix_str, color) = if *enable_live_updates {
                    ("Enabled", egui::Color32::from_rgb(100, 255, 100))
                } else {
                    ("Disabled", egui::Color32::from_rgb(255, 100, 100))
                };
                *status_message = Some((
                    vec![
                        (
                            "Live Editor Updates: ".to_string(),
                            egui::Color32::LIGHT_BLUE,
                        ),
                        (suffix_str.to_string(), color),
                    ],
                    std::time::Instant::now(),
                ));
            }
            ui.label(
                egui::RichText::new(
                    "Disables hot reload. Useful when debugging core engine systems.",
                )
                .size(11.0)
                .color(egui::Color32::from_gray(140)),
            );
        });
}