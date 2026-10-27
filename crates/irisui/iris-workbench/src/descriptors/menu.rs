// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Menu Bar and Action Descriptors
//!
//! Provides declarative descriptors for configuring application menu bars,
//! cascading submenus, shortcuts, and command actions.

use iris_core::Color;
use iris_core::style::Style;
use iris_widgets::declarative::UiScope;

/// Represents an item within a dropdown menu or cascading submenu.
///
/// Supports interactive actions, nested submenus, and visual separators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItemDescriptor {
    /// Interactive action triggering an application or engine command.
    Action {
        /// Optional leading icon glyph or character (e.g. `"🗎"`, `"💾"`).
        icon: Option<String>,

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
        /// Optional leading icon glyph or character.
        icon: Option<String>,

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
            icon: None,
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
            icon: None,
            label: label.into(),
            shortcut: Some(shortcut.into()),
            tag,
            enabled: true,
        }
    }

    /// Creates an enabled action item with an icon but without a shortcut.
    #[inline]
    #[must_use]
    pub fn action_with_icon(icon: impl Into<String>, label: impl Into<String>, tag: u64) -> Self {
        Self::Action {
            icon: Some(icon.into()),
            label: label.into(),
            shortcut: None,
            tag,
            enabled: true,
        }
    }

    /// Creates an enabled action item with an icon and keyboard shortcut.
    ///
    /// # Arguments
    /// * `icon` - Icon character or glyph.
    /// * `label` - User-facing text label.
    /// * `shortcut` - Optional keyboard shortcut representation.
    /// * `tag` - Semantic interaction tag.
    #[inline]
    #[must_use]
    pub fn action_full(
        icon: impl Into<String>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        tag: u64,
    ) -> Self {
        Self::Action {
            icon: Some(icon.into()),
            label: label.into(),
            shortcut: shortcut.map(Into::into),
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
            icon: None,
            label: label.into(),
            shortcut: None,
            tag,
            enabled: false,
        }
    }

    /// Creates a disabled action item with an optional icon and shortcut.
    #[inline]
    #[must_use]
    pub fn disabled_action_full(
        icon: Option<impl Into<String>>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        tag: u64,
    ) -> Self {
        Self::Action {
            icon: icon.map(Into::into),
            label: label.into(),
            shortcut: shortcut.map(Into::into),
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
            icon: None,
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

/// Describes the application branding logo displayed on the far left of the menu bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuBarBrandingDescriptor {
    /// Texture atlas UV coordinates `[u_min, v_min, u_max, v_max]` for the logo icon.
    pub uv: [f32; 4],

    /// Color tint applied to the branding icon.
    pub tint: Color,

    /// Icon square dimensions in physical pixels (e.g. `28.0`).
    pub size: f32,
}

impl MenuBarBrandingDescriptor {
    /// Creates a new branding logo descriptor.
    #[inline]
    #[must_use]
    pub const fn new(uv: [f32; 4], tint: Color, size: f32) -> Self {
        Self { uv, tint, size }
    }
}

/// Describes an item or control displayed on the right edge of the menu bar.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuRightItemDescriptor {
    /// Primary or secondary action button (e.g. Play, Stop, Build).
    ActionButton {
        /// User-visible text label.
        label: String,

        /// Background fill color.
        bg: Color,

        /// Foreground text color.
        text: Color,

        /// Semantic tag dispatched when clicked.
        tag: u64,
    },

    /// Industrial desktop window control button (Minimize, Maximize, Close).
    WindowControl {
        /// Display text or glyph (e.g. `"—"`, `"□"`, `"✕"`).
        label: String,

        /// Whether this is a close button applying elevated danger hover styling.
        is_close: bool,

        /// Semantic tag dispatched when clicked.
        tag: u64,
    },
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

    /// Appends an action item with an icon and optional shortcut via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_action_full(
        self,
        icon: impl Into<String>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        tag: u64,
    ) -> Self {
        self.with_item(MenuItemDescriptor::action_full(icon, label, shortcut, tag))
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
#[derive(Debug, Clone, PartialEq)]
pub struct MenuBarDescriptor {
    /// Ordered list of top-level menu categories.
    pub menus: Vec<MenuSectionDescriptor>,

    /// Optional leading branding logo descriptor.
    pub branding: Option<MenuBarBrandingDescriptor>,

    /// Right-aligned action buttons and window management controls.
    pub right_items: Vec<MenuRightItemDescriptor>,

    /// Height of the top menubar panel in physical pixels (defaults to `36.0`).
    pub height: f32,

    /// Horizontal padding applied to each menu item button (defaults to `6.0`).
    pub item_padding: f32,

    /// Horizontal gap between menu item buttons (defaults to `4.0`).
    pub item_gap: f32,

    /// Horizontal gap separating the branding logo from the first menu item (defaults to `10.0`).
    pub logo_gap: f32,
}

impl Default for MenuBarDescriptor {
    fn default() -> Self {
        Self {
            menus: Vec::new(),
            branding: None,
            right_items: Vec::new(),
            height: 36.0,
            item_padding: 6.0,
            item_gap: 4.0,
            logo_gap: 10.0,
        }
    }
}

impl MenuBarDescriptor {
    /// Creates an empty menu bar descriptor with default studio dimensions.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches an application branding logo via fluent chaining.
    #[inline]
    #[must_use]
    pub const fn with_branding(mut self, branding: MenuBarBrandingDescriptor) -> Self {
        self.branding = Some(branding);
        self
    }

    /// Sets the height of the menubar in physical pixels.
    #[inline]
    #[must_use]
    pub const fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Appends a menu section via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_menu(mut self, section: MenuSectionDescriptor) -> Self {
        self.menus.push(section);
        self
    }

    /// Appends a right-aligned action button via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_action_button(
        mut self,
        label: impl Into<String>,
        bg: Color,
        text: Color,
        tag: u64,
    ) -> Self {
        self.right_items
            .push(MenuRightItemDescriptor::ActionButton {
                label: label.into(),
                bg,
                text,
                tag,
            });
        self
    }

    /// Appends a right-aligned window management control button via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_window_control(
        mut self,
        label: impl Into<String>,
        is_close: bool,
        tag: u64,
    ) -> Self {
        self.right_items
            .push(MenuRightItemDescriptor::WindowControl {
                label: label.into(),
                is_close,
                tag,
            });
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

    /// Declaratively renders the top application menu bar and baseline divider into [`UiScope`].
    ///
    /// Emits the top menu bar, dropdown items, branding icon, right-aligned controls,
    /// and the 1px baseline bottom divider. Returns the semantic tag of any clicked action.
    ///
    /// # Arguments
    /// * `scope` - Active declarative UI scope attached to the root container.
    /// * `screen_width` - Available width in physical pixels.
    pub fn render(&self, scope: &mut UiScope<'_>, screen_width: f32) -> Option<u64> {
        let mut builder = scope.menu_bar(screen_width, self.height);
        builder = builder
            .item_padding(self.item_padding)
            .item_gap(self.item_gap)
            .logo_gap(self.logo_gap);

        let action = builder.show(|bar| {
            // 0. Branding Logo
            if let Some(branding) = self.branding {
                bar.branding_icon(branding.uv, branding.tint, branding.size);
            }

            // 1. Cascading Menus
            for menu in &self.menus {
                bar.menu(&menu.title, |m| {
                    for item in &menu.items {
                        match item {
                            MenuItemDescriptor::Action {
                                icon,
                                label,
                                shortcut,
                                tag,
                                enabled,
                            } => {
                                let icon_str = icon.as_deref().unwrap_or("");
                                if *enabled {
                                    m.item(icon_str, label, shortcut.as_deref(), *tag);
                                } else {
                                    m.item_disabled(icon_str, label, shortcut.as_deref());
                                }
                            }
                            MenuItemDescriptor::Separator => {
                                m.separator();
                            }
                            MenuItemDescriptor::Submenu { .. } => {
                                // Submenu placeholder
                            }
                        }
                    }
                });
            }

            // 2. Right-Aligned Action Buttons and Window Controls
            if !self.right_items.is_empty() {
                bar.right(|right| {
                    for item in &self.right_items {
                        match item {
                            MenuRightItemDescriptor::ActionButton {
                                label,
                                bg,
                                text,
                                tag,
                            } => {
                                right.action_button(label, *bg, *text, *tag);
                            }
                            MenuRightItemDescriptor::WindowControl {
                                label,
                                is_close,
                                tag,
                            } => {
                                right.window_button(label, *is_close, *tag);
                            }
                        }
                    }
                });
            }
        });

        // 3. Baseline Divider under MenuBar (positioned absolutely with 1.0px height)
        scope.empty_box_passive_named(
            "IrisTopMenuBarDivider",
            Style::new()
                .position_absolute()
                .left(0.0)
                .top(self.height - 1.0)
                .width(screen_width)
                .height(1.0)
                .background(Color::hex("#1c1e24")),
        );

        action
    }
}