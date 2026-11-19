// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top-Level Application Menu Bar Descriptor & Rendering
//!
//! Provides [`MenuBarDescriptor`] for composing sections, branding logos,
//! right-aligned controls, and directly emitting typed actions into [`ActionBus`].

use super::items::{
    MenuBarBrandingDescriptor, MenuItemDescriptor, MenuRightItemDescriptor, MenuSectionDescriptor,
};
use crate::action_bus::ActionBus;
use irisui_core::style::Style;
use irisui_core::{Color, UiIcon, UiTag};
use irisui_widgets::declarative::UiScope;

enum MenuClickOutcome<A> {
    Action { tag: UiTag, action: A },
    TagOnly(UiTag),
}

/// Top-level application menu bar descriptor defining menus, submenus, modal triggers, and actions.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuBarDescriptor<A = ()> {
    /// Ordered list of top-level menu categories.
    pub menus: Vec<MenuSectionDescriptor<A>>,

    /// Optional leading branding logo descriptor.
    pub branding: Option<MenuBarBrandingDescriptor>,

    /// Right-aligned action buttons and window management controls.
    pub right_items: Vec<MenuRightItemDescriptor<A>>,

    /// Height of the top menubar panel in physical pixels (defaults to `36.0`).
    pub height: f32,

    /// Horizontal padding applied to each menu item button (defaults to `6.0`).
    pub item_padding: f32,

    /// Horizontal gap between menu item buttons (defaults to `4.0`).
    pub item_gap: f32,

    /// Horizontal gap separating the branding logo from the first menu item (defaults to `10.0`).
    pub logo_gap: f32,
}

impl<A> Default for MenuBarDescriptor<A> {
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

impl<A> MenuBarDescriptor<A> {
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
    pub fn with_menu(mut self, section: MenuSectionDescriptor<A>) -> Self {
        self.menus.push(section);
        self
    }

    /// Appends a right-aligned typed command button via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_command_button(
        mut self,
        label: impl Into<String>,
        bg: Color,
        text: Color,
        action: A,
    ) -> Self {
        let label = label.into();
        let tag = UiTag::from_str("menubar.right").child(&label);
        self.right_items
            .push(MenuRightItemDescriptor::ActionButton {
                label,
                bg,
                text,
                tag,
                action: Some(action),
            });
        self
    }

    /// Appends a right-aligned action button with an explicit tag and optional command.
    #[inline]
    #[must_use]
    pub fn with_action_button(
        mut self,
        label: impl Into<String>,
        bg: Color,
        text: Color,
        tag: impl Into<UiTag>,
        action: Option<A>,
    ) -> Self {
        self.right_items
            .push(MenuRightItemDescriptor::ActionButton {
                label: label.into(),
                bg,
                text,
                tag: tag.into(),
                action,
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
        tag: impl Into<UiTag>,
    ) -> Self {
        self.right_items
            .push(MenuRightItemDescriptor::WindowControl {
                label: label.into(),
                is_close,
                tag: tag.into(),
            });
        self
    }

    /// Appends a menu section directly to the menu bar.
    #[inline]
    pub fn add_menu(&mut self, section: MenuSectionDescriptor<A>) {
        self.menus.push(section);
    }

    /// Finds a menu section by its title label.
    #[must_use]
    pub fn find_menu(&self, title: &str) -> Option<&MenuSectionDescriptor<A>> {
        self.menus.iter().find(|m| m.title == title)
    }

    /// Mutably finds a menu section by its title label.
    pub fn find_menu_mut(&mut self, title: &str) -> Option<&mut MenuSectionDescriptor<A>> {
        self.menus.iter_mut().find(|m| m.title == title)
    }

    /// Returns `true` when this descriptor defines no branding, menu sections, or right-aligned items.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.branding.is_none() && self.menus.is_empty() && self.right_items.is_empty()
    }
}

impl<A: Clone> MenuBarDescriptor<A> {
    fn render_internal(
        &self,
        scope: &mut UiScope<'_>,
        screen_width: f32,
        mut on_action: impl FnMut(A),
    ) -> Option<UiTag> {
        let outcome = scope
            .menu_bar(screen_width, self.height)
            .item_padding(self.item_padding)
            .item_gap(self.item_gap)
            .logo_gap(self.logo_gap)
            .show(|bar| {
                if let Some(branding) = self.branding {
                    bar.branding_icon(branding.uv, branding.tint, branding.size);
                }

                for menu in &self.menus {
                    let section_tag = menu.effective_tag();
                    bar.menu(&menu.title, |m| {
                        for item in &menu.items {
                            emit_menu_item(m, item, section_tag);
                        }
                    });
                }

                if !self.right_items.is_empty() {
                    bar.right(|right| {
                        for item in &self.right_items {
                            emit_right_item(right, item);
                        }
                    });
                }
            });

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

        match outcome {
            Some(MenuClickOutcome::Action { tag, action }) => {
                on_action(action);
                Some(tag)
            }
            Some(MenuClickOutcome::TagOnly(tag)) => Some(tag),
            None => None,
        }
    }

