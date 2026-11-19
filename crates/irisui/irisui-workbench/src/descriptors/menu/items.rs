// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Type-Safe Menu Bar Item and Section Descriptors
//!
//! Defines [`MenuItemDescriptor`], [`MenuBarBrandingDescriptor`], [`MenuRightItemDescriptor`],
//! and [`MenuSectionDescriptor`] for strongly-typed menu construction.

use irisui_core::{Color, UiIcon, UiTag};

/// Represents an item within a dropdown menu or cascading submenu.
///
/// Supports typed interactive actions, checkmark toggles, declarative modal triggers,
/// nested submenus, and visual separators.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemDescriptor<A = ()> {
    /// Interactive command emitting `action` directly into [`crate::action_bus::ActionBus`] when clicked.
    Action {
        /// Optional leading glyph or GPU texture atlas icon.
        icon: Option<UiIcon>,

        /// User-visible text label (e.g. `"Save Scene"`).
        label: String,

        /// Optional keyboard shortcut annotation displayed on the right edge (e.g. `"Ctrl+S"`).
        shortcut: Option<String>,

        /// Explicit interaction tag (`UiTag::NONE` derives one automatically from section + label).
        tag: UiTag,

        /// Strongly-typed command emitted on click when enabled.
        action: Option<A>,

        /// Whether the item is active and clickable (`false` renders disabled).
        enabled: bool,
    },

    /// Interactive toggle item displaying a checkmark indicator (`✓`) when `checked` is `true`.
    ToggleCheck {
        /// User-visible text label (e.g. `"Hierarchy"`).
        label: String,

        /// Optional keyboard shortcut annotation.
        shortcut: Option<String>,

        /// Explicit interaction tag (`UiTag::NONE` derives one automatically from section + label).
        tag: UiTag,

        /// Active toggle state controlling checkmark indicator visibility.
        checked: bool,

        /// Command emitted when clicked to toggle the setting or panel.
        action: Option<A>,

        /// Whether the toggle item is active and interactive.
        enabled: bool,
    },

    /// Declarative item that opens a target modal dialog identified by [`UiTag`] directly on click.
    ModalTrigger {
        /// Optional leading glyph or GPU texture atlas icon.
        icon: Option<UiIcon>,

        /// User-visible text label (e.g. `"About..."`).
        label: String,

        /// Optional keyboard shortcut annotation.
        shortcut: Option<String>,

        /// Explicit interaction tag (`UiTag::NONE` derives one automatically from section + label).
        tag: UiTag,

        /// Identifier of the modal opened via `UiScope::open_modal` upon click.
        modal_tag: UiTag,

        /// Whether the trigger is active and interactive.
        enabled: bool,
    },

    /// Nested cascading submenu branching into secondary options.
    Submenu {
        /// Optional leading glyph or GPU texture atlas icon.
        icon: Option<UiIcon>,

        /// User-visible text label for the submenu entry (e.g. `"Export As..."`).
        label: String,

        /// Semantic interaction tag for the submenu trigger.
        tag: UiTag,

        /// Child items contained within this cascading submenu.
        items: Vec<MenuItemDescriptor<A>>,
    },

    /// Visual horizontal divider separating logical groups of actions.
    Separator,
}

impl<A> MenuItemDescriptor<A> {
    /// Creates an enabled command item emitting `action` on click.
    #[inline]
    #[must_use]
    pub fn command(label: impl Into<String>, action: A) -> Self {
        Self::Action {
            icon: None,
            label: label.into(),
            shortcut: None,
            tag: UiTag::NONE,
            action: Some(action),
            enabled: true,
        }
    }

    /// Creates an enabled command item with a keyboard shortcut annotation.
    #[inline]
    #[must_use]
    pub fn command_with_shortcut(
        label: impl Into<String>,
        shortcut: impl Into<String>,
        action: A,
    ) -> Self {
        Self::Action {
            icon: None,
            label: label.into(),
            shortcut: Some(shortcut.into()),
            tag: UiTag::NONE,
            action: Some(action),
            enabled: true,
        }
    }

    /// Creates an enabled command item with a unified icon.
    #[inline]
    #[must_use]
    pub fn command_with_icon(icon: impl Into<UiIcon>, label: impl Into<String>, action: A) -> Self {
        Self::Action {
            icon: Some(icon.into()),
            label: label.into(),
            shortcut: None,
            tag: UiTag::NONE,
            action: Some(action),
            enabled: true,
        }
    }

