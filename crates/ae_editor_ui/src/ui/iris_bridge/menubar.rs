// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top Application Menu Bar & Floating Dropdown Subsystem
//!
//! Provides the primary desktop application header menu bar (File, Edit, View,
//! Window, Help) and elevated floating dropdown popup cascades using 100% pure
//! declarative [`UiScope`].
//!

use crate::ui::EngineUiAction;
use crate::ui::iris_bridge::types::{ActiveMenu, DropdownAction};
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::prelude::*;

/// Height of the top menubar panel in physical pixels.
pub const MENUBAR_HEIGHT: f32 = 26.0;

/// Default width of floating dropdown popup menus in physical pixels.
pub const DROPDOWN_WIDTH: f32 = 260.0;

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

/// Builds the top application menu bar widget tree directly using declarative [`UiScope`].
///
/// Renders the full-width header strip in deep obsidian black (`#090a0d`) with `SpaceBetween`
/// flexbox distribution: desktop menu triggers on the left, primary action controls on the right.
///
/// Hover and active highlights are resolved natively by the framework without manual
/// external coordinate hit-testing.
pub fn build_top_menu_bar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    screen_width: f32,
    active_menu: Option<ActiveMenu>,
    is_editing: bool,
    cursor_pos: Point,
) -> TopMenuBarOutput {
    let mut scope = UiScope::new(tree, parent_id);
    let mut btn_file = WidgetId::default();
    let mut btn_edit = WidgetId::default();
    let mut btn_view = WidgetId::default();
    let mut btn_window = WidgetId::default();
    let mut btn_help = WidgetId::default();

    // Sleek, modern deep obsidian black theme as requested by user
    let bar_style = Style::new()
        .flex_row()
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::Center)
        .width(screen_width)
        .height(MENUBAR_HEIGHT)
        .padding_insets(Insets::new(0.0, 6.0, 0.0, 10.0))
        .background(Color::rgba(0.035, 0.040, 0.050, 1.0));

    let root_id = scope.container(bar_style, |bar| {
        // 1. Left Group: Desktop Menu Headers (File, Edit, View, Window, Help)
        let left_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(2.0);

        bar.container(left_style, |left| {
            btn_file = left
                .menu_bar_item("File", TAG_MENU_FILE, active_menu == Some(ActiveMenu::File))
                .id;
            btn_edit = left
                .menu_bar_item("Edit", TAG_MENU_EDIT, active_menu == Some(ActiveMenu::Edit))
                .id;
            btn_view = left
                .menu_bar_item("View", TAG_MENU_VIEW, active_menu == Some(ActiveMenu::View))
                .id;
            btn_window = left
                .menu_bar_item(
                    "Window",
                    TAG_MENU_WINDOW,
                    active_menu == Some(ActiveMenu::Window),
                )
                .id;
            btn_help = left
                .menu_bar_item("Help", TAG_MENU_HELP, active_menu == Some(ActiveMenu::Help))
                .id;
        });

        // 2. Right Group: Play/Stop Action Button
        let right_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(6.0);

        bar.container(right_style, |right| {
            if is_editing {
                right.menu_action_button(
                    "▶ Play",
                    TAG_ACTION_PLAY_PAUSE,
                    Color::hex("#228b22"),
                    Color::WHITE,
                );
            } else {
                right.menu_action_button(
                    "⏹ Stop",
                    TAG_ACTION_PLAY_PAUSE,
                    Color::hex("#dc2626"),
                    Color::WHITE,
                );
            }
        });
    });

    // Reconcile 0ms hover styling across top menu items natively
    let mut bar_scope = UiScope::new(tree, root_id);
    bar_scope.finish_layout_with_hover(
        Rect::new(0.0, 0.0, screen_width, MENUBAR_HEIGHT),
        cursor_pos,
    );

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

/// Parameters required to construct an elevated floating dropdown cascade.
///
/// Groups configuration values into a cohesive descriptor struct adhering to
/// zero-suppression architecture rules.
#[derive(Debug, Clone, Copy)]
pub struct DropdownMenuParams<'a> {
    /// Active top-level menu category currently opened.
    pub active: ActiveMenu,
    /// Left horizontal screen coordinate anchor in physical pixels.
    pub anchor_x: f32,
    /// Reference to the active panel docking and visibility state.
    pub layout_state: &'a PanelLayoutState,
    /// Whether the editor command stack supports undo operations.
    pub can_undo: bool,
    /// Whether the editor command stack supports redo operations.
    pub can_redo: bool,
    /// Current mouse cursor location in physical pixels.
    pub cursor_pos: Point,
}

