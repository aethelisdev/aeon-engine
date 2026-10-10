// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level retained Overlay and Portal Tree subsystem (Phase 5.2).
//!
//! Provides a dedicated, full-screen retained [`UiTree`] for transient top-level
//! UI elements such as menubar dropdowns, right-click context menus, modal dialogs,
//! and inspector color picker popups. Because overlays render outside of panel
//! boundaries, they remain immune to dock panel `.clip_children(true)` and GPU scissor rects.
//!

use crate::ui::iris_bridge::types::ActiveMenu;
use irisui::prelude::{LayoutEngine, Point, Rect, Size, Style, UiTree, WidgetId};

/// Kind of modal dialog rendered inside the overlay layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalKind {
    /// Preferences settings modal dialog.
    Preferences,
    /// About Aeon Engine informational dialog.
    About,
    /// File / asset delete confirmation modal.
    DeleteConfirmation,
    /// New directory creation modal.
    NewFolder,
    /// File or entity rename modal.
    Rename,
    /// Asset loading splash screen overlay.
    LoadingSplash,
}

/// Active overlay category or portal payload managed by [`OverlayTree`].
#[derive(Debug, Clone, PartialEq)]
pub enum ActiveOverlay {
    /// Menubar top dropdown menu category.
    MenubarDropdown(ActiveMenu),
    /// Hierarchy panel context menu or add entity menu.
    HierarchyContextMenu,
    /// Assets browser item or folder context menu.
    AssetsContextMenu,
    /// Inspector color picker popup.
    InspectorColorPicker,
    /// Inspector cascading add-component menu.
    InspectorAddComponent,
    /// Inspector dropdown combobox popup.
    InspectorDropdown,
    /// 2D Visual UI Designer add-element menu.
    UiDesignerAddElement,
    /// 2D Visual UI Designer aspect-ratio menu.
    UiDesignerAspectRatio,
    /// Modal dialog overlay.
    Modal(ModalKind),
    /// Generic or custom floating popup with an assigned anchor rect.
    CustomPopup {
        /// String identifier for debugging and inspection.
        id: String,
        /// Screen-space anchor rectangle where the popup originated.
        anchor: Rect,
    },
}

/// Dismissal behavior when a user clicks outside the active overlay boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayDismissPolicy {
    /// Automatically dismisses the overlay when clicking anywhere outside of its bounds.
    ClickOutside,
    /// Requires an explicit close button, confirmation, or cancellation trigger.
    ExplicitOnly,
}

/// Result of evaluating a pointer down event against active overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayDismissResult {
    /// No active overlay was open; event passes through to background layers.
    PassThrough,
    /// Pointer fell inside active overlay bounds; event consumed by overlay.
    Consumed,
    /// Pointer fell outside active overlay bounds; overlay was dismissed.
    Dismissed,
    /// Pointer fell outside active modal overlay with [`OverlayDismissPolicy::ExplicitOnly`]; dismissal ignored.
    Ignored,
}

/// Dedicated top-level retained UI tree and lifecycle manager for popups, menus, and modals.
///
/// Maintains a decoupled [`UiTree`] that lives above all dock panel boundaries and GPU scissor
/// rectangles, resolving the Clipping & Scissor Trap (Mine 2).
pub struct OverlayTree {
    /// Retained UI tree dedicated to top-level overlay widgets.
    tree: UiTree,
    /// Dedicated Taffy layout engine for overlay node computations.
    layout_engine: LayoutEngine,
    /// Currently open overlay item, if any.
    active_overlay: Option<ActiveOverlay>,
    /// Bounding box of the active overlay for hit-testing and dismissal resolution.
    overlay_bounds: Option<Rect>,
    /// Dismissal policy governing click-outside behavior.
    dismiss_policy: OverlayDismissPolicy,
    /// Whether the overlay contents need rebuild or re-evaluation.
    is_dirty: bool,
}

impl Default for OverlayTree {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayTree {
    /// Constructs a new empty overlay tree with a clean retained arena.
    pub fn new() -> Self {
        Self {
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            active_overlay: None,
            overlay_bounds: None,
            dismiss_policy: OverlayDismissPolicy::ClickOutside,
            is_dirty: false,
        }
    }

