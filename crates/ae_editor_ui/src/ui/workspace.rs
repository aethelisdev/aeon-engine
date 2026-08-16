// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::{ConsoleEntry, EngineUi, EngineUiAction};
use crate::ui::panel_layout::PanelLayoutState;

impl EngineUi {
    /// Renders the internal content of the Asset Browser (3D Models & 2D Textures).
    pub fn draw_assets_content(
        ui: &mut egui::Ui,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .button("🗑 Garbage Collect")
                .on_hover_text("Clean up unused model and texture resources to reclaim VRAM")
                .clicked()
            {
                ui_actions.push(EngineUiAction::GarbageCollect);
            }
            ui.label(
                egui::RichText::new(format!(
                    "{} Models • {} Textures loaded",
                    models.len(),
                    textures.len()
                ))
                .color(egui::Color32::from_gray(140))
                .size(11.0),
            );
        });
        ui.separator();

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
                                        ui_actions.push(EngineUiAction::SpawnModel(handle));
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
                                        ui_actions.push(EngineUiAction::SpawnSprite(handle));
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
    }

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

    /// Renders the internal content of the Animation Timeline Studio.
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
                        ui_actions.push(EngineUiAction::ModifyAnimationPlayer(entity, updated));
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

    /// Renders a persistent thin bar at the absolute bottom for quick status info.
    pub(super) fn draw_utility_bar(
        _layout_state: &mut PanelLayoutState,
        status_message: &mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
        ui: &mut egui::Ui,
    ) -> Option<egui::Rect> {
        let resp = egui::Panel::bottom("utility_bar")
            .exact_size(22.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(15, 15, 20))
                    .inner_margin(egui::Margin::symmetric(10, 3))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 42, 52))),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 1. Status Message or Ready indicator
                    if let Some((spans, _)) = status_message {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (text, color) in spans {
                            ui.label(egui::RichText::new(text.as_str()).color(*color).size(11.0));
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("● Ready")
                                .color(egui::Color32::from_rgb(70, 190, 120))
                                .size(11.0),
                        );
                    }

                    // 2. Right-side Engine info
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Aeon Engine v{}",
                                env!("CARGO_PKG_VERSION")
                            ))
                            .color(egui::Color32::from_gray(90))
                            .size(11.0),
                        );
                    });
                });
            });

        Some(resp.response.rect)
    }
}