// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::EngineUi;

/// Telemetry data and viewport overlay references passed into the Stats & Profiler panel.
pub struct StatsPanelContext<'a> {
    pub wireframe_enabled: &'a mut bool,
    pub grid_enabled: &'a mut bool,
    pub fps: f32,
    pub profiler_ecs_ms: f32,
    pub profiler_physics_ms: f32,
    pub profiler_render_ms: f32,
    pub profiler_present_ms: f32,
    pub profiler_ui_ms: f32,
    pub profiler_frame_ms: f32,
    pub memory_models_mb: f32,
    pub memory_textures_mb: f32,
    pub render_draw_calls: u32,
    pub render_triangles: u64,
    pub render_vertices: u64,
    pub gpu_adapter_name: &'a str,
    pub gpu_backend: &'a str,
    pub active_entities_count: usize,
    pub selected_entity: Option<hecs::Entity>,
}

/// Helper function to draw a section card with a dark background and subtle border.
fn draw_stats_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    icon: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let frame = egui::Frame::NONE
        .fill(egui::Color32::from_rgb(18, 20, 26))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 42, 54)))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(10, 8));

    frame
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", icon, title))
                        .strong()
                        .color(egui::Color32::from_rgb(220, 225, 235)),
                );
            });
            ui.add_space(4.0);
            add_contents(ui)
        })
        .inner
}

impl EngineUi {
    /// Renders the internal content of the CPU, GPU, Memory, and Scene Geometry Stats panel.
    /// Uses modular section cards and real-time engine telemetry without linter suppression.
    pub fn draw_stats_content(ui: &mut egui::Ui, ctx: StatsPanelContext<'_>) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);

            // 1. Performance & Hardware Card
            draw_stats_card(ui, "Performance & Hardware", "⚡", |ui| {
                let fps_color = if ctx.fps >= 55.0 {
                    egui::Color32::from_rgb(50, 205, 50)
                } else if ctx.fps >= 30.0 {
                    egui::Color32::from_rgb(255, 185, 0)
                } else {
                    egui::Color32::from_rgb(235, 60, 60)
                };

                let frame_time_ms = if ctx.fps > 0.0 { 1000.0 / ctx.fps } else { 0.0 };

                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.label(
                        egui::RichText::new(format!("{:.0}", ctx.fps))
                            .strong()
                            .color(fps_color),
                    );
                    ui.label(format!("({:.2} ms)", frame_time_ms));
                });

                if !ctx.gpu_adapter_name.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("GPU:")
                                .color(egui::Color32::from_rgb(150, 155, 170)),
                        );
                        ui.label(
                            egui::RichText::new(ctx.gpu_adapter_name)
                                .color(egui::Color32::from_rgb(200, 210, 225)),
                        );
                    });
                }

                if !ctx.gpu_backend.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Backend:")
                                .color(egui::Color32::from_rgb(150, 155, 170)),
                        );
                        ui.label(
                            egui::RichText::new(ctx.gpu_backend)
                                .color(egui::Color32::from_rgb(0, 190, 230)),
                        );
                    });
                }
            });

            ui.add_space(6.0);

            // 2. CPU & Physics Profiler Card
            draw_stats_card(ui, "CPU & Physics Profiler", "⏱", |ui| {
                let bar_max = ctx.profiler_frame_ms.max(1.0);

                let draw_prof_row =
                    |ui: &mut egui::Ui, label: &str, ms: f32, fill: egui::Color32| {
                        ui.horizontal(|ui| {
                            ui.add_sized([100.0, 14.0], egui::Label::new(label));
                            ui.add(
                                egui::ProgressBar::new(ms / bar_max)
                                    .fill(fill)
                                    .text(format!("{:.2} ms", ms)),
                            );
                        });
                    };

                draw_prof_row(
                    ui,
                    "ECS / Logic:",
                    ctx.profiler_ecs_ms,
                    egui::Color32::from_rgb(0, 190, 230),
                );
                draw_prof_row(
                    ui,
                    "Physics:",
                    ctx.profiler_physics_ms,
                    egui::Color32::from_rgb(245, 145, 30),
                );
                draw_prof_row(
                    ui,
                    "Render (Prep):",
                    ctx.profiler_render_ms,
                    egui::Color32::from_rgb(90, 130, 240),
                );
                draw_prof_row(
                    ui,
                    "Present (VSync):",
                    ctx.profiler_present_ms,
                    egui::Color32::from_rgb(120, 90, 180),
                );
                draw_prof_row(
                    ui,
                    "UI / Editor:",
                    ctx.profiler_ui_ms,
                    egui::Color32::from_rgb(220, 180, 50),
                );

                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Total Frame:").strong());
                    ui.label(format!("{:.2} ms", ctx.profiler_frame_ms));
                });
            });

            ui.add_space(6.0);

            // 3. Scene & Geometry Metrics Card
            draw_stats_card(ui, "Scene & Geometry", "📐", |ui| {
                let mut tri_buf = core::fmt::NumBuffer::new();
                let tri_str = ctx.render_triangles.format_into(&mut tri_buf);
                ui.horizontal(|ui| {
                    ui.label("Triangles:");
                    ui.label(
                        egui::RichText::new(tri_str)
                            .strong()
                            .color(egui::Color32::from_rgb(200, 230, 255)),
                    );
                });

                let mut vert_buf = core::fmt::NumBuffer::new();
                let vert_str = ctx.render_vertices.format_into(&mut vert_buf);
                ui.horizontal(|ui| {
                    ui.label("Vertices:");
                    ui.label(
                        egui::RichText::new(vert_str)
                            .strong()
                            .color(egui::Color32::from_rgb(200, 230, 255)),
                    );
                });

                let mut dc_buf = core::fmt::NumBuffer::new();
                let dc_str = ctx.render_draw_calls.format_into(&mut dc_buf);
                ui.horizontal(|ui| {
                    ui.label("Draw Calls:");
                    ui.label(
                        egui::RichText::new(dc_str)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 210, 160)),
                    );
                });

                let mut ent_buf = core::fmt::NumBuffer::new();
                let ent_str = ctx.active_entities_count.format_into(&mut ent_buf);
                ui.horizontal(|ui| {
                    ui.label("Entities:");
                    let selected_str = if ctx.selected_entity.is_some() {
                        " (1 selected)"
                    } else {
                        ""
                    };
                    ui.label(format!("{}{}", ent_str, selected_str));
                });
            });

            ui.add_space(6.0);

            // 4. Memory & Assets Card
            draw_stats_card(ui, "Memory & Assets", "💾", |ui| {
                let total_mb = ctx.memory_models_mb + ctx.memory_textures_mb;
                ui.horizontal(|ui| {
                    ui.label("Models (RAM+VRAM):");
                    ui.label(format!("{:.2} MB", ctx.memory_models_mb));
                });
                ui.horizontal(|ui| {
                    ui.label("Textures (VRAM):");
                    ui.label(format!("{:.2} MB", ctx.memory_textures_mb));
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Total Allocated:").strong());
                    ui.label(egui::RichText::new(format!("{:.2} MB", total_mb)).strong());
                });
            });

            ui.add_space(6.0);

            // 5. Viewport Overlays Card
            draw_stats_card(ui, "Viewport Overlays", "🎛", |ui| {
                ui.checkbox(ctx.wireframe_enabled, "🕸 Wireframe Mode (Edges)");
                ui.checkbox(ctx.grid_enabled, "🔲 Show Grid");
            });

            ui.add_space(6.0);
        });
    }
}