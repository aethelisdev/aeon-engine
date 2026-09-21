// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Menu Bar and Floating Dropdown Builders
//!
//! Provides factory routines constructing the top horizontal application menu bar
//! and floating dropdown overlays utilizing standard [`iris_widgets::MenuBarBuilder`]
//! and [`iris_widgets::DropdownMenuBuilder`].
//!

use crate::ui::menubar::types::{
    ActiveMenu, DROPDOWN_WIDTH, DropdownAction, MENUBAR_HEIGHT, MenuBarState,
    TAG_ACTION_PLAY_PAUSE, TAG_MENU_EDIT, TAG_MENU_FILE, TAG_MENU_HELP, TAG_MENU_VIEW,
    TAG_MENU_WINDOW,
};
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::types::EngineUiAction;
use iris_widgets::{DropdownMenuBuilder, MenuBarBuilder};
use irisui::prelude::{Color, Point, Rect, UiTree, WidgetId};

/// Output geometry and button identifiers generated during top menu bar construction.
#[derive(Debug, Clone)]
pub struct TopMenuBarOutput {
    /// Root container widget ID of the top menubar.
    pub root_id: WidgetId,
    /// Generated widget identifiers for each top menu category button.
    pub menu_button_ids: [(ActiveMenu, WidgetId); 5],
}

/// Builds the top application menu bar widget tree with semantic roles and tags.
///
/// Constructs the horizontal bar spanning `screen_width`, attaching menu category buttons
/// ("File", "Edit", "View", "Window", "Help") and the Play/Stop mode toggle action button.
///
/// # Arguments
/// * `tree` - Mutable reference to the active UI widget tree.
/// * `parent_id` - Optional parent widget node (e.g. root canvas) to attach the menubar to.
/// * `screen_width` - Full width of the display surface in logical pixels.
/// * `state` - Current retained state of the menu bar indicating active and hovered buttons.
/// * `is_editing` - Whether the engine is currently in editing mode (true) or playing mode (false).
///
/// # Return Value
/// Returns a [`TopMenuBarOutput`] containing the root widget ID and button IDs.
pub fn build_top_menu_bar(
    tree: &mut UiTree,
    parent_id: Option<WidgetId>,
    screen_width: f32,
    state: &MenuBarState,
    is_editing: bool,
) -> TopMenuBarOutput {
    let mut menu_builder = MenuBarBuilder::new(tree, screen_width);

    let btn_file = menu_builder.add_menu_button(
        TAG_MENU_FILE,
        "File",
        state.active_menu == Some(ActiveMenu::File),
        state.hovered_menu == Some(ActiveMenu::File),
    );
    let btn_edit = menu_builder.add_menu_button(
        TAG_MENU_EDIT,
        "Edit",
        state.active_menu == Some(ActiveMenu::Edit),
        state.hovered_menu == Some(ActiveMenu::Edit),
    );
    let btn_view = menu_builder.add_menu_button(
        TAG_MENU_VIEW,
        "View",
        state.active_menu == Some(ActiveMenu::View),
        state.hovered_menu == Some(ActiveMenu::View),
    );
    let btn_window = menu_builder.add_menu_button(
        TAG_MENU_WINDOW,
        "Window",
        state.active_menu == Some(ActiveMenu::Window),
        state.hovered_menu == Some(ActiveMenu::Window),
    );
    let btn_help = menu_builder.add_menu_button(
        TAG_MENU_HELP,
        "Help",
        state.active_menu == Some(ActiveMenu::Help),
        state.hovered_menu == Some(ActiveMenu::Help),
    );

    if is_editing {
        menu_builder.add_action_button(
            TAG_ACTION_PLAY_PAUSE,
            "▶ Play",
            Color::hex("#228b22"),
            Color::WHITE,
            state.is_play_hovered,
        );
    } else {
        menu_builder.add_action_button(
            TAG_ACTION_PLAY_PAUSE,
            "⏹ Stop",
            Color::hex("#dc2626"),
            Color::WHITE,
            state.is_play_hovered,
        );
    }

    let root_id = menu_builder.build(parent_id);

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

/// Parameters for constructing a floating dropdown menu.
#[derive(Debug, Clone)]
pub struct DropdownBuildParams<'a> {
    /// Active menu category being expanded.
    pub active: ActiveMenu,
    /// Horizontal screen coordinate aligning with the anchor button.
    pub anchor_x: f32,
    /// Current mouse cursor position for hover evaluation.
    pub cursor_pos: Point,
    /// Current dock panel layout state to query panel visibility.
    pub layout_state: &'a PanelLayoutState,
    /// Whether an undo command is currently available.
    pub can_undo: bool,
    /// Whether a redo command is currently available.
    pub can_redo: bool,
}

