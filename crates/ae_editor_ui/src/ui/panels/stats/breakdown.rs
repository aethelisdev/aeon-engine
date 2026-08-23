// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! CPU Thread Synchronization and GPU Pass Execution Breakdown Widgets.
//!
//! Provides granular visualization of CPU subsystems (ECS, Physics, Render Prep, VSync wait, UI)
//! and GPU render passes (Shadows, Main Opaque, Post-Process, UI) with multi-segmented timing bars.
//!

use ae_core::telemetry::{CpuSyncTimings, GpuPassTimings};
use egui::{Color32, Rect, Stroke, Vec2};

/// Renders the detailed CPU Thread & Synchronization breakdown card.
pub fn draw_cpu_breakdown_card(ui: &mut egui::Ui, cpu: &CpuSyncTimings, total_gpu_ms: f32) {
    let is_cpu_bound = cpu.is_cpu_bound(total_gpu_ms);
    let total_frame_ms = cpu.total_cpu_ms.max(total_gpu_ms);

    // If frame is running fast under 8.33ms (120 FPS) or 16.67ms (60 FPS), mark as optimal
    let (bottleneck_text, bottleneck_color) = if total_frame_ms <= 8.33 {
        ("Optimal (120+ FPS)", Color32::from_rgb(0, 210, 160))
    } else if total_frame_ms <= 16.67 {
        ("Within Budget (60+ FPS)", Color32::from_rgb(0, 190, 230))
    } else if is_cpu_bound {
        ("CPU Bound", Color32::from_rgb(245, 145, 30))
    } else {
        ("GPU Bound / VSync", Color32::from_rgb(240, 100, 80))
    };

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Thread Balance:")
                .color(Color32::from_rgb(150, 155, 170))
                .font(egui::FontId::proportional(11.0)),
        );
        ui.label(
            egui::RichText::new(bottleneck_text)
                .strong()
                .color(bottleneck_color)
                .font(egui::FontId::proportional(11.0)),
        );
    });

    ui.add_space(4.0);

    // Multi-segmented composite bar for CPU distribution
    let bar_total = cpu.total_cpu_ms.max(0.001);
    let segments = [
        (
            cpu.main_logic_ms / bar_total,
            Color32::from_rgb(0, 190, 230),
        ), // ECS (Cyan)
        (cpu.physics_ms / bar_total, Color32::from_rgb(245, 145, 30)), // Physics (Orange)
        (
            cpu.render_prep_ms / bar_total,
            Color32::from_rgb(90, 130, 240),
        ), // Render Prep (Blue)
        (
            cpu.wait_for_gpu_ms / bar_total,
            Color32::from_rgb(160, 100, 220),
        ), // Wait for GPU (Purple)
        (
            cpu.ui_editor_ms / bar_total,
            Color32::from_rgb(220, 180, 50),
        ), // UI (Yellow)
    ];
    draw_multi_segment_bar(ui, &segments);

    ui.add_space(5.0);

    // Clean, aligned timing subrows (no overlapping dots or progress bar glitches)
    draw_timing_row(
        ui,
        "ECS / Logic",
        cpu.main_logic_ms,
        bar_total,
        Color32::from_rgb(0, 190, 230),
    );
    draw_timing_row(
        ui,
        "Physics Simulation",
        cpu.physics_ms,
        bar_total,
        Color32::from_rgb(245, 145, 30),
    );
    draw_timing_row(
        ui,
        "Render Preparation",
        cpu.render_prep_ms,
        bar_total,
        Color32::from_rgb(90, 130, 240),
    );
    draw_timing_row(
        ui,
        "Wait for GPU (VSync)",
        cpu.wait_for_gpu_ms,
        bar_total,
        Color32::from_rgb(160, 100, 220),
    );
    draw_timing_row(
        ui,
        "UI / Editor Passes",
        cpu.ui_editor_ms,
        bar_total,
        Color32::from_rgb(220, 180, 50),
    );

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Total CPU Frame:")
                .strong()
                .color(Color32::from_rgb(220, 225, 235)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2} ms", cpu.total_cpu_ms))
                    .strong()
                    .color(Color32::from_rgb(200, 230, 255)),
            );
        });
    });
}