    /// Returns a shared reference to the retained overlay UI tree.
    #[inline]
    pub fn tree(&self) -> &UiTree {
        &self.tree
    }

    /// Returns a mutable reference to the retained overlay UI tree.
    #[inline]
    pub fn tree_mut(&mut self) -> &mut UiTree {
        &mut self.tree
    }

    /// Ensures that the overlay tree has an allocated root node configured spanning the screen dimensions.
    ///
    /// If the tree currently has no root, allocates a new root node styled as an absolute container
    /// matching `(0.0, 0.0, screen_width, screen_height)`.
    pub fn ensure_root(&mut self, screen_width: f32, screen_height: f32) -> WidgetId {
        if let Some(root) = self.tree.root() {
            root
        } else {
            let root = self.tree.create_root().unwrap_or_default();
            let mut scope = irisui::prelude::UiScope::new(&mut self.tree, root);
            scope.configure_container(
                "OverlayTreeRoot",
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(screen_width)
                    .height(screen_height)
                    .flex_col()
                    .align_items(irisui::prelude::AlignItems::Stretch),
                false,
            );
            root
        }
    }

    /// Returns a mutable reference to the dedicated overlay layout engine.
    #[inline]
    pub fn layout_engine(&mut self) -> &mut LayoutEngine {
        &mut self.layout_engine
    }

    /// Returns `true` if an overlay is currently active.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.active_overlay.is_some()
    }

    /// Returns the currently active overlay payload, if any.
    #[inline]
    pub fn active_overlay(&self) -> Option<&ActiveOverlay> {
        self.active_overlay.as_ref()
    }

    /// Returns the cached bounding box of the active overlay, if computed.
    #[inline]
    pub fn overlay_bounds(&self) -> Option<Rect> {
        self.overlay_bounds
    }

    /// Sets the bounding box of the active overlay.
    #[inline]
    pub fn set_overlay_bounds(&mut self, bounds: Rect) {
        self.overlay_bounds = Some(bounds);
    }

    /// Opens an overlay with the specified dismissal policy and marks the tree dirty.
    ///
    /// Preserves nodes populated by builders in the current frame and registers active state.
    pub fn open(&mut self, overlay: ActiveOverlay, policy: OverlayDismissPolicy) {
        self.active_overlay = Some(overlay);
        self.dismiss_policy = policy;
        self.overlay_bounds = None;
        self.is_dirty = true;
    }

    /// Dismisses and closes the current overlay, clearing tree nodes and cached bounds.
    pub fn close(&mut self) {
        self.active_overlay = None;
        self.overlay_bounds = None;
        self.is_dirty = false;
        self.tree.clear();
    }

