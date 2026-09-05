// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event Routing & Hit-Testing Interaction Logic for Iris UI Asset Browser.
//!
//! Intercepts mouse clicks, right-click context menus, double clicks, wheel scrolling,
//! search box focus, 3D orbit dragging, and keyboard shortcuts (Space, F2, Delete, Escape).
//!

use super::types::{AssetsContextMenuTarget, AssetsPanelAction, AssetsPanelTargets};
use crate::ui::panels::assets::types::{AssetItem, AssetViewMode};
use irisui::prelude::Point;
use std::path::{Path, PathBuf};
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{Key, NamedKey};

/// State tracking for double-click asset spawning detection and 3D preview orbital dragging.
#[derive(Debug, Clone, Default)]
pub struct AssetClickTracker {
    /// Last clicked asset path.
    pub last_path: Option<PathBuf>,
    /// Instant of the previous click.
    pub last_instant: Option<Instant>,
    /// Whether the user is actively dragging the 3D model preview canvas.
    pub is_orbit_dragging: bool,
    /// Last cursor position recorded during orbit dragging.
    pub last_drag_pos: Option<Point>,
    /// Candidate asset item queued for potential drag and drop upon cursor move threshold.
    pub potential_drag_item: Option<AssetItem>,
    /// Initial screen coordinates where the candidate asset item was clicked.
    pub drag_start_pos: Option<Point>,
    /// Whether an asset item is actively being dragged across the editor.
    pub is_dragging_asset: bool,
}

/// Context descriptor bundling query and target state for Asset Browser event processing.
pub struct AssetsEventContext<'a> {
    /// Current window cursor position in physical or logical coordinates.
    pub cursor_pos: Point,
    /// Hit target registry for active asset browser UI elements.
    pub targets: &'a AssetsPanelTargets,
    /// Current active folder path.
    pub current_folder: &'a Path,
    /// Active search filter string buffer.
    pub search_query: &'a str,
    /// Whether the search input box currently has keyboard focus.
    pub is_search_focused: bool,
    /// Currently selected asset path, if any.
    pub selected_asset: Option<&'a Path>,
}

