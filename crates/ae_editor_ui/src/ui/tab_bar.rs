// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Unified tab bar widget with smooth corner roundings,
/// top accent highlights, and  panel integration.
use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};

/// Descriptor for a single tab in the unified tab bar.
pub struct EditorTab<'a> {
    pub id: usize,
    pub icon: &'a str,
    pub title: &'a str,
}

impl<'a> EditorTab<'a> {
    /// Creates a new EditorTab definition with an integer ID, unicode icon, and display title.
    pub fn new(id: usize, icon: &'a str, title: &'a str) -> Self {
        Self { id, icon, title }
    }
}

/// Renders a unified tab bar where the active tab  merges with the panel content below.
/// Returns `true` if the user clicked a different tab.
pub fn draw_tab_bar(ui: &mut egui::Ui, current_tab: &mut usize, tabs: &[EditorTab]) -> bool {
    let mut changed = false;
    let tab_height = 26.0;
    let tab_spacing = 3.0;

    // Calculate exact width needed by tabs
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let mut required_width = 8.0;
    for tab in tabs {
        let text = if tab.icon.is_empty() {
            tab.title.to_string()
        } else {
            format!("{} {}", tab.icon, tab.title)
        };
        let text_layout = ui
            .painter()
            .layout_no_wrap(text, font_id.clone(), Color32::WHITE);
        let tab_width = (text_layout.size().x + 22.0).max(75.0);
        required_width += tab_width + tab_spacing;
    }

    let (rect, _response) =
        ui.allocate_exact_size(Vec2::new(required_width, tab_height), egui::Sense::hover());

    let painter = ui.painter_at(rect);

    let baseline_y = rect.max.y;
    let panel_bg = Color32::from_rgb(20, 20, 25);
    let border_color = Color32::from_rgb(45, 48, 60);
    let bar_bg = Color32::from_rgb(15, 15, 20);

    // Draw tab bar container background
    painter.rect_filled(rect, CornerRadius::ZERO, bar_bg);

    let mut current_x = rect.min.x + 4.0;

    for tab in tabs {
        let text = if tab.icon.is_empty() {
            tab.title.to_string()
        } else {
            format!("{} {}", tab.icon, tab.title)
        };

        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let text_layout = painter.layout_no_wrap(text.clone(), font_id.clone(), Color32::WHITE);
        let tab_width = (text_layout.size().x + 22.0).max(75.0);
        let tab_rect = Rect::from_min_size(
            Pos2::new(current_x, rect.min.y + 2.0),
            Vec2::new(tab_width, tab_height - 2.0),
        );

        let is_active = *current_tab == tab.id;
        let tab_id = ui.make_persistent_id(format!("editor_tab_{}_{}", tab.id, tab.title));
        let tab_resp = ui.interact(tab_rect, tab_id, egui::Sense::click());

        if tab_resp.clicked() && !is_active {
            *current_tab = tab.id;
            changed = true;
        }

        let is_hovered = tab_resp.hovered();

        if is_active {
            // Active Tab:  merge with bottom panel
            painter.rect_filled(
                tab_rect,
                CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                },
                panel_bg,
            );
            // Top accent cyan highlight line
            let accent_color = Color32::from_rgb(0, 195, 255);
            painter.line_segment(
                [
                    Pos2::new(tab_rect.min.x + 2.0, tab_rect.min.y),
                    Pos2::new(tab_rect.max.x - 2.0, tab_rect.min.y),
                ],
                Stroke::new(2.0, accent_color),
            );
            // Left & Right subtle borders
            painter.line_segment(
                [
                    Pos2::new(tab_rect.min.x, tab_rect.min.y + 4.0),
                    Pos2::new(tab_rect.min.x, tab_rect.max.y),
                ],
                Stroke::new(1.0, border_color),
            );
            painter.line_segment(
                [
                    Pos2::new(tab_rect.max.x, tab_rect.min.y + 4.0),
                    Pos2::new(tab_rect.max.x, tab_rect.max.y),
                ],
                Stroke::new(1.0, border_color),
            );
        } else {
            // Inactive Tab
            let inactive_fill = if is_hovered {
                Color32::from_rgb(28, 30, 38)
            } else {
                Color32::from_rgb(18, 19, 24)
            };
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(tab_rect.min.x, tab_rect.min.y + 3.0),
                    Vec2::new(tab_width, tab_height - 5.0),
                ),
                CornerRadius {
                    nw: 4,
                    ne: 4,
                    sw: 0,
                    se: 0,
                },
                inactive_fill,
            );
            // Inactive tab bottom border line
            painter.line_segment(
                [
                    Pos2::new(tab_rect.min.x, baseline_y),
                    Pos2::new(tab_rect.max.x, baseline_y),
                ],
                Stroke::new(1.0, border_color),
            );
        }

        // Text & Icon
        let text_color = if is_active {
            Color32::from_rgb(240, 245, 255)
        } else if is_hovered {
            Color32::from_rgb(200, 205, 220)
        } else {
            Color32::from_rgb(140, 145, 160)
        };

        let text_pos = Pos2::new(
            tab_rect.min.x + (tab_width - text_layout.size().x) * 0.5,
            tab_rect.min.y
                + (tab_rect.height() - text_layout.size().y) * 0.5
                + if is_active { 0.0 } else { 1.0 },
        );

        painter.text(text_pos, egui::Align2::LEFT_TOP, text, font_id, text_color);

        current_x += tab_width + tab_spacing;
    }

    // Remaining baseline separator to the right edge
    if current_x < rect.max.x {
        painter.line_segment(
            [
                Pos2::new(current_x, baseline_y),
                Pos2::new(rect.max.x, baseline_y),
            ],
            Stroke::new(1.0, border_color),
        );
    }

    changed
}