/// Builds floating dropdown popup items using declarative [`UiScope`].
///
/// Dispatches menu actions and sub-options inside an elevated [`UiLayer::Popup`] card.
/// Automatically formats leading symbols, labels, and right-aligned shortcut keys.
pub fn build_floating_dropdown(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: DropdownMenuParams<'_>,
) -> (WidgetId, Vec<DropdownAction>, Rect) {
    let width = DROPDOWN_WIDTH;
    let mut actions = Vec::new();
    let mut scope = UiScope::new(tree, parent_id);

    let dropdown_id = scope.dropdown_menu_card(params.anchor_x, MENUBAR_HEIGHT, width, |card| {
        let mut add_item = |scope: &mut UiScope<'_>,
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
            scope.dropdown_item(tag, icon, label, shortcut, enabled);
        };

        match params.active {
            ActiveMenu::File => {
                add_item(card, "🗎", "New Project", Some("Ctrl N"), true, None);
                add_item(
                    card,
                    "🗁",
                    "Load Scene",
                    Some("Ctrl O"),
                    true,
                    Some(DropdownAction::UiAction(
                        EngineUiAction::OpenLoadSceneDialog,
                    )),
                );
                card.dropdown_separator();
                add_item(
                    card,
                    "🖫",
                    "Save Scene",
                    Some("Ctrl S"),
                    true,
                    Some(DropdownAction::UiAction(EngineUiAction::SaveScene)),
                );
                add_item(
                    card,
                    "🖫",
                    "Save Scene As",
                    Some("Ctrl Shift S"),
                    true,
                    Some(DropdownAction::UiAction(
                        EngineUiAction::OpenSaveSceneDialog,
                    )),
                );
                card.dropdown_separator();
                add_item(
                    card,
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
                add_item(card, "↩", "Undo", Some("Ctrl Z"), params.can_undo, undo_act);

                let redo_act = if params.can_redo {
                    Some(DropdownAction::UiAction(EngineUiAction::Redo))
                } else {
                    None
                };
                add_item(card, "↪", "Redo", Some("Ctrl Y"), params.can_redo, redo_act);

                card.dropdown_separator();

                add_item(
                    card,
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
                    let is_open = params.layout_state.is_panel_visible(pid);
                    let check = if is_open { "✓" } else { " " };
                    add_item(
                        card,
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
                    let is_open = params.layout_state.is_panel_visible(pid);
                    let check = if is_open { "✓" } else { " " };
                    add_item(
                        card,
                        "",
                        label,
                        Some(check),
                        true,
                        Some(DropdownAction::TogglePanel(pid)),
                    );
                }

                card.dropdown_separator();

                add_item(
                    card,
                    "↺",
                    "Reset Layout to Default",
                    None,
                    true,
                    Some(DropdownAction::ResetLayout),
                );
            }
            ActiveMenu::Help => {
                add_item(
                    card,
                    "ℹ",
                    "About Aeon Engine",
                    Some("F1"),
                    true,
                    Some(DropdownAction::OpenAbout),
                );
            }
        }
    });

    let dd_h = irisui::prelude::measure_height(tree, dropdown_id);
    let mut dd_scope = UiScope::new(tree, dropdown_id);
    let dd_bounds = Rect::new(params.anchor_x, MENUBAR_HEIGHT, width, dd_h);
    dd_scope.finish_layout_with_hover(dd_bounds, params.cursor_pos);

    let dd_rect = tree.get(dropdown_id).map_or(
        Rect::new(params.anchor_x, MENUBAR_HEIGHT, width, dd_h),
        |n| n.computed_rect,
    );

    (dropdown_id, actions, dd_rect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_menu_bar_declarative_build_and_roles() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");

        let output = build_top_menu_bar(
            &mut tree,
            root,
            1920.0,
            Some(ActiveMenu::File),
            true,
            Point::new(10.0, 10.0),
        );

        let root_node = tree.get(output.root_id).expect("Root menubar must exist");
        assert_eq!(root_node.style.height, Some(MENUBAR_HEIGHT));
        assert_eq!(root_node.style.width, Some(1920.0));
        assert_eq!(
            root_node.style.justify_content,
            JustifyContent::SpaceBetween
        );

        // Verify all 5 menu headers exist with MenuBarItem role and semantic tags
        for (idx, (menu, btn_id)) in output.menu_button_ids.iter().enumerate() {
            let btn_node = tree.get(*btn_id).expect("Menu button node must exist");
            assert_eq!(btn_node.role, WidgetRole::MenuBarItem);
            assert_eq!(btn_node.tag, idx as u64);
            if *menu == ActiveMenu::File {
                assert_eq!(btn_node.text_color, Color::hex("#00e5ff"));
            }
        }
    }

    #[test]
    fn test_floating_dropdown_declarative_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let layout_state = PanelLayoutState::default();

        let (dd_id, actions, dd_rect) = build_floating_dropdown(
            &mut tree,
            root,
            DropdownMenuParams {
                active: ActiveMenu::File,
                anchor_x: 20.0,
                layout_state: &layout_state,
                can_undo: true,
                can_redo: true,
                cursor_pos: Point::new(25.0, 40.0),
            },
        );

        let dd_node = tree.get(dd_id).expect("Dropdown card must exist");
        assert_eq!(dd_node.role, WidgetRole::DropdownPopup);
        assert_eq!(dd_node.layer, UiLayer::Popup);
        assert!(dd_rect.width >= DROPDOWN_WIDTH);
        assert!(actions.len() >= 4);

        // Verify height matches measured content and does not elongate unnecessarily
        assert!(
            dd_rect.height > 100.0 && dd_rect.height < 200.0,
            "Dropdown card height must snugly fit items, got {}",
            dd_rect.height
        );

        // Verify that children are DropdownItem or dividers
        for child_id in &dd_node.children {
            let child = tree.get(*child_id).expect("Dropdown child must exist");
            assert!(
                child.role == WidgetRole::DropdownItem || child.role == WidgetRole::Separator,
                "Child role must be DropdownItem or divider"
            );
            if child.role == WidgetRole::DropdownItem {
                for grandchild_id in &child.children {
                    let gc = tree.get(*grandchild_id).expect("Grandchild must exist");
                    if gc.role == WidgetRole::DropdownLabel {
                        assert!(
                            gc.computed_rect.width >= 100.0,
                            "DropdownLabel width must be at least 100.0 to prevent truncation, got {}",
                            gc.computed_rect.width
                        );
                    }
                }
            }
        }
    }
}