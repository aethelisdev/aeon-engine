// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified and draggable tab bar widget with smooth corner roundings,
//! in-bar sliding reordering, cross-zone detachment, and  panel integration.
//!

use super::panel_layout::{PanelId, PanelLayoutState, PanelZone, TabDragState};
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

/// Renders a dynamic, draggable tab bar that supports in-bar sliding reorder and cross-zone relocation.
/// Implements Chrome-like tab behavior:
/// - Horizontal dragging inside the bar slides the tab smoothly within the bar.
/// - Dragging away vertically or to another zone detaches the tab, displaying a floating preview badge.
/// Returns `true` if the active tab changed.
pub fn draw_draggable_tab_bar(
    ui: &mut egui::Ui,
    zone: PanelZone,
    layout_state: &mut PanelLayoutState,
    tab_drag_state: &mut Option<TabDragState>,
) -> bool {
    let mut changed = false;
    let tab_height = 26.0;
    let tab_spacing = 3.0;

    let tabs: Vec<PanelId> = layout_state.get_zone_tabs(zone).to_vec();
    let active_tab = layout_state.get_active_tab(zone);

    // Calculate total required width for all tabs in this zone
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let mut required_width = 8.0;
    let mut tab_widths = Vec::with_capacity(tabs.len());

    for panel in &tabs {
        let text = format!("{} {}", panel.icon(), panel.title());
        let text_layout = ui
            .painter()
            .layout_no_wrap(text, font_id.clone(), Color32::WHITE);
        let tab_width = (text_layout.size().x + 22.0).max(75.0);
        tab_widths.push(tab_width);
        required_width += tab_width + tab_spacing;
    }

    let allocated_width = required_width.max(120.0);
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(allocated_width, tab_height), egui::Sense::hover());

    let painter = ui.painter_at(rect);
    let baseline_y = rect.max.y;
    let panel_bg = Color32::from_rgb(20, 20, 25);
    let border_color = Color32::from_rgb(45, 48, 60);
    let bar_bg = Color32::from_rgb(15, 15, 20);

    // Draw tab bar container background
    painter.rect_filled(rect, CornerRadius::ZERO, bar_bg);

    let mouse_pos = ui.input(|i| i.pointer.hover_pos());
    let is_pointer_in_bar =
        mouse_pos.is_some_and(|p| rect.expand2(Vec2::new(8.0, 14.0)).contains(p));
    let is_pointer_far_y = mouse_pos.is_some_and(|p| (p.y - rect.center().y).abs() > 20.0);

    // Insertion slot boundary X-coordinates for drag-and-drop indicator
    let mut slot_x_coords = Vec::with_capacity(tabs.len() + 1);
    let mut current_x = rect.min.x + 4.0;
    slot_x_coords.push(current_x);

    for (idx, panel) in tabs.iter().enumerate() {
        let tab_width = tab_widths[idx];
        let base_tab_rect = Rect::from_min_size(
            Pos2::new(current_x, rect.min.y + 2.0),
            Vec2::new(tab_width, tab_height - 2.0),
        );

        let is_active = active_tab == Some(*panel);
        let tab_id = ui.make_persistent_id(format!("draggable_tab_{:?}_{}", zone, panel.title()));
        let tab_resp = ui.interact(base_tab_rect, tab_id, egui::Sense::click_and_drag());

        // Check if this specific tab is currently being dragged
        let is_this_tab_dragging = tab_drag_state
            .as_ref()
            .is_some_and(|d| d.panel_id == *panel && d.source_zone == zone);

        // 1. Drag start handling
        if tab_resp.drag_started()
            && tab_drag_state.is_none()
            && let Some(origin) = mouse_pos
        {
            *tab_drag_state = Some(TabDragState::new(*panel, zone, idx, origin));
        }

        // Update detachment state if currently dragging this tab
        if is_this_tab_dragging && let Some(drag) = tab_drag_state.as_mut() {
            drag.is_detached = is_pointer_far_y;
        }

        let is_detached = tab_drag_state
            .as_ref()
            .is_some_and(|d| d.panel_id == *panel && d.is_detached);

        // 2. Click handling (only when not dragging)
        if tab_resp.clicked() && !is_active && !is_this_tab_dragging {
            layout_state.set_active_tab(zone, *panel);
            changed = true;
        }

        // Calculate rendered tab rect (in-bar sliding vs static)
        let draw_tab_rect = if is_this_tab_dragging
            && !is_detached
            && let (Some(drag), Some(pos)) = (tab_drag_state.as_ref(), mouse_pos)
        {
            let max_left = rect.min.x + 4.0 - base_tab_rect.min.x;
            let max_right = rect.max.x - 4.0 - base_tab_rect.max.x;
            let slide_dx = (pos.x - drag.drag_origin.x).clamp(max_left, max_right);
            base_tab_rect.translate(Vec2::new(slide_dx, 0.0))
        } else {
            base_tab_rect
        };

        let is_hovered = tab_resp.hovered();

        // 3. Tab Visual Styling
        let tab_title = format!("{} {}", panel.icon(), panel.title());
        let text_layout =
            painter.layout_no_wrap(tab_title.clone(), font_id.clone(), Color32::WHITE);

        if is_detached {
            // Detached Tab Placeholder (Ghosted in the source bar)
            painter.rect_filled(
                draw_tab_rect,
                CornerRadius::same(5),
                Color32::from_rgba_premultiplied(18, 20, 26, 120),
            );
            painter.rect_stroke(
                draw_tab_rect,
                CornerRadius::same(5),
                Stroke::new(1.0, Color32::from_rgba_premultiplied(60, 65, 80, 160)),
                egui::StrokeKind::Inside,
            );
            let text_pos = Pos2::new(
                draw_tab_rect.min.x + (tab_width - text_layout.size().x) * 0.5,
                draw_tab_rect.min.y + (tab_height - text_layout.size().y) * 0.5,
            );
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                tab_title,
                font_id.clone(),
                Color32::from_rgba_premultiplied(90, 95, 110, 150),
            );
        } else if is_this_tab_dragging {
            // In-Bar Sliding Tab (Elevated highlight)
            painter.rect_filled(
                draw_tab_rect,
                CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                },
                Color32::from_rgb(26, 30, 42),
            );
            painter.rect_stroke(
                draw_tab_rect,
                CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                },
                Stroke::new(1.5, Color32::from_rgb(0, 220, 255)),
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    Pos2::new(draw_tab_rect.min.x, draw_tab_rect.min.y),
                    Pos2::new(draw_tab_rect.max.x, draw_tab_rect.min.y),
                ],
                Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
            );
            let text_pos = Pos2::new(
                draw_tab_rect.min.x + (tab_width - text_layout.size().x) * 0.5,
                draw_tab_rect.min.y + (tab_height - text_layout.size().y) * 0.5,
            );
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                tab_title,
                font_id.clone(),
                Color32::WHITE,
            );
        } else if is_active {
            // Active Tab:  merge with panel
            painter.rect_filled(
                draw_tab_rect,
                CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                },
                panel_bg,
            );
            painter.line_segment(
                [
                    Pos2::new(draw_tab_rect.min.x, draw_tab_rect.min.y),
                    Pos2::new(draw_tab_rect.max.x, draw_tab_rect.min.y),
                ],
                Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
            );
            let text_pos = Pos2::new(
                draw_tab_rect.min.x + (tab_width - text_layout.size().x) * 0.5,
                draw_tab_rect.min.y + (tab_height - text_layout.size().y) * 0.5,
            );
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                tab_title,
                font_id.clone(),
                Color32::from_rgb(240, 245, 255),
            );
        } else {
            // Inactive Tab
            let tab_fill = if is_hovered {
                Color32::from_rgb(32, 35, 46)
            } else {
                Color32::from_rgb(18, 19, 24)
            };
            painter.rect_filled(
                draw_tab_rect,
                CornerRadius {
                    nw: 5,
                    ne: 5,
                    sw: 0,
                    se: 0,
                },
                tab_fill,
            );
            painter.line_segment(
                [
                    Pos2::new(draw_tab_rect.min.x, baseline_y),
                    Pos2::new(draw_tab_rect.max.x, baseline_y),
                ],
                Stroke::new(1.0, border_color),
            );
            let text_color = if is_hovered {
                Color32::from_rgb(200, 205, 220)
            } else {
                Color32::from_rgb(140, 145, 160)
            };
            let text_pos = Pos2::new(
                draw_tab_rect.min.x + (tab_width - text_layout.size().x) * 0.5,
                draw_tab_rect.min.y + (tab_height - text_layout.size().y) * 0.5,
            );
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                tab_title,
                font_id.clone(),
                text_color,
            );
        }

        current_x += tab_width + tab_spacing;
        slot_x_coords.push(current_x);
    }

    // Right tail baseline border line
    if current_x < rect.max.x {
        painter.line_segment(
            [
                Pos2::new(current_x, baseline_y),
                Pos2::new(rect.max.x, baseline_y),
            ],
            Stroke::new(1.0, border_color),
        );
    }

    // 4. Drag-and-Drop Slot Hover Detection & Indicator
    if let (Some(drag), Some(pos)) = (tab_drag_state.as_mut(), mouse_pos)
        && is_pointer_in_bar
    {
        // Find closest slot index based on mouse X position
        let mut best_slot = 0;
        let mut min_dist = f32::MAX;
        for (slot_idx, &slot_x) in slot_x_coords.iter().enumerate() {
            let dist = (pos.x - slot_x).abs();
            if dist < min_dist {
                min_dist = dist;
                best_slot = slot_idx;
            }
        }

        drag.hovered_zone = Some(zone);
        drag.hovered_index = best_slot;

        // Draw bright cyan insertion vertical indicator line
        if best_slot < slot_x_coords.len() {
            let indicator_x = slot_x_coords[best_slot];
            painter.line_segment(
                [
                    Pos2::new(indicator_x, rect.min.y + 2.0),
                    Pos2::new(indicator_x, rect.max.y - 2.0),
                ],
                Stroke::new(3.0, Color32::from_rgb(0, 220, 255)),
            );
        }
    }

    changed
}

