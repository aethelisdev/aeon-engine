// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Pointer Hit-Testing & Occlusion Subsystem
//!
//! Evaluates whether pointer coordinates fall over interactive application chrome,
//! elevated floating windows, active modal dialogs, or transparent 3D viewports.

use super::runner::Workbench;
use irisui_core::WidgetId;
use irisui_core::event::{MouseButton, UiEvent};
use irisui_core::geometry::{Point, Rect};
use irisui_core::node::UiLayer;
use irisui_core::tree::UiTree;
use irisui_dock::chrome::DockChromeFrame;
use irisui_dock::state::DockState;
use std::fmt::Debug;
use std::hash::Hash;

/// Result of multi-layered pointer hover resolution across a retained [`UiTree`]
/// and active docking frame.
///
/// Captures the highest-priority widget hit, resolved 64-bit semantic tag, overlay
/// occlusion flag, underlying dock panel (excluding any designated non-UI viewport),
/// and frame-to-frame transition indicators with zero heap allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverTargetResolution<TabId> {
    /// Identifier of the topmost hit widget node, if any.
    pub hit_widget: Option<WidgetId>,
    /// Semantic 64-bit interaction tag resolved from the hit widget or nearest tagged ancestor.
    pub hovered_tag: Option<u64>,
    /// Hovered non-excluded dock panel tab when not occluded by elevated overlays.
    pub hovered_panel: Option<TabId>,
    /// Whether the pointer lands on an elevated overlay layer (`UiLayer::Floating` or higher).
    pub is_over_overlay: bool,
    /// Whether `hovered_tag` differs from the previously recorded hover tag.
    pub hover_target_changed: bool,
    /// Whether the hover transition affects any label or button text color.
    pub hover_affects_text: bool,
    /// Whether the hover transition involves a live node without an invalidation boundary.
    pub hover_requires_full_upload: bool,
    /// Whether `hovered_panel` differs from the previously recorded hovered panel.
    pub hovered_panel_changed: bool,
}

impl<TabId> Default for HoverTargetResolution<TabId> {
    #[inline]
    fn default() -> Self {
        Self {
            hit_widget: None,
            hovered_tag: None,
            hovered_panel: None,
            is_over_overlay: false,
            hover_target_changed: false,
            hover_affects_text: false,
            hover_requires_full_upload: false,
            hovered_panel_changed: false,
        }
    }
}

/// Evaluates layered hit-testing across the UI widget tree and docking chrome frame
/// at `point` with zero heap allocations.
///
/// Resolution priority:
/// 1. Layered tree hit-test ([`UiTree::hit_test_tag_layered`]) extracting effective stacking
///    [`UiLayer`] and inherited semantic interaction tag.
/// 2. Overlay occlusion check (`layer >= UiLayer::Floating`).
/// 3. Underlying dock panel lookup from `dock_frame.panel_rects` when unoccluded, skipping
///    `non_ui_tab` (e.g. an interactive 3D viewport canvas).
#[must_use]
pub fn resolve_hover_targets<TabId: Copy + PartialEq>(
    tree: &UiTree,
    dock_frame: Option<&DockChromeFrame<TabId>>,
    non_ui_tab: Option<TabId>,
    point: Point,
) -> HoverTargetResolution<TabId> {
    let (hit_widget, hit_layer, hovered_tag) = match tree.hit_test_tag_layered(point) {
        Some((id, layer, tag)) => (Some(id), Some(layer), (tag != 0).then_some(tag)),
        None => (None, None, None),
    };
    let is_over_overlay = hit_layer.is_some_and(|layer| layer >= UiLayer::Floating);
    let hovered_panel = if is_over_overlay {
        None
    } else {
        dock_frame.and_then(|frame| {
            frame
                .panel_rects
                .iter()
                .find(|(pid, bounds)| non_ui_tab != Some(*pid) && bounds.contains_point(point))
                .map(|(pid, _)| *pid)
        })
    };

    HoverTargetResolution {
        hit_widget,
        hovered_tag,
        hovered_panel,
        is_over_overlay,
        hover_target_changed: false,
        hover_affects_text: false,
        hover_requires_full_upload: false,
        hovered_panel_changed: false,
    }
}

/// Tracks the state of the system pointer device across interaction events,
/// recording cursor coordinates and mouse button down states.
///
/// Designed to provide high-level, decoupled pointer telemetry to the workbench orchestrator
/// and application shells without leaking low-level window event structures.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerState {
    /// Most recently observed cursor coordinate in logical screen points.
    pub position: Option<Point>,

    /// Whether the primary (left) mouse button is currently held down.
    pub primary_down: bool,

    /// Whether the secondary (right) mouse button is currently held down.
    pub secondary_down: bool,

    /// Whether the middle mouse button is currently held down.
    pub middle_down: bool,
}

