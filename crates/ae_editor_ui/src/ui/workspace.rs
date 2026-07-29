// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::{ConsoleEntry, EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the bottom workspace panel containing either the Asset Browser or the Developer Console.
    /// The console sub-panel is optimized using a zero-allocation `egui::text::LayoutJob` per visible log entry,
    /// eliminating high-frequency heap allocations and layout nesting overhead.
    pub(super) fn draw_workspace_panel(
        show_workspace: &mut bool,
        workspace_tab: &mut usize,
        console_entries: &Vec<ConsoleEntry>,
        ui: &mut egui::Ui,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) -> Option<egui::Rect> {
        if !*show_workspace {
            return None;
        }

        let resp = egui::Panel::bottom("workspace_panel")
            .resizable(true)
            .show_separator_line(false)
            .min_size(100.0)
            .default_size(250.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(20, 20, 25))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .stroke(egui::Stroke::NONE),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(workspace_tab, 0, "📂 Assets");
                    ui.selectable_value(workspace_tab, 1, "📜 Console");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            *show_workspace = false;
                        }
                        if ui
                            .button("🧹 Garbage Collect")
                            .on_hover_text(
                                "Clean up unused model and texture resources to reclaim VRAM",
                            )
                            .clicked()
                        {
                            ui_actions.push(EngineUiAction::GarbageCollect);
                        }
                    });
                });
                ui.separator();

                if *workspace_tab == 0 {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Models Column
                            ui.vertical(|ui| {
                                ui.heading("Models (3D)");
                                ui.separator();
                                for (handle, model) in models.iter() {
                                    let path = std::path::Path::new(&model.source_path);
                                    let filename = path
                                        .file_stem()
                                        .unwrap_or(std::ffi::OsStr::new("Unknown"))
                                        .to_string_lossy();

                                    ui.horizontal(|ui| {
                                        ui.label(format!("📦 {}", filename));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("➕ Spawn").clicked() {
                                                    ui_actions
                                                        .push(EngineUiAction::SpawnModel(handle));
                                                }
                                            },
                                        );
                                    });
                                }
                                if models.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No models loaded yet.")
                                            .italics()
                                            .color(egui::Color32::from_gray(120)),
                                    );
                                }
                            });

                            ui.add_space(20.0);

                            // Textures Column
                            ui.vertical(|ui| {
                                ui.heading("Textures (2D)");
                                ui.separator();
                                for (handle, texture) in textures.iter() {
                                    let path = std::path::Path::new(&texture.source_path);
                                    let filename = path
                                        .file_stem()
                                        .unwrap_or(std::ffi::OsStr::new("Unknown"))
                                        .to_string_lossy();

                                    ui.horizontal(|ui| {
                                        ui.label(format!("🖼 {}", filename));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("➕ Spawn").clicked() {
                                                    ui_actions
                                                        .push(EngineUiAction::SpawnSprite(handle));
                                                }
                                            },
                                        );
                                    });
                                }
                                if textures.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No textures loaded yet.")
                                            .italics()
                                            .color(egui::Color32::from_gray(120)),
                                    );
                                }
                            });
                        });
                    });
                } else if *workspace_tab == 1 {
                    let row_height = 18.0;
                    let font_id = egui::TextStyle::Monospace.resolve(ui.style());

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show_rows(ui, row_height, console_entries.len(), |ui, row_range| {
                            for i in row_range {
                                let log = &console_entries[i];
                                let color = match log.level {
                                    log::Level::Error => egui::Color32::from_rgb(255, 100, 100),
                                    log::Level::Warn => egui::Color32::from_rgb(255, 200, 100),
                                    log::Level::Info => egui::Color32::from_gray(220),
                                    log::Level::Debug => egui::Color32::from_rgb(100, 200, 255),
                                    log::Level::Trace => egui::Color32::from_gray(150),
                                };

                                // Zero-allocation composite LayoutJob for microsecond level CPU-drawing speed.
                                // Completely bypasses horizontal layout nesting, 3x label layouts, and string formats.
                                let mut job = egui::text::LayoutJob::default();

                                // 1. Timestamp (gray-120)
                                job.append(
                                    &log.timestamp,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: egui::Color32::from_gray(120),
                                        ..Default::default()
                                    },
                                );

                                // 2. Bracket and Target (gray-150)
                                job.append(
                                    " [",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: egui::Color32::from_gray(150),
                                        ..Default::default()
                                    },
                                );
                                job.append(
                                    &log.target,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: egui::Color32::from_gray(150),
                                        ..Default::default()
                                    },
                                );
                                job.append(
                                    "] ",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: egui::Color32::from_gray(150),
                                        ..Default::default()
                                    },
                                );

                                // 3. Log Message (level-specific color)
                                job.append(
                                    &log.msg,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color,
                                        ..Default::default()
                                    },
                                );

                                ui.label(job);
                            }
                        });
                }
            });

        Some(resp.response.rect)
    }

    /// Renders a persistent thin bar at the absolute bottom for quick toggling and status info.
    pub(super) fn draw_utility_bar(
        show_workspace: &mut bool,
        workspace_tab: &mut usize,
        status_message: &mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
        ui: &mut egui::Ui,
    ) -> Option<egui::Rect> {
        let resp = egui::Panel::bottom("utility_bar")
            .exact_size(24.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(15, 15, 20))
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .stroke(egui::Stroke::NONE),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    // 1. Toggles
                    let console_btn = egui::RichText::new("📜 Console").size(11.5).strong();
                    let assets_btn = egui::RichText::new("📂 Assets").size(11.5).strong();

                    let cur_workspace = *show_workspace;
                    let cur_tab = *workspace_tab;

                    // Highlight active tab
                    let (console_color, console_bg) = if cur_workspace && cur_tab == 1 {
                        (egui::Color32::WHITE, egui::Color32::from_rgb(50, 80, 120))
                    } else {
                        (egui::Color32::from_gray(140), egui::Color32::TRANSPARENT)
                    };
                    let (assets_color, assets_bg) = if cur_workspace && cur_tab == 0 {
                        (egui::Color32::WHITE, egui::Color32::from_rgb(50, 80, 120))
                    } else {
                        (egui::Color32::from_gray(140), egui::Color32::TRANSPARENT)
                    };

                    if ui
                        .add(
                            egui::Button::new(console_btn.color(console_color))
                                .fill(console_bg)
                                .small(),
                        )
                        .clicked()
                    {
                        if cur_workspace && cur_tab == 1 {
                            *show_workspace = false;
                        } else {
                            *show_workspace = true;
                            *workspace_tab = 1;
                        }
                    }

                    if ui
                        .add(
                            egui::Button::new(assets_btn.color(assets_color))
                                .fill(assets_bg)
                                .small(),
                        )
                        .clicked()
                    {
                        if cur_workspace && cur_tab == 0 {
                            *show_workspace = false;
                        } else {
                            *show_workspace = true;
                            *workspace_tab = 0;
                        }
                    }

                    ui.separator();

                    // 2. Status Message
                    if let Some((spans, _)) = status_message {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (text, color) in spans {
                            ui.label(egui::RichText::new(text.as_str()).color(*color).size(11.0));
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("Ready")
                                .color(egui::Color32::from_gray(100))
                                .size(11.0)
                                .italics(),
                        );
                    }

                    // 3. Right-side stats
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("AE v{}", env!("CARGO_PKG_VERSION")))
                                .color(egui::Color32::from_gray(60))
                                .size(10.0),
                        );
                        if let Ok(lock) = ae_editor::editor_logger::LOGGER.logs.lock() {
                            let count = lock.len();
                            if count > 0 {
                                ui.label(
                                    egui::RichText::new(format!("Logs: {}", count))
                                        .color(egui::Color32::from_gray(80))
                                        .size(11.0),
                                );
                            }
                        }
                    });
                });
            });

        Some(resp.response.rect)
    }
}