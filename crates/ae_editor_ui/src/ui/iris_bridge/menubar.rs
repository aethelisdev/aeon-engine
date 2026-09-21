// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top application menu bar and floating dropdown popup construction routines.

use crate::ui::EngineUiAction;
use crate::ui::iris_bridge::types::{ActiveMenu, DropdownAction};
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::prelude::*;

/// Height of the top menubar panel in physical pixels.
pub const MENUBAR_HEIGHT: f32 = 26.0;

/// Default width of floating dropdown popup menus in physical pixels.
pub const DROPDOWN_WIDTH: f32 = 250.0;

/// Numeric widget tag for the File menu header button.
pub const TAG_MENU_FILE: u64 = 0;
/// Numeric widget tag for the Edit menu header button.
pub const TAG_MENU_EDIT: u64 = 1;
/// Numeric widget tag for the View menu header button.
pub const TAG_MENU_VIEW: u64 = 2;
/// Numeric widget tag for the Window menu header button.
pub const TAG_MENU_WINDOW: u64 = 3;
/// Numeric widget tag for the Help menu header button.
pub const TAG_MENU_HELP: u64 = 4;
/// Numeric widget tag for the Play/Stop editor toolbar action button.
pub const TAG_ACTION_PLAY_PAUSE: u64 = 100;

/// Output geometry and button identifiers returned from top menubar construction.
#[derive(Debug, Clone)]
pub struct TopMenuBarOutput {
    /// Root container widget ID of the top menubar.
    pub root_id: WidgetId,
    /// Generated widget identifiers for each active menu button.
    pub menu_button_ids: [(ActiveMenu, WidgetId); 5],
}

