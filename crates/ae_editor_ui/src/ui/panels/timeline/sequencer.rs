// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};

impl EngineUi {
    /// Renders the internal content of the Animation Timeline Studio.
    /// Exposes skeletal animation transport controls (Play/Pause/Stop/Loop/Speed),
    /// interactive drag scrubber, time ruler markings, and active clip playback tracking.
    pub fn draw_timeline_content(
        ui: &mut egui::Ui,
        world: &hecs::World,
        selected_entity: Option<hecs::Entity>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.spacing_mut().item_spacing.y = 6.0;
        if let Some(entity) = selected_entity {
            if world.contains(entity) {
                if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                    let mut updated = (*player).clone();
                    let mut changed = false;

                    let duration = updated
                        .current_clip
                        .as_ref()
                        .map_or(1.0, |c| c.duration.max(0.1));

                    // 1. TRANSPORT CONTROLS BAR
                    ui.horizontal(|ui| {
                        if ui
                            .button(if updated.state == ae_animation::AnimationState::Playing {
                                "⏸ Pause"
                            } else {
                                "▶ Play"
                            })
                            .clicked()
                        {
                            updated.state =
                                if updated.state == ae_animation::AnimationState::Playing {
                                    ae_animation::AnimationState::Paused
                                } else {
                                    ae_animation::AnimationState::Playing
                                };
                            changed = true;
                        }
                        if ui.button("⏹ Stop").clicked() {
                            updated.state = ae_animation::AnimationState::Stopped;
                            updated.current_time = 0.0;
                            changed = true;
                        }

                        ui.separator();

                        ui.checkbox(&mut updated.looping, "🔁 Loop");
                        if updated.looping != player.looping {
                            changed = true;
                        }

                        ui.add(
                            egui::Slider::new(&mut updated.speed, -3.0..=3.0)
                                .text("Speed")
                                .suffix("x"),
                        );
                        if (updated.speed - player.speed).abs() > 0.001 {
                            changed = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.2}s / {:.2}s",
                                    updated.current_time, duration
                                ))
                                .monospace()
                                .color(egui::Color32::from_rgb(0, 200, 255)),
                            );
                        });
                    });

                    ui.separator();

                    // 2. TIMELINE SCRUBBER & RULER
                    let timeline_height = 36.0;
                    let (timeline_rect, scrubber_resp) = ui.allocate_exact_size(
                        egui::Vec2::new(ui.available_width(), timeline_height),
                        egui::Sense::click_and_drag(),
                    );

                    let painter = ui.painter_at(timeline_rect);

                    // Scrubber background
                    painter.rect_filled(
                        timeline_rect,
                        egui::CornerRadius::same(4),
                        egui::Color32::from_rgb(25, 25, 30),
                    );

                    if (scrubber_resp.dragged() || scrubber_resp.clicked())
                        && let Some(mouse_pos) = scrubber_resp.interact_pointer_pos()
                    {
                        let t = ((mouse_pos.x - timeline_rect.min.x) / timeline_rect.width())
                            .clamp(0.0, 1.0);
                        updated.current_time = t * duration;
                        changed = true;
                    }

                    let progress_ratio = (updated.current_time / duration).clamp(0.0, 1.0);
                    let fill_width = timeline_rect.width() * progress_ratio;
                    if fill_width > 0.0 {
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                timeline_rect.min,
                                egui::Vec2::new(fill_width, timeline_rect.height()),
                            ),
                            egui::CornerRadius::same(4),
                            egui::Color32::from_rgba_unmultiplied(0, 150, 255, 60),
                        );
                    }

                    for i in 0..=10 {
                        let frac = i as f32 / 10.0;
                        let x = timeline_rect.min.x + timeline_rect.width() * frac;
                        let is_major = i % 2 == 0;
                        let tick_h = if is_major { 8.0 } else { 4.0 };
                        painter.line_segment(
                            [
                                egui::Pos2::new(x, timeline_rect.min.y),
                                egui::Pos2::new(x, timeline_rect.min.y + tick_h),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
                        );
                        if is_major {
                            painter.text(
                                egui::Pos2::new(x + 2.0, timeline_rect.min.y + 10.0),
                                egui::Align2::LEFT_TOP,
                                format!("{:.1}s", frac * duration),
                                egui::FontId::proportional(9.0),
                                egui::Color32::from_gray(120),
                            );
                        }
                    }

                    let needle_x = timeline_rect.min.x + fill_width;
                    painter.line_segment(
                        [
                            egui::Pos2::new(needle_x, timeline_rect.min.y),
                            egui::Pos2::new(needle_x, timeline_rect.max.y),
                        ],
                        egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 220, 255)),
                    );

                    if changed {
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Selected entity does not have an AnimationPlayer component.");
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No entity selected. Select a 3D animated model to inspect animation timeline.");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    "No entity selected. Select a 3D animated model to inspect animation timeline.",
                );
            });
        }
    }
}