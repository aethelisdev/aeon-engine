// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Zero-Allocation Static Cascading Menu Tree Primitives (`irisui-widgets::declarative::scope::menu::spec`)
//!
//! Provides `const`-constructible static slice menu trees ([`CascadingMenuEntry`]) and
//! declarative [`UiScope`] renderers that eliminate per-frame `Vec` allocations, string cloning,
//! and manual submenu open/close plumbing in consumer panels.

use super::super::core::UiScope;
use super::types::CascadingPopupConfig;
use irisui_core::{Rect, UiIcon, UiTag};

/// Static-slice descriptor for a single entry inside a zero-allocation cascading menu hierarchy.
///
/// All constructors are `const fn`, allowing callers to declare arbitrary multi-level menus
/// in read-only static memory (`const &[CascadingMenuEntry<'static, Action>]`) with zero
/// per-frame heap allocations or cloning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CascadingMenuEntry<'a, A: 'a> {
    /// Interactive actionable leaf entry emitting a borrowed action payload when clicked.
    Item {
        /// Human-readable label displayed on the menu row.
        label: &'a str,
        /// Optional leading icon (`Glyph` or `Atlas`).
        icon: Option<UiIcon>,
        /// Optional right-aligned shortcut hint (e.g. `"Del"`, `"Ctrl+D"`).
        shortcut: Option<&'a str>,
        /// Semantic tag assigned to the clickable item row.
        tag: UiTag,
        /// Domain action payload forwarded to the caller's handler on click.
        action: A,
        /// Whether this entry can be hovered and clicked.
        enabled: bool,
    },
    /// Branching submenu entry expanding a child slice of [`CascadingMenuEntry`] when hovered.
    Submenu {
        /// Category label displayed on the parent menu row.
        label: &'a str,
        /// Optional leading icon (`Glyph` or `Atlas`).
        icon: Option<UiIcon>,
        /// Deterministic semantic tag identifying this submenu popup in `UiTree`.
        tag: UiTag,
        /// Nested child entries rendered when this submenu branch is open.
        children: &'a [CascadingMenuEntry<'a, A>],
    },
    /// Horizontal divider line separating logical groups within a popup card.
    Separator,
}

impl<'a, A: 'a> CascadingMenuEntry<'a, A> {
    /// Constructs an enabled leaf item with no icon or shortcut, deriving its [`UiTag`] from `label`.
    #[inline]
    #[must_use]
    pub const fn item(label: &'a str, action: A) -> Self {
        Self::Item {
            label,
            icon: None,
            shortcut: None,
            tag: UiTag::from_str(label),
            action,
            enabled: true,
        }
    }

    /// Constructs an enabled leaf item with a unified [`UiIcon`], deriving its [`UiTag`] from `label`.
    #[inline]
    #[must_use]
    pub const fn item_with_icon(label: &'a str, icon: UiIcon, action: A) -> Self {
        Self::Item {
            label,
            icon: Some(icon),
            shortcut: None,
            tag: UiTag::from_str(label),
            action,
            enabled: true,
        }
    }

    /// Constructs a leaf item with explicit tag, optional icon, shortcut hint, and enabled state.
    #[inline]
    #[must_use]
    pub const fn item_full(
        label: &'a str,
        icon: Option<UiIcon>,
        shortcut: Option<&'a str>,
        tag: UiTag,
        action: A,
        enabled: bool,
    ) -> Self {
        Self::Item {
            label,
            icon,
            shortcut,
            tag,
            action,
            enabled,
        }
    }

    /// Constructs a branching submenu entry expanding a static slice of child entries on hover.
    #[inline]
    #[must_use]
    pub const fn submenu(
        label: &'a str,
        icon: Option<UiIcon>,
        tag: UiTag,
        children: &'a [CascadingMenuEntry<'a, A>],
    ) -> Self {
        Self::Submenu {
            label,
            icon,
            tag,
            children,
        }
    }

    /// Constructs a horizontal separator divider entry.
    #[inline]
    #[must_use]
    pub const fn separator() -> Self {
        Self::Separator
    }
}

