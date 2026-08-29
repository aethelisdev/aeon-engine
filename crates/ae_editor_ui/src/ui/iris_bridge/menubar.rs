// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top application menu bar and floating dropdown popup construction routines.

use crate::ui::EngineUiAction;
use crate::ui::iris_bridge::types::{ActiveMenu, DropdownAction};
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::prelude::*;

/// Height of the top menubar panel in physical pixels (matching egui geometry).
pub const MENUBAR_HEIGHT: f32 = 26.0;

/// Default width of floating dropdown popup menus in physical pixels.
pub const DROPDOWN_WIDTH: f32 = 250.0;

/// Tests if a coordinate is within horizontal bounds on the menubar.
#[inline]
pub fn is_in_menubar_range(p: Point, x1: f32, x2: f32) -> bool {
    p.y >= 0.0 && p.y <= MENUBAR_HEIGHT && p.x >= x1 && p.x <= x2
}

/// Builds the top application menu bar widget tree.
pub fn build_top_menu_bar(
    tree: &mut UiTree,
    screen_width: f32,
    cursor_pos: Point,
    active_menu: Option<ActiveMenu>,
    is_editing: bool,
) -> WidgetId {
    let is_file_hovered = is_in_menubar_range(cursor_pos, 6.0, 44.0);
    let is_edit_hovered = is_in_menubar_range(cursor_pos, 44.0, 84.0);
    let is_view_hovered = is_in_menubar_range(cursor_pos, 84.0, 126.0);
    let is_window_hovered = is_in_menubar_range(cursor_pos, 126.0, 186.0);
    let is_help_hovered = is_in_menubar_range(cursor_pos, 186.0, 226.0);
    let is_play_hovered = is_in_menubar_range(cursor_pos, screen_width - 90.0, screen_width);

    let mut menu_builder = MenuBarBuilder::new(tree, screen_width);

    menu_builder.add_menu_button(
        "File",
        active_menu == Some(ActiveMenu::File),
        is_file_hovered,
    );
    menu_builder.add_menu_button(
        "Edit",
        active_menu == Some(ActiveMenu::Edit),
        is_edit_hovered,
    );
    menu_builder.add_menu_button(
        "View",
        active_menu == Some(ActiveMenu::View),
        is_view_hovered,
    );
    menu_builder.add_menu_button(
        "Window",
        active_menu == Some(ActiveMenu::Window),
        is_window_hovered,
    );
    menu_builder.add_menu_button(
        "Help",
        active_menu == Some(ActiveMenu::Help),
        is_help_hovered,
    );

    if is_editing {
        menu_builder.add_action_button(
            "▶ Play",
            Color::hex("#228b22"),
            Color::WHITE,
            is_play_hovered,
        );
    } else {
        menu_builder.add_action_button(
            "⏹ Stop",
            Color::hex("#dc2626"),
            Color::WHITE,
            is_play_hovered,
        );
    }

    menu_builder.build()
}