/// Renders the detailed GPU Pass Timings breakdown card.
pub fn draw_gpu_breakdown_card(
    ui: &mut egui::Ui,
    gpu: &GpuPassTimings,
    gpu_name: &str,
    gpu_backend: &str,
) {
    if !gpu_name.is_empty() {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(gpu_name)
                    .color(Color32::from_rgb(200, 210, 225))
                    .font(egui::FontId::proportional(11.0)),
            );
            if !gpu_backend.is_empty() {
                ui.label(
                    egui::RichText::new(format!("({})", gpu_backend))
                        .color(Color32::from_rgb(0, 190, 230))
                        .font(egui::FontId::proportional(10.5)),
                );
            }
        });
        ui.add_space(3.0);
    }

    let bar_total = gpu.total_gpu_ms.max(0.001);
    let segments = [
        (
            gpu.shadow_pass_ms / bar_total,
            Color32::from_rgb(230, 90, 60),
        ), // Shadows (Coral)
        (
            gpu.main_opaque_pass_ms / bar_total,
            Color32::from_rgb(60, 140, 240),
        ), // Main Opaque (Blue)
        (
            gpu.post_process_pass_ms / bar_total,
            Color32::from_rgb(200, 70, 180),
        ), // Post-Process (Magenta)
        (gpu.ui_pass_ms / bar_total, Color32::from_rgb(220, 180, 50)), // UI Pass (Yellow)
    ];
    draw_multi_segment_bar(ui, &segments);

    ui.add_space(5.0);

    draw_timing_row(
        ui,
        "Shadow Pass (Cascades)",
        gpu.shadow_pass_ms,
        bar_total,
        Color32::from_rgb(230, 90, 60),
    );
    draw_timing_row(
        ui,
        "Main Forward Pass",
        gpu.main_opaque_pass_ms,
        bar_total,
        Color32::from_rgb(60, 140, 240),
    );
    draw_timing_row(
        ui,
        "Post-Process & Outline",
        gpu.post_process_pass_ms,
        bar_total,
        Color32::from_rgb(200, 70, 180),
    );
    draw_timing_row(
        ui,
        "UI Composite Pass",
        gpu.ui_pass_ms,
        bar_total,
        Color32::from_rgb(220, 180, 50),
    );

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Total GPU Workload:")
                .strong()
                .color(Color32::from_rgb(220, 225, 235)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2} ms", gpu.total_gpu_ms))
                    .strong()
                    .color(Color32::from_rgb(200, 230, 255)),
            );
        });
    });
}

/// Helper to render a multi-colored segmented progress bar.
fn draw_multi_segment_bar(ui: &mut egui::Ui, segments: &[(f32, Color32)]) {
    let (response, painter) =
        ui.allocate_painter(Vec2::new(ui.available_width(), 7.0), egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 3.0, Color32::from_rgb(14, 16, 22));

    let mut current_x = rect.min.x;
    for (fraction, color) in segments {
        if *fraction <= 0.001 {
            continue;
        }
        let seg_w = (rect.width() * fraction).min(rect.max.x - current_x);
        let seg_rect = Rect::from_min_size(
            egui::Pos2::new(current_x, rect.min.y),
            Vec2::new(seg_w, rect.height()),
        );
        painter.rect_filled(seg_rect, 0.0, *color);
        current_x += seg_w;
    }

    painter.rect_stroke(
        rect,
        3.0,
        Stroke::new(1.0, Color32::from_rgb(34, 38, 48)),
        egui::StrokeKind::Inside,
    );
}

/// Helper to draw a clean timing row with a colored bullet, label, and right-aligned value & percentage.
fn draw_timing_row(ui: &mut egui::Ui, label: &str, ms: f32, max_ms: f32, dot_color: Color32) {
    let fraction = (ms / max_ms).clamp(0.0, 1.0);
    let pct = fraction * 100.0;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("●")
                .font(egui::FontId::proportional(9.0))
                .color(dot_color),
        );
        ui.label(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(11.0))
                .color(Color32::from_rgb(175, 180, 195)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("({:.0}%)", pct))
                    .font(egui::FontId::proportional(10.5))
                    .color(Color32::from_rgb(130, 135, 150)),
            );
            ui.label(
                egui::RichText::new(format!("{:.2} ms", ms))
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if ms > 0.05 {
                        Color32::from_rgb(220, 230, 245)
                    } else {
                        Color32::from_rgb(120, 125, 140)
                    }),
            );
        });
    });
}