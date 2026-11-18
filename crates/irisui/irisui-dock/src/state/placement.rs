// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Declarative tab placement policies, visibility bitmasks, and lifecycle operations on
//! [`DockState`].

use crate::drag_drop::DropZone;
use crate::state::{DockState, collect_all_tabs_recursive};
use irisui_core::{Rect, Size};

/// Declarative placement strategy describing where a tab opens or re-docks in a [`DockState`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabPlacement<T> {
    /// Opens inside the workspace split tree: tries docking alongside `partner` first,
    /// then splitting `anchor` along `zone`, falling back to the focused or root leaf.
    Docked {
        /// Optional sibling tab whose leaf is reused when open.
        partner: Option<T>,
        /// Optional primary anchor tab (e.g. a main viewport) to split relative to.
        anchor: Option<T>,
        /// Edge or center drop zone used when splitting `anchor`.
        zone: DropZone,
    },
    /// Opens or focuses as a centered detached floating window sized to `size` logical pixels.
    FloatingCentered {
        /// Preferred window dimensions in logical pixels.
        size: Size,
    },
}

/// Computes a centered floating window [`Rect`] within `screen_size`, clamped above the menubar
/// and falling back to `1280x720` when screen dimensions are missing or degenerate (`<= 100px`).
#[must_use]
pub fn resolve_centered_floating_rect(desired_size: Size, screen_size: Option<Size>) -> Rect {
    let screen = screen_size
        .filter(|s| s.width > 100.0 && s.height > 100.0)
        .unwrap_or_else(|| Size::new(1280.0, 720.0));
    let width = desired_size.width.min(screen.width.max(300.0));
    let height = desired_size.height.min(screen.height.max(200.0));
    let x = ((screen.width - width) * 0.5).max(10.0);
    let y = ((screen.height - height) * 0.5).max(36.0);
    Rect::new(x, y, width, height)
}

impl<T: PartialEq + Clone> DockState<T> {
    /// Returns `true` if `tab` is present either in the docked split tree or in any floating window.
    #[inline]
    #[must_use]
    pub fn contains_tab(&self, tab: &T) -> bool {
        self.tree.find_tab(tab).is_some() || self.is_tab_in_floating_window(tab)
    }

    /// Computes a 64-bit visibility bitmask for `tabs`, setting bit `i` when `tabs[i]` is open.
    ///
    /// Tabs beyond index `63` are ignored.
    #[must_use]
    pub fn tab_visibility_mask(&self, tabs: &[T]) -> u64 {
        let mut mask = 0u64;
        for (idx, tab) in tabs.iter().take(64).enumerate() {
            if self.contains_tab(tab) {
                mask |= 1u64 << idx;
            }
        }
        mask
    }

    /// Focuses `tab` if it already exists in the docked tree or in a floating window.
    ///
    /// Returns `true` if `tab` was found and focused, or `false` if it is currently closed.
    pub fn focus_tab(&mut self, tab: &T) -> bool {
        if let Some((leaf, idx)) = self.tree.find_tab(tab) {
            let _ = self.tree.set_active_tab(leaf, idx);
            self.tree.set_focused_leaf(Some(leaf));
            return true;
        }
        if let Some(win_id) = self
            .floating_windows
            .iter()
            .find(|w| w.tree.find_tab(tab).is_some())
            .map(|w| w.id)
        {
            self.bring_floating_to_front(win_id);
            return true;
        }
        false
    }

    /// Closes `tab` whether hosted in a detached floating window or inside the docked split tree,
    /// automatically collapsing empty leaves.
    ///
    /// Returns `true` if `tab` was open and removed.
    pub fn close_tab_by_id(&mut self, tab: &T) -> bool {
        if let Some(win_id) = self
            .floating_windows
            .iter()
            .find(|w| w.tree.find_tab(tab).is_some())
            .map(|w| w.id)
        {
            let _ = self.close_floating_window(win_id);
            return true;
        }
        if let Some((leaf, idx)) = self.tree.find_tab(tab) {
            let _ = self.close_tab(leaf, idx);
            return true;
        }
        false
    }