/// Evaluates a mouse click event against the active Asset Browser panel hit targets.
pub fn handle_assets_click(
    ctx: &AssetsEventContext<'_>,
    tracker: &mut AssetClickTracker,
    out_actions: &mut Vec<AssetsPanelAction>,
) -> bool {
    let targets = ctx.targets;
    let cursor_pos = ctx.cursor_pos;
    let current_folder = ctx.current_folder;
    let is_search_focused = ctx.is_search_focused;

    // 0a. Quick Asset Preview Modal Interactions (Highest Z-Order)
    if let Some(ref pm) = targets.preview_modal {
        if pm.close_btn_rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::CloseInspectModal);
            return true;
        }
        if let Some(act_rect) = pm.action_btn_rect
            && act_rect.contains_point(cursor_pos)
        {
            out_actions.push(AssetsPanelAction::SpawnAsset(
                pm.item.path.clone(),
                pm.item.category,
            ));
            out_actions.push(AssetsPanelAction::CloseInspectModal);
            return true;
        }
        if pm.reveal_btn_rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::RevealFolder(pm.item.path.clone()));
            return true;
        }
        if pm.dialog_rect.contains_point(cursor_pos) {
            // Click inside preview modal card
            return true;
        }
        // Click on semi-transparent backdrop dismisses modal
        out_actions.push(AssetsPanelAction::CloseInspectModal);
        return true;
    }

    // 0b. Floating Context Menu Interactions
    if let Some(ref cm) = targets.context_menu {
        if cm.card_rect.contains_point(cursor_pos) {
            if let Some(r) = cm.inspect_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                if let AssetsContextMenuTarget::Asset(ref item) = cm.target {
                    out_actions.push(AssetsPanelAction::OpenInspectModal(item.clone()));
                }
                return true;
            }
            if let Some(r) = cm.spawn_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                if let AssetsContextMenuTarget::Asset(ref item) = cm.target {
                    out_actions.push(AssetsPanelAction::SpawnAsset(
                        item.path.clone(),
                        item.category,
                    ));
                }
                return true;
            }
            if let Some(r) = cm.new_folder_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                let parent = match &cm.target {
                    AssetsContextMenuTarget::Folder(path) => path.clone(),
                    AssetsContextMenuTarget::Asset(_) => current_folder.to_path_buf(),
                };
                out_actions.push(AssetsPanelAction::OpenCreateSubfolder(parent));
                return true;
            }
            if let Some(r) = cm.rename_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                match &cm.target {
                    AssetsContextMenuTarget::Asset(item) => {
                        out_actions.push(AssetsPanelAction::OpenRename(
                            item.path.clone(),
                            item.name.clone(),
                            false,
                        ));
                    }
                    AssetsContextMenuTarget::Folder(path) => {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        out_actions.push(AssetsPanelAction::OpenRename(path.clone(), name, true));
                    }
                }
                return true;
            }
            if let Some(r) = cm.delete_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                let path = match &cm.target {
                    AssetsContextMenuTarget::Asset(item) => item.path.clone(),
                    AssetsContextMenuTarget::Folder(path) => path.clone(),
                };
                out_actions.push(AssetsPanelAction::OpenDelete(path));
                return true;
            }
            if let Some(r) = cm.copy_path_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                if let AssetsContextMenuTarget::Asset(ref item) = cm.target {
                    out_actions.push(AssetsPanelAction::CopyPath(item.path.clone()));
                }
                return true;
            }
            if let Some(r) = cm.reveal_rect
                && r.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                let path = match &cm.target {
                    AssetsContextMenuTarget::Asset(item) => item.path.clone(),
                    AssetsContextMenuTarget::Folder(path) => path.clone(),
                };
                out_actions.push(AssetsPanelAction::RevealFolder(path));
                return true;
            }
            return true;
        }
        // Clicked outside context menu card: dismiss context menu and proceed with click
        out_actions.push(AssetsPanelAction::CloseContextMenu);
    }

    if !targets.panel_rect.contains_point(cursor_pos) {
        if is_search_focused {
            out_actions.push(AssetsPanelAction::FocusSearch(false));
        }
        return false;
    }

    // 1. Breadcrumbs Navigation
    for crumb in &targets.breadcrumbs {
        if crumb.rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::NavigateFolder(crumb.path.clone()));
            return true;
        }
    }

    // 2. Search Clear Button "✖"
    if let Some(clr_rect) = targets.search_clear_btn_rect
        && clr_rect.contains_point(cursor_pos)
    {
        out_actions.push(AssetsPanelAction::ClearSearch);
        return true;
    }

    // 3. Search Input Box Click
    if targets.search_input_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::FocusSearch(true));
        return true;
    } else if is_search_focused {
        out_actions.push(AssetsPanelAction::FocusSearch(false));
    }

    // 4. View Mode Toggles
    if targets.grid_toggle_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::SetViewMode(AssetViewMode::Grid));
        return true;
    }
    if targets.list_toggle_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::SetViewMode(AssetViewMode::List));
        return true;
    }

    // 5. Action Buttons: Import, Reveal, Clean
    if targets.import_btn_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::OpenImportDialog);
        return true;
    }
    if targets.reveal_btn_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::RevealFolder(
            current_folder.to_path_buf(),
        ));
        return true;
    }
    if targets.clean_btn_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::CleanVram);
        return true;
    }

    // 6. Category Chips
    for &(cat, rect) in &targets.category_chips {
        if rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::SelectCategory(cat));
            return true;
        }
    }

    // 7. Sidebar Toggle Button in Footer
    if targets.sidebar_toggle_btn_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::ToggleSidebar);
        return true;
    }

    // 8. New Subfolder "+" in Sidebar Header
    if let Some(plus_rect) = targets.new_subfolder_btn_rect
        && plus_rect.contains_point(cursor_pos)
    {
        out_actions.push(AssetsPanelAction::OpenCreateSubfolder(
            current_folder.to_path_buf(),
        ));
        return true;
    }

    // 9. Folder Tree Rows
    for node in &targets.folder_nodes {
        if node.row_rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::NavigateFolder(node.path.clone()));
            return true;
        }
    }

    // 10. Grid Cards (Click or Double Click)
    for card in &targets.grid_cards {
        if card.rect.contains_point(cursor_pos) {
            let now = Instant::now();
            let is_double_click = if let (Some(last_path), Some(last_time)) =
                (&tracker.last_path, tracker.last_instant)
            {
                last_path == &card.path && now.duration_since(last_time).as_millis() < 400
            } else {
                false
            };

            if is_double_click {
                tracker.last_path = None;
                tracker.last_instant = None;
                tracker.potential_drag_item = None;
                tracker.drag_start_pos = None;
                out_actions.push(AssetsPanelAction::SpawnAsset(
                    card.path.clone(),
                    card.category,
                ));
            } else {
                tracker.last_path = Some(card.path.clone());
                tracker.last_instant = Some(now);
                tracker.potential_drag_item = Some(card.item.clone());
                tracker.drag_start_pos = Some(cursor_pos);
                tracker.is_dragging_asset = false;
                out_actions.push(AssetsPanelAction::SelectAsset(Some(card.path.clone())));
            }
            return true;
        }
    }

    // 11. List Rows (Click, Double Click, Spawn or Inspect buttons)
    for row in &targets.list_rows {
        if row.rect.contains_point(cursor_pos) {
            if let Some(sp_rect) = row.spawn_btn_rect
                && sp_rect.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::SpawnAsset(
                    row.path.clone(),
                    row.category,
                ));
                return true;
            }
            if let Some(ins_rect) = row.inspect_btn_rect
                && ins_rect.contains_point(cursor_pos)
            {
                out_actions.push(AssetsPanelAction::OpenInspectModal(row.item.clone()));
                return true;
            }

            let now = Instant::now();
            let is_double_click = if let (Some(last_path), Some(last_time)) =
                (&tracker.last_path, tracker.last_instant)
            {
                last_path == &row.path && now.duration_since(last_time).as_millis() < 400
            } else {
                false
            };

            if is_double_click {
                tracker.last_path = None;
                tracker.last_instant = None;
                tracker.potential_drag_item = None;
                tracker.drag_start_pos = None;
                out_actions.push(AssetsPanelAction::SpawnAsset(
                    row.path.clone(),
                    row.category,
                ));
            } else {
                tracker.last_path = Some(row.path.clone());
                tracker.last_instant = Some(now);
                tracker.potential_drag_item = Some(row.item.clone());
                tracker.drag_start_pos = Some(cursor_pos);
                tracker.is_dragging_asset = false;
                out_actions.push(AssetsPanelAction::SelectAsset(Some(row.path.clone())));
            }
            return true;
        }
    }

    // Clicking empty area deselects
    if targets.content_viewport_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::SelectAsset(None));
        return true;
    }

    true
}