impl<'a> UiScope<'a> {
    /// Emits an entire [`CascadingMenuEntry`] slice recursively inside the active dropdown card scope.
    ///
    /// Submenu hover expansion is driven autonomously by `UiTree::active_submenus`; leaf item clicks
    /// auto-close all active popups and invoke `on_action` with the clicked item's borrowed payload.
    ///
    /// # Arguments
    /// * `entries` - Static or borrowed slice of hierarchical menu entries.
    /// * `on_action` - Callback invoked once with `&A` when an enabled leaf item is clicked.
    pub fn render_cascading_entries<A>(
        &mut self,
        entries: &[CascadingMenuEntry<'_, A>],
        on_action: &mut impl FnMut(&A),
    ) {
        for entry in entries {
            match entry {
                CascadingMenuEntry::Separator => {
                    self.separator();
                }
                CascadingMenuEntry::Submenu {
                    label,
                    icon,
                    tag,
                    children,
                } => {
                    self.submenu_named_with_ui_icon_inner(
                        "CascadingMenuSubmenu",
                        *label,
                        *icon,
                        *tag,
                        |sub_card| {
                            sub_card.render_cascading_entries(children, on_action);
                        },
                    );
                }
                CascadingMenuEntry::Item {
                    label,
                    icon,
                    shortcut,
                    tag,
                    action,
                    enabled,
                } => {
                    self.dropdown_item_autoclose_ui_icon(*tag, *icon, label, *shortcut, *enabled)
                        .on_click(|| on_action(action));
                }
            }
        }
    }

    /// Emits a zero-allocation scoped cascading popup menu anchored to `anchor_tag` when open.
    ///
    /// Reads open/close state directly from `UiTree::active_popup_tags` via `anchor_tag` and
    /// manages multi-level submenu hovers via `UiTree::active_submenus`, eliminating panel boolean
    /// flags and manual `Vec` menu reconstruction.
    ///
    /// Returns the union bounding [`Rect`] of all open cascading cards when active.
    pub fn scoped_cascading_menu_anchored<A>(
        &mut self,
        name: &'static str,
        anchor_tag: UiTag,
        config: CascadingPopupConfig,
        entries: &[CascadingMenuEntry<'_, A>],
        mut on_action: impl FnMut(&A),
    ) -> Option<Rect> {
        self.scoped_cascading_popup_anchored_to_tag(
            name,
            anchor_tag.raw(),
            config.fallback_rect,
            config,
            |card| {
                card.render_cascading_entries(entries, &mut on_action);
            },
        )
        .map(|res| res.response.rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::scope::menu::PopupPlacement;
    use irisui_core::{UiLayer, UiTree, WidgetRole};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MockMenuAction {
        SpawnCube,
        SpawnUiPanel,
    }

    const SUB_SHAPES: &[CascadingMenuEntry<'static, MockMenuAction>] =
        &[CascadingMenuEntry::item_with_icon(
            "Cube",
            UiIcon::Glyph("🧊"),
            MockMenuAction::SpawnCube,
        )];

    const ROOT_MENU: &[CascadingMenuEntry<'static, MockMenuAction>] = &[
        CascadingMenuEntry::submenu(
            "3D Shapes",
            Some(UiIcon::Glyph("📦")),
            UiTag::from_str("mock.submenu.3d"),
            SUB_SHAPES,
        ),
        CascadingMenuEntry::separator(),
        CascadingMenuEntry::item("UI Panel", MockMenuAction::SpawnUiPanel),
    ];

    #[test]
    fn test_scoped_cascading_menu_anchored_static_slice_and_click() {
        const ANCHOR: UiTag = UiTag::from_str("mock.anchor");
        const SUB_TAG: UiTag = UiTag::from_str("mock.submenu.3d");

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        // Closed initially -> returns None without emitting nodes
        let mut triggered = None;
        {
            let mut scope = UiScope::new(&mut tree, root);
            let rect = scope.scoped_cascading_menu_anchored(
                "TestMenu",
                ANCHOR,
                CascadingPopupConfig::new(180.0, PopupPlacement::Below)
                    .with_fallback_rect(Rect::new(20.0, 20.0, 24.0, 24.0)),
                ROOT_MENU,
                |act| triggered = Some(*act),
            );
            assert!(rect.is_none());
        }

        // Activate anchor + submenu in UiTree SSOT and simulate click on nested leaf
        tree.open_popup(ANCHOR.raw());
        let menu_name_tag = crate::declarative::types::hash_label("TestMenu");
        tree.set_active_submenu(menu_name_tag, SUB_TAG.raw());
        tree.event_state.frame_clicked_tag = Some(UiTag::from_str("Cube").raw());

        {
            let mut scope = UiScope::new(&mut tree, root);
            let rect = scope.scoped_cascading_menu_anchored(
                "TestMenu",
                ANCHOR,
                CascadingPopupConfig::new(180.0, PopupPlacement::Below)
                    .with_fallback_rect(Rect::new(20.0, 20.0, 24.0, 24.0)),
                ROOT_MENU,
                |act| triggered = Some(*act),
            );
            assert!(rect.is_some());
        }

        assert_eq!(triggered, Some(MockMenuAction::SpawnCube));
        assert!(
            !tree.is_popup_tag_active(ANCHOR.raw()),
            "Leaf item click must auto-close all active popup tags"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| { n.layer == UiLayer::Popup && n.role == WidgetRole::DropdownPopup })
        );
    }
}