    /// Focuses an existing docked/floating `tab`, or opens a centered detached floating window
    /// constrained to `screen_size`.
    pub fn open_or_focus_centered_floating(
        &mut self,
        tab: T,
        title: impl Into<String>,
        desired_size: Size,
        screen_size: Option<Size>,
    ) {
        if let Some((leaf, idx)) = self.tree.find_tab(&tab) {
            let _ = self.tree.set_active_tab(leaf, idx);
            self.tree.set_focused_leaf(Some(leaf));
            return;
        }
        let rect = resolve_centered_floating_rect(desired_size, screen_size);
        let _ = self.open_or_focus_floating_tab(tab, title, rect);
    }

    /// Focuses `tab` if open, or creates it according to its declarative [`TabPlacement`] rule.
    pub fn open_or_focus_tab(
        &mut self,
        tab: T,
        title: impl Into<String>,
        placement: &TabPlacement<T>,
        screen_size: Option<Size>,
    ) {
        if self.focus_tab(&tab) {
            return;
        }

        match placement {
            TabPlacement::FloatingCentered { size } => {
                self.open_or_focus_centered_floating(tab, title, *size, screen_size);
            }
            TabPlacement::Docked {
                partner,
                anchor,
                zone,
            } => {
                if let Some(partner_tab) = partner
                    && let Some((partner_leaf, _)) = self.tree.find_tab(partner_tab)
                    && let Ok(idx) = self.tree.add_tab(partner_leaf, tab.clone())
                {
                    let _ = self.tree.set_active_tab(partner_leaf, idx);
                    self.tree.set_focused_leaf(Some(partner_leaf));
                    return;
                }

                if let Some(anchor_tab) = anchor
                    && let Some((anchor_leaf, _)) = self.tree.find_tab(anchor_tab)
                    && let Ok(new_leaf) = self.tree.dock_tab(anchor_leaf, *zone, tab.clone())
                {
                    if let Some((leaf, idx)) = self.tree.find_tab(&tab) {
                        let _ = self.tree.set_active_tab(leaf, idx);
                    }
                    self.tree.set_focused_leaf(Some(new_leaf));
                    return;
                }

                let _ = self.tree.push_to_focused_leaf(tab);
            }
        }
    }

    /// Toggles `tab` visibility: closes it when open, or opens it with `placement` when closed.
    pub fn toggle_tab(
        &mut self,
        tab: T,
        title: impl Into<String>,
        placement: &TabPlacement<T>,
        screen_size: Option<Size>,
    ) {
        if !self.close_tab_by_id(&tab) {
            self.open_or_focus_tab(tab, title, placement, screen_size);
        }
    }

