// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Multi-surface native Iris UI floating window rendering, titlebar interaction, and intelligent redocking.
//!
//! Renders independent detached panels directly with native Iris UI styling (dark sleek frame,
//! slightly rounded corners, exact tab strip parities), supports tab dragging from floating surfaces
//! back to the workspace tree, edge resizing, free window dragging, and smart canonical redocking.

use crate::ui::docking::EditorTabViewer;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use egui::{Color32, CornerRadius, CursorIcon, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use irisui::dock::{DockLayoutOptions, DockNode, DropZone};

/// Interaction mode active during mouse drag on a floating window.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) enum FloatingInteraction {
    #[default]
    None,
    DraggingTitle {
        grab_offset: Pos2,
    },
    Resizing(ResizeEdge),
}

/// Identifies the border or corner being resized on a floating window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Returns true if a floating window title drag or edge resize action is currently active.
pub(crate) fn is_floating_action_active(ctx: &egui::Context) -> bool {
    ctx.data(|d| {
        d.get_temp::<(u64, FloatingInteraction)>(egui::Id::new("iris_floating_native_action"))
    })
    .is_some()
}

/// Docks a floating window back to its canonical home leaf in the tree.
pub fn smart_dock_back_panel(layout_state: &mut PanelLayoutState, win_id: u64) {
    let target_panel = layout_state
        .dock_state
        .floating_windows
        .iter()
        .find(|w| w.id == win_id)
        .and_then(|w| w.tree.all_tabs().first().copied());

    let Some(panel) = target_panel else {
        let _ = layout_state.dock_state.close_floating_window(win_id);
        return;
    };

    let ideal_partner = match panel {
        PanelId::Hierarchy => PanelId::Stats,
        PanelId::Stats => PanelId::Hierarchy,
        PanelId::Inspector => PanelId::MaterialEditor,
        PanelId::MaterialEditor => PanelId::Inspector,
        PanelId::Assets => PanelId::Console,
        PanelId::Console | PanelId::AnimationTimeline => PanelId::Assets,
        PanelId::Viewport => PanelId::UiDesigner,
        PanelId::UiDesigner => PanelId::Viewport,
    };

    // 1. Try docking as a tab alongside ideal partner leaf
    if let Some((partner_leaf, _)) = layout_state.dock_state.tree.find_tab(&ideal_partner) {
        let _ =
            layout_state
                .dock_state
                .dock_floating_window(win_id, partner_leaf, DropZone::Center);
        return;
    }

    // 2. Try docking relative to Viewport (the central anchor of the editor)
    if let Some((viewport_leaf, _)) = layout_state.dock_state.tree.find_tab(&PanelId::Viewport) {
        let target_zone = match panel {
            PanelId::Hierarchy | PanelId::Stats => DropZone::Left,
            PanelId::Inspector | PanelId::MaterialEditor => DropZone::Right,
            PanelId::Assets | PanelId::Console | PanelId::AnimationTimeline => DropZone::Bottom,
            PanelId::Viewport | PanelId::UiDesigner => DropZone::Center,
        };
        let _ = layout_state
            .dock_state
            .dock_floating_window(win_id, viewport_leaf, target_zone);
        return;
    }

    // 3. Fallback to any existing leaf in the tree (preventing split-node errors)
    if let Some(fallback_leaf) = layout_state.dock_state.tree.find_first_leaf() {
        let _ =
            layout_state
                .dock_state
                .dock_floating_window(win_id, fallback_leaf, DropZone::Center);
    }
}