/// Builds floating dropdown popup items and returns hit targets with bounding box.
pub fn build_floating_dropdown(
    tree: &mut UiTree,
    active: ActiveMenu,
    anchor_x: f32,
    cursor_pos: Point,
    layout_state: &PanelLayoutState,
    can_undo: bool,
    can_redo: bool,
) -> (WidgetId, Vec<(Rect, DropdownAction)>, Rect) {
    let width = DROPDOWN_WIDTH;
    let mut dropdown_builder = DropdownMenuBuilder::new(tree, anchor_x, MENUBAR_HEIGHT, width);
    let mut items = Vec::new();
    let mut curr_y = MENUBAR_HEIGHT + 4.0;

    let add_item = |builder: &mut DropdownMenuBuilder,
                    items: &mut Vec<(Rect, DropdownAction)>,
                    curr_y: &mut f32,
                    icon: &str,
                    label: &str,
                    shortcut: Option<&str>,
                    enabled: bool,
                    action: Option<DropdownAction>| {
        let row_rect = Rect::new(anchor_x + 4.0, *curr_y, width - 8.0, 24.0);
        let is_hovered = row_rect.contains_point(cursor_pos);
        builder.add_item(icon, label, shortcut, enabled, is_hovered);
        if let Some(act) = action {
            items.push((row_rect, act));
        }
        *curr_y += 25.0;
    };

    let add_separator = |builder: &mut DropdownMenuBuilder, curr_y: &mut f32| {
        builder.add_separator();
        *curr_y += 7.0;
    };

    match active {
        ActiveMenu::File => {
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "🗎",
                "New Project",
                Some("Ctrl N"),
                true,
                None,
            );
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "🗁",
                "Load Scene",
                Some("Ctrl O"),
                true,
                Some(DropdownAction::UiAction(
                    EngineUiAction::OpenLoadSceneDialog,
                )),
            );
            add_separator(&mut dropdown_builder, &mut curr_y);
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "🖫",
                "Save Scene",
                Some("Ctrl S"),
                true,
                Some(DropdownAction::UiAction(EngineUiAction::SaveScene)),
            );
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "🖫",
                "Save Scene As",
                Some("Ctrl Shift S"),
                true,
                Some(DropdownAction::UiAction(
                    EngineUiAction::OpenSaveSceneDialog,
                )),
            );
            add_separator(&mut dropdown_builder, &mut curr_y);
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "⏻",
                "Exit",
                Some("Alt F4"),
                true,
                Some(DropdownAction::UiAction(EngineUiAction::Exit)),
            );
        }
        ActiveMenu::Edit => {
            let undo_act = if can_undo {
                Some(DropdownAction::UiAction(EngineUiAction::Undo))
            } else {
                None
            };
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "↩",
                "Undo",
                Some("Ctrl Z"),
                can_undo,
                undo_act,
            );

            let redo_act = if can_redo {
                Some(DropdownAction::UiAction(EngineUiAction::Redo))
            } else {
                None
            };
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "↪",
                "Redo",
                Some("Ctrl Y"),
                can_redo,
                redo_act,
            );

            add_separator(&mut dropdown_builder, &mut curr_y);
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "⚙",
                "Preferences...",
                None,
                true,
                Some(DropdownAction::OpenPreferences),
            );
        }
        ActiveMenu::View => {
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "⛶",
                "Toggle Fullscreen",
                Some("F11"),
                true,
                None,
            );
            add_separator(&mut dropdown_builder, &mut curr_y);

            let panel_icons = [
                ("☲", "Hierarchy", PanelId::Hierarchy),
                ("🗠", "Stats", PanelId::Stats),
                ("🎛", "Inspector", PanelId::Inspector),
                ("◐", "Material Editor", PanelId::MaterialEditor),
                ("🗀", "Assets", PanelId::Assets),
                ("⌨", "Console", PanelId::Console),
                ("▷", "Timeline", PanelId::AnimationTimeline),
                ("◩", "UI Designer", PanelId::UiDesigner),
            ];

            for &(icon, label, panel) in &panel_icons {
                let is_open = layout_state.is_panel_visible(panel);
                let shortcut = if is_open { Some("✓") } else { None };
                add_item(
                    &mut dropdown_builder,
                    &mut items,
                    &mut curr_y,
                    icon,
                    label,
                    shortcut,
                    true,
                    Some(DropdownAction::TogglePanel(panel)),
                );
            }

            add_separator(&mut dropdown_builder, &mut curr_y);
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "↺",
                "Reset Layout to Default",
                None,
                true,
                Some(DropdownAction::ResetLayout),
            );
        }
        ActiveMenu::Window => {
            let panel_icons = [
                ("☲", "Hierarchy", PanelId::Hierarchy),
                ("🗠", "Stats", PanelId::Stats),
                ("🎛", "Inspector", PanelId::Inspector),
                ("◐", "Material Editor", PanelId::MaterialEditor),
                ("🗀", "Assets", PanelId::Assets),
                ("⌨", "Console", PanelId::Console),
                ("▷", "Timeline", PanelId::AnimationTimeline),
                ("◩", "UI Designer", PanelId::UiDesigner),
            ];

            for &(icon, label, panel) in &panel_icons {
                let is_open = layout_state.is_panel_visible(panel);
                let shortcut = if is_open { Some("✓") } else { None };
                add_item(
                    &mut dropdown_builder,
                    &mut items,
                    &mut curr_y,
                    icon,
                    label,
                    shortcut,
                    true,
                    Some(DropdownAction::TogglePanel(panel)),
                );
            }

            add_separator(&mut dropdown_builder, &mut curr_y);
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "↺",
                "Reset Layout to Default",
                None,
                true,
                Some(DropdownAction::ResetLayout),
            );
        }
        ActiveMenu::Help => {
            add_item(
                &mut dropdown_builder,
                &mut items,
                &mut curr_y,
                "ℹ",
                "About Aeon Engine",
                Some("F1"),
                true,
                Some(DropdownAction::OpenAbout),
            );
        }
    }

    let dropdown_id = dropdown_builder.build();

    // Layout dropdown items and sub-nodes (labels & right-aligned shortcuts)
    let mut layout_y = MENUBAR_HEIGHT + 4.0;
    let children = tree
        .get(dropdown_id)
        .map(|n| n.children.clone())
        .unwrap_or_default();

    for child_id in children {
        if let Some(child_node) = tree.get_mut(child_id) {
            if child_node
                .name
                .as_deref()
                .unwrap_or_default()
                .contains("Separator")
            {
                child_node.computed_rect =
                    Rect::new(anchor_x + 6.0, layout_y + 3.0, width - 12.0, 1.0);
                layout_y += 7.0;
            } else {
                let row_rect = Rect::new(anchor_x + 4.0, layout_y, width - 8.0, 24.0);
                child_node.computed_rect = row_rect;

                let row_children = child_node.children.clone();
                let has_icon = row_children.iter().any(|&cid| {
                    tree.get(cid).and_then(|n| n.name.as_deref()) == Some("DropdownIcon")
                });
                let has_shortcut = row_children.iter().any(|&cid| {
                    tree.get(cid).and_then(|n| n.name.as_deref()) == Some("DropdownShortcut")
                });

                for sub_id in row_children {
                    if let Some(sub_node) = tree.get_mut(sub_id) {
                        match sub_node.name.as_deref() {
                            Some("DropdownIcon") => {
                                sub_node.computed_rect =
                                    Rect::new(row_rect.x + 6.0, layout_y + 5.0, 18.0, 14.0);
                            }
                            Some("DropdownShortcut") => {
                                sub_node.computed_rect = Rect::new(
                                    row_rect.x + row_rect.width - 70.0,
                                    layout_y + 5.0,
                                    62.0,
                                    14.0,
                                );
                            }
                            _ => {
                                let label_x = if has_icon {
                                    row_rect.x + 28.0
                                } else {
                                    row_rect.x + 8.0
                                };
                                let label_w = if has_shortcut {
                                    row_rect.width - (label_x - row_rect.x) - 72.0
                                } else {
                                    row_rect.width - (label_x - row_rect.x) - 8.0
                                };
                                sub_node.computed_rect =
                                    Rect::new(label_x, layout_y + 5.0, label_w, 14.0);
                            }
                        }
                    }
                }
                layout_y += 25.0;
            }
        }
    }

    let total_height = layout_y - MENUBAR_HEIGHT + 4.0;
    if let Some(node) = tree.get_mut(dropdown_id) {
        node.computed_rect = Rect::new(anchor_x, MENUBAR_HEIGHT, width, total_height);
    }
    let dropdown_rect = Rect::new(anchor_x, MENUBAR_HEIGHT, width, total_height);

    (dropdown_id, items, dropdown_rect)
}