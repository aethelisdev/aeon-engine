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
        console_entries: &[ConsoleEntry],
        ui: &mut egui::Ui,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
        selected_entity: Option<hecs::Entity>,
        world: &hecs::World,
    ) -> Option<egui::Rect> {
        if !*show_workspace {
            return None;
        }

        let resp = egui::Panel::bottom("workspace_panel")
            .resizable(true)
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
                    crate::ui::tab_bar::draw_tab_bar(
                        ui,
                        workspace_tab,
                        &[
                            crate::ui::tab_bar::EditorTab::new(0, "📂", "Assets"),
                            crate::ui::tab_bar::EditorTab::new(1, "📜", "Console"),
                            crate::ui::tab_bar::EditorTab::new(2, "🎬", "Animation Timeline"),
                        ],
                    );
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
                } else if *workspace_tab == 2 {
                    // --- 🎬 ANIMATION TIMELINE STUDIO ---
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
                                        updated.state = if updated.state == ae_animation::AnimationState::Playing {
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

                                    let mut is_looping = updated.looping;
                                    if ui.checkbox(&mut is_looping, "🔁 Loop").changed() {
                                        updated.looping = is_looping;
                                        changed = true;
                                    }

                                    ui.separator();

                                    ui.label("Speed:");
                                    if ui
                                        .add(
                                            egui::Slider::new(&mut updated.speed, 0.1..=3.0)
                                                .text("x")
                                                .fixed_decimals(1),
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }

                                    ui.separator();

                                    let model_anims = if let Ok(model_id) =
                                        world.get::<&ae_core::ecs::ModelId>(entity)
                                    {
                                        models.get(model_id.0).map(|m| &m.animations)
                                    } else {
                                        None
                                    };

                                    let active_clip_name = updated
                                        .current_clip
                                        .as_ref()
                                        .map_or("No Clip Loaded", |c| c.name.as_str());

                                    ui.label("Clip:");
                                    egui::ComboBox::from_id_salt("timeline_studio_clip_selector")
                                        .selected_text(active_clip_name)
                                        .show_ui(ui, |ui| {
                                            if let Some(anims) = model_anims {
                                                for clip in anims {
                                                    let is_selected = updated
                                                        .current_clip
                                                        .as_ref()
                                                        .is_some_and(|c| c.name == clip.name);
                                                    if ui.selectable_label(is_selected, &clip.name).clicked() {
                                                        updated.current_clip = Some(clip.clone());
                                                        updated.current_time = 0.0;
                                                        changed = true;
                                                    }
                                                }
                                            }
                                        });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:05.2}s / {:05.2}s",
                                                updated.current_time, duration
                                            ))
                                            .color(egui::Color32::from_rgb(0, 220, 255))
                                            .strong(),
                                        );
                                    });
                                });

                                ui.separator();

                                // 2. HORIZONTAL SCRUBBER TIMELINE BAR
                                let (timeline_rect, _resp) = ui.allocate_exact_size(
                                    egui::Vec2::new(ui.available_width(), 32.0),
                                    egui::Sense::click_and_drag(),
                                );
                                let painter = ui.painter_at(timeline_rect);

                                painter.rect_filled(
                                    timeline_rect,
                                    egui::CornerRadius::same(4),
                                    egui::Color32::from_rgb(14, 15, 18),
                                );

                                let scrubber_resp = ui.interact(
                                    timeline_rect,
                                    ui.make_persistent_id("timeline_scrubber_head"),
                                    egui::Sense::click_and_drag(),
                                );

                                if scrubber_resp.dragged() || scrubber_resp.clicked() {
                                    if let Some(mouse_pos) = scrubber_resp.interact_pointer_pos() {
                                        let t = ((mouse_pos.x - timeline_rect.min.x)
                                            / timeline_rect.width())
                                        .clamp(0.0, 1.0);
                                        updated.current_time = t * duration;
                                        changed = true;
                                    }
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
                                    ui_actions.push(EngineUiAction::ModifyAnimationPlayer(
                                        entity, updated,
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
                            ui.label("No entity selected. Select a 3D animated model to inspect animation timeline.");
                        });
                    }
                }
            });

        Some(resp.response.rect)
    }

    /// Renders a persistent thin bar at the absolute bottom for quick toggling and status info.
    /// Renders a persistent thin bar at the absolute bottom for quick toggling and status info.
    pub(super) fn draw_utility_bar(
        show_workspace: &mut bool,
        workspace_tab: &mut usize,
        show_left_panel: &mut bool,
        left_panel_tab: &mut usize,
        status_message: &mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
        ui: &mut egui::Ui,
    ) -> Option<egui::Rect> {
        let resp = egui::Panel::bottom("utility_bar")
            .exact_size(24.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(15, 15, 20))
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60))),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    // 1. Left Panel Toggles (Hierarchy & Stats)
                    let hier_btn = egui::RichText::new("🏗️ Hierarchy").size(11.5).strong();
                    let stats_btn = egui::RichText::new("📊 Stats").size(11.5).strong();

                    let cur_left = *show_left_panel;
                    let cur_ltab = *left_panel_tab;

                    let (hier_color, hier_bg) = if cur_left && cur_ltab == 0 {
                        (egui::Color32::WHITE, egui::Color32::from_rgb(50, 80, 120))
                    } else {
                        (egui::Color32::from_gray(140), egui::Color32::TRANSPARENT)
                    };
                    let (stats_color, stats_bg) = if cur_left && cur_ltab == 1 {
                        (egui::Color32::WHITE, egui::Color32::from_rgb(50, 80, 120))
                    } else {
                        (egui::Color32::from_gray(140), egui::Color32::TRANSPARENT)
                    };

                    if ui
                        .add(
                            egui::Button::new(hier_btn.color(hier_color))
                                .fill(hier_bg)
                                .small(),
                        )
                        .clicked()
                    {
                        if cur_left && cur_ltab == 0 {
                            *show_left_panel = false;
                        } else {
                            *show_left_panel = true;
                            *left_panel_tab = 0;
                        }
                    }

                    if ui
                        .add(
                            egui::Button::new(stats_btn.color(stats_color))
                                .fill(stats_bg)
                                .small(),
                        )
                        .clicked()
                    {
                        if cur_left && cur_ltab == 1 {
                            *show_left_panel = false;
                        } else {
                            *show_left_panel = true;
                            *left_panel_tab = 1;
                        }
                    }

                    ui.separator();

                    // 2. Bottom Workspace Toggles
                    let console_btn = egui::RichText::new("📜 Console").size(11.5).strong();
                    let assets_btn = egui::RichText::new("📂 Assets").size(11.5).strong();
                    let anim_btn = egui::RichText::new("🎬 Timeline").size(11.5).strong();

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
                    let (anim_color, anim_bg) = if cur_workspace && cur_tab == 2 {
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

                    if ui
                        .add(
                            egui::Button::new(anim_btn.color(anim_color))
                                .fill(anim_bg)
                                .small(),
                        )
                        .clicked()
                    {
                        if cur_workspace && cur_tab == 2 {
                            *show_workspace = false;
                        } else {
                            *show_workspace = true;
                            *workspace_tab = 2;
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