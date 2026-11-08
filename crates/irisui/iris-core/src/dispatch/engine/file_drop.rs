// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # File Drag & Drop Dispatch Handlers
//!
//! Handles external OS file hover, drop, and cancellation events against the active UI hierarchy.

use super::motion::handle_mouse_move;
use crate::dispatch::types::DispatchResult;
use crate::geometry::Point;
use crate::tree::UiTree;

/// Processes an OS file hovering over the window, updating hover tracking and hover state.
///
/// Avoids redundant heap allocations by mutating only the position coordinate when the hovered
/// path has not changed across consecutive cursor movement events.
pub(crate) fn handle_file_hovered(
    tree: &mut UiTree,
    point: Point,
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
    handle_mouse_move(tree, point)
}

/// Processes an OS file drop event onto the UI hierarchy.
pub(crate) fn handle_file_dropped(
    tree: &mut UiTree,
    point: Point,
    paths: &[std::path::PathBuf],
) -> DispatchResult {
    tree.event_state.hovered_file = None;
    let hit_id = tree.hit_test_layered(point);
    let target_tag = hit_id.and_then(|id| tree.get(id)).map_or(0, |n| n.tag);
    tree.event_state.frame_dropped_files = Some((hit_id, target_tag, paths.to_vec(), point));

    DispatchResult {
        consumed: hit_id.is_some(),
        needs_redraw: true,
        target: hit_id,
        target_tag,
        ..DispatchResult::default()
    }
}

/// Cancels an in-progress file drag/hover interaction.
pub(crate) fn handle_file_cancelled(tree: &mut UiTree) -> DispatchResult {
    tree.event_state.hovered_file = None;
    tree.event_state.frame_dropped_files = None;
    DispatchResult::default()
}