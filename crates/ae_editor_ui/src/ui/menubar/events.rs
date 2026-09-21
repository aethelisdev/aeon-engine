// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Menu Bar Event Handling Routines
//!
//! Provides event dispatching for the top horizontal application menu bar
//! and popup dropdown items, driven 100% by [`UiTree::hit_test_target`]
//! with zero manual coordinate heuristics or parallel hit target collections.
//!

use crate::ui::menubar::types::{ActiveMenu, DropdownAction, MenuBarState, TAG_ACTION_PLAY_PAUSE};
use crate::ui::panel_layout::PanelLayoutState;
use crate::ui::types::EngineUiAction;
use irisui::prelude::{Point, UiLayer, UiTree, WidgetRole};
use winit::event::{ElementState, MouseButton, WindowEvent};

/// Contextual mutable state and event queues required for menubar event handling.
pub struct MenuBarEventContext<'a> {
    /// Current mouse cursor position in logical coordinates.
    pub cursor_pos: Point,
    /// Queue of engine UI actions to enqueue commands into.
    pub pending_actions: &'a mut Vec<EngineUiAction>,
    /// Dock panel layout state for toggling panel visibility or resetting layouts.
    pub layout_state: &'a mut PanelLayoutState,
    /// Mutable flag toggling visibility of the preferences modal dialog.
    pub show_preferences: &'a mut bool,
    /// Mutable flag toggling visibility of the about engine modal dialog.
    pub show_about: &'a mut bool,
    /// Whether the editor is currently in Edit mode or Play mode.
    pub is_editing: bool,
    /// Mutable flag requesting application exit.
    pub should_exit: &'a mut bool,
}