/// Evaluates a right-click mouse press event to open context menus for assets or folders.
pub fn handle_assets_right_click(
    ctx: &AssetsEventContext<'_>,
    out_actions: &mut Vec<AssetsPanelAction>,
) -> bool {
    let targets = ctx.targets;
    let cursor_pos = ctx.cursor_pos;

    // If preview modal is open, ignore right clicks
    if targets.preview_modal.is_some() {
        return false;
    }

    if !targets.panel_rect.contains_point(cursor_pos) {
        return false;
    }

    // 1. Right click on an asset grid card
    for card in &targets.grid_cards {
        if card.rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::SelectAsset(Some(card.path.clone())));
            out_actions.push(AssetsPanelAction::OpenContextMenu(
                AssetsContextMenuTarget::Asset(card.item.clone()),
                cursor_pos,
            ));
            return true;
        }
    }

    // 2. Right click on a list row
    for row in &targets.list_rows {
        if row.rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::SelectAsset(Some(row.path.clone())));
            out_actions.push(AssetsPanelAction::OpenContextMenu(
                AssetsContextMenuTarget::Asset(row.item.clone()),
                cursor_pos,
            ));
            return true;
        }
    }

    // 3. Right click on a folder tree node
    for node in &targets.folder_nodes {
        if node.row_rect.contains_point(cursor_pos) {
            out_actions.push(AssetsPanelAction::OpenContextMenu(
                AssetsContextMenuTarget::Folder(node.path.clone()),
                cursor_pos,
            ));
            return true;
        }
    }

    // 4. Right click on empty viewport / panel area
    if targets.content_viewport_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::OpenContextMenu(
            AssetsContextMenuTarget::Folder(ctx.current_folder.to_path_buf()),
            cursor_pos,
        ));
        return true;
    }

    false
}