    /// Docks a floating window back into the main split tree using the declarative [`TabPlacement`]
    /// resolved for its primary tab.
    pub fn dock_floating_by_placement(
        &mut self,
        window_id: u64,
        resolve_placement: impl FnOnce(&T) -> TabPlacement<T>,
    ) {
        let Some(pos) = self.floating_windows.iter().position(|w| w.id == window_id) else {
            return;
        };
        let first_tab = self.floating_windows[pos].tree.all_tabs().first().cloned();
        let Some(tab) = first_tab else {
            let _ = self.close_floating_window(window_id);
            return;
        };

        let placement = resolve_placement(&tab);
        if let TabPlacement::Docked {
            partner,
            anchor,
            zone,
        } = placement
        {
            if let Some(ref partner_tab) = partner
                && let Some((partner_leaf, _)) = self.tree.find_tab(partner_tab)
            {
                let _ = self.dock_floating_window(window_id, partner_leaf, DropZone::Center);
                return;
            }
            if let Some(ref anchor_tab) = anchor
                && let Some((anchor_leaf, _)) = self.tree.find_tab(anchor_tab)
            {
                let _ = self.dock_floating_window(window_id, anchor_leaf, zone);
                return;
            }
        }

        if let Some(fallback_leaf) = self.tree.find_first_leaf() {
            let _ = self.dock_floating_window(window_id, fallback_leaf, DropZone::Center);
            return;
        }

        let mut window = self.floating_windows.remove(pos);
        let mut tabs = Vec::new();
        if let Some(root) = window.tree.root() {
            collect_all_tabs_recursive(&mut window.tree, root, &mut tabs);
        }
        if !tabs.is_empty() {
            let leaf = self.tree.create_leaf(tabs);
            self.tree.set_root(leaf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::DockTree;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum SampleTab {
        Viewport,
        Hierarchy,
        Stats,
        Inspector,
        Preferences,
    }

    fn placement_for(tab: &SampleTab) -> TabPlacement<SampleTab> {
        match tab {
            SampleTab::Preferences => TabPlacement::FloatingCentered {
                size: Size::new(760.0, 540.0),
            },
            SampleTab::Hierarchy => TabPlacement::Docked {
                partner: Some(SampleTab::Stats),
                anchor: Some(SampleTab::Viewport),
                zone: DropZone::Left,
            },
            SampleTab::Stats => TabPlacement::Docked {
                partner: Some(SampleTab::Hierarchy),
                anchor: Some(SampleTab::Viewport),
                zone: DropZone::Left,
            },
            SampleTab::Inspector => TabPlacement::Docked {
                partner: None,
                anchor: Some(SampleTab::Viewport),
                zone: DropZone::Right,
            },
            SampleTab::Viewport => TabPlacement::Docked {
                partner: None,
                anchor: None,
                zone: DropZone::Center,
            },
        }
    }

    #[test]
    fn test_centered_floating_rect_and_screen_fallback() {
        let wide = resolve_centered_floating_rect(
            Size::new(760.0, 540.0),
            Some(Size::new(2560.0, 1440.0)),
        );
        assert_eq!(wide, Rect::new(900.0, 450.0, 760.0, 540.0));

        let fallback = resolve_centered_floating_rect(Size::new(760.0, 540.0), None);
        assert_eq!(fallback, Rect::new(260.0, 90.0, 760.0, 540.0));

        let tiny =
            resolve_centered_floating_rect(Size::new(760.0, 540.0), Some(Size::new(50.0, 1080.0)));
        assert_eq!(tiny, fallback);
    }

    #[test]
    fn test_tab_placement_open_toggle_close_and_mask() {
        let mut tree = DockTree::new();
        let leaf = tree.create_leaf(vec![SampleTab::Viewport]);
        tree.set_root(leaf);
        let mut dock = DockState::new(tree);

        dock.open_or_focus_tab(
            SampleTab::Hierarchy,
            "Hierarchy",
            &placement_for(&SampleTab::Hierarchy),
            Some(Size::new(1920.0, 1080.0)),
        );
        dock.open_or_focus_tab(
            SampleTab::Stats,
            "Stats",
            &placement_for(&SampleTab::Stats),
            Some(Size::new(1920.0, 1080.0)),
        );
        dock.open_or_focus_tab(
            SampleTab::Preferences,
            "Preferences",
            &placement_for(&SampleTab::Preferences),
            Some(Size::new(1920.0, 1080.0)),
        );

        assert!(dock.contains_tab(&SampleTab::Hierarchy));
        assert!(dock.contains_tab(&SampleTab::Stats));
        assert!(dock.is_tab_in_floating_window(&SampleTab::Preferences));

        let tabs = [
            SampleTab::Viewport,
            SampleTab::Hierarchy,
            SampleTab::Stats,
            SampleTab::Inspector,
            SampleTab::Preferences,
        ];
        assert_eq!(dock.tab_visibility_mask(&tabs), 0b10111);

        dock.toggle_tab(
            SampleTab::Preferences,
            "Preferences",
            &placement_for(&SampleTab::Preferences),
            Some(Size::new(1920.0, 1080.0)),
        );
        assert!(!dock.contains_tab(&SampleTab::Preferences));
        assert!(dock.close_tab_by_id(&SampleTab::Hierarchy));
        assert!(!dock.contains_tab(&SampleTab::Hierarchy));
    }

    #[test]
    fn test_dock_floating_by_placement_docks_back_to_partner() {
        let mut tree = DockTree::new();
        let leaf = tree.create_leaf(vec![SampleTab::Viewport]);
        tree.set_root(leaf);
        let mut dock = DockState::new(tree);
        dock.open_or_focus_centered_floating(
            SampleTab::Inspector,
            "Inspector",
            Size::new(320.0, 400.0),
            Some(Size::new(1920.0, 1080.0)),
        );
        let win_id = dock.floating_windows[0].id;
        dock.dock_floating_by_placement(win_id, placement_for);
        assert!(dock.floating_windows.is_empty());
        assert!(dock.tree.find_tab(&SampleTab::Inspector).is_some());
    }
}