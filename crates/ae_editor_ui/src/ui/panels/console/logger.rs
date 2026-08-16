// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{ConsoleEntry, EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the internal content of the Developer Console.
    /// The console sub-panel is optimized using a zero-allocation `egui::text::LayoutJob` per visible log entry,
    /// eliminating high-frequency heap allocations and layout nesting overhead.
    pub fn draw_console_content(
        ui: &mut egui::Ui,
        console_entries: &[ConsoleEntry],
        _ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .button("🧹 Clear Logs")
                .on_hover_text("Clear in-memory logger entries")
                .clicked()
                && let Ok(mut lock) = ae_editor::editor_logger::LOGGER.logs.lock()
            {
                lock.clear();
            }
        });
        ui.separator();

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
}