    /// Creates an enabled command item with an icon and optional keyboard shortcut.
    #[inline]
    #[must_use]
    pub fn command_full(
        icon: impl Into<UiIcon>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        action: A,
    ) -> Self {
        Self::Action {
            icon: Some(icon.into()),
            label: label.into(),
            shortcut: shortcut.map(Into::into),
            tag: UiTag::NONE,
            action: Some(action),
            enabled: true,
        }
    }

    /// Creates a disabled (non-interactive) action item with optional icon and shortcut.
    #[inline]
    #[must_use]
    pub fn disabled(
        icon: Option<UiIcon>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
    ) -> Self {
        Self::Action {
            icon,
            label: label.into(),
            shortcut: shortcut.map(Into::into),
            tag: UiTag::NONE,
            action: None,
            enabled: false,
        }
    }

    /// Creates a checkmark toggle menu item emitting `action` on click.
    #[inline]
    #[must_use]
    pub fn toggle_check(label: impl Into<String>, checked: bool, action: A) -> Self {
        Self::ToggleCheck {
            label: label.into(),
            shortcut: None,
            tag: UiTag::NONE,
            checked,
            action: Some(action),
            enabled: true,
        }
    }

    /// Creates a menu item that opens `modal_tag` on click without routing through `ActionBus`.
    #[inline]
    #[must_use]
    pub fn modal_trigger(
        icon: Option<UiIcon>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        modal_tag: impl Into<UiTag>,
    ) -> Self {
        Self::ModalTrigger {
            icon,
            label: label.into(),
            shortcut: shortcut.map(Into::into),
            tag: UiTag::NONE,
            modal_tag: modal_tag.into(),
            enabled: true,
        }
    }

    /// Creates a tag-only action item without an associated payload.
    #[inline]
    #[must_use]
    pub fn tagged(label: impl Into<String>, tag: impl Into<UiTag>) -> Self {
        Self::Action {
            icon: None,
            label: label.into(),
            shortcut: None,
            tag: tag.into(),
            action: None,
            enabled: true,
        }
    }

    /// Creates a nested cascading submenu item.
    #[inline]
    #[must_use]
    pub fn submenu(
        label: impl Into<String>,
        tag: impl Into<UiTag>,
        items: Vec<MenuItemDescriptor<A>>,
    ) -> Self {
        Self::Submenu {
            icon: None,
            label: label.into(),
            tag: tag.into(),
            items,
        }
    }

    /// Creates a horizontal separator divider.
    #[inline]
    #[must_use]
    pub const fn separator() -> Self {
        Self::Separator
    }

    /// Assigns an explicit [`UiTag`] to this menu item via fluent builder chaining.
    #[inline]
    #[must_use]
    pub fn with_tag(mut self, explicit_tag: impl Into<UiTag>) -> Self {
        let next = explicit_tag.into();
        match &mut self {
            Self::Action { tag, .. }
            | Self::ToggleCheck { tag, .. }
            | Self::ModalTrigger { tag, .. }
            | Self::Submenu { tag, .. } => *tag = next,
            Self::Separator => {}
        }
        self
    }

    /// Returns the explicit or parent-scoped [`UiTag`] for this item, or `None` for separators.
    #[must_use]
    pub fn resolved_tag(&self, section_tag: UiTag) -> Option<UiTag> {
        match self {
            Self::Action { tag, label, .. }
            | Self::ToggleCheck { tag, label, .. }
            | Self::ModalTrigger { tag, label, .. }
            | Self::Submenu { tag, label, .. } => {
                if tag.is_empty() {
                    Some(section_tag.child(label))
                } else {
                    Some(*tag)
                }
            }
            Self::Separator => None,
        }
    }

    /// Returns the explicit semantic [`UiTag`] assigned to this item, if set.
    #[inline]
    #[must_use]
    pub const fn tag(&self) -> Option<UiTag> {
        match self {
            Self::Action { tag, .. }
            | Self::ToggleCheck { tag, .. }
            | Self::ModalTrigger { tag, .. }
            | Self::Submenu { tag, .. } => Some(*tag),
            Self::Separator => None,
        }
    }

    /// Returns `true` if this item is a separator.
    #[inline]
    #[must_use]
    pub const fn is_separator(&self) -> bool {
        matches!(self, Self::Separator)
    }