    /// Tests whether a screen coordinate falls inside the active overlay boundaries.
    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        if let Some(bounds) = self.overlay_bounds {
            point.x >= bounds.x
                && point.x <= bounds.right()
                && point.y >= bounds.y
                && point.y <= bounds.bottom()
        } else {
            false
        }
    }

    /// Evaluates a pointer-down event against active overlays.
    ///
    /// Automatically closes the overlay if clicked outside and policy is [`OverlayDismissPolicy::ClickOutside`].
    pub fn handle_pointer_down(&mut self, cursor: Point) -> OverlayDismissResult {
        if !self.is_open() {
            return OverlayDismissResult::PassThrough;
        }

        if self.contains_point(cursor) {
            OverlayDismissResult::Consumed
        } else {
            match self.dismiss_policy {
                OverlayDismissPolicy::ClickOutside => {
                    self.close();
                    OverlayDismissResult::Dismissed
                }
                OverlayDismissPolicy::ExplicitOnly => OverlayDismissResult::Ignored,
            }
        }
    }

    /// Marks the overlay tree dirty, requesting a layout or visual rebuild.
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Returns `true` if the overlay tree requires reconstruction.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Clears the dirty flag.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    /// Computes Taffy flexbox layout on the internal overlay tree.
    pub fn compute_layout(&mut self, screen_size: Size) -> bool {
        self.layout_engine
            .compute_layout(&mut self.tree, screen_size)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_tree_initial_state() {
        let tree = OverlayTree::new();
        assert!(!tree.is_open());
        assert_eq!(tree.active_overlay(), None);
        assert_eq!(tree.overlay_bounds(), None);
        assert!(!tree.is_dirty());
        assert!(tree.tree().is_empty());
    }

    #[test]
    fn test_overlay_tree_open_and_close_lifecycle() {
        let mut tree = OverlayTree::new();
        let overlay = ActiveOverlay::MenubarDropdown(ActiveMenu::File);

        tree.open(overlay.clone(), OverlayDismissPolicy::ClickOutside);
        assert!(tree.is_open());
        assert_eq!(tree.active_overlay(), Some(&overlay));
        assert!(tree.is_dirty());

        tree.set_overlay_bounds(Rect::new(10.0, 30.0, 180.0, 200.0));
        assert_eq!(
            tree.overlay_bounds(),
            Some(Rect::new(10.0, 30.0, 180.0, 200.0))
        );

        tree.close();
        assert!(!tree.is_open());
        assert_eq!(tree.active_overlay(), None);
        assert_eq!(tree.overlay_bounds(), None);
        assert!(!tree.is_dirty());
    }

    #[test]
    fn test_overlay_tree_click_inside_consumed() {
        let mut tree = OverlayTree::new();
        tree.open(
            ActiveOverlay::InspectorColorPicker,
            OverlayDismissPolicy::ClickOutside,
        );
        tree.set_overlay_bounds(Rect::new(100.0, 100.0, 200.0, 200.0));

        let inside_point = Point::new(150.0, 150.0);
        assert!(tree.contains_point(inside_point));
        assert_eq!(
            tree.handle_pointer_down(inside_point),
            OverlayDismissResult::Consumed
        );
        assert!(tree.is_open());
    }

    #[test]
    fn test_overlay_tree_click_outside_dismiss_policy() {
        let mut tree = OverlayTree::new();
        tree.open(
            ActiveOverlay::HierarchyContextMenu,
            OverlayDismissPolicy::ClickOutside,
        );
        tree.set_overlay_bounds(Rect::new(50.0, 50.0, 100.0, 100.0));

        let outside_point = Point::new(200.0, 200.0);
        assert!(!tree.contains_point(outside_point));
        assert_eq!(
            tree.handle_pointer_down(outside_point),
            OverlayDismissResult::Dismissed
        );
        assert!(!tree.is_open());
    }

    #[test]
    fn test_overlay_tree_modal_explicit_policy_not_dismissed_on_outside() {
        let mut tree = OverlayTree::new();
        tree.open(
            ActiveOverlay::Modal(ModalKind::Preferences),
            OverlayDismissPolicy::ExplicitOnly,
        );
        tree.set_overlay_bounds(Rect::new(100.0, 100.0, 400.0, 300.0));

        let outside_point = Point::new(20.0, 20.0);
        assert!(!tree.contains_point(outside_point));
        assert_eq!(
            tree.handle_pointer_down(outside_point),
            OverlayDismissResult::Ignored
        );
        assert!(tree.is_open());
    }

    #[test]
    fn test_overlay_tree_layout_computation() {
        let mut tree = OverlayTree::new();
        let root = tree.tree_mut().create_root().expect("root widget");
        let _ = root;
        assert!(tree.compute_layout(Size::new(800.0, 600.0)));
    }

    #[test]
    fn test_overlay_tree_open_preserves_populated_nodes() {
        let mut tree = OverlayTree::new();
        let root = tree.tree_mut().create_root().expect("root widget");
        let mut scope = irisui::prelude::UiScope::new(tree.tree_mut(), root);
        scope.label(
            "Overlay Test Item",
            12.0,
            irisui::prelude::Color::WHITE,
            irisui::prelude::TextAlign::Left,
        );

        assert_eq!(tree.tree().iter().count(), 2);
        tree.open(
            ActiveOverlay::MenubarDropdown(ActiveMenu::File),
            OverlayDismissPolicy::ClickOutside,
        );
        assert_eq!(
            tree.tree().iter().count(),
            2,
            "open() must preserve builder-created nodes"
        );
        assert!(tree.tree().root().is_some());
    }
}