/// Builds floating dropdown popup items and returns the widget ID, indexed actions, and bounding box.
///
/// Dynamically populates items for the specified [`ActiveMenu`] category, resolving keyboard shortcuts,
/// status checks (e.g. visible panels in View/Window), and assigns sequential numeric tags corresponding
/// to the generated action payload list for zero-allocation event dispatching.
///
/// # Arguments
/// * `tree` - Mutable reference to the UI widget tree.
/// * `parent_id` - Optional parent widget node (e.g. root canvas) to attach the popup container to.
/// * `params` - Sizing, anchor, visibility, and capability parameters.
///
/// # Return Value
/// Returns a tuple of `(WidgetId, Vec<DropdownAction>, Rect)` containing the popup node ID,
/// indexed action array, and the computed bounding rectangle.
pub fn build_floating_dropdown(
    tree: &mut UiTree,
    parent_id: Option<WidgetId>,
    params: DropdownBuildParams<'_>,
) -> (WidgetId, Vec<DropdownAction>, Rect) {
    let width = DROPDOWN_WIDTH;
    let mut dropdown_builder =
        DropdownMenuBuilder::new(tree, params.anchor_x, MENUBAR_HEIGHT, width);
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
        builder.add_item(tag, icon, label, shortcut, enabled, params.cursor_pos);
    };

    let add_separator = |builder: &mut DropdownMenuBuilder| {
        builder.add_separator();
    };

    match params.active {
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
            let undo_act = if params.can_undo {
                Some(DropdownAction::UiAction(EngineUiAction::Undo))
            } else {
                None
            };
            add_item(
                &mut dropdown_builder,
                "↩",
                "Undo",
                Some("Ctrl Z"),
                params.can_undo,
                undo_act,
            );

            let redo_act = if params.can_redo {
                Some(DropdownAction::UiAction(EngineUiAction::Redo))
            } else {
                None
            };
            add_item(
                &mut dropdown_builder,
                "↪",
                "Redo",
                Some("Ctrl Y"),
                params.can_redo,
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
            for &pid in PanelId::all_tool_panels() {
                let is_open = params.layout_state.is_panel_visible(pid);
                let check = if is_open { "✓" } else { " " };
                add_item(
                    &mut dropdown_builder,
                    "",
                    pid.title(),
                    Some(check),
                    true,
                    Some(DropdownAction::TogglePanel(pid)),
                );
            }
        }
        ActiveMenu::Window => {
            for &pid in PanelId::all_tool_panels() {
                let is_open = params.layout_state.is_panel_visible(pid);
                let check = if is_open { "✓" } else { " " };
                add_item(
                    &mut dropdown_builder,
                    "",
                    pid.title(),
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

    let (dropdown_id, dropdown_rect) = dropdown_builder.build(parent_id);
    (dropdown_id, actions, dropdown_rect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_top_menu_bar_structure() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "TestRoot");
        let state = MenuBarState::default();

        let output = build_top_menu_bar(&mut tree, Some(root), 1920.0, &state, true);
        assert!(tree.get(output.root_id).is_some());
        assert_eq!(output.menu_button_ids.len(), 5);

        for (menu, btn_id) in output.menu_button_ids {
            let node = tree.get(btn_id).expect("Menu button node should exist");
            assert_eq!(node.tag, menu.to_tag());
        }
    }

    #[test]
    fn test_build_floating_dropdown_all_categories() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "TestRoot");
        let layout_state = PanelLayoutState::new_default();
        let cursor_pos = Point::new(50.0, 50.0);

        let menus = [
            ActiveMenu::File,
            ActiveMenu::Edit,
            ActiveMenu::View,
            ActiveMenu::Window,
            ActiveMenu::Help,
        ];

        for menu in menus {
            let (dd_id, actions, rect) = build_floating_dropdown(
                &mut tree,
                Some(root),
                DropdownBuildParams {
                    active: menu,
                    anchor_x: 100.0,
                    cursor_pos,
                    layout_state: &layout_state,
                    can_undo: true,
                    can_redo: true,
                },
            );
            assert!(tree.get(dd_id).is_some());
            assert!(!actions.is_empty());
            assert_eq!(rect.x, 100.0);
            assert_eq!(rect.y, MENUBAR_HEIGHT);
            assert_eq!(rect.width, DROPDOWN_WIDTH);
        }
    }
}