/// Builds the top application menu bar widget tree with semantic roles and tags.
pub fn build_top_menu_bar(
    tree: &mut UiTree,
    screen_width: f32,
    active_menu: Option<ActiveMenu>,
    hovered_menu: Option<ActiveMenu>,
    is_play_hovered: bool,
    is_editing: bool,
) -> TopMenuBarOutput {
    let mut menu_builder = MenuBarBuilder::new(tree, screen_width);

    let btn_file = menu_builder.add_menu_button(
        TAG_MENU_FILE,
        "File",
        active_menu == Some(ActiveMenu::File),
        hovered_menu == Some(ActiveMenu::File),
    );
    let btn_edit = menu_builder.add_menu_button(
        TAG_MENU_EDIT,
        "Edit",
        active_menu == Some(ActiveMenu::Edit),
        hovered_menu == Some(ActiveMenu::Edit),
    );
    let btn_view = menu_builder.add_menu_button(
        TAG_MENU_VIEW,
        "View",
        active_menu == Some(ActiveMenu::View),
        hovered_menu == Some(ActiveMenu::View),
    );
    let btn_window = menu_builder.add_menu_button(
        TAG_MENU_WINDOW,
        "Window",
        active_menu == Some(ActiveMenu::Window),
        hovered_menu == Some(ActiveMenu::Window),
    );
    let btn_help = menu_builder.add_menu_button(
        TAG_MENU_HELP,
        "Help",
        active_menu == Some(ActiveMenu::Help),
        hovered_menu == Some(ActiveMenu::Help),
    );

    if is_editing {
        menu_builder.add_action_button(
            TAG_ACTION_PLAY_PAUSE,
            "▶ Play",
            Color::hex("#228b22"),
            Color::WHITE,
            is_play_hovered,
        );
    } else {
        menu_builder.add_action_button(
            TAG_ACTION_PLAY_PAUSE,
            "⏹ Stop",
            Color::hex("#dc2626"),
            Color::WHITE,
            is_play_hovered,
        );
    }

    let root_id = menu_builder.build();
    TopMenuBarOutput {
        root_id,
        menu_button_ids: [
            (ActiveMenu::File, btn_file),
            (ActiveMenu::Edit, btn_edit),
            (ActiveMenu::View, btn_view),
            (ActiveMenu::Window, btn_window),
            (ActiveMenu::Help, btn_help),
        ],
    }
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
) -> (WidgetId, Vec<DropdownAction>, Rect) {
    let width = DROPDOWN_WIDTH;
    let mut dropdown_builder = DropdownMenuBuilder::new(tree, anchor_x, MENUBAR_HEIGHT, width);
    let mut actions = Vec::new();

    let mut add_item = |builder: &mut DropdownMenuBuilder,
                        icon: &str,
                        label: &str,
                        shortcut: Option<&str>,
                        enabled: bool,
                        action: Option<DropdownAction>| {
        let tag = if let Some(act) = action {
            let idx = actions.len() as u64;
            actions.push(act);
            idx
        } else {
            u64::MAX
        };
        builder.add_item(tag, icon, label, shortcut, enabled, cursor_pos);
    };

    let add_separator = |builder: &mut DropdownMenuBuilder| {
        builder.add_separator();
    };

    match active {
        ActiveMenu::File => {
            add_item(
                &mut dropdown_builder,
                "🗎",
                "New Project",
                Some("Ctrl N"),
                true,
                None,
            );
            add_item(
                &mut dropdown_builder,
                "🗁",
                "Load Scene",
                Some("Ctrl O"),
                true,
                Some(DropdownAction::UiAction(
                    EngineUiAction::OpenLoadSceneDialog,
                )),
            );
            add_separator(&mut dropdown_builder);
            add_item(
                &mut dropdown_builder,
                "🖫",
                "Save Scene",
                Some("Ctrl S"),
                true,
                Some(DropdownAction::UiAction(EngineUiAction::SaveScene)),
            );
            add_item(
                &mut dropdown_builder,
                "🖫",
                "Save Scene As",
                Some("Ctrl Shift S"),
                true,
                Some(DropdownAction::UiAction(
                    EngineUiAction::OpenSaveSceneDialog,
                )),
            );
            add_separator(&mut dropdown_builder);
            add_item(
                &mut dropdown_builder,
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
                "↪",
                "Redo",
                Some("Ctrl Y"),
                can_redo,
                redo_act,
            );

            add_separator(&mut dropdown_builder);

            add_item(
                &mut dropdown_builder,
                "⚙",
                "Preferences...",
                Some("Ctrl ,"),
                true,
                Some(DropdownAction::OpenPreferences),
            );
        }
        ActiveMenu::View => {
            let panels = [
                (PanelId::Hierarchy, "Hierarchy"),
                (PanelId::Inspector, "Inspector"),
                (PanelId::Assets, "Asset Browser"),
                (PanelId::Console, "Console"),
                (PanelId::MaterialEditor, "Material Editor"),
                (PanelId::AnimationTimeline, "Timeline"),
                (PanelId::Stats, "Statistics"),
                (PanelId::UiDesigner, "UI Designer"),
            ];

            for (pid, label) in panels {
                let is_open = layout_state.is_panel_visible(pid);
                let check = if is_open { "✓" } else { " " };
                add_item(
                    &mut dropdown_builder,
                    "",
                    label,
                    Some(check),
                    true,
                    Some(DropdownAction::TogglePanel(pid)),
                );
            }
        }
        ActiveMenu::Window => {
            let panels = [
                (PanelId::Hierarchy, "Hierarchy"),
                (PanelId::Inspector, "Inspector"),
                (PanelId::Assets, "Asset Browser"),
                (PanelId::Console, "Console"),
                (PanelId::MaterialEditor, "Material Editor"),
                (PanelId::AnimationTimeline, "Timeline"),
                (PanelId::Stats, "Statistics"),
                (PanelId::UiDesigner, "UI Designer"),
            ];

            for (pid, label) in panels {
                let is_open = layout_state.is_panel_visible(pid);
                let check = if is_open { "✓" } else { " " };
                add_item(
                    &mut dropdown_builder,
                    "",
                    label,
                    Some(check),
                    true,
                    Some(DropdownAction::TogglePanel(pid)),
                );
            }

            add_separator(&mut dropdown_builder);

            add_item(
                &mut dropdown_builder,
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
                "ℹ",
                "About Aeon Engine",
                Some("F1"),
                true,
                Some(DropdownAction::OpenAbout),
            );
        }
    }

    let (dropdown_id, dropdown_rect) = dropdown_builder.build();
    (dropdown_id, actions, dropdown_rect)
}