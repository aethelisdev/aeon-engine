// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use ae_renderer::graphics_settings::GraphicsSettings;

/// Renders the Graphics tab content in the Preferences window.
/// Controls shadows, MSAA, bloom, environment/sky, sun position, atmosphere,
/// and depth fog — all modifying `GraphicsSettings` in real-time.
pub fn draw(ui: &mut egui::Ui, gs: &mut GraphicsSettings) {
    ui.label(
        egui::RichText::new("Graphics Settings")
            .strong()
            .size(18.0)
            .color(egui::Color32::WHITE),
    );
    ui.separator();
    ui.add_space(12.0);

    // ── Shadows ──
    egui::CollapsingHeader::new("Shadows")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.checkbox(&mut gs.shadow_enabled, "Enable Shadows");
            ui.add_space(8.0);
            ui.add_enabled_ui(gs.shadow_enabled, |ui| {
                egui::Grid::new("sh_grid")
                    .num_columns(2)
                    .spacing([20.0, 12.0])
                    .show(ui, |ui| {
                        ui.label("Resolution");
                        egui::ComboBox::from_id_salt("sh_res")
                            .width(200.0)
                            .selected_text(gs.shadow_resolution.label())
                            .show_ui(ui, |ui| {
                                use ae_renderer::graphics_settings::ShadowResolution::*;
                                ui.selectable_value(&mut gs.shadow_resolution, Low, Low.label());
                                ui.selectable_value(
                                    &mut gs.shadow_resolution,
                                    Medium,
                                    Medium.label(),
                                );
                                ui.selectable_value(&mut gs.shadow_resolution, High, High.label());
                                ui.selectable_value(
                                    &mut gs.shadow_resolution,
                                    Ultra,
                                    Ultra.label(),
                                );
                            });
                        ui.end_row();

                        ui.label("Cascade Count");
                        egui::ComboBox::from_id_salt("sh_cascades")
                            .width(200.0)
                            .selected_text(format!("{} Cascades", gs.shadow_cascades))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut gs.shadow_cascades,
                                    3,
                                    "3 Cascades (Faster)",
                                );
                                ui.selectable_value(
                                    &mut gs.shadow_cascades,
                                    4,
                                    "4 Cascades (Detailed)",
                                );
                            });
                        ui.end_row();

                        ui.label("PCF Quality");
                        egui::ComboBox::from_id_salt("sh_pcf")
                            .width(200.0)
                            .selected_text(gs.shadow_pcf.label())
                            .show_ui(ui, |ui| {
                                use ae_renderer::graphics_settings::PcfQuality::*;
                                ui.selectable_value(&mut gs.shadow_pcf, Off, Off.label());
                                ui.selectable_value(&mut gs.shadow_pcf, Soft, Soft.label());
                                ui.selectable_value(
                                    &mut gs.shadow_pcf,
                                    UltraSoft,
                                    UltraSoft.label(),
                                );
                            });
                        ui.end_row();

                        ui.label("Bias");
                        ui.add(
                            egui::Slider::new(&mut gs.shadow_bias, 0.0001..=0.05)
                                .logarithmic(true)
                                .fixed_decimals(4),
                        );
                        ui.end_row();
                    });
            });
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── Performance ──
    egui::CollapsingHeader::new("Performance")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("perf_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    ui.label("Framerate Limit");
                    egui::ComboBox::from_id_salt("perf_fps")
                        .width(200.0)
                        .selected_text(gs.fps_limit.label())
                        .show_ui(ui, |ui| {
                            use ae_renderer::graphics_settings::FpsLimit::*;
                            ui.selectable_value(&mut gs.fps_limit, Limit60, Limit60.label());
                            ui.selectable_value(&mut gs.fps_limit, Limit120, Limit120.label());
                            ui.selectable_value(&mut gs.fps_limit, Uncapped, Uncapped.label());
                        });
                    ui.end_row();
                });
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── Anti-Aliasing ──
    egui::CollapsingHeader::new("Anti-Aliasing")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("aa_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    ui.label("MSAA Samples");
                    egui::ComboBox::from_id_salt("aa_msaa")
                        .width(200.0)
                        .selected_text(match gs.msaa_samples {
                            1 => "Off (1x)",
                            2 => "2x",
                            _ => "4x (Default)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut gs.msaa_samples, 1, "Off (1x)");
                            ui.selectable_value(&mut gs.msaa_samples, 2, "2x");
                            ui.selectable_value(&mut gs.msaa_samples, 4, "4x (Default)");
                        });
                    ui.end_row();
                });
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── Post-Processing ──
    egui::CollapsingHeader::new("Post-Processing")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.checkbox(&mut gs.bloom_enabled, "Enable Bloom");
            ui.add_space(8.0);
            ui.add_enabled_ui(gs.bloom_enabled, |ui| {
                egui::Grid::new("pp_grid")
                    .num_columns(2)
                    .spacing([20.0, 12.0])
                    .show(ui, |ui| {
                        ui.label("Bloom Intensity");
                        ui.add(
                            egui::Slider::new(&mut gs.bloom_intensity, 0.0..=3.0).fixed_decimals(2),
                        );
                        ui.end_row();
                    });
            });
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // ── Environment & Sky ──
    egui::CollapsingHeader::new("Environment & Sky")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);

            egui::Grid::new("env_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    ui.label("Sky Quality");
                    egui::ComboBox::from_id_salt("sky_quality_combo")
                        .selected_text(gs.sky_quality.label())
                        .show_ui(ui, |ui| {
                            use ae_renderer::graphics_settings::SkyQuality;
                            ui.selectable_value(
                                &mut gs.sky_quality,
                                SkyQuality::Low,
                                SkyQuality::Low.label(),
                            );
                            ui.selectable_value(
                                &mut gs.sky_quality,
                                SkyQuality::Medium,
                                SkyQuality::Medium.label(),
                            );
                            ui.selectable_value(
                                &mut gs.sky_quality,
                                SkyQuality::High,
                                SkyQuality::High.label(),
                            );
                        });
                    ui.end_row();

                    ui.label("Base/Horizon Color");
                    ui.color_edit_button_rgb(&mut gs.environment_color);
                    ui.end_row();
                });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Sun Position").strong());
            ui.add_space(4.0);

            egui::Grid::new("sun_pos_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    ui.label("Pitch");
                    ui.add(egui::Slider::new(
                        &mut gs.sun_pitch,
                        -std::f32::consts::PI..=std::f32::consts::PI,
                    ));
                    ui.end_row();

                    ui.label("Yaw");
                    ui.add(egui::Slider::new(
                        &mut gs.sun_yaw,
                        -std::f32::consts::PI..=std::f32::consts::PI,
                    ));
                    ui.end_row();
                });

            if gs.sky_quality != ae_renderer::graphics_settings::SkyQuality::Low {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Atmosphere Parameters").strong());
                ui.add_space(4.0);

                egui::Grid::new("atmos_grid")
                    .num_columns(2)
                    .spacing([20.0, 12.0])
                    .show(ui, |ui| {
                        ui.label("Density");
                        ui.add(egui::Slider::new(&mut gs.atmosphere_density, 0.0..=5.0));
                        ui.end_row();

                        ui.label("Sun Disc Size");
                        ui.add(egui::Slider::new(&mut gs.sun_disc_size, 0.1..=5.0));
                        ui.end_row();

                        ui.label("Sun Glow Strength");
                        ui.add(egui::Slider::new(&mut gs.sun_glow_strength, 0.0..=5.0));
                        ui.end_row();
                    });
            }

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Depth Fog").strong());
            ui.add_space(4.0);

            ui.checkbox(&mut gs.fog_enabled, "Enable Atmospheric Depth Fog");
            ui.add_enabled_ui(gs.fog_enabled, |ui| {
                egui::Grid::new("fog_grid")
                    .num_columns(2)
                    .spacing([20.0, 12.0])
                    .show(ui, |ui| {
                        ui.label("Fog Distance");
                        ui.add(egui::Slider::new(&mut gs.fog_distance, 100.0..=2000.0));
                        ui.end_row();
                    });
            });
        });
}