/// Renders all detached floating windows with 100% native Iris UI styling, draggable tabs, edge resizing, and smart docking back.
pub fn render_floating_windows(
    ui: &mut egui::Ui,
    layout_state: &mut PanelLayoutState,
    tab_viewer: &mut EditorTabViewer<'_>,
    _options: DockLayoutOptions,
) {
    if layout_state.dock_state.floating_windows.is_empty() {
        return;
    }

    let pointer_pos = ui.ctx().pointer_latest_pos();
    let mouse_down = ui.input(|i| i.pointer.primary_down());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());
    let mouse_double_clicked = ui.input(|i| {
        i.pointer
            .button_double_clicked(egui::PointerButton::Primary)
    });
    let mouse_delta = ui.input(|i| i.pointer.delta());
    let mouse_moving = ui.input(|i| i.pointer.is_moving());

    let mut floating_to_close = None;
    let mut floating_to_dock = None;
    let mut floating_to_front = None;

    let interaction_id = egui::Id::new("iris_floating_native_action");
    let mut active_action: Option<(u64, FloatingInteraction)> =
        ui.ctx().data(|d| d.get_temp(interaction_id));

    if !mouse_down {
        active_action = None;
    }

    for win in &mut layout_state.dock_state.floating_windows {
        const RESIZE_MARGIN: f32 = 6.0;
        const TAB_BAR_HEIGHT: f32 = 26.0;

        // 1. Handle Active Resizing / Title Dragging
        if let Some((active_id, interaction)) = active_action
            && active_id == win.id
        {
            match interaction {
                FloatingInteraction::DraggingTitle { grab_offset } => {
                    if let Some(p) = pointer_pos {
                        let screen = ui
                            .input(|i| i.raw.screen_rect)
                            .unwrap_or_else(|| ui.max_rect());
                        win.rect.x = (p.x - grab_offset.x)
                            .clamp(-win.rect.width + 60.0, (screen.width() - 60.0).max(0.0));
                        win.rect.y =
                            (p.y - grab_offset.y).clamp(0.0, (screen.height() - 32.0).max(0.0));
                    }
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                }
                FloatingInteraction::Resizing(edge) => match edge {
                    ResizeEdge::Right => {
                        win.rect.width = (win.rect.width + mouse_delta.x).max(220.0);
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                    }
                    ResizeEdge::Bottom => {
                        win.rect.height = (win.rect.height + mouse_delta.y).max(140.0);
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
                    }
                    ResizeEdge::Left => {
                        let old_w = win.rect.width;
                        win.rect.width = (win.rect.width - mouse_delta.x).max(220.0);
                        win.rect.x += old_w - win.rect.width;
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                    }
                    ResizeEdge::Top => {
                        let old_h = win.rect.height;
                        win.rect.height = (win.rect.height - mouse_delta.y).max(140.0);
                        win.rect.y += old_h - win.rect.height;
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
                    }
                    ResizeEdge::TopLeft => {
                        let old_w = win.rect.width;
                        win.rect.width = (win.rect.width - mouse_delta.x).max(220.0);
                        win.rect.x += old_w - win.rect.width;
                        let old_h = win.rect.height;
                        win.rect.height = (win.rect.height - mouse_delta.y).max(140.0);
                        win.rect.y += old_h - win.rect.height;
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
                    }
                    ResizeEdge::TopRight => {
                        win.rect.width = (win.rect.width + mouse_delta.x).max(220.0);
                        let old_h = win.rect.height;
                        win.rect.height = (win.rect.height - mouse_delta.y).max(140.0);
                        win.rect.y += old_h - win.rect.height;
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeNeSw);
                    }
                    ResizeEdge::BottomLeft => {
                        let old_w = win.rect.width;
                        win.rect.width = (win.rect.width - mouse_delta.x).max(220.0);
                        win.rect.x += old_w - win.rect.width;
                        win.rect.height = (win.rect.height + mouse_delta.y).max(140.0);
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeNeSw);
                    }
                    ResizeEdge::BottomRight => {
                        win.rect.width = (win.rect.width + mouse_delta.x).max(220.0);
                        win.rect.height = (win.rect.height + mouse_delta.y).max(140.0);
                        ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
                    }
                },
                FloatingInteraction::None => {}
            }
        }

        // Recompute win_rect after potential movement/resizing
        let win_rect = Rect::from_min_size(
            Pos2::new(win.rect.x, win.rect.y),
            Vec2::new(win.rect.width, win.rect.height),
        );
        let bar_rect =
            Rect::from_min_size(win_rect.min, Vec2::new(win_rect.width(), TAB_BAR_HEIGHT));
        let content_rect = Rect::from_min_max(
            Pos2::new(win_rect.min.x, win_rect.min.y + TAB_BAR_HEIGHT),
            win_rect.max,
        );

        // Bring to front on window interaction
        if let Some(p) = pointer_pos
            && win_rect.contains(p)
            && mouse_clicked
        {
            floating_to_front = Some(win.id);
        }

        // 2. Edge & Corner Hit-Testing for Resizing
        let mut hovered_resize_edge = None;
        if active_action.is_none()
            && let Some(p) = pointer_pos
        {
            let outer_rect = win_rect.expand(RESIZE_MARGIN);
            if outer_rect.contains(p) {
                let on_left = p.x <= win_rect.min.x + RESIZE_MARGIN;
                let on_right = p.x >= win_rect.max.x - RESIZE_MARGIN;
                let on_top = p.y <= win_rect.min.y + RESIZE_MARGIN;
                let on_bottom = p.y >= win_rect.max.y - RESIZE_MARGIN;

                if on_top && on_left {
                    hovered_resize_edge = Some(ResizeEdge::TopLeft);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
                } else if on_top && on_right {
                    hovered_resize_edge = Some(ResizeEdge::TopRight);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeNeSw);
                } else if on_bottom && on_left {
                    hovered_resize_edge = Some(ResizeEdge::BottomLeft);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeNeSw);
                } else if on_bottom && on_right {
                    hovered_resize_edge = Some(ResizeEdge::BottomRight);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
                } else if on_left {
                    hovered_resize_edge = Some(ResizeEdge::Left);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                } else if on_right {
                    hovered_resize_edge = Some(ResizeEdge::Right);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                } else if on_top {
                    hovered_resize_edge = Some(ResizeEdge::Top);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
                } else if on_bottom {
                    hovered_resize_edge = Some(ResizeEdge::Bottom);
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
                }

                if mouse_down && mouse_moving && hovered_resize_edge.is_some() {
                    active_action =
                        hovered_resize_edge.map(|e| (win.id, FloatingInteraction::Resizing(e)));
                    floating_to_front = Some(win.id);
                }
            }
        }

        // 3. Render Floating Window Shadow, Pitch-Black Background, and Border
        ui.painter().rect_filled(
            win_rect.expand(6.0),
            CornerRadius::same(10),
            Color32::from_rgba_unmultiplied(0, 0, 0, 30),
        );
        ui.painter().rect_filled(
            win_rect.expand(3.0),
            CornerRadius::same(8),
            Color32::from_rgba_unmultiplied(0, 0, 0, 70),
        );
        ui.painter().rect_filled(
            win_rect.expand(1.0),
            CornerRadius::same(7),
            Color32::from_rgba_unmultiplied(0, 0, 0, 140),
        );

        // Window background: exact pitch-black theme matching docked panels
        ui.painter().rect_filled(
            win_rect,
            CornerRadius::same(6),
            Color32::from_rgb(14, 16, 22),
        );
        ui.painter().rect_stroke(
            win_rect,
            CornerRadius::same(6),
            Stroke::new(1.0, Color32::from_rgb(38, 42, 54)),
            StrokeKind::Inside,
        );

        // 4. Render Native Iris UI Tab Bar Header
        ui.painter().rect_filled(
            bar_rect,
            CornerRadius {
                nw: 6,
                ne: 6,
                sw: 0,
                se: 0,
            },
            Color32::from_rgb(15, 15, 20),
        );
        ui.painter().line_segment(
            [bar_rect.left_bottom(), bar_rect.right_bottom()],
            Stroke::new(1.0, Color32::from_rgb(38, 41, 52)),
        );

        // Window controls on the right (Dock-back `⤢` and Close `✖`)
        let close_btn_rect = Rect::from_min_size(
            Pos2::new(bar_rect.right() - 24.0, bar_rect.top() + 2.0),
            Vec2::new(20.0, 22.0),
        );
        let dock_back_btn_rect = Rect::from_min_size(
            Pos2::new(bar_rect.right() - 48.0, bar_rect.top() + 2.0),
            Vec2::new(20.0, 22.0),
        );

        let is_close_hovered = pointer_pos.is_some_and(|p| close_btn_rect.contains(p));
        let is_dock_hovered = pointer_pos.is_some_and(|p| dock_back_btn_rect.contains(p));

        if is_close_hovered && mouse_clicked {
            floating_to_close = Some(win.id);
        } else if is_dock_hovered && mouse_clicked {
            floating_to_dock = Some(win.id);
        }

        ui.painter().text(
            dock_back_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "⤢",
            FontId::proportional(12.0),
            if is_dock_hovered {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(140, 145, 160)
            },
        );

        ui.painter().text(
            close_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "✖",
            FontId::proportional(10.0),
            if is_close_hovered {
                Color32::from_rgb(255, 100, 100)
            } else {
                Color32::from_rgb(140, 145, 160)
            },
        );

        // Render Tabs
        let mut current_tab_x = bar_rect.left() + 4.0;

        // Iterate through all leaf nodes in this window's tree
        let leaf_info: Vec<(irisui::dock::DockNodeId, Vec<PanelId>, usize)> = win
            .tree
            .iter()
            .filter_map(|(id, node)| match node {
                DockNode::Leaf { tabs, active_tab } => Some((id, tabs.clone(), *active_tab)),
                _ => None,
            })
            .collect();

        for (leaf_id, tabs, active_idx) in leaf_info {
            for (tab_idx, panel) in tabs.iter().enumerate() {
                let title = format!("{} {}", panel.icon(), panel.title());
                let text_w = (title.len() as f32) * 7.5 + 28.0;
                let tab_w = text_w.clamp(60.0, 180.0);

                let tab_rect = Rect::from_min_size(
                    Pos2::new(current_tab_x, bar_rect.top()),
                    Vec2::new(tab_w, TAB_BAR_HEIGHT),
                );

                let is_tab_active = tab_idx == active_idx;
                let is_tab_hovered = pointer_pos.is_some_and(|p| tab_rect.contains(p));

                if is_tab_hovered {
                    if mouse_double_clicked {
                        floating_to_dock = Some(win.id);
                    } else if mouse_clicked {
                        let _ = win.tree.set_active_tab(leaf_id, tab_idx);
                        floating_to_front = Some(win.id);
                    }
                }

                // Tab visual presentation
                let tab_bg = if is_tab_active {
                    Color32::from_rgb(22, 24, 30)
                } else if is_tab_hovered {
                    Color32::from_rgb(28, 30, 38)
                } else {
                    Color32::from_rgb(16, 17, 22)
                };

                ui.painter().rect_filled(
                    tab_rect,
                    CornerRadius {
                        nw: 4,
                        ne: 4,
                        sw: 0,
                        se: 0,
                    },
                    tab_bg,
                );

                if is_tab_active {
                    ui.painter().line_segment(
                        [
                            Pos2::new(tab_rect.left(), tab_rect.bottom() - 1.0),
                            Pos2::new(tab_rect.right(), tab_rect.bottom() - 1.0),
                        ],
                        Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
                    );
                }

                let text_color = if is_tab_active {
                    Color32::from_rgb(0, 229, 255)
                } else if is_tab_hovered {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(155, 160, 175)
                };

                ui.painter().text(
                    Pos2::new(tab_rect.left() + 8.0, tab_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &title,
                    FontId::proportional(12.0),
                    text_color,
                );

                current_tab_x += tab_w + 2.0;
            }

            // Render Content for this Leaf
            if active_idx < tabs.len() {
                let active_panel = tabs[active_idx];
                tab_viewer.render_content(ui, active_panel, content_rect);
            }
        }

        // Titlebar & Tab Bar Dragging: Freely translates the floating window across the screen
        let is_bar_hovered = pointer_pos.is_some_and(|p| bar_rect.contains(p));
        if is_bar_hovered
            && !is_close_hovered
            && !is_dock_hovered
            && hovered_resize_edge.is_none()
            && active_action.is_none()
        {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
            if mouse_double_clicked {
                floating_to_dock = Some(win.id);
            } else if mouse_down
                && layout_state.dock_state.active_drag.is_none()
                && let Some(p) = pointer_pos
            {
                let grab_offset = Pos2::new(p.x - win.rect.x, p.y - win.rect.y);
                active_action = Some((win.id, FloatingInteraction::DraggingTitle { grab_offset }));
                floating_to_front = Some(win.id);
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
        }
    }

    // Save active action state: exact matching type (u64, FloatingInteraction)
    ui.ctx().data_mut(|d| {
        if let Some(action) = active_action {
            d.insert_temp(interaction_id, action);
        } else {
            d.remove_temp::<(u64, FloatingInteraction)>(interaction_id);
        }
    });

    // Execute Deferred Actions
    if let Some(front_id) = floating_to_front
        && let Some(idx) = layout_state
            .dock_state
            .floating_windows
            .iter()
            .position(|w| w.id == front_id)
    {
        let win = layout_state.dock_state.floating_windows.remove(idx);
        layout_state.dock_state.floating_windows.push(win);
    }
    if let Some(win_id) = floating_to_close {
        let _ = layout_state.dock_state.close_floating_window(win_id);
    }
    if let Some(win_id) = floating_to_dock {
        smart_dock_back_panel(layout_state, win_id);
    }
}