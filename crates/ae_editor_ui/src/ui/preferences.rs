// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::EngineUi;

pub mod editor;
pub mod graphics;
pub mod modules;

impl EngineUi {
    /// Renders the Preferences window of the engine editor.
    pub(super) fn draw_preferences_window(
        show_preferences: &mut bool,
        preferences_tab: &mut u8,
        ctx: &egui::Context,
        graphics_settings: &mut ae_renderer::graphics_settings::GraphicsSettings,
        snapping_settings: &mut ae_editor::snapping::SnapSettings,
        enable_live_updates: &mut bool,
        editor_config: &mut ae_editor::editor_state::EditorConfig,
        enabled_modules: &std::collections::HashSet<ae_core::modules::EngineModule>,
        ui_actions: &mut Vec<super::EngineUiAction>,
        status_message: &mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
    ) -> Option<egui::Rect> {
        let mut show_pref = *show_preferences;
        let mut rect = None;
        if show_pref {
            let response = egui::Window::new("⚙  Preferences")
                .id(egui::Id::new("preferences_window"))
                .collapsible(false)
                .resizable(true)
                .default_size([700.0, 520.0])
                .min_size([550.0, 400.0])
                .open(&mut show_pref)
                .show(ctx, |ui| {
                    let pref_tab = preferences_tab;
                    let gs = &mut *graphics_settings;

                    ui.horizontal(|ui| {
                        // ── LEFT SIDEBAR ──────────────────────
                        let sidebar_w = 140.0;
                        ui.allocate_ui_with_layout(
                            egui::vec2(sidebar_w, ui.available_height()),
                            egui::Layout::top_down_justified(egui::Align::Min),
                            |ui| {
                                // Paint sidebar background for the FULL height of the window content
                                let rect = ui.max_rect();
                                ui.painter().rect_filled(
                                    rect,
                                    0.0,
                                    egui::Color32::from_rgb(22, 22, 28),
                                );

                                ui.add_space(12.0);

                                let tabs = [
                                    ("General", 0u8),
                                    ("Graphics", 1u8),
                                    ("Editor", 2u8),
                                    ("Navigation", 3u8),
                                    ("Input", 7u8),
                                    ("Keymap", 4u8),
                                    ("System", 5u8),
                                    ("Add-ons", 6u8),
                                    ("Modules", 9u8),
                                    ("Experimental", 8u8),
                                ];

                                for &(label, idx) in &tabs {
                                    let selected = *pref_tab == idx;
                                    let _fg = if selected {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::from_rgb(140, 140, 155)
                                    };
                                    let bg = if selected {
                                        egui::Color32::from_rgb(45, 70, 120)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    let (rect, resp) = ui.allocate_at_least(
                                        egui::vec2(sidebar_w, 32.0),
                                        egui::Sense::click(),
                                    );
                                    if resp.clicked() {
                                        *pref_tab = idx;
                                    }

                                    if ui.is_rect_visible(rect) {
                                        if selected {
                                            ui.painter().rect_filled(rect, 4.0, bg);
                                            ui.painter().rect_filled(
                                                egui::Rect::from_min_size(
                                                    rect.left_top(),
                                                    egui::vec2(4.0, rect.height()),
                                                ),
                                                2.0,
                                                egui::Color32::from_rgb(77, 163, 255),
                                            );
                                        } else if resp.hovered() {
                                            ui.painter().rect_filled(
                                                rect,
                                                4.0,
                                                egui::Color32::from_rgba_premultiplied(
                                                    255, 255, 255, 15,
                                                ),
                                            );
                                        }

                                        let text_color = if selected {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::from_rgb(160, 160, 175)
                                        };
                                        ui.painter().text(
                                            rect.left_center() + egui::vec2(16.0, 0.0),
                                            egui::Align2::LEFT_CENTER,
                                            label,
                                            egui::FontId::proportional(13.0),
                                            text_color,
                                        );
                                    }
                                }
                            },
                        );

                        ui.separator();

                        // ── RIGHT CONTENT AREA ──────────────────────────
                        ui.vertical(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.add_space(8.0);

                            egui::ScrollArea::vertical()
                                .id_salt("pref_scroll")
                                .auto_shrink([false; 2])
                                .show(ui, |ui| match *pref_tab {
                                    0 => {
                                        ui.heading("General");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Global engine settings, language, and theme.");
                                    }
                                    1 => {
                                        graphics::draw(ui, gs);
                                    }
                                    2 => {
                                        editor::draw(
                                            ui,
                                            snapping_settings,
                                            enable_live_updates,
                                            editor_config,
                                            status_message,
                                        );
                                    }
                                    3 => {
                                        ui.heading("Navigation");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Orbit and viewport navigation settings.");
                                    }
                                    4 => {
                                        ui.heading("Keymap");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Manage editor keyboard shortcuts.");
                                    }
                                    5 => {
                                        ui.heading("System");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Hardware and script engine engine info.");
                                    }
                                    6 => {
                                        ui.heading("Add-ons");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Manage Aeon Engine extensions.");
                                    }
                                    7 => {
                                        ui.heading("Input");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Controller and mouse input configuration.");
                                    }
                                    8 => {
                                        ui.heading("Experimental");
                                        ui.separator();
                                        ui.add_space(10.0);
                                        ui.label("Trial features (Use with caution).");
                                    }
                                    9 => {
                                        modules::draw(ui, enabled_modules, ui_actions);
                                    }
                                    _ => {}
                                });
                        });
                    });
                });
            *show_preferences = show_pref;
            rect = response.map(|r| r.response.rect);
        }
        rect
    }
}