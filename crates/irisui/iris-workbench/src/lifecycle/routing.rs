// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Targeted Event Invalidation Routing
//!
//! Routes platform interaction events and dispatch results to their owning dock panel
//! leaf containers or escalates to global shell reconstruction when interactions occur
//! in application chrome or outside any docking leaf.

use iris_core::dispatch::DispatchResult;
use iris_core::event::UiEvent;
use iris_core::tree::UiTree;
use iris_dock::chrome::DockChromeFrame;
use iris_dock::notifier::UiNotifier;
use std::hash::Hash;

/// Categorization of event-driven invalidation routing across the workbench.
///
/// Enables desktop IDE architectures to isolate frame invalidation to the exact
/// dock panel containing the interacted widget node, avoiding costly global shell rebuilds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventInvalidationRoute<TabId> {
    /// Invalidation successfully localized to a specific docking panel leaf container.
    ///
    /// The hosting coordinator will trigger an isolated partial subtree layout (Branch 2)
    /// exclusively for this panel, leaving the rest of the application shell intact.
    Panel(TabId),

    /// Interaction occurred outside docking panel leaves (e.g. MenuBar, StatusBar,
    /// modal dialog backdrop, or floating window titlebar) or during active pointer dragging,
    /// requiring coarse application shell reconstruction (Branch 1).
    Shell,

    /// Interaction requires zero structural or panel invalidation (e.g. idle mouse motion
    /// without active pointer dragging).
    None,
}

impl<TabId> EventInvalidationRoute<TabId> {
    /// Returns `true` if the event was routed to a specific panel leaf container.
    #[inline]
    #[must_use]
    pub const fn is_panel(&self) -> bool {
        matches!(self, Self::Panel(_))
    }

    /// Returns `true` if the event requires a coarse application shell reconstruction.
    #[inline]
    #[must_use]
    pub const fn is_shell(&self) -> bool {
        matches!(self, Self::Shell)
    }

    /// Returns `true` if the event requires zero structural or panel invalidation.
    #[inline]
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns a reference to the target panel identifier if routed to a panel.
    #[inline]
    #[must_use]
    pub fn panel(&self) -> Option<&TabId> {
        match self {
            Self::Panel(tab) => Some(tab),
            _ => None,
        }
    }

    /// Applies the invalidation routing decision directly to an external [`UiNotifier`].
    ///
    /// Automatically calls [`UiNotifier::tag_redraw`] for panel routes, [`UiNotifier::tag_all`]
    /// for shell routes, or performs no-op for dormant events.
    #[inline]
    pub fn apply_to_notifier(&self, notifier: &mut UiNotifier<TabId>)
    where
        TabId: Copy + Eq + Hash + Send + Sync + 'static,
    {
        match self {
            Self::Panel(tab) => {
                notifier.tag_redraw(*tab);
            }
            Self::Shell => {
                notifier.tag_all();
            }
            Self::None => {}
        }
    }
}

/// Routes an interaction event and its dispatch result to its owning dock panel leaf container,
/// or escalates to global shell invalidation if the interaction occurred in the shell chrome.
///
/// # Resolution Order
/// 1. **Primary Target Boundary**: Resolves `dispatch.target` to its nearest ancestor
///    invalidation boundary via [`UiTree::find_invalidation_boundary`]. If that boundary matches
///    an active leaf container in [`DockChromeFrame::tab_for_leaf_container`], routes directly to
///    [`EventInvalidationRoute::Panel`].
/// 2. **Semantic / Focused Tag Fallback**: If unrouted, checks `dispatch.target_tag` (if non-zero)
///    or `tree.event_state.focused_tag` (if text input is focused). Resolves the tag to a node via
///    [`UiTree::find_node_by_tag`], then inspects its invalidation boundary against the dock frame.
/// 3. **Unrouted Gesture Escalation**: If the interaction occurred outside any dock leaf (e.g. MenuBar,
///    StatusBar, Modal backdrop, or active Popup dragging) and is NOT idle continuous motion
///    (`MouseMove` / `FileHovered` without active drag), escalates to [`EventInvalidationRoute::Shell`].
/// 4. **Dormant Motion**: Idle hover motion outside dock leaves produces [`EventInvalidationRoute::None`].
pub fn route_event_invalidation<TabId: Copy + Eq>(
    tree: &UiTree,
    dock_frame: Option<&DockChromeFrame<TabId>>,
    event: &UiEvent,
    dispatch: &DispatchResult,
) -> EventInvalidationRoute<TabId> {
    let Some(dock_frame) = dock_frame else {
        let is_continuous_motion = matches!(
            event,
            UiEvent::MouseMove { .. } | UiEvent::FileHovered { .. }
        );
        let is_pointer_dragging = tree.event_state.is_pointer_drag_active;
        if !is_continuous_motion || is_pointer_dragging {
            return EventInvalidationRoute::Shell;
        }
        return EventInvalidationRoute::None;
    };

    // 1. Direct target node invalidation boundary lookup
    if let Some(target_id) = dispatch.target
        && let Some(boundary_id) = tree.find_invalidation_boundary(target_id)
        && let Some(tab) = dock_frame.tab_for_leaf_container(boundary_id)
    {
        return EventInvalidationRoute::Panel(tab);
    }

    // 2. Semantic tag or focused tag fallback
    let target_tag = if dispatch.target_tag != 0 {
        Some(dispatch.target_tag)
    } else if tree.is_text_input_focused() {
        tree.event_state.focused_tag
    } else {
        None
    };

    if let Some(tag) = target_tag
        && let Some(node_id) = tree.find_node_by_tag(tag)
        && let Some(boundary_id) = tree.find_invalidation_boundary(node_id)
        && let Some(tab) = dock_frame.tab_for_leaf_container(boundary_id)
    {
        return EventInvalidationRoute::Panel(tab);
    }

    // 3. Unrouted event escalation: Discrete gestures or pointer dragging outside any dock leaf
    // (e.g. clicking top menubar, statusbar, modal dialogs, or moving a dragging window/popup)
    let is_continuous_motion = matches!(
        event,
        UiEvent::MouseMove { .. } | UiEvent::FileHovered { .. }
    );
    let is_pointer_dragging = tree.event_state.is_pointer_drag_active;

    if !is_continuous_motion || is_pointer_dragging {
        EventInvalidationRoute::Shell
    } else {
        EventInvalidationRoute::None
    }
}

/// Same as [`route_event_invalidation`], but allows a custom fallback resolver closure
/// (e.g. for domain-specific tag mappings before falling back to shell escalation).
pub fn route_event_invalidation_with_fallback<TabId: Copy + Eq, F>(
    tree: &UiTree,
    dock_frame: Option<&DockChromeFrame<TabId>>,
    event: &UiEvent,
    dispatch: &DispatchResult,
    custom_resolver: F,
) -> EventInvalidationRoute<TabId>
where
    F: FnOnce(&DispatchResult, &UiEvent) -> Option<TabId>,
{
    let base_route = route_event_invalidation(tree, dock_frame, event, dispatch);
    if (base_route.is_none() || base_route.is_shell())
        && let Some(custom_tab) = custom_resolver(dispatch, event)
    {
        return EventInvalidationRoute::Panel(custom_tab);
    }
    base_route
}