/// Dispatches window input events to the application menu bar and active dropdown menus.
///
/// Evaluates mouse clicks and hover coordinates against the active [`UiTree`] utilizing
/// standard tree hit-testing. If an active dropdown menu is open and the user clicks outside,
/// dismisses the dropdown and consumes the event to prevent unintended interactions.
///
/// # Arguments
/// * `state` - Retained menu bar state storing active/hovered categories and cached actions.
/// * `tree` - Immutable reference to the active UI widget tree.
/// * `event` - Window input event from Winit.
/// * `ctx` - Contextual event dispatching targets and mutable state references.
///
/// # Return Value
/// Returns `true` if the event was intercepted and consumed by the menu bar or dropdown popup,
/// or `false` if it should continue propagating down to other editor subsystems.
pub fn handle_menubar_event(
    state: &mut MenuBarState,
    tree: &UiTree,
    event: &WindowEvent,
    ctx: &mut MenuBarEventContext<'_>,
) -> bool {
    match event {
        WindowEvent::CursorMoved { .. } => {
            if let Some(target) = tree.hit_test_target(ctx.cursor_pos) {
                if target.role == WidgetRole::MenuBarItem {
                    if let Some(menu) = ActiveMenu::from_tag(target.tag) {
                        state.hovered_menu = Some(menu);
                        // If any menu is already active/open, hovering over another top menu button
                        //  switches the open dropdown category.
                        if state.active_menu.is_some() && state.active_menu != Some(menu) {
                            state.active_menu = Some(menu);
                        }
                    }
                    state.is_play_hovered = false;
                } else if target.role == WidgetRole::Button && target.tag == TAG_ACTION_PLAY_PAUSE {
                    state.is_play_hovered = true;
                    state.hovered_menu = None;
                } else {
                    state.hovered_menu = None;
                    state.is_play_hovered = false;
                }
            } else {
                state.hovered_menu = None;
                state.is_play_hovered = false;
            }
            false
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => {
            let hit = tree.hit_test_target(ctx.cursor_pos);

            // 1. Popup Dropdown Item or Container Clicks (Highest priority)
            if let Some(ref target) = hit
                && target.layer == UiLayer::Popup
            {
                if target.role == WidgetRole::DropdownItem {
                    let action_idx = target.tag as usize;
                    if action_idx < state.actions.len() {
                        let action = state.actions[action_idx].clone();
                        match action {
                            DropdownAction::UiAction(ui_act) => {
                                if matches!(ui_act, EngineUiAction::Exit) {
                                    *ctx.should_exit = true;
                                }
                                ctx.pending_actions.push(ui_act);
                            }
                            DropdownAction::TogglePanel(pid) => {
                                ctx.layout_state.toggle_panel(pid);
                            }
                            DropdownAction::ResetLayout => {
                                ctx.layout_state.reset_to_default();
                            }
                            DropdownAction::OpenPreferences => {
                                *ctx.show_preferences = true;
                            }
                            DropdownAction::OpenAbout => {
                                *ctx.show_about = true;
                            }
                        }
                    }
                    state.close_dropdown();
                    return true;
                }
                // Non-item click inside popup container (e.g. separator or padding)
                return true;
            }

            // 2. Menu Bar Header Buttons
            if let Some(ref target) = hit {
                if target.role == WidgetRole::MenuBarItem {
                    if let Some(menu) = ActiveMenu::from_tag(target.tag) {
                        if state.active_menu == Some(menu) {
                            state.close_dropdown();
                        } else {
                            state.active_menu = Some(menu);
                        }
                    }
                    return true;
                }

                if target.role == WidgetRole::Button && target.tag == TAG_ACTION_PLAY_PAUSE {
                    let next_mode = if ctx.is_editing {
                        ae_core::modules::EngineMode::Play
                    } else {
                        ae_core::modules::EngineMode::Edit
                    };
                    ctx.pending_actions
                        .push(EngineUiAction::ChangeMode(next_mode));
                    return true;
                }
            }

            // 3. Dismiss Active Dropdown on Click Outside
            if state.active_menu.is_some() {
                state.close_dropdown();
                return true;
            }

            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::menubar::builder::build_top_menu_bar;
    use irisui::prelude::Rect;

    #[test]
    fn test_handle_menubar_click_toggles_active_menu() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "TestRoot");
        let mut state = MenuBarState::default();
        let mut actions = Vec::new();
        let mut layout = PanelLayoutState::new_default();
        let mut show_pref = false;
        let mut show_about = false;
        let mut should_exit = false;

        let _out = build_top_menu_bar(&mut tree, Some(root), 1920.0, &state, true);
        let mut layout_engine = irisui::prelude::LayoutEngine::new();
        let _ = layout_engine.compute_layout(&mut tree, irisui::prelude::Size::new(1920.0, 1080.0));

        // Click over File button (approx x=15.0, y=10.0)
        let click_pos = Point::new(15.0, 10.0);
        let event = WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };

        let mut ctx = MenuBarEventContext {
            cursor_pos: click_pos,
            pending_actions: &mut actions,
            layout_state: &mut layout,
            show_preferences: &mut show_pref,
            show_about: &mut show_about,
            is_editing: true,
            should_exit: &mut should_exit,
        };

        let consumed = handle_menubar_event(&mut state, &tree, &event, &mut ctx);

        assert!(consumed);
        assert_eq!(state.active_menu, Some(ActiveMenu::File));

        // Clicking File again toggles it off
        let mut ctx_again = MenuBarEventContext {
            cursor_pos: click_pos,
            pending_actions: &mut actions,
            layout_state: &mut layout,
            show_preferences: &mut show_pref,
            show_about: &mut show_about,
            is_editing: true,
            should_exit: &mut should_exit,
        };
        let consumed_again = handle_menubar_event(&mut state, &tree, &event, &mut ctx_again);

        assert!(consumed_again);
        assert_eq!(state.active_menu, None);
    }

    #[test]
    fn test_dismiss_active_menu_on_click_outside() {
        let tree = UiTree::new();
        let mut state = MenuBarState {
            active_menu: Some(ActiveMenu::File),
            ..Default::default()
        };
        let mut actions = Vec::new();
        let mut layout = PanelLayoutState::new_default();
        let mut show_pref = false;
        let mut show_about = false;
        let mut should_exit = false;

        let click_outside = Point::new(500.0, 500.0);
        let event = WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };

        let mut ctx = MenuBarEventContext {
            cursor_pos: click_outside,
            pending_actions: &mut actions,
            layout_state: &mut layout,
            show_preferences: &mut show_pref,
            show_about: &mut show_about,
            is_editing: true,
            should_exit: &mut should_exit,
        };

        let consumed = handle_menubar_event(&mut state, &tree, &event, &mut ctx);

        assert!(consumed);
        assert_eq!(state.active_menu, None);
    }

    #[test]
    fn test_handle_menubar_hover_switches_category_when_active() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "TestRoot");
        let mut state = MenuBarState {
            active_menu: Some(ActiveMenu::File),
            ..Default::default()
        };
        let mut actions = Vec::new();
        let mut layout = PanelLayoutState::new_default();
        let mut show_pref = false;
        let mut show_about = false;
        let mut should_exit = false;

        let _out = build_top_menu_bar(&mut tree, Some(root), 1920.0, &state, true);
        let mut layout_engine = irisui::prelude::LayoutEngine::new();
        let _ = layout_engine.compute_layout(&mut tree, irisui::prelude::Size::new(1920.0, 1080.0));

        // Find Edit button position
        let edit_node = tree
            .find_node_by_tag(ActiveMenu::Edit.to_tag())
            .expect("Edit button should exist");
        let edit_center = Point::new(
            edit_node.1.computed_rect.x + 5.0,
            edit_node.1.computed_rect.y + 5.0,
        );

        let move_event = WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(edit_center.x as f64, edit_center.y as f64),
        };

        let mut ctx = MenuBarEventContext {
            cursor_pos: edit_center,
            pending_actions: &mut actions,
            layout_state: &mut layout,
            show_preferences: &mut show_pref,
            show_about: &mut show_about,
            is_editing: true,
            should_exit: &mut should_exit,
        };

        let consumed = handle_menubar_event(&mut state, &tree, &move_event, &mut ctx);

        assert!(!consumed); // CursorMoved is not consumed
        assert_eq!(state.hovered_menu, Some(ActiveMenu::Edit));
        assert_eq!(state.active_menu, Some(ActiveMenu::Edit));
    }

    #[test]
    fn test_dropdown_item_click_dispatches_action() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "TestRoot");
        let mut actions = Vec::new();
        let mut layout = PanelLayoutState::new_default();
        let mut show_pref = false;
        let mut show_about = false;
        let mut should_exit = false;

        // Build File dropdown
        let (_dd_id, dd_actions, dd_rect) = crate::ui::menubar::builder::build_floating_dropdown(
            &mut tree,
            Some(root),
            crate::ui::menubar::builder::DropdownBuildParams {
                active: ActiveMenu::File,
                anchor_x: 10.0,
                cursor_pos: Point::new(0.0, 0.0),
                layout_state: &layout,
                can_undo: false,
                can_redo: false,
            },
        );
        let mut state = MenuBarState {
            active_menu: Some(ActiveMenu::File),
            actions: dd_actions,
            dropdown_rect: Some(dd_rect),
            ..Default::default()
        };

        // Click on "Save Scene" (item index 1 in actions, tag 1: item y is approx 85.0..109.0)
        let save_click = Point::new(30.0, 95.0);
        let click_event = WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };

        let mut ctx = MenuBarEventContext {
            cursor_pos: save_click,
            pending_actions: &mut actions,
            layout_state: &mut layout,
            show_preferences: &mut show_pref,
            show_about: &mut show_about,
            is_editing: true,
            should_exit: &mut should_exit,
        };

        let consumed = handle_menubar_event(&mut state, &tree, &click_event, &mut ctx);

        assert!(consumed);
        assert_eq!(state.active_menu, None); // Menu closed on click
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], EngineUiAction::SaveScene));
    }

    #[test]
    fn test_play_pause_button_dispatches_mode_change() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "TestRoot");
        let mut state = MenuBarState::default();
        let mut actions = Vec::new();
        let mut layout = PanelLayoutState::new_default();
        let mut show_pref = false;
        let mut show_about = false;
        let mut should_exit = false;

        let _out = build_top_menu_bar(&mut tree, Some(root), 1920.0, &state, true);
        let mut layout_engine = irisui::prelude::LayoutEngine::new();
        let _ = layout_engine.compute_layout(&mut tree, irisui::prelude::Size::new(1920.0, 1080.0));

        let play_node = tree
            .find_node_by_tag(TAG_ACTION_PLAY_PAUSE)
            .expect("Play button should exist");
        let play_center = Point::new(
            play_node.1.computed_rect.x + 5.0,
            play_node.1.computed_rect.y + 5.0,
        );

        let click_event = WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };

        let mut ctx = MenuBarEventContext {
            cursor_pos: play_center,
            pending_actions: &mut actions,
            layout_state: &mut layout,
            show_preferences: &mut show_pref,
            show_about: &mut show_about,
            is_editing: true,
            should_exit: &mut should_exit,
        };

        let consumed = handle_menubar_event(&mut state, &tree, &click_event, &mut ctx);

        assert!(consumed);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            EngineUiAction::ChangeMode(ae_core::modules::EngineMode::Play)
        ));
    }
}