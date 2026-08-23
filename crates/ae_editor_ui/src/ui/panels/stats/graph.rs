// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame Pacing and Stutter Analyzer Oscilloscope Canvas Widget.
//!
//! Visualizes 240-frame historical frametimes with a smooth polyline curve,
//! target milestone threshold lines (144 FPS / 120 FPS / 60 FPS / 30 FPS),
//! dynamic spike indicators, and 2x2 statistical metrics.
//!

use ae_core::telemetry::{FramePacingStats, FrameRingBuffer};
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

/// Renders the Frame Pacing & Stutter Analyzer card with the live oscilloscope canvas.
pub fn draw_frame_pacing_card(
    ui: &mut egui::Ui,
    frame_pacing: &FrameRingBuffer<240>,
    stats: &FramePacingStats,
) {
    // 1. Metric Badges Grid (2x2 layout to prevent horizontal clipping in narrow panels)
    egui::Grid::new("frame_pacing_metrics_grid")
        .num_columns(2)
        .spacing([6.0, 4.0])
        .min_col_width((ui.available_width() - 8.0) * 0.5)
        .show(ui, |ui| {
            draw_metric_pill(
                ui,
                "Avg FPS",
                &format!(
                    "{:.0} ({:.2}ms)",
                    stats.average_fps, stats.average_frametime_ms
                ),
                get_fps_color(stats.average_fps),
            );
            draw_metric_pill(
                ui,
                "1% Low",
                &format!("{:.0} FPS", stats.low_1_percent_fps),
                get_fps_color(stats.low_1_percent_fps),
            );
            ui.end_row();

            draw_metric_pill(
                ui,
                "0.1% Low",
                &format!("{:.0} FPS", stats.low_0_1_percent_fps),
                get_fps_color(stats.low_0_1_percent_fps),
            );
            draw_metric_pill(
                ui,
                "Jitter",
                &format!("±{:.2} ms", stats.variance_ms),
                if stats.variance_ms < 1.5 {
                    Color32::from_rgb(0, 210, 160)
                } else if stats.variance_ms < 4.0 {
                    Color32::from_rgb(255, 185, 0)
                } else {
                    Color32::from_rgb(235, 60, 60)
                },
            );
            ui.end_row();
        });

    ui.add_space(6.0);

    // 2. Oscilloscope Canvas
    let available_w = ui.available_width().max(180.0);
    let canvas_height = 110.0;
    let (response, painter) =
        ui.allocate_painter(Vec2::new(available_w, canvas_height), egui::Sense::hover());
    let rect = response.rect;

    // Dark sleek oscilloscope canvas background
    painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 14, 18));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, Color32::from_rgb(30, 34, 44)),
        egui::StrokeKind::Inside,
    );

    let max_ms_scale = 36.0f32; // Predictable vertical ceiling (0 - 36ms)

    // Draw horizontal reference milestone lines
    draw_milestone_line(
        &painter,
        rect,
        6.94,
        max_ms_scale,
        "144 FPS (6.9ms)",
        Color32::from_rgba_unmultiplied(0, 210, 240, 60),
    );
    draw_milestone_line(
        &painter,
        rect,
        8.33,
        max_ms_scale,
        "120 FPS (8.3ms)",
        Color32::from_rgba_unmultiplied(50, 205, 50, 70),
    );
    draw_milestone_line(
        &painter,
        rect,
        16.67,
        max_ms_scale,
        "60 FPS (16.6ms)",
        Color32::from_rgba_unmultiplied(255, 185, 0, 80),
    );
    draw_milestone_line(
        &painter,
        rect,
        33.33,
        max_ms_scale,
        "30 FPS (33.3ms)",
        Color32::from_rgba_unmultiplied(235, 60, 60, 80),
    );

    let sample_count = frame_pacing.count();
    if sample_count >= 2 {
        let step_x = rect.width() / (sample_count - 1) as f32;

        let mut hovered_info: Option<(usize, f32, Pos2)> = None;
        let hover_pos = response.hover_pos();

        // 1. Build curve points for smooth polyline rendering
        let mut points = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let val_ms = frame_pacing.get_chronological(i).unwrap_or(0.0);
            let clamped_val = val_ms.min(max_ms_scale).max(0.0);
            let x = rect.min.x + i as f32 * step_x;
            let y = rect.max.y - (clamped_val / max_ms_scale) * rect.height();
            let pt = Pos2::new(x, y);
            points.push((pt, val_ms));

            if let Some(mouse) = hover_pos
                && (mouse.x - x).abs() <= (step_x * 0.6).max(2.5)
            {
                hovered_info = Some((i, val_ms, pt));
            }
        }

        // 2. Draw subtle translucent filled area under the curve
        for window in points.windows(2) {
            let (p0, v0) = window[0];
            let (p1, _v1) = window[1];
            let fill_color = if v0 <= 8.33 {
                Color32::from_rgba_unmultiplied(0, 200, 160, 25)
            } else if v0 <= 16.67 {
                Color32::from_rgba_unmultiplied(0, 180, 230, 25)
            } else if v0 <= 33.33 {
                Color32::from_rgba_unmultiplied(255, 185, 0, 35)
            } else {
                Color32::from_rgba_unmultiplied(235, 60, 60, 45)
            };

            let poly = [
                Pos2::new(p0.x, rect.max.y - 1.0),
                p0,
                p1,
                Pos2::new(p1.x, rect.max.y - 1.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                poly.to_vec(),
                fill_color,
                Stroke::NONE,
            ));
        }

        // 3. Draw smooth curve polyline stroke
        for window in points.windows(2) {
            let (p0, v0) = window[0];
            let (p1, v1) = window[1];
            let line_color = get_frametime_color(v0.max(v1));
            painter.line_segment([p0, p1], Stroke::new(1.8, line_color));
        }

        // 4. Highlight spike points that exceed 16.6ms (stutters)
        for (pt, val_ms) in &points {
            if *val_ms > 16.67 {
                let spike_color = if *val_ms > 33.33 {
                    Color32::from_rgb(255, 60, 60)
                } else {
                    Color32::from_rgb(255, 190, 30)
                };
                painter.circle_filled(*pt, 2.5, spike_color);
            }
        }

        // 5. Draw interactive hover inspector line and tooltip badge
        if let Some((idx, ms, pt)) = hovered_info {
            painter.line_segment(
                [Pos2::new(pt.x, rect.min.y), Pos2::new(pt.x, rect.max.y)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
            );
            painter.circle_filled(pt, 3.5, Color32::WHITE);
            painter.circle_stroke(pt, 4.5, Stroke::new(1.0, Color32::from_rgb(0, 190, 230)));

            let instant_fps = if ms > 0.001 { 1000.0 / ms } else { 0.0 };
            let tooltip_text = format!("Frame #{}: {:.2} ms ({:.0} FPS)", idx, ms, instant_fps);
            let text_pos = Pos2::new(
                (pt.x + 8.0).min(rect.max.x - 145.0),
                (pt.y - 16.0).max(rect.min.y + 4.0),
            );

            painter.rect_filled(
                Rect::from_min_size(text_pos, Vec2::new(142.0, 17.0)),
                3.0,
                Color32::from_rgba_unmultiplied(18, 22, 30, 240),
            );
            painter.rect_stroke(
                Rect::from_min_size(text_pos, Vec2::new(142.0, 17.0)),
                3.0,
                Stroke::new(1.0, Color32::from_rgb(45, 52, 68)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                Pos2::new(text_pos.x + 5.0, text_pos.y + 2.5),
                egui::Align2::LEFT_TOP,
                tooltip_text,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(220, 230, 245),
            );
        }
    }

    ui.add_space(4.0);

    // 3. Pacing Summary Footer
    ui.horizontal(|ui| {
        let (spike_label_color, status_icon) = if stats.spikes_over_16ms == 0 {
            (Color32::from_rgb(0, 210, 160), "✓")
        } else if stats.spikes_over_16ms < 10 {
            (Color32::from_rgb(255, 185, 0), "⚠")
        } else {
            (Color32::from_rgb(235, 60, 60), "⚡")
        };

        ui.label(
            egui::RichText::new(format!(
                "{} Spikes (>16ms): {}  •  Stutter Rate: {:.1}%",
                status_icon, stats.spikes_over_16ms, stats.stutter_rate_percent
            ))
            .color(spike_label_color)
            .font(egui::FontId::proportional(11.0)),
        );
    });
}

