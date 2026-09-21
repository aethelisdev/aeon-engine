// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Menu Bar Types and State Model
//!
//! Provides the enumeration of menu categories, interactive dropdown action payloads,
//! layout sizing constants, and the retained state model for the top application menu bar.
//!

use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use irisui::prelude::{Rect, WidgetId};

/// Height of the top application menu bar panel in physical/logical pixels.
pub const MENUBAR_HEIGHT: f32 = 26.0;

/// Default width of floating dropdown popup menus in logical pixels.
pub const DROPDOWN_WIDTH: f32 = 250.0;

/// Numeric widget tag indicating an unassigned or neutral widget node.
pub const TAG_NONE: u64 = 0;

/// Numeric widget tag identifying the File menu button in the UI tree.
pub const TAG_MENU_FILE: u64 = 1001;

/// Numeric widget tag identifying the Edit menu button in the UI tree.
pub const TAG_MENU_EDIT: u64 = 1002;

/// Numeric widget tag identifying the View menu button in the UI tree.
pub const TAG_MENU_VIEW: u64 = 1003;

/// Numeric widget tag identifying the Window menu button in the UI tree.
pub const TAG_MENU_WINDOW: u64 = 1004;

/// Numeric widget tag identifying the Help menu button in the UI tree.
pub const TAG_MENU_HELP: u64 = 1005;

/// Numeric widget tag identifying the Play/Stop editor toolbar action button.
pub const TAG_ACTION_PLAY_PAUSE: u64 = 100;

/// Top menu bar categories for active open dropdown menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActiveMenu {
    /// File operations (New Project, Load Scene, Save Scene, Save Scene As, Exit).
    File,
    /// Edit actions (Undo, Redo, Preferences).
    Edit,
    /// View layout and tool panel visibility toggles.
    View,
    /// Tool windows and workspace resets.
    Window,
    /// Documentation, engine information, and shortcuts.
    Help,
}

impl ActiveMenu {
    /// Converts this menu category into its unique numeric widget tag.
    ///
    /// # Return Value
    /// Returns the corresponding `u64` tag matching constants [`TAG_MENU_FILE`] through [`TAG_MENU_HELP`].
    #[inline]
    #[must_use]
    pub const fn to_tag(self) -> u64 {
        match self {
            Self::File => TAG_MENU_FILE,
            Self::Edit => TAG_MENU_EDIT,
            Self::View => TAG_MENU_VIEW,
            Self::Window => TAG_MENU_WINDOW,
            Self::Help => TAG_MENU_HELP,
        }
    }

    /// Resolves an active menu category from its numeric widget tag.
    ///
    /// # Arguments
    /// * `tag` - The numeric identifier assigned to the widget node during construction.
    ///
    /// # Return Value
    /// Returns `Some(ActiveMenu)` if the tag corresponds to a known menu header, or `None` otherwise.
    #[inline]
    #[must_use]
    pub const fn from_tag(tag: u64) -> Option<Self> {
        match tag {
            TAG_MENU_FILE => Some(Self::File),
            TAG_MENU_EDIT => Some(Self::Edit),
            TAG_MENU_VIEW => Some(Self::View),
            TAG_MENU_WINDOW => Some(Self::Window),
            TAG_MENU_HELP => Some(Self::Help),
            _ => None,
        }
    }
}

/// Action payload dispatched from clicking a dropdown menu item.
///
/// Encapsulates actions originating from the floating dropdown popup so that they can be
/// handled uniformly by the editor workbench without direct UI tree coupling.
#[derive(Debug, Clone)]
pub enum DropdownAction {
    /// Dispatches an event-bus UI action directly to the engine runtime.
    UiAction(EngineUiAction),
    /// Toggles the docking visibility of a specific tool panel.
    TogglePanel(PanelId),
    /// Resets docking layout and floating panels to the default engine preset.
    ResetLayout,
    /// Opens the engine settings and preferences modal dialog.
    OpenPreferences,
    /// Opens the engine information and about modal dialog.
    OpenAbout,
}

/// Retained interaction and overlay state for the top application menu bar.
///
/// Stores active/hovered menu identifiers, cached dropdown actions for O(1) tag lookup,
/// button node IDs for dynamic position anchors, and floating dropdown bounds.
#[derive(Debug, Default, Clone)]
pub struct MenuBarState {
    /// Currently open dropdown menu category, or `None` if all menus are closed.
    pub active_menu: Option<ActiveMenu>,
    /// Menu category button currently hovered by the mouse pointer.
    pub hovered_menu: Option<ActiveMenu>,
    /// Whether the Play/Stop mode toggle toolbar button is currently hovered.
    pub is_play_hovered: bool,
    /// Indexed list of dispatched actions corresponding to active dropdown items.
    pub actions: Vec<DropdownAction>,
    /// Widget identifiers of active top menu header buttons for dynamic coordinate resolution.
    pub button_ids: Vec<(ActiveMenu, WidgetId)>,
    /// Cached bounding box of the active floating dropdown popup.
    pub dropdown_rect: Option<Rect>,
}

impl MenuBarState {
    /// Creates a new, default menu bar state with no active or hovered menus.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets transient dropdown state and closes any active floating menu.
    pub fn close_dropdown(&mut self) {
        self.active_menu = None;
        self.actions.clear();
        self.dropdown_rect = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_menu_tag_roundtrip() {
        let menus = [
            ActiveMenu::File,
            ActiveMenu::Edit,
            ActiveMenu::View,
            ActiveMenu::Window,
            ActiveMenu::Help,
        ];
        for menu in menus {
            let tag = menu.to_tag();
            assert_eq!(ActiveMenu::from_tag(tag), Some(menu));
        }
        assert_eq!(ActiveMenu::from_tag(999), None);
    }

    #[test]
    fn test_menu_bar_state_close_dropdown() {
        let mut state = MenuBarState::new();
        state.active_menu = Some(ActiveMenu::File);
        state.actions.push(DropdownAction::ResetLayout);
        state.dropdown_rect = Some(Rect::new(0.0, 26.0, 250.0, 100.0));

        state.close_dropdown();
        assert_eq!(state.active_menu, None);
        assert!(state.actions.is_empty());
        assert_eq!(state.dropdown_rect, None);
    }
}