// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::EngineUi;

pub mod editor;
pub mod graphics;
pub mod modules;

/// Parameters for drawing the editor preferences window.
pub struct PreferencesWindowParams<'a> {
    pub show_preferences: &'a mut bool,
    pub preferences_tab: &'a mut u8,
    pub ctx: &'a egui::Context,
    pub graphics_settings: &'a mut ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping_settings: &'a mut ae_editor::snapping::SnapSettings,
    pub enable_live_updates: &'a mut bool,
    pub editor_config: &'a mut ae_editor::editor_state::EditorConfig,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub ui_actions: &'a mut Vec<super::EngineUiAction>,
    pub status_message: &'a mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
}

impl EngineUi {
    /// Renders the floating modal/window for configuring project and engine preferences.
    /// Supports the following configurations:
    ///   - `General`: Live updates toggle, layout reset.
    ///   - `Snapping`: Movement, rotation, scale grids.
    ///   - `Graphics`: Directly modifies `GraphicsSettings` (Shadows, MSAA, Bloom).
    ///   - `Navigation`: Tab persistence via `preferences_tab`.
    /// - UI PATTERN: Split-pane layout with a vertical sidebar and scrollable content area.
    /// - PERSISTENCE: Modifies the `graphics_settings` which are usually passed from the main loop.
    pub(super) fn draw_preferences_window(
        params: PreferencesWindowParams<'_>,
    ) -> Option<egui::Rect> {
        let show_preferences = params.show_preferences;
        let preferences_tab = params.preferences_tab;
        let ctx = params.ctx;
        let graphics_settings = params.graphics_settings;
        let snapping_settings = params.snapping_settings;
        let enable_live_updates = params.enable_live_updates;
        let editor_config = params.editor_config;
        let enabled_modules = params.enabled_modules;
        let ui_actions = params.ui_actions;
        let status_message = params.status_message;

        let mut show_pref = *show_preferences;
        let mut rect = None;
        if show_pref {
            let response = egui::Window::new("Preferences")
                .id(egui::Id::new("preferences_window"))
                .title_bar(false)
                .collapsible(false)
                .resizable(true)
                .default_size([720.0, 520.0])
                .min_size([550.0, 400.0])
                .frame(
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(20, 20, 25))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60)))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::ZERO)
                        .shadow(egui::Shadow {
                            offset: [0, 8],
                            blur: 24,
                            spread: 0,
                            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
                        }),
                )
                .show(ctx, |ui| {
                    let pref_tab = preferences_tab;
                    let gs = &mut *graphics_settings;

                    // ── 1. CUSTOM SLEEK HEADER BAR (Dark Header) ──
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(15, 15, 20))
                        .inner_margin(egui::Margin::symmetric(14, 8))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60)))
                        .corner_radius(egui::CornerRadius {
                            nw: 8,
                            ne: 8,
                            sw: 0,
                            se: 0,
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("⚙  Preferences")
                                        .strong()
                                        .size(13.0)
                                        .color(egui::Color32::from_gray(225)),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("✖")
                                                        .size(11.0)
                                                        .color(egui::Color32::from_gray(160)),
                                                )
                                                .fill(egui::Color32::TRANSPARENT)
                                                .frame(false),
                                            )
                                            .on_hover_text("Close Preferences")
                                            .clicked()
                                        {
                                            show_pref = false;
                                        }
                                    },
                                );
                            });
                        });

                    // ── 2. WINDOW BODY (SIDEBAR + CONTENT) ──
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // ── LEFT SIDEBAR ──────────────────────
                                let sidebar_w = 140.0;
                                ui.allocate_ui_with_layout(
                                    egui::vec2(sidebar_w, ui.available_height()),
                                    egui::Layout::top_down_justified(egui::Align::Min),
                                    |ui| {
                                        ui.add_space(4.0);

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
                                            let bg = if selected {
                                                egui::Color32::from_rgb(0, 60, 80)
                                            } else {
                                                egui::Color32::TRANSPARENT
                                            };

                                            let (tab_rect, resp) = ui.allocate_at_least(
                                                egui::vec2(sidebar_w, 32.0),
                                                egui::Sense::click(),
                                            );
                                            if resp.clicked() {
                                                *pref_tab = idx;
                                            }

                                            if ui.is_rect_visible(tab_rect) {
                                                if selected {
                                                    ui.painter().rect_filled(tab_rect, 4.0, bg);
                                                    ui.painter().rect_filled(
                                                        egui::Rect::from_min_size(
                                                            tab_rect.left_top(),
                                                            egui::vec2(4.0, tab_rect.height()),
                                                        ),
                                                        2.0,
                                                        egui::Color32::from_rgb(0, 229, 255),
                                                    );
                                                } else if resp.hovered() {
                                                    ui.painter().rect_filled(
                                                        tab_rect,
                                                        4.0,
                                                        egui::Color32::from_rgb(28, 32, 42),
                                                    );
                                                }

                                                let text_color = if selected {
                                                    egui::Color32::from_rgb(0, 229, 255)
                                                } else if resp.hovered() {
                                                    egui::Color32::WHITE
                                                } else {
                                                    egui::Color32::from_rgb(160, 160, 175)
                                                };
                                                ui.painter().text(
                                                    tab_rect.left_center() + egui::vec2(16.0, 0.0),
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
                                    ui.add_space(4.0);

                                    egui::ScrollArea::vertical()
                                        .id_salt("pref_scroll")
                                        .auto_shrink([false; 2])
                                        .show(ui, |ui| match *pref_tab {
                                            0 => {
                                                ui.heading("General & Interface");
                                                ui.separator();
                                                ui.add_space(8.0);
                                                ui.label(
                                                    "Global engine settings, language, and display scaling.",
                                                );
                                                ui.add_space(12.0);

                                                ui.group(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("🔍 Display & UI Scale")
                                                            .strong()
                                                            .color(egui::Color32::WHITE),
                                                    );
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        "Adjust interface scale for different monitor resolutions (or use Ctrl + / Ctrl - shortcuts):",
                                                    );
                                                    ui.add_space(8.0);

                                                    let scales: [(f32, &str); 7] = [
                                                        (0.75, "75%"),
                                                        (0.80, "80%"),
                                                        (0.90, "90%"),
                                                        (1.00, "100% (Default)"),
                                                        (1.10, "110%"),
                                                        (1.25, "125%"),
                                                        (1.50, "150%"),
                                                    ];

                                                    let current_zoom = ctx.zoom_factor();
                                                    let selected_text = scales
                                                        .iter()
                                                        .find(|(val, _)| (current_zoom - *val).abs() < 0.01)
                                                        .map(|(_, l)| *l)
                                                        .unwrap_or_else(|| {
                                                            if (current_zoom - 1.0).abs() < 0.01 {
                                                                "100% (Default)"
                                                            } else {
                                                                ""
                                                            }
                                                        });
                                                    let display_text = if selected_text.is_empty() {
                                                        format!("{:.0}%", (current_zoom * 100.0).round())
                                                    } else {
                                                        selected_text.to_string()
                                                    };

                                                    egui::ComboBox::from_id_salt("ui_scale_combo")
                                                        .width(220.0)
                                                        .selected_text(display_text)
                                                        .show_ui(ui, |ui| {
                                                            for (scale_val, label) in scales {
                                                                let is_selected = (current_zoom - scale_val).abs() < 0.01;
                                                                if ui.selectable_label(is_selected, label).clicked() {
                                                                    ui_actions.push(
                                                                        crate::ui::EngineUiAction::SetUiScale(
                                                                            scale_val,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        });
                                                });
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
                                                ui.label("Hardware and script engine info.");
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
                                                ui.label(
                                                    "Controller and mouse input configuration.",
                                                );
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
                });

            rect = response.map(|r| r.response.rect);
        }
        *show_preferences = show_pref;
        rect
    }
}