// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # File Drag & Drop Dispatch Handlers
//!
//! Handles external OS file hover, drop, and cancellation events against the active UI hierarchy.

use super::motion::handle_mouse_move;
use crate::dispatch::types::DispatchResult;
use crate::geometry::Point;
use crate::node::WidgetRole;
use crate::tree::UiTree;

/// Processes an OS file hovering over the window, updating hover tracking and hover state.
///
/// Avoids redundant heap allocations by mutating only the position coordinate when the hovered
/// path has not changed across consecutive cursor movement events.
pub(crate) fn handle_file_hovered(
    tree: &mut UiTree,
    point: Option<Point>,
    path: &std::path::Path,
) -> DispatchResult {
    match &mut tree.event_state.hovered_file {
        Some((current_path, current_point)) if current_path.as_path() == path => {
            *current_point = point;
        }
        slot => {
            *slot = Some((path.to_path_buf(), point));
        }
    }
    if let Some(pt) = point {
        handle_mouse_move(tree, pt)
    } else {
        DispatchResult {
            needs_redraw: true,
            ..DispatchResult::default()
        }
    }
}

/// Processes an OS file drop event onto the UI hierarchy.
///
/// Only explicit [`WidgetRole::DropTarget`] containers (or active modal occlusion barriers)
/// mark the event as `consumed`, allowing drops over passive UI regions, viewport HUD badges,
/// tabs, or unpositioned compositor drops (`point == None`) to route to host workbench handlers.
pub(crate) fn handle_file_dropped(
    tree: &mut UiTree,
    point: Option<Point>,
    paths: &[std::path::PathBuf],
) -> DispatchResult {
    tree.event_state.hovered_file = None;
    let valid_point = point.filter(|pt| pt.x.is_finite() && pt.y.is_finite());
    let hit_id = valid_point.and_then(|pt| tree.hit_test_layered(pt));
    let drop_target_id =
        hit_id.and_then(|id| tree.find_ancestor_with_role(id, WidgetRole::DropTarget));
    let modal_active = tree.is_modal_active();

    if modal_active && drop_target_id.is_none() {
        tree.event_state.frame_dropped_files = None;
        return DispatchResult {
            consumed: true,
            needs_redraw: true,
            target: hit_id,
            target_tag: hit_id.and_then(|id| tree.get(id)).map_or(0, |n| n.tag),
            ..DispatchResult::default()
        };
    }

    let target_id = drop_target_id.or(hit_id);
    let target_tag = target_id.and_then(|id| tree.get(id)).map_or(0, |n| n.tag);
    tree.event_state.frame_dropped_files =
        Some((target_id, target_tag, paths.to_vec(), valid_point));

    DispatchResult {
        consumed: drop_target_id.is_some(),
        needs_redraw: true,
        target: target_id,
        target_tag,
        ..DispatchResult::default()
    }
}

/// Cancels an in-progress file drag/hover interaction.
pub(crate) fn handle_file_cancelled(tree: &mut UiTree) -> DispatchResult {
    let was_hovering = tree.event_state.hovered_file.is_some();
    tree.event_state.hovered_file = None;
    tree.event_state.frame_dropped_files = None;
    DispatchResult {
        needs_redraw: was_hovering,
        ..DispatchResult::default()
    }
}