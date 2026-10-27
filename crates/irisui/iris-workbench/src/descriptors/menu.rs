// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Menu Bar and Action Descriptors
//!
//! Provides declarative descriptors for configuring application menu bars,
//! cascading submenus, shortcuts, and command actions.

/// Represents an item within a dropdown menu or cascading submenu.
///
/// Supports interactive actions, nested submenus, and visual separators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItemDescriptor {
    /// Interactive action triggering an application or engine command.
    Action {
        /// User-visible text label (e.g. `"Save Scene"`).
        label: String,

        /// Optional keyboard shortcut label displayed on the right edge (e.g. `"Ctrl+S"`).
        shortcut: Option<String>,

        /// Semantic interaction tag dispatched when the action is triggered.
        tag: u64,

        /// Whether the action is active and interactive (if `false`, rendered disabled).
        enabled: bool,
    },

    /// Nested cascading submenu branching into secondary options.
    Submenu {
        /// User-visible text label for the submenu entry (e.g. `"Export As..."`).
        label: String,

        /// Semantic interaction tag for the submenu trigger.
        tag: u64,

        /// Child items contained within this cascading submenu.
        items: Vec<MenuItemDescriptor>,
    },

    /// Visual horizontal divider separating logical groups of actions.
    Separator,
}

impl MenuItemDescriptor {
    /// Creates an enabled action item without a keyboard shortcut.
    ///
    /// # Arguments
    /// * `label` - User-facing text label.
    /// * `tag` - Semantic interaction tag.
    #[inline]
    #[must_use]
    pub fn action(label: impl Into<String>, tag: u64) -> Self {
        Self::Action {
            label: label.into(),
            shortcut: None,
            tag,
            enabled: true,
        }
    }

    /// Creates an enabled action item with a keyboard shortcut.
    ///
    /// # Arguments
    /// * `label` - User-facing text label.
    /// * `shortcut` - Keyboard shortcut representation (e.g. `"Ctrl+Z"`).
    /// * `tag` - Semantic interaction tag.
    #[inline]
    #[must_use]
    pub fn action_with_shortcut(
        label: impl Into<String>,
        shortcut: impl Into<String>,
        tag: u64,
    ) -> Self {
        Self::Action {
            label: label.into(),
            shortcut: Some(shortcut.into()),
            tag,
            enabled: true,
        }
    }

    /// Creates a disabled action item (rendered grayed out and non-interactive).
    ///
    /// # Arguments
    /// * `label` - User-facing text label.
    /// * `tag` - Semantic interaction tag.
    #[inline]
    #[must_use]
    pub fn disabled_action(label: impl Into<String>, tag: u64) -> Self {
        Self::Action {
            label: label.into(),
            shortcut: None,
            tag,
            enabled: false,
        }
    }

    /// Creates a nested cascading submenu item.
    ///
    /// # Arguments
    /// * `label` - Submenu label.
    /// * `tag` - Semantic interaction tag for hovering or expanding.
    /// * `items` - Child menu items within the submenu.
    #[inline]
    #[must_use]
    pub fn submenu(label: impl Into<String>, tag: u64, items: Vec<MenuItemDescriptor>) -> Self {
        Self::Submenu {
            label: label.into(),
            tag,
            items,
        }
    }

    /// Creates a horizontal separator divider.
    #[inline]
    #[must_use]
    pub const fn separator() -> Self {
        Self::Separator
    }

    /// Returns the semantic interaction tag associated with this item, if applicable.
    #[inline]
    #[must_use]
    pub const fn tag(&self) -> Option<u64> {
        match self {
            Self::Action { tag, .. } | Self::Submenu { tag, .. } => Some(*tag),
            Self::Separator => None,
        }
    }

    /// Returns `true` if this item is a separator.
    #[inline]
    #[must_use]
    pub const fn is_separator(&self) -> bool {
        matches!(self, Self::Separator)
    }

    /// Returns `true` if this item is an interactive action that is enabled.
    #[inline]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        match self {
            Self::Action { enabled, .. } => *enabled,
            Self::Submenu { .. } => true,
            Self::Separator => false,
        }
    }
}

/// A top-level menu category displayed on the menu bar (e.g. `"File"`, `"Edit"`, `"Window"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSectionDescriptor {
    /// User-facing label displayed on the menu bar (e.g. `"File"`).
    pub title: String,

    /// Semantic interaction tag for the top-level trigger button.
    pub tag: u64,

    /// Ordered list of child items contained in this dropdown menu.
    pub items: Vec<MenuItemDescriptor>,
}

impl MenuSectionDescriptor {
    /// Creates a new menu section with the specified title and trigger tag.
    ///
    /// # Arguments
    /// * `title` - Label for the menu bar button.
    /// * `tag` - Unique semantic tag for tracking click and hover triggers.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, tag: u64) -> Self {
        Self {
            title: title.into(),
            tag,
            items: Vec::new(),
        }
    }

    /// Appends a menu item via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_item(mut self, item: MenuItemDescriptor) -> Self {
        self.items.push(item);
        self
    }

    /// Appends an action item via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_action(self, label: impl Into<String>, tag: u64) -> Self {
        self.with_item(MenuItemDescriptor::action(label, tag))
    }

    /// Appends an action item with a keyboard shortcut via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_action_shortcut(
        self,
        label: impl Into<String>,
        shortcut: impl Into<String>,
        tag: u64,
    ) -> Self {
        self.with_item(MenuItemDescriptor::action_with_shortcut(
            label, shortcut, tag,
        ))
    }

    /// Appends a separator divider via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_separator(self) -> Self {
        self.with_item(MenuItemDescriptor::separator())
    }

    /// Appends a cascading submenu via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_submenu(
        self,
        label: impl Into<String>,
        tag: u64,
        items: Vec<MenuItemDescriptor>,
    ) -> Self {
        self.with_item(MenuItemDescriptor::submenu(label, tag, items))
    }

    /// Appends a child item directly to this menu section.
    #[inline]
    pub fn add_item(&mut self, item: MenuItemDescriptor) {
        self.items.push(item);
    }
}

/// Top-level application menu bar descriptor defining menus, submenus, actions, and keybindings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MenuBarDescriptor {
    /// Ordered list of top-level menu categories.
    pub menus: Vec<MenuSectionDescriptor>,
}

impl MenuBarDescriptor {
    /// Creates an empty menu bar descriptor.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Appends a menu section via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_menu(mut self, section: MenuSectionDescriptor) -> Self {
        self.menus.push(section);
        self
    }

    /// Appends a menu section directly to the menu bar.
    #[inline]
    pub fn add_menu(&mut self, section: MenuSectionDescriptor) {
        self.menus.push(section);
    }

    /// Finds a menu section by its title label.
    #[must_use]
    pub fn find_menu(&self, title: &str) -> Option<&MenuSectionDescriptor> {
        self.menus.iter().find(|m| m.title == title)
    }

    /// Mutably finds a menu section by its title label.
    pub fn find_menu_mut(&mut self, title: &str) -> Option<&mut MenuSectionDescriptor> {
        self.menus.iter_mut().find(|m| m.title == title)
    }
}