    /// Declaratively renders the menu bar and divider into [`UiScope`], returning any clicked item tag.
    pub fn render(&self, scope: &mut UiScope<'_>, screen_width: f32) -> Option<u64> {
        self.render_internal(scope, screen_width, |_| {})
            .map(UiTag::raw)
    }

    /// Declaratively renders the menu bar and divider into [`UiScope`], automatically opening modal
    /// triggers on click and pushing strongly-typed actions into `action_bus`.
    ///
    /// Eliminates consumer-side numeric tag lookup tables and manual modal dispatch shims.
    pub fn render_into(
        &self,
        scope: &mut UiScope<'_>,
        screen_width: f32,
        action_bus: &mut ActionBus<A>,
    ) -> Option<UiTag>
    where
        A: 'static + Send + Sync,
    {
        self.render_internal(scope, screen_width, |action| {
            action_bus.emit(action);
        })
    }
}

fn emit_menu_item<A: Clone>(
    menu_scope: &mut irisui_widgets::declarative::scope::MenuScope<'_, '_, MenuClickOutcome<A>>,
    item: &MenuItemDescriptor<A>,
    section_tag: UiTag,
) {
    let item_tag = item.resolved_tag(section_tag).unwrap_or(UiTag::NONE);
    match item {
        MenuItemDescriptor::Action {
            icon,
            label,
            shortcut,
            action,
            enabled,
            ..
        } => {
            if !*enabled {
                menu_scope.item_ui_icon_disabled(item_tag, *icon, label, shortcut.as_deref());
                return;
            }
            let payload = match action {
                Some(act) => MenuClickOutcome::Action {
                    tag: item_tag,
                    action: act.clone(),
                },
                None => MenuClickOutcome::TagOnly(item_tag),
            };
            menu_scope.item_ui_icon(item_tag, *icon, label, shortcut.as_deref(), payload);
        }
        MenuItemDescriptor::ToggleCheck {
            label,
            shortcut,
            checked,
            action,
            enabled,
            ..
        } => {
            let check_icon = Some(UiIcon::Glyph(if *checked { "✓" } else { " " }));
            if !*enabled {
                menu_scope.item_ui_icon_disabled(item_tag, check_icon, label, shortcut.as_deref());
                return;
            }
            let payload = match action {
                Some(act) => MenuClickOutcome::Action {
                    tag: item_tag,
                    action: act.clone(),
                },
                None => MenuClickOutcome::TagOnly(item_tag),
            };
            menu_scope.item_ui_icon(item_tag, check_icon, label, shortcut.as_deref(), payload);
        }
        MenuItemDescriptor::ModalTrigger {
            icon,
            label,
            shortcut,
            modal_tag,
            enabled,
            ..
        } => {
            menu_scope.item_ui_icon_modal(
                item_tag,
                *icon,
                label,
                shortcut.as_deref(),
                *modal_tag,
                *enabled,
            );
        }
        MenuItemDescriptor::Separator => {
            menu_scope.separator();
        }
        MenuItemDescriptor::Submenu { .. } => {}
    }
}

fn emit_right_item<A: Clone>(
    right: &mut irisui_widgets::declarative::scope::MenuBarRightScope<'_, '_, MenuClickOutcome<A>>,
    item: &MenuRightItemDescriptor<A>,
) {
    match item {
        MenuRightItemDescriptor::ActionButton {
            label,
            bg,
            text,
            tag,
            action,
        } => {
            let effective_tag = if tag.is_empty() {
                UiTag::from_str("menubar.right").child(label)
            } else {
                *tag
            };
            let payload = match action {
                Some(act) => MenuClickOutcome::Action {
                    tag: effective_tag,
                    action: act.clone(),
                },
                None => MenuClickOutcome::TagOnly(effective_tag),
            };
            right.action_button_tagged(effective_tag.raw(), label, *bg, *text, payload);
        }
        MenuRightItemDescriptor::WindowControl {
            label,
            is_close,
            tag,
        } => {
            let effective_tag = if tag.is_empty() {
                UiTag::from_str("window.control").child(label)
            } else {
                *tag
            };
            right.window_button_tagged(
                effective_tag.raw(),
                label,
                *is_close,
                MenuClickOutcome::TagOnly(effective_tag),
            );
        }
    }
}