/// Evaluates mouse wheel scrolling against the active Asset Browser panel hit targets.
pub fn handle_assets_scroll(
    cursor_pos: Point,
    scroll_delta: f32,
    targets: &AssetsPanelTargets,
    out_actions: &mut Vec<AssetsPanelAction>,
) -> bool {
    // 0. Quick Asset Preview modal zoom scrolling
    if let Some(ref pm) = targets.preview_modal
        && pm.dialog_rect.contains_point(cursor_pos)
    {
        let zoom_delta = scroll_delta * 0.002;
        out_actions.push(AssetsPanelAction::InspectZoomDelta(zoom_delta));
        return true;
    }

    if !targets.panel_rect.contains_point(cursor_pos) {
        return false;
    }

    if let Some(sb_rect) = targets.sidebar_rect
        && sb_rect.contains_point(cursor_pos)
    {
        out_actions.push(AssetsPanelAction::TreeScroll(scroll_delta));
        return true;
    }

    if targets.content_viewport_rect.contains_point(cursor_pos) {
        out_actions.push(AssetsPanelAction::Scroll(scroll_delta));
        return true;
    }

    true
}

/// Evaluates a window event against the active Asset Browser panel hit targets.
pub fn handle_assets_panel_event(
    event: &WindowEvent,
    ctx: &AssetsEventContext<'_>,
    tracker: &mut AssetClickTracker,
    out_actions: &mut Vec<AssetsPanelAction>,
) -> bool {
    // 1. Mouse Button Pressed
    if let WindowEvent::MouseInput {
        state: ElementState::Pressed,
        button,
        ..
    } = event
    {
        match button {
            MouseButton::Left => {
                // Check if user clicked inside 3D preview orbit canvas to initiate drag
                if let Some(ref pm) = ctx.targets.preview_modal
                    && let Some(orbit_rect) = pm.orbit_canvas_rect
                    && orbit_rect.contains_point(ctx.cursor_pos)
                {
                    tracker.is_orbit_dragging = true;
                    tracker.last_drag_pos = Some(ctx.cursor_pos);
                }
                return handle_assets_click(ctx, tracker, out_actions);
            }
            MouseButton::Right => {
                return handle_assets_right_click(ctx, out_actions);
            }
            _ => {}
        }
    }

    // 2. Mouse Button Released
    if let WindowEvent::MouseInput {
        state: ElementState::Released,
        button: MouseButton::Left,
        ..
    } = event
    {
        if tracker.is_orbit_dragging {
            tracker.is_orbit_dragging = false;
            tracker.last_drag_pos = None;
            return true;
        }

        let was_dragging = tracker.is_dragging_asset;
        tracker.is_dragging_asset = false;
        tracker.potential_drag_item = None;
        tracker.drag_start_pos = None;

        if was_dragging {
            out_actions.push(AssetsPanelAction::EndAssetDrag);
            // Allow release event to propagate to docking viewport drop handler
            return false;
        }
    }

    // 3. Cursor Moved (for 3D preview orbital rotation & drag-and-drop initiation)
    if let WindowEvent::CursorMoved { position, .. } = event {
        let current = Point::new(position.x as f32, position.y as f32);

        if tracker.is_orbit_dragging {
            if let Some(prev) = tracker.last_drag_pos {
                let dx = (current.x - prev.x) * 0.01;
                let dy = (current.y - prev.y) * 0.01;
                out_actions.push(AssetsPanelAction::InspectOrbitDelta(dx, dy));
            }
            tracker.last_drag_pos = Some(current);
            return true;
        }

        if let (Some(item), Some(start_pos)) =
            (&tracker.potential_drag_item, tracker.drag_start_pos)
        {
            let dx = current.x - start_pos.x;
            let dy = current.y - start_pos.y;
            if (dx * dx + dy * dy) > 25.0 {
                tracker.is_dragging_asset = true;
                out_actions.push(AssetsPanelAction::StartAssetDrag(item.clone()));
                tracker.potential_drag_item = None;
                tracker.drag_start_pos = None;
            }
        }

        if tracker.is_dragging_asset {
            return false;
        }
    }

    // 4. Mouse Wheel Scrolling
    if let WindowEvent::MouseWheel { delta, .. } = event {
        let scroll_delta = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y * 28.0,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };
        return handle_assets_scroll(ctx.cursor_pos, scroll_delta, ctx.targets, out_actions);
    }

    // 5. Keyboard Navigation & Action Shortcuts
    if let WindowEvent::KeyboardInput {
        event:
            KeyEvent {
                logical_key,
                state: ElementState::Pressed,
                ..
            },
        ..
    } = event
    {
        // 5a. Escape dismisses modal, context menu, or search focus
        if matches!(logical_key, Key::Named(NamedKey::Escape)) {
            if ctx.targets.preview_modal.is_some() {
                out_actions.push(AssetsPanelAction::CloseInspectModal);
                return true;
            }
            if ctx.targets.context_menu.is_some() {
                out_actions.push(AssetsPanelAction::CloseContextMenu);
                return true;
            }
            if ctx.is_search_focused {
                out_actions.push(AssetsPanelAction::FocusSearch(false));
                return true;
            }
        }

        // 5b. Search box typing (when active)
        if ctx.is_search_focused {
            match logical_key {
                Key::Named(NamedKey::Backspace) => {
                    let mut new_query = ctx.search_query.to_string();
                    new_query.pop();
                    out_actions.push(AssetsPanelAction::SearchInput(new_query));
                    return true;
                }
                Key::Named(NamedKey::Enter) => {
                    out_actions.push(AssetsPanelAction::FocusSearch(false));
                    return true;
                }
                Key::Character(text) => {
                    let mut new_query = ctx.search_query.to_string();
                    new_query.push_str(text.as_str());
                    out_actions.push(AssetsPanelAction::SearchInput(new_query));
                    return true;
                }
                _ => {}
            }
        } else if ctx.targets.preview_modal.is_none() {
            // 5c. Spacebar: Quick Asset Preview ( style)
            if matches!(logical_key, Key::Named(NamedKey::Space))
                && let Some(sel_path) = ctx.selected_asset
            {
                if let Some(card) = ctx.targets.grid_cards.iter().find(|c| c.path == sel_path) {
                    out_actions.push(AssetsPanelAction::OpenInspectModal(card.item.clone()));
                    return true;
                }
                if let Some(row) = ctx.targets.list_rows.iter().find(|r| r.path == sel_path) {
                    out_actions.push(AssetsPanelAction::OpenInspectModal(row.item.clone()));
                    return true;
                }
            }

            // 5d. F2: Rename selected asset
            if matches!(logical_key, Key::Named(NamedKey::F2))
                && let Some(sel_path) = ctx.selected_asset
            {
                let name = sel_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                out_actions.push(AssetsPanelAction::OpenRename(
                    sel_path.to_path_buf(),
                    name,
                    false,
                ));
                return true;
            }

            // 5e. Delete: Request deletion of selected asset
            if matches!(logical_key, Key::Named(NamedKey::Delete))
                && let Some(sel_path) = ctx.selected_asset
            {
                out_actions.push(AssetsPanelAction::OpenDelete(sel_path.to_path_buf()));
                return true;
            }
        }
    }

    false
}