    /// Returns `true` if this item is interactive and enabled.
    #[inline]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        match self {
            Self::Action { enabled, .. }
            | Self::ToggleCheck { enabled, .. }
            | Self::ModalTrigger { enabled, .. } => *enabled,
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
pub enum MenuRightItemDescriptor<A = ()> {
    /// Styled colored action button (e.g., Play/Stop button in green/red).
    ActionButton {
        /// Text label displayed inside the button.
        label: String,
        /// Button background color.
        bg: Color,
        /// Foreground text color.
        text: Color,
        /// Interaction tag assigned to the rendered button.
        tag: UiTag,
        /// Optional strongly-typed command emitted when clicked.
        action: Option<A>,
    },
    /// Desktop window control button (minimize, maximize, close).
    WindowControl {
        /// Control symbol (e.g., `"—"`, `"□"`, `"✕"`).
        label: String,
        /// Whether this control closes the window (activates red hover highlight).
        is_close: bool,
        /// Semantic interaction tag monitored by the workbench host.
        tag: UiTag,
    },
}

/// A top-level menu category displayed on the menu bar (e.g. `"File"`, `"Edit"`, `"Window"`).
#[derive(Debug, Clone, PartialEq)]
pub struct MenuSectionDescriptor<A = ()> {
    /// User-facing label displayed on the menu bar (e.g. `"File"`).
    pub title: String,

    /// Semantic interaction tag for scoping menu items and triggers.
    pub tag: UiTag,

    /// Ordered list of child items contained in this dropdown menu.
    pub items: Vec<MenuItemDescriptor<A>>,
}

impl<A> MenuSectionDescriptor<A> {
    /// Creates a new menu section deriving its semantic tag from `title`.
    #[inline]
    #[must_use]
    pub fn titled(title: impl Into<String>) -> Self {
        let title = title.into();
        let tag = UiTag::from_str(&title);
        Self {
            title,
            tag,
            items: Vec::new(),
        }
    }

    /// Creates a new menu section with an explicit title and semantic trigger tag.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, tag: impl Into<UiTag>) -> Self {
        Self {
            title: title.into(),
            tag: tag.into(),
            items: Vec::new(),
        }
    }

    /// Returns the section's effective namespace [`UiTag`].
    #[inline]
    #[must_use]
    pub fn effective_tag(&self) -> UiTag {
        if self.tag.is_empty() {
            UiTag::from_str(&self.title)
        } else {
            self.tag
        }
    }

    /// Appends a menu item via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_item(mut self, item: MenuItemDescriptor<A>) -> Self {
        self.items.push(item);
        self
    }

    /// Appends an enabled typed command item via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_command(self, label: impl Into<String>, action: A) -> Self {
        self.with_item(MenuItemDescriptor::command(label, action))
    }

    /// Appends an enabled typed command item with keyboard shortcut via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_command_shortcut(
        self,
        label: impl Into<String>,
        shortcut: impl Into<String>,
        action: A,
    ) -> Self {
        self.with_item(MenuItemDescriptor::command_with_shortcut(
            label, shortcut, action,
        ))
    }

    /// Appends an enabled typed command item with icon and optional shortcut via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_command_full(
        self,
        icon: impl Into<UiIcon>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        action: A,
    ) -> Self {
        self.with_item(MenuItemDescriptor::command_full(
            icon, label, shortcut, action,
        ))
    }

    /// Appends an enabled typed command item with an icon via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_command_with_icon(
        self,
        icon: impl Into<UiIcon>,
        label: impl Into<String>,
        action: A,
    ) -> Self {
        self.with_item(MenuItemDescriptor::command_with_icon(icon, label, action))
    }

    /// Appends a disabled action item with optional icon and shortcut via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_disabled(
        self,
        icon: Option<UiIcon>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
    ) -> Self {
        self.with_item(MenuItemDescriptor::disabled(icon, label, shortcut))
    }

    /// Appends a checkmark toggle item emitting `action` on click via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_toggle_check(self, label: impl Into<String>, checked: bool, action: A) -> Self {
        self.with_item(MenuItemDescriptor::toggle_check(label, checked, action))
    }

    /// Appends an item that opens `modal_tag` on click via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_modal_trigger(
        self,
        icon: Option<UiIcon>,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        modal_tag: impl Into<UiTag>,
    ) -> Self {
        self.with_item(MenuItemDescriptor::modal_trigger(
            icon, label, shortcut, modal_tag,
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
        tag: impl Into<UiTag>,
        items: Vec<MenuItemDescriptor<A>>,
    ) -> Self {
        self.with_item(MenuItemDescriptor::submenu(label, tag, items))
    }

    /// Appends a child item directly to this menu section.
    #[inline]
    pub fn add_item(&mut self, item: MenuItemDescriptor<A>) {
        self.items.push(item);
    }
}