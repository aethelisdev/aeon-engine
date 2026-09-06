// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native rendering and interaction engine for `iris-dock` inside the Aeon Engine editor.
//!
//! Renders split containers, draggable divider lines, tab strips with active highlights,
//! dirty modification bullets, close `✖` buttons, 5-way navigator overlays, and floating windows.

use crate::ui::docking::EditorTabViewer;
use crate::ui::panel_layout::PanelLayoutState;
use egui::{Color32, CornerRadius, CursorIcon, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use irisui::core::geometry::{Point, Rect as IrisRect};
use irisui::dock::{
    DockLayoutOptions, DockNavigatorGeometry, DropZone, SplitDirection, TabViewer,
    calculate_drop_preview_rect, compute_dock_layout_advanced,
};

/// Renders the complete `iris-dock` tree, handles interactive gestures, and manages floating windows.
pub fn render_iris_dock(
    ui: &mut egui::Ui,
    layout_state: &mut PanelLayoutState,
    tab_viewer: &mut EditorTabViewer<'_>,
) {
    let available_rect = ui.available_rect_before_wrap();
    if available_rect.width() <= 10.0 || available_rect.height() <= 10.0 {
        return;
    }

    let iris_rect = IrisRect::new(
        available_rect.min.x,
        available_rect.min.y,
        available_rect.width(),
        available_rect.height(),
    );

    let options = DockLayoutOptions {
        splitter_thickness: 3.0,
        tab_bar_height: 26.0,
        maximized_leaf: layout_state.dock_state.maximized_leaf,
        auto_hide_single_tab_bar: layout_state.dock_state.auto_hide_single_tab_bar,
    };

    let layout = compute_dock_layout_advanced(
        &layout_state.dock_state.tree,
        iris_rect,
        options,
        tab_viewer,
    );

    let pointer_pos = ui.ctx().pointer_latest_pos();
    let mouse_down = ui.input(|i| i.pointer.primary_down());
    let mouse_released = ui.input(|i| i.pointer.any_released());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());
    let mouse_double_clicked = ui.input(|i| {
        i.pointer
            .button_double_clicked(egui::PointerButton::Primary)
    });

    // Mask pointer interactions for docked elements when interacting with or hovering over floating windows
    let is_floating_action_active =
        crate::ui::docking_floating::is_floating_action_active(ui.ctx());
    let is_pointer_over_floating = pointer_pos.is_some_and(|p| {
        layout_state.dock_state.floating_windows.iter().any(|win| {
            p.x >= win.rect.x
                && p.x <= win.rect.x + win.rect.width
                && p.y >= win.rect.y
                && p.y <= win.rect.y + win.rect.height
        })
    });
    let is_floating_busy = is_floating_action_active || is_pointer_over_floating;

    let docked_pointer_pos = if is_floating_busy { None } else { pointer_pos };
    let docked_mouse_down = if is_floating_busy { false } else { mouse_down };
    let docked_mouse_clicked = if is_floating_busy {
        false
    } else {
        mouse_clicked
    };
    let docked_mouse_double_clicked = if is_floating_busy {
        false
    } else {
        mouse_double_clicked
    };

    // 1. Handle Active Splitter Drag
    if layout_state.dock_state.active_splitter.is_some() {
        if mouse_released {
            layout_state.dock_state.end_splitter_drag();
        } else if let Some(pos) = pointer_pos {
            let axis_coord = match layout_state.dock_state.active_splitter.map(|s| s.direction) {
                Some(SplitDirection::Horizontal) => pos.x,
                _ => pos.y,
            };
            layout_state.dock_state.update_splitter_drag(axis_coord);
        }
    }

    // 2. Render Splitters & Hit Testing
    for splitter in &layout.splitters {
        let egui_rect = Rect::from_min_size(
            Pos2::new(splitter.rect.x, splitter.rect.y),
            Vec2::new(splitter.rect.width, splitter.rect.height),
        );

        let hit_rect = egui_rect.expand2(match splitter.direction {
            SplitDirection::Horizontal => Vec2::new(3.0, 0.0),
            SplitDirection::Vertical => Vec2::new(0.0, 3.0),
        });

        let is_hovered = docked_pointer_pos.is_some_and(|p| hit_rect.contains(p));
        let is_dragged = layout_state
            .dock_state
            .active_splitter
            .as_ref()
            .is_some_and(|d| d.node_id == splitter.node_id);

        if is_hovered || is_dragged {
            ui.ctx().set_cursor_icon(match splitter.direction {
                SplitDirection::Horizontal => CursorIcon::ResizeHorizontal,
                SplitDirection::Vertical => CursorIcon::ResizeVertical,
            });

            if docked_mouse_double_clicked && is_hovered {
                let _ = layout_state.dock_state.reset_splitter(splitter.node_id);
            } else if docked_mouse_down
                && is_hovered
                && layout_state.dock_state.active_splitter.is_none()
                && layout_state.dock_state.active_drag.is_none()
            {
                let (total_dim, start_coord) = match splitter.direction {
                    SplitDirection::Horizontal => {
                        (available_rect.width(), pointer_pos.unwrap_or_default().x)
                    }
                    SplitDirection::Vertical => {
                        (available_rect.height(), pointer_pos.unwrap_or_default().y)
                    }
                };
                layout_state.dock_state.start_splitter_drag(
                    splitter.node_id,
                    splitter.direction,
                    start_coord,
                    total_dim,
                );
            }
        }

        let splitter_color = if is_dragged || is_hovered {
            Color32::from_rgb(0, 229, 255)
        } else {
            Color32::from_rgb(38, 41, 52)
        };

        ui.painter()
            .rect_filled(egui_rect, CornerRadius::ZERO, splitter_color);
    }

    // 3. Render Leaves, Tab Strips, and Panel Content
    let mut tab_to_close = None;
    let mut tab_to_activate = None;
    let mut tab_to_drag = None;
    let mut tab_to_float = None;

    for leaf in &layout.leaves {
        let leaf_rect = Rect::from_min_size(
            Pos2::new(leaf.rect.x, leaf.rect.y),
            Vec2::new(leaf.rect.width, leaf.rect.height),
        );

        // Tab Bar
        if leaf.tab_bar_rect.height > 0.0 {
            let bar_rect = Rect::from_min_size(
                Pos2::new(leaf.tab_bar_rect.x, leaf.tab_bar_rect.y),
                Vec2::new(leaf.tab_bar_rect.width, leaf.tab_bar_rect.height),
            );

            // Tab bar background
            ui.painter()
                .rect_filled(bar_rect, CornerRadius::ZERO, Color32::from_rgb(15, 15, 20));
            ui.painter().line_segment(
                [bar_rect.left_bottom(), bar_rect.right_bottom()],
                Stroke::new(1.0, Color32::from_rgb(38, 41, 52)),
            );

            if let Some(tab_bar) = &leaf.tab_bar_layout {
                for tab in &tab_bar.tabs {
                    let tab_rect = Rect::from_min_size(
                        Pos2::new(tab.rect.x, tab.rect.y),
                        Vec2::new(tab.rect.width, tab.rect.height),
                    );

                    let is_tab_hovered = docked_pointer_pos.is_some_and(|p| tab_rect.contains(p));

                    // Close button hitbox
                    let mut is_close_hovered = false;
                    let close_rect_opt = tab.close_btn_rect.map(|cb| {
                        Rect::from_min_size(Pos2::new(cb.x, cb.y), Vec2::new(cb.width, cb.height))
                    });

                    if let Some(cr) = close_rect_opt {
                        is_close_hovered = docked_pointer_pos.is_some_and(|p| cr.contains(p));
                    }

                    // Click & Drag Gestures
                    let is_tab_right_clicked = is_tab_hovered
                        && ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary));

                    if is_tab_right_clicked {
                        tab_to_float = Some((leaf.node_id, tab.index));
                    } else if is_tab_hovered && docked_mouse_clicked {
                        if is_close_hovered {
                            tab_to_close = Some((leaf.node_id, tab.index));
                        } else {
                            tab_to_activate = Some((leaf.node_id, tab.index));
                        }
                    } else if is_tab_hovered
                        && docked_mouse_down
                        && !is_close_hovered
                        && layout_state.dock_state.active_drag.is_none()
                        && layout_state.dock_state.active_splitter.is_none()
                        && tab.is_draggable
                        && ui.input(|i| i.pointer.is_moving())
                    {
                        tab_to_drag = Some((leaf.node_id, tab.index));
                    }

                    // Visual Tab Presentation
                    let tab_bg = if tab.is_active {
                        Color32::from_rgb(22, 24, 30)
                    } else if is_tab_hovered {
                        Color32::from_rgb(28, 30, 38)
                    } else {
                        Color32::from_rgb(16, 17, 22)
                    };

                    ui.painter().rect_filled(
                        tab_rect,
                        CornerRadius {
                            nw: 3,
                            ne: 3,
                            sw: 0,
                            se: 0,
                        },
                        tab_bg,
                    );

                    if tab.is_active {
                        ui.painter().line_segment(
                            [
                                Pos2::new(tab_rect.left(), tab_rect.bottom() - 1.0),
                                Pos2::new(tab_rect.right(), tab_rect.bottom() - 1.0),
                            ],
                            Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
                        );
                    }

                    let text_color = if tab.is_active {
                        Color32::from_rgb(0, 229, 255)
                    } else if is_tab_hovered {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(155, 160, 175)
                    };

                    ui.painter().text(
                        Pos2::new(tab_rect.left() + 8.0, tab_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &tab.title,
                        FontId::proportional(12.0),
                        text_color,
                    );

                    // Dirty indicator bullet
                    if tab.is_modified {
                        let dot_x = tab_rect.left() + 4.0;
                        let dot_y = tab_rect.center().y;
                        ui.painter().circle_filled(
                            Pos2::new(dot_x, dot_y),
                            2.5,
                            Color32::from_rgb(0, 229, 255),
                        );
                    }

                    // Close button `✖`
                    if let Some(cr) = close_rect_opt {
                        let close_color = if is_close_hovered {
                            Color32::from_rgb(255, 100, 100)
                        } else {
                            Color32::from_rgb(130, 135, 150)
                        };
                        ui.painter().text(
                            cr.center(),
                            egui::Align2::CENTER_CENTER,
                            "✖",
                            FontId::proportional(10.0),
                            close_color,
                        );
                    }
                }
            }

            // Restore / Maximize Indicator Button
            if leaf.is_maximized {
                let restore_rect = Rect::from_min_size(
                    Pos2::new(bar_rect.right() - 28.0, bar_rect.top() + 2.0),
                    Vec2::new(24.0, 22.0),
                );
                let is_rest_hovered = docked_pointer_pos.is_some_and(|p| restore_rect.contains(p));
                if is_rest_hovered && docked_mouse_clicked {
                    layout_state.dock_state.restore();
                }
                let rest_color = if is_rest_hovered {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(0, 229, 255)
                };
                ui.painter().text(
                    restore_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "❐",
                    FontId::proportional(13.0),
                    rest_color,
                );
            }
        }

        // Panel Content Area
        let content_rect = Rect::from_min_size(
            Pos2::new(leaf.content_rect.x, leaf.content_rect.y),
            Vec2::new(leaf.content_rect.width, leaf.content_rect.height),
        );

        if leaf.active_tab < leaf.tabs.len() {
            let active_panel = leaf.tabs[leaf.active_tab];
            tab_viewer.render_content(ui, active_panel, content_rect);
        }

        // Leaf border stroke
        ui.painter().rect_stroke(
            leaf_rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_rgb(30, 32, 40)),
            StrokeKind::Inside,
        );
    }

    // Apply deferred tab actions
    if let Some((leaf_id, tab_idx)) = tab_to_close
        && let Some(leaf_node) = layout_state.dock_state.tree.get_mut(leaf_id)
        && let irisui::dock::DockNode::Leaf { tabs, .. } = leaf_node
        && tab_idx < tabs.len()
        && tab_viewer.on_close(&mut tabs[tab_idx])
    {
        let _ = layout_state.dock_state.tree.remove_tab(leaf_id, tab_idx);
        layout_state.dock_state.tree.collapse_empty_leaves();
    }

    if let Some((leaf_id, tab_idx)) = tab_to_activate {
        let _ = layout_state
            .dock_state
            .tree
            .set_active_tab(leaf_id, tab_idx);
        layout_state.dock_state.tree.set_focused_leaf(Some(leaf_id));
    }

    if let Some((leaf_id, tab_idx)) = tab_to_float
        && let Some(leaf_node) = layout_state.dock_state.tree.get(leaf_id)
        && let irisui::dock::DockNode::Leaf { tabs, .. } = leaf_node
        && tab_idx < tabs.len()
    {
        let panel = tabs[tab_idx];
        let leaf_geom = layout.leaves.iter().find(|l| l.node_id == leaf_id);
        let (init_pos, init_size) = if let Some(l) = leaf_geom {
            (
                Point::new(l.rect.x, l.rect.y),
                Point::new(l.rect.width, l.rect.height),
            )
        } else {
            let pos = pointer_pos.unwrap_or(available_rect.center());
            (
                Point::new(pos.x - 140.0, pos.y - 20.0),
                Point::new(420.0, 300.0),
            )
        };
        let _ = layout_state.dock_state.detach_tab_to_floating(
            leaf_id,
            tab_idx,
            format!("{} {}", panel.icon(), panel.title()),
            init_pos,
            init_size,
        );
    }

    if let Some((leaf_id, tab_idx)) = tab_to_drag {
        let cursor = pointer_pos
            .map(|p| Point::new(p.x, p.y))
            .unwrap_or_default();
        let source_rect = layout
            .leaves
            .iter()
            .find(|l| l.node_id == leaf_id)
            .map(|l| l.rect)
            .unwrap_or_default();
        let _ = layout_state
            .dock_state
            .start_tab_drag(leaf_id, tab_idx, cursor, source_rect);
        layout_state.dock_state.tree.collapse_empty_leaves();
    }

    // 4. Tab Dragging, 5-Way Docking Cross Navigator, and Drop Zone Overlays
    let mut hovered_leaf_nav: Option<(DockNavigatorGeometry, Option<DropZone>)> = None;
    let mut resolved_zone: Option<(irisui::dock::DockNodeId, DropZone, IrisRect)> = None;

    if let Some(drag) = &mut layout_state.dock_state.active_drag
        && let Some(pos) = pointer_pos
    {
        drag.cursor_pos = Point::new(pos.x, pos.y);
        let screen_point = drag.cursor_pos;

        // Reset per-frame drag targets so cursor movement re-evaluates cleanly every frame
        drag.tab_reorder_target = None;
        drag.hover_target = None;

        // If cursor is over any floating window, DO NOT snap to background leaves under the floating window!
        let is_over_floating = layout_state.dock_state.floating_windows.iter().any(|win| {
            screen_point.x >= win.rect.x
                && screen_point.x <= win.rect.x + win.rect.width
                && screen_point.y >= win.rect.y
                && screen_point.y <= win.rect.y + win.rect.height
        });

        if !is_over_floating {
            // 1. PRIORITY 1: Check Tab Bar Strip Reordering & Insertion across all leaves
            // If cursor is within any leaf's tab bar strip, prioritize combining/inserting into that tab bar.
            // This ensures dragging onto upper tab bars (Viewport, Hierarchy, Inspector) merges tabs
            // instead of triggering root screen splits.
            for leaf in &layout.leaves {
                let is_over_tab_bar = (leaf.tab_bar_rect.height > 0.0
                    && leaf.tab_bar_rect.contains_point(screen_point))
                    || leaf
                        .tab_bar_layout
                        .as_ref()
                        .is_some_and(|tb| tb.rect.contains_point(screen_point));

                if is_over_tab_bar {
                    let insert_idx = if let Some(ref tab_bar) = leaf.tab_bar_layout {
                        irisui::dock::calculate_tab_reorder_index(tab_bar, screen_point)
                            .unwrap_or(leaf.tabs.len())
                    } else {
                        leaf.tabs.len()
                    };
                    drag.tab_reorder_target = Some((leaf.node_id, insert_idx));
                    break;
                }
            }

            // 2. PRIORITY 2: Check 5-Way Docking Cross Navigator over the hovered leaf
            // Only snaps to docking zones if the cursor is directly on one of the 5 cross buttons.
            // If cursor is on the leaf body outside the buttons (e.g. 3D viewport), active_zone is None,
            // allowing the tab to be effortlessly dropped as a floating window!
            if drag.tab_reorder_target.is_none() {
                for leaf in &layout.leaves {
                    if leaf.rect.contains_point(screen_point) {
                        let nav =
                            DockNavigatorGeometry::from_content_rect(leaf.content_rect, 42.0, 6.0);
                        let hit = nav.hit_test(screen_point);
                        if let Some(zone) = hit {
                            let prev = calculate_drop_preview_rect(leaf.content_rect, zone);
                            resolved_zone = Some((leaf.node_id, zone, prev));
                            hovered_leaf_nav = Some((nav, Some(zone)));
                        } else {
                            hovered_leaf_nav = Some((nav, None));
                        }
                        break;
                    }
                }
            }
        }

        if let Some((target_leaf, zone, prev)) = resolved_zone {
            drag.hover_target = Some((target_leaf, zone, prev));
        }
    }

    // Drop or Float on Mouse Release
    if layout_state.dock_state.active_drag.is_some() && mouse_released {
        let _ = layout_state
            .dock_state
            .drop_tab_or_float(Point::new(420.0, 300.0));
    }

    // Render Drag Overlays
    if let Some(drag) = &layout_state.dock_state.active_drag {
        // A. Translucent drop preview rect
        if let Some((_, _, prev)) = drag.hover_target {
            let prev_egui = Rect::from_min_size(
                Pos2::new(prev.x, prev.y),
                Vec2::new(prev.width, prev.height),
            );
            ui.painter().rect_filled(
                prev_egui,
                CornerRadius::same(4),
                Color32::from_rgba_unmultiplied(0, 229, 255, 38),
            );
            ui.painter().rect_stroke(
                prev_egui,
                CornerRadius::same(4),
                Stroke::new(2.0, Color32::from_rgb(0, 229, 255)),
                StrokeKind::Inside,
            );
        }

        // B. Tab strip insertion marker indicator
        if let Some((target_leaf, insert_idx)) = drag.tab_reorder_target
            && let Some(leaf) = layout.leaves.iter().find(|l| l.node_id == target_leaf)
        {
            let bar_egui = Rect::from_min_size(
                Pos2::new(leaf.tab_bar_rect.x, leaf.tab_bar_rect.y),
                Vec2::new(leaf.tab_bar_rect.width, leaf.tab_bar_rect.height),
            );
            // Subtle highlight over target tab bar
            ui.painter().rect_filled(
                bar_egui,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(0, 229, 255, 30),
            );

            // Compute insertion X coordinate
            let insert_x = if let Some(ref tab_bar) = leaf.tab_bar_layout {
                if insert_idx < tab_bar.tabs.len() {
                    tab_bar.tabs[insert_idx].rect.x
                } else {
                    tab_bar
                        .tabs
                        .last()
                        .map(|t| t.rect.x + t.rect.width)
                        .unwrap_or(leaf.tab_bar_rect.x + 4.0)
                }
            } else {
                leaf.tab_bar_rect.x + 4.0
            };

            // Draw glowing vertical insertion marker
            ui.painter().line_segment(
                [
                    Pos2::new(insert_x, bar_egui.top() + 2.0),
                    Pos2::new(insert_x, bar_egui.bottom() - 2.0),
                ],
                Stroke::new(3.0, Color32::from_rgb(0, 229, 255)),
            );
        }

        // C. 5-Way Docking Cross Navigator over the hovered leaf
        if let Some((nav, active_zone)) = &hovered_leaf_nav {
            render_dock_navigator(ui, nav, *active_zone);
        }

        // D. Floating drag cursor badge
        if let Some(pos) = pointer_pos {
            let badge_rect = Rect::from_min_size(
                Pos2::new(pos.x + 12.0, pos.y + 12.0),
                Vec2::new(125.0, 26.0),
            );
            ui.painter().rect_filled(
                badge_rect,
                CornerRadius::same(13),
                Color32::from_rgba_unmultiplied(18, 20, 28, 240),
            );
            ui.painter().rect_stroke(
                badge_rect,
                CornerRadius::same(13),
                Stroke::new(1.0, Color32::from_rgb(0, 229, 255)),
                StrokeKind::Inside,
            );
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} {}", drag.tab_data.icon(), drag.tab_data.title()),
                FontId::proportional(11.5),
                Color32::WHITE,
            );
        }
    }

    // 5. Render Independent Floating Windows
    crate::ui::docking_floating::render_floating_windows(ui, layout_state, tab_viewer, options);
}