/// Helper to render a small metric pill badge.
fn draw_metric_pill(ui: &mut egui::Ui, label: &str, value: &str, val_color: Color32) {
    let frame = egui::Frame::NONE
        .fill(Color32::from_rgb(14, 16, 22))
        .stroke(Stroke::new(1.0, Color32::from_rgb(32, 36, 46)))
        .corner_radius(egui::CornerRadius::same(3))
        .inner_margin(egui::Margin::symmetric(6, 4));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(egui::FontId::proportional(10.5))
                    .color(Color32::from_rgb(140, 145, 160)),
            );
            ui.label(
                egui::RichText::new(value)
                    .strong()
                    .font(egui::FontId::proportional(11.0))
                    .color(val_color),
            );
        });
    });
}

/// Helper to draw a horizontal milestone line with right-aligned label.
fn draw_milestone_line(
    painter: &egui::Painter,
    rect: Rect,
    ms: f32,
    max_ms: f32,
    label: &str,
    color: Color32,
) {
    if ms > max_ms {
        return;
    }
    let y = rect.max.y - (ms / max_ms) * rect.height();
    painter.line_segment(
        [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
        Stroke::new(1.0, color),
    );
    painter.text(
        Pos2::new(rect.max.x - 4.0, y - 2.0),
        egui::Align2::RIGHT_BOTTOM,
        label,
        egui::FontId::proportional(8.5),
        color,
    );
}

/// Helper to pick color for frame time value.
fn get_frametime_color(ms: f32) -> Color32 {
    if ms <= 8.33 {
        Color32::from_rgb(0, 210, 160) // Mint Emerald (120+ FPS)
    } else if ms <= 16.67 {
        Color32::from_rgb(0, 190, 230) // Cyan (60+ FPS)
    } else if ms <= 33.33 {
        Color32::from_rgb(255, 185, 0) // Amber Yellow (30-60 FPS)
    } else {
        Color32::from_rgb(235, 60, 60) // Bright Crimson Spike (<30 FPS)
    }
}

/// Helper to pick color for FPS value.
fn get_fps_color(fps: f32) -> Color32 {
    if fps >= 115.0 {
        Color32::from_rgb(0, 210, 160)
    } else if fps >= 55.0 {
        Color32::from_rgb(0, 190, 230)
    } else if fps >= 30.0 {
        Color32::from_rgb(255, 185, 0)
    } else {
        Color32::from_rgb(235, 60, 60)
    }
}