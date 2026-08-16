// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;

impl EngineUi {
    /// Renders the internal content of the CPU, GPU, and Memory Stats / Profiler panel.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_stats_content(
        ui: &mut egui::Ui,
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Performance");
            ui.label(format!("FPS: {:.0}", fps));
            ui.label(format!("Frame Time: {:.2} ms", 1000.0 / fps));
            ui.separator();

            ui.heading("⏱ CPU Profiler");
            let bar_max = profiler_frame_ms.max(1.0);
            ui.horizontal(|ui| {
                ui.label("ECS/Logic:");
                ui.add(
                    egui::ProgressBar::new(profiler_ecs_ms / bar_max)
                        .text(format!("{:.2} ms", profiler_ecs_ms)),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Render:   ");
                ui.add(
                    egui::ProgressBar::new(profiler_render_ms / bar_max)
                        .text(format!("{:.2} ms", profiler_render_ms)),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Present:  ");
                ui.add(
                    egui::ProgressBar::new(profiler_present_ms / bar_max)
                        .fill(egui::Color32::from_rgb(80, 80, 140))
                        .text(format!("{:.2} ms", profiler_present_ms)),
                );
            });
            ui.horizontal(|ui| {
                ui.label("UI:       ");
                ui.add(
                    egui::ProgressBar::new(profiler_ui_ms / bar_max)
                        .text(format!("{:.2} ms", profiler_ui_ms)),
                );
            });
            ui.label(format!("Total Frame: {:.2} ms", profiler_frame_ms));
            ui.separator();

            ui.heading("💾 Memory");
            let total_mb = memory_models_mb + memory_textures_mb;
            ui.label(format!("Models (RAM+VRAM): {:.2} MB", memory_models_mb));
            ui.label(format!("Textures (VRAM):   {:.2} MB", memory_textures_mb));
            ui.label(format!("Total (Estimate):  {:.2} MB", total_mb));
            ui.separator();

            ui.checkbox(wireframe_enabled, "🕸 Wireframe Mode (Edges)");
            ui.checkbox(grid_enabled, "🔲 Show Grid");
        });
    }
}