/// Renders the 5-way docking cross navigator centered within the hovered leaf pane.
/// Displays Center, Left, Right, Top, and Bottom anchor buttons:
/// - Center (`▣`): Inserts the tab into the leaf alongside existing tabs (tab grouping).
/// - Left (`◧`), Right (`◨`), Top (`⬒`), Bottom (`⬓`): Triggers 50/50 directional splits.
fn render_dock_navigator(
    ui: &mut egui::Ui,
    nav: &DockNavigatorGeometry,
    active_zone: Option<DropZone>,
) {
    let cross_bounds = Rect::from_min_max(
        Pos2::new(nav.left_button.x - 6.0, nav.top_button.y - 6.0),
        Pos2::new(
            nav.right_button.x + nav.right_button.width + 6.0,
            nav.bottom_button.y + nav.bottom_button.height + 6.0,
        ),
    );

    ui.painter().rect_filled(
        cross_bounds,
        CornerRadius::same(10),
        Color32::from_rgba_unmultiplied(12, 14, 20, 225),
    );
    ui.painter().rect_stroke(
        cross_bounds,
        CornerRadius::same(10),
        Stroke::new(1.0, Color32::from_rgb(45, 50, 65)),
        StrokeKind::Inside,
    );

    // 5-way directional split & center tab grouping buttons
    let buttons = [
        (nav.center_button, DropZone::Center, "▣"),
        (nav.left_button, DropZone::Left, "◧"),
        (nav.right_button, DropZone::Right, "◨"),
        (nav.top_button, DropZone::Top, "⬒"),
        (nav.bottom_button, DropZone::Bottom, "⬓"),
    ];

    for (btn_rect, zone, icon) in buttons {
        let is_selected = active_zone == Some(zone);
        let egui_rect = Rect::from_min_size(
            Pos2::new(btn_rect.x, btn_rect.y),
            Vec2::new(btn_rect.width, btn_rect.height),
        );

        let bg_color = if is_selected {
            Color32::from_rgba_unmultiplied(0, 180, 220, 190)
        } else {
            Color32::from_rgba_unmultiplied(26, 29, 39, 240)
        };

        let stroke_color = if is_selected {
            Color32::from_rgb(0, 229, 255)
        } else {
            Color32::from_rgb(55, 60, 75)
        };

        let stroke_width = if is_selected { 2.0 } else { 1.0 };

        ui.painter()
            .rect_filled(egui_rect, CornerRadius::same(5), bg_color);
        ui.painter().rect_stroke(
            egui_rect,
            CornerRadius::same(5),
            Stroke::new(stroke_width, stroke_color),
            StrokeKind::Inside,
        );

        let icon_color = if is_selected {
            Color32::WHITE
        } else {
            Color32::from_rgb(170, 180, 200)
        };

        ui.painter().text(
            egui_rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(16.0),
            icon_color,
        );
    }
}