/// Legacy tab bar widget for non-draggable static dialog tabs (such as Preferences).
pub fn draw_tab_bar(ui: &mut egui::Ui, current_tab: &mut usize, tabs: &[EditorTab]) -> bool {
    let mut changed = false;
    let tab_height = 26.0;
    let tab_spacing = 3.0;

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
        required_width += text_layout.size().x + 22.0 + tab_spacing;
    }

    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(required_width.max(ui.available_width()), tab_height),
        egui::Sense::hover(),
    );

    let painter = ui.painter_at(rect);
    let baseline_y = rect.max.y;
    let panel_bg = Color32::from_rgb(20, 20, 25);
    let border_color = Color32::from_rgb(45, 48, 60);
    let tab_inactive_bg = Color32::from_rgb(18, 19, 24);

    let mut current_x = rect.min.x + 4.0;

    for tab in tabs {
        let is_active = *current_tab == tab.id;
        let text = if tab.icon.is_empty() {
            tab.title.to_string()
        } else {
            format!("{} {}", tab.icon, tab.title)
        };
        let text_layout = painter.layout_no_wrap(text.clone(), font_id.clone(), Color32::WHITE);
        let tab_width = text_layout.size().x + 20.0;

        let tab_rect = Rect::from_min_size(
            Pos2::new(current_x, rect.min.y + 2.0),
            Vec2::new(tab_width, tab_height - 2.0),
        );

        let tab_resp = ui.interact(
            tab_rect,
            ui.make_persistent_id(format!("static_tab_{}", tab.id)),
            egui::Sense::click(),
        );

        if tab_resp.clicked() && !is_active {
            *current_tab = tab.id;
            changed = true;
        }

        let is_hovered = tab_resp.hovered();

        if is_active {
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
            painter.line_segment(
                [
                    Pos2::new(tab_rect.min.x, tab_rect.min.y),
                    Pos2::new(tab_rect.max.x, tab_rect.min.y),
                ],
                Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
            );
        } else {
            let tab_fill = if is_hovered {
                Color32::from_rgb(32, 35, 46)
            } else {
                tab_inactive_bg
            };
            painter.rect_filled(
                tab_rect,
                CornerRadius {
                    nw: 5,
                    ne: 5,
                    sw: 0,
                    se: 0,
                },
                tab_fill,
            );
            painter.line_segment(
                [
                    Pos2::new(tab_rect.min.x, baseline_y),
                    Pos2::new(tab_rect.max.x, baseline_y),
                ],
                Stroke::new(1.0, border_color),
            );
        }

        let text_color = if is_active {
            Color32::from_rgb(240, 245, 255)
        } else if is_hovered {
            Color32::from_rgb(200, 205, 220)
        } else {
            Color32::from_rgb(140, 145, 160)
        };

        let text_pos = Pos2::new(
            tab_rect.min.x + (tab_width - text_layout.size().x) * 0.5,
            tab_rect.min.y + (tab_height - text_layout.size().y) * 0.5,
        );
        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            text,
            font_id.clone(),
            text_color,
        );

        current_x += tab_width + tab_spacing;
    }

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