impl PointerState {
    /// Creates a default, uninitialized pointer state with no active buttons or coordinates.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: None,
            primary_down: false,
            secondary_down: false,
            middle_down: false,
        }
    }

    /// Updates internal cursor position and button pressed states from an incoming [`UiEvent`].
    pub fn update_from_event(&mut self, event: &UiEvent) {
        if let Some(pos) = event.pointer_position() {
            self.position = Some(pos);
        }
        match event {
            UiEvent::MouseDown { button, .. } => match button {
                MouseButton::Left => self.primary_down = true,
                MouseButton::Right => self.secondary_down = true,
                MouseButton::Middle => self.middle_down = true,
                MouseButton::Other(_) => {}
            },
            UiEvent::MouseUp { button, .. } => match button {
                MouseButton::Left => self.primary_down = false,
                MouseButton::Right => self.secondary_down = false,
                MouseButton::Middle => self.middle_down = false,
                MouseButton::Other(_) => {}
            },
            _ => {}
        }
    }

    /// Returns the most recently observed pointer position in logical points.
    #[must_use]
    #[inline]
    pub const fn position(&self) -> Option<Point> {
        self.position
    }

    /// Sets or overrides the recorded pointer position.
    #[inline]
    pub fn set_position(&mut self, position: Option<Point>) {
        self.position = position;
    }

    /// Returns whether the primary (left) mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_primary_down(&self) -> bool {
        self.primary_down
    }

    /// Returns whether the secondary (right) mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_secondary_down(&self) -> bool {
        self.secondary_down
    }

    /// Returns whether the middle mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_middle_down(&self) -> bool {
        self.middle_down
    }

    /// Returns whether any mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_any_down(&self) -> bool {
        self.primary_down || self.secondary_down || self.middle_down
    }

    /// Resets all mouse button states to released.
    #[inline]
    pub fn release_all_buttons(&mut self) {
        self.primary_down = false;
        self.secondary_down = false;
        self.middle_down = false;
    }
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Returns an immutable reference to the autonomous pointer state tracker.
    #[must_use]
    #[inline]
    pub fn pointer_state(&self) -> &PointerState {
        &self.pointer_state
    }

    /// Returns a mutable reference to the autonomous pointer state tracker.
    #[inline]
    pub fn pointer_state_mut(&mut self) -> &mut PointerState {
        &mut self.pointer_state
    }
    /// Resolves and records hovered UI tags and dock panel targets at `point`
    /// using the workbench's retained widget tree and docking frame.
    pub fn resolve_hover_targets(&mut self, point: Option<Point>) -> HoverTargetResolution<TabId> {
        self.resolve_hover_targets_with_frame(None, point)
    }

    /// Resolves and records hovered UI tags and dock panel targets using an explicit
    /// docking chrome frame (falling back to [`Self::active_dock_frame`] when `None`).
    pub fn resolve_hover_targets_with_frame(
        &mut self,
        dock_frame: Option<&DockChromeFrame<TabId>>,
        point: Option<Point>,
    ) -> HoverTargetResolution<TabId> {
        let resolved_point = point.or(self.last_cursor_pos);
        let effective_frame = dock_frame.or(self.dock_frame.as_ref());
        let resolution = match resolved_point {
            Some(pt) => resolve_hover_targets(&self.tree, effective_frame, self.non_ui_tab, pt),
            None => HoverTargetResolution::default(),
        };
        let tree_ref = &self.tree;
        let mut last_hovered_tag = self.last_hovered_tag;
        let mut last_hit_widget = self.last_hit_widget;
        let mut last_hovered_panel = self.last_hovered_panel;
        let committed = commit_hover_resolution_state(
            resolution,
            tree_ref,
            &mut self.hovered_tag,
            &mut self.hovered_panel,
            &mut last_hovered_tag,
            &mut last_hit_widget,
            &mut last_hovered_panel,
        );
        self.last_hovered_tag = last_hovered_tag;
        self.last_hit_widget = last_hit_widget;
        self.last_hovered_panel = last_hovered_panel;
        committed
    }

    /// Evaluates whether the UI currently captures the pointer or if the pointer rests over any
    /// active workbench UI element using the workbench's owned docking state.
    #[inline]
    #[must_use]
    pub fn wants_pointer_self(&self) -> bool {
        self.wants_pointer_input(
            Some(self.effective_cursor_pos()),
            &self.descriptor.dock_state,
        )
    }

    /// Evaluates whether `pos` rests over active workbench UI elements using the workbench's owned
    /// docking state rather than the interactive viewport canvas.
    #[inline]
    #[must_use]
    pub fn wants_pointer_at_self(&self, pos: [f32; 2]) -> bool {
        self.wants_pointer_at(pos, &self.descriptor.dock_state)
    }

    /// Evaluates whether the UI currently captures the pointer or if the pointer is positioned over
    /// any active workbench UI element (menubar, statusbar, panels, floating windows, popups).
    #[must_use]
    pub fn wants_pointer_input(
        &self,
        cursor_pos: Option<Point>,
        dock_state: &DockState<TabId>,
    ) -> bool {
        let pt = cursor_pos.or(self.last_cursor_pos);
        let Some(point) = pt else {
            return false;
        };
        self.wants_pointer_at_full([point.x, point.y], self.interactive_viewport, dock_state)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, or the top menubar rather than transparent canvas/viewport areas.
    #[must_use]
    pub fn wants_pointer_at_default(&self, pos: [f32; 2], non_ui_viewport: Option<Rect>) -> bool {
        self.wants_pointer_at_with_dock(pos, non_ui_viewport, &self.descriptor.dock_state)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, floating windows, or elevated UI tree layers using the configured interactive viewport.
    #[must_use]
    pub fn wants_pointer_at(&self, pos: [f32; 2], dock_state: &DockState<TabId>) -> bool {
        self.wants_pointer_at_full(pos, self.interactive_viewport, dock_state)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, or floating windows using an explicitly provided docking state.
    #[must_use]
    pub fn wants_pointer_at_with_dock(
        &self,
        pos: [f32; 2],
        non_ui_viewport: Option<Rect>,
        dock_state: &DockState<TabId>,
    ) -> bool {
        self.wants_pointer_at_full(pos, non_ui_viewport, dock_state)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, floating windows, or elevated UI tree layers.
    #[must_use]
    pub fn wants_pointer_at_full(
        &self,
        pos: [f32; 2],
        non_ui_viewport: Option<Rect>,
        dock_state: &DockState<TabId>,
    ) -> bool {
        let point = Point::new(pos[0], pos[1]);

        let is_over_statusbar = if self.descriptor.statusbar.is_some() {
            if let Some(win_size) = self.last_window_size {
                pos[1] >= (win_size.height - self.statusbar_height).max(0.0)
                    && pos[1] <= win_size.height
            } else {
                false
            }
        } else {
            false
        };

        if (self.descriptor.menubar.is_some() && pos[1] <= self.menubar_height)
            || is_over_statusbar
            || self.dialog_host.is_active()
        {
            return true;
        }

        let in_occluding_floating = dock_state.floating_windows.iter().any(|w| {
            if let Some(non_ui) = self.non_ui_tab
                && w.tree.all_tabs().contains(&non_ui)
            {
                return false;
            }
            w.rect.contains_point(point)
        });

        let effective_viewport = non_ui_viewport.or(self.interactive_viewport);
        if let Some(vp) = effective_viewport
            && vp.contains_point(point)
        {
            if in_occluding_floating {
                return true;
            }

            if self.tree.wants_pointer_at(point) {
                return true;
            }

            return false;
        }

        if in_occluding_floating {
            return true;
        }

        true
    }
}

pub(super) fn commit_hover_resolution_state<TabId: Copy + PartialEq>(
    mut resolution: HoverTargetResolution<TabId>,
    tree: &UiTree,
    hovered_tag: &mut Option<u64>,
    hovered_panel: &mut Option<TabId>,
    last_hovered_tag: &mut Option<u64>,
    last_hit_widget: &mut Option<WidgetId>,
    last_hovered_panel: &mut Option<TabId>,
) -> HoverTargetResolution<TabId> {
    resolution.hover_target_changed = resolution.hovered_tag != *last_hovered_tag
        || (resolution.hovered_tag.is_none() && resolution.hit_widget != *last_hit_widget);
    if resolution.hover_target_changed {
        let target_affects_text = |widget: Option<WidgetId>, tag: Option<u64>| {
            widget.is_some_and(|id| tree.node_hover_affects_text(id))
                || tag.is_some_and(|t| tree.tag_hover_affects_text(t))
        };
        let target_lacks_boundary = |widget: Option<WidgetId>, tag: Option<u64>| {
            widget.is_some_and(|id| {
                tree.get(id).is_some() && tree.find_invalidation_boundary(id).is_none()
            }) || tag.is_some_and(|t| {
                tree.find_node_by_tag(t)
                    .is_some_and(|id| tree.find_invalidation_boundary(id).is_none())
            })
        };
        resolution.hover_affects_text = target_affects_text(*last_hit_widget, *last_hovered_tag)
            || target_affects_text(resolution.hit_widget, resolution.hovered_tag);
        resolution.hover_requires_full_upload =
            target_lacks_boundary(*last_hit_widget, *last_hovered_tag)
                || target_lacks_boundary(resolution.hit_widget, resolution.hovered_tag);
    }
    resolution.hovered_panel_changed = resolution.hovered_panel != *last_hovered_panel;
    *hovered_tag = resolution.hovered_tag;
    *hovered_panel = resolution.hovered_panel;
    *last_hovered_tag = resolution.hovered_tag;
    *last_hit_widget = resolution.hit_widget;
    *last_hovered_panel = resolution.hovered_panel;
    resolution
}