// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event Routing & Hit-Testing Interaction Logic for Iris UI Asset Browser.
//!
//! Intercepts mouse clicks, right-click context menus, double clicks, wheel scrolling,
//! search box focus, 3D orbit dragging, and keyboard shortcuts (Space, F2, Delete, Escape).
//!

use super::types::{
    AssetItemAction, AssetPreviewModalState, AssetsContextMenuAction, AssetsContextMenuTarget,
    AssetsPanelAction, AssetsTagTarget, resolve_assets_ctx_action, resolve_assets_tag,
};
use crate::assets::types::{AssetCategory, AssetItem, AssetViewMode};
use irisui::prelude::{HitTargetInfo, Point, Rect, UiLayer, WidgetRole};
use std::path::{Path, PathBuf};
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{Key, NamedKey};

#[inline]
fn is_point_in_rect(rect: Rect, p: Point) -> bool {
    p.x >= rect.x && p.x <= rect.x + rect.width && p.y >= rect.y && p.y <= rect.y + rect.height
}

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

/// Context descriptor bundling query and geometry state for Asset Browser event processing.
pub struct AssetsEventContext<'a> {
    /// Current window cursor position in physical or logical coordinates.
    pub cursor_pos: Point,
    /// Outer panel bounds.
    pub panel_rect: Rect,
    /// Bounding rectangle of the folder tree sidebar if not collapsed.
    pub sidebar_rect: Option<Rect>,
    /// Bounding rectangle of the scrollable content viewport.
    pub content_viewport_rect: Rect,
    /// Active right-click context menu subject and click position, if open.
    pub context_menu: Option<&'a (AssetsContextMenuTarget, Point)>,
    /// Bounding rectangle of the floating context menu card, if open.
    pub context_menu_card_rect: Option<Rect>,
    /// Active Quick Asset Preview modal state, if open.
    pub preview_modal: Option<&'a AssetPreviewModalState>,
    /// Current active folder path.
    pub current_folder: &'a Path,
    /// Active search filter string buffer.
    pub search_query: &'a str,
    /// Whether the search input box currently has keyboard focus.
    pub is_search_focused: bool,
    /// Currently selected asset path, if any.
    pub selected_asset: Option<&'a Path>,
    /// Filtered asset items currently displayed.
    pub filtered_items: &'a [AssetItem],
    /// Discovered subfolders tree cached in engine asset state.
    pub subfolders: &'a [PathBuf],
    /// Optional hit target information evaluated via `UiTree::hit_test_target` at cursor position.
    pub hit_target: Option<HitTargetInfo>,
}

/// Evaluates a mouse click event against the active Asset Browser panel hit targets.
pub fn handle_assets_click(
    ctx: &AssetsEventContext<'_>,
    tracker: &mut AssetClickTracker,
    out_actions: &mut Vec<AssetsPanelAction>,
) -> bool {
    let cursor_pos = ctx.cursor_pos;
    let current_folder = ctx.current_folder;
    let is_search_focused = ctx.is_search_focused;

    // 0a. Quick Asset Preview Modal Interactions (Zero-Allocation O(1) Semantic Dispatch)
    if let Some(pm) = ctx.preview_modal {
        if let Some(ref hit) = ctx.hit_target {
            if hit.tag == irisui::prelude::MODAL_TAG_CLOSE
                || hit.tag == irisui::prelude::MODAL_TAG_SCRIM
            {
                out_actions.push(AssetsPanelAction::CloseInspectModal);
                return true;
            } else if hit.tag == irisui::prelude::MODAL_TAG_CONFIRM {
                out_actions.push(AssetsPanelAction::SpawnAsset(
                    pm.item.path.clone(),
                    pm.item.category,
                ));
                out_actions.push(AssetsPanelAction::CloseInspectModal);
                return true;
            } else if hit.tag == super::types::ASSET_PREVIEW_TAG_REVEAL {
                out_actions.push(AssetsPanelAction::RevealFolder(pm.item.path.clone()));
                return true;
            } else if hit.layer == UiLayer::Modal
                || hit.tag == super::types::ASSET_PREVIEW_TAG_ORBIT
            {
                // Clicked inside modal card or on orbit canvas - consume click
                return true;
            }
        }
        // Outside click on scrim dismisses modal
        out_actions.push(AssetsPanelAction::CloseInspectModal);
        return true;
    }

    // 0b. Floating Context Menu Interactions (Zero-Allocation O(1) Dispatch)
    if let Some((target, _)) = ctx.context_menu {
        if let Some(ref hit) = ctx.hit_target
            && hit.layer == UiLayer::Popup
            && hit.role == WidgetRole::DropdownItem
            && let Some(act) = resolve_assets_ctx_action(hit.tag)
        {
            out_actions.push(AssetsPanelAction::CloseContextMenu);
            match act {
                AssetsContextMenuAction::Inspect => {
                    if let AssetsContextMenuTarget::Asset(item) = target {
                        out_actions.push(AssetsPanelAction::OpenInspectModal(item.clone()));
                    }
                }
                AssetsContextMenuAction::Spawn => {
                    if let AssetsContextMenuTarget::Asset(item) = target {
                        out_actions.push(AssetsPanelAction::SpawnAsset(
                            item.path.clone(),
                            item.category,
                        ));
                    }
                }
                AssetsContextMenuAction::NewFolder => {
                    let parent = match target {
                        AssetsContextMenuTarget::Folder(path) => path.clone(),
                        AssetsContextMenuTarget::Asset(_) => current_folder.to_path_buf(),
                    };
                    out_actions.push(AssetsPanelAction::OpenCreateSubfolder(parent));
                }
                AssetsContextMenuAction::Rename => match target {
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
                },
                AssetsContextMenuAction::Delete => {
                    let path = match target {
                        AssetsContextMenuTarget::Asset(item) => item.path.clone(),
                        AssetsContextMenuTarget::Folder(path) => path.clone(),
                    };
                    out_actions.push(AssetsPanelAction::OpenDelete(path));
                }
                AssetsContextMenuAction::CopyPath => {
                    if let AssetsContextMenuTarget::Asset(item) = target {
                        out_actions.push(AssetsPanelAction::CopyPath(item.path.clone()));
                    }
                }
                AssetsContextMenuAction::Reveal => {
                    let path = match target {
                        AssetsContextMenuTarget::Asset(item) => item.path.clone(),
                        AssetsContextMenuTarget::Folder(path) => path.clone(),
                    };
                    out_actions.push(AssetsPanelAction::RevealFolder(path));
                }
            }
            return true;
        }

        if ctx
            .context_menu_card_rect
            .is_some_and(|r| is_point_in_rect(r, cursor_pos))
        {
            // Clicked inside context menu card background
            return true;
        }

        // Clicked outside context menu card: dismiss context menu and proceed with click
        out_actions.push(AssetsPanelAction::CloseContextMenu);
    }

    if !is_point_in_rect(ctx.panel_rect, cursor_pos) {
        if is_search_focused {
            out_actions.push(AssetsPanelAction::FocusSearch(false));
        }
        return false;
    }

    // --- 100% Declarative O(1) Semantic Tag Dispatch ---
    if let Some(ref hit) = ctx.hit_target
        && let Some(target) = resolve_assets_tag(hit.tag)
    {
        match target {
            AssetsTagTarget::Item(item_idx, action) => {
                if let Some(item) = ctx.filtered_items.get(item_idx as usize) {
                    match action {
                        AssetItemAction::SelectOrOpen => {
                            let now = Instant::now();
                            let is_double_click = if let (Some(last_path), Some(last_time)) =
                                (&tracker.last_path, tracker.last_instant)
                            {
                                last_path == &item.path
                                    && now.duration_since(last_time).as_millis() < 400
                            } else {
                                false
                            };

                            if is_double_click {
                                tracker.last_path = None;
                                tracker.last_instant = None;
                                tracker.potential_drag_item = None;
                                tracker.drag_start_pos = None;
                                out_actions.push(AssetsPanelAction::SpawnAsset(
                                    item.path.clone(),
                                    item.category,
                                ));
                            } else {
                                tracker.last_path = Some(item.path.clone());
                                tracker.last_instant = Some(now);
                                tracker.potential_drag_item = Some(item.clone());
                                tracker.drag_start_pos = Some(cursor_pos);
                                tracker.is_dragging_asset = false;
                                out_actions
                                    .push(AssetsPanelAction::SelectAsset(Some(item.path.clone())));
                            }
                        }
                        AssetItemAction::Spawn => {
                            out_actions.push(AssetsPanelAction::SpawnAsset(
                                item.path.clone(),
                                item.category,
                            ));
                        }
                        AssetItemAction::Inspect => {
                            out_actions.push(AssetsPanelAction::OpenInspectModal(item.clone()));
                        }
                    }
                    return true;
                }
            }
            AssetsTagTarget::Tree(node_idx, is_chevron) => {
                let _ = is_chevron;
                let target_path = if node_idx == 0 {
                    PathBuf::from("assets")
                } else if let Some(folder) = ctx.subfolders.get((node_idx - 1) as usize) {
                    folder.clone()
                } else {
                    PathBuf::from("assets")
                };
                out_actions.push(AssetsPanelAction::NavigateFolder(target_path));
                return true;
            }
            AssetsTagTarget::Breadcrumb(segment_idx) => {
                let segments: Vec<&str> =
                    current_folder.iter().filter_map(|s| s.to_str()).collect();
                if !segments.is_empty() {
                    let end_idx = (segment_idx as usize).min(segments.len() - 1);
                    let target_path: PathBuf = segments[..=end_idx].iter().collect();
                    out_actions.push(AssetsPanelAction::NavigateFolder(target_path));
                    return true;
                }
            }
            AssetsTagTarget::Chip(cat_idx) => {
                if let Some(&cat) = AssetCategory::ALL.get(cat_idx as usize) {
                    out_actions.push(AssetsPanelAction::SelectCategory(cat));
                    return true;
                }
            }
            AssetsTagTarget::ToggleSidebar => {
                out_actions.push(AssetsPanelAction::ToggleSidebar);
                return true;
            }
            AssetsTagTarget::Import => {
                out_actions.push(AssetsPanelAction::OpenImportDialog);
                return true;
            }
            AssetsTagTarget::CleanVram => {
                out_actions.push(AssetsPanelAction::CleanVram);
                return true;
            }
            AssetsTagTarget::ViewGrid => {
                out_actions.push(AssetsPanelAction::SetViewMode(AssetViewMode::Grid));
                return true;
            }
            AssetsTagTarget::ViewList => {
                out_actions.push(AssetsPanelAction::SetViewMode(AssetViewMode::List));
                return true;
            }
            AssetsTagTarget::EngineContent => {
                out_actions.push(AssetsPanelAction::ToggleEngineContent);
                return true;
            }
            AssetsTagTarget::SearchInput => {
                out_actions.push(AssetsPanelAction::FocusSearch(true));
                return true;
            }
            AssetsTagTarget::SearchClear => {
                out_actions.push(AssetsPanelAction::ClearSearch);
                return true;
            }
            AssetsTagTarget::NewSubfolder => {
                out_actions.push(AssetsPanelAction::OpenCreateSubfolder(
                    current_folder.to_path_buf(),
                ));
                return true;
            }
            AssetsTagTarget::Reveal => {
                out_actions.push(AssetsPanelAction::RevealFolder(
                    current_folder.to_path_buf(),
                ));
                return true;
            }
            AssetsTagTarget::PreviewClose
            | AssetsTagTarget::PreviewReveal
            | AssetsTagTarget::PreviewOrbit
            | AssetsTagTarget::PreviewWireframe => {}
        }
    }

    // Unfocus search box if clicked elsewhere
    if is_search_focused {
        out_actions.push(AssetsPanelAction::FocusSearch(false));
    }

    // Clicking empty area in content viewport deselects active asset
    if is_point_in_rect(ctx.content_viewport_rect, cursor_pos) {
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
    let cursor_pos = ctx.cursor_pos;

    // If preview modal is open, ignore right clicks
    if ctx.preview_modal.is_some() {
        return false;
    }

    if !is_point_in_rect(ctx.panel_rect, cursor_pos) {
        return false;
    }

    // 1. O(1) Semantic Tag Dispatch for Right-Clicks
    if let Some(ref hit) = ctx.hit_target
        && let Some(target) = resolve_assets_tag(hit.tag)
    {
        match target {
            AssetsTagTarget::Item(item_idx, _) => {
                if let Some(item) = ctx.filtered_items.get(item_idx as usize) {
                    out_actions.push(AssetsPanelAction::SelectAsset(Some(item.path.clone())));
                    out_actions.push(AssetsPanelAction::OpenContextMenu(
                        AssetsContextMenuTarget::Asset(item.clone()),
                        cursor_pos,
                    ));
                    return true;
                }
            }
            AssetsTagTarget::Tree(node_idx, _) => {
                let target_path = if node_idx == 0 {
                    PathBuf::from("assets")
                } else if let Some(folder) = ctx.subfolders.get((node_idx - 1) as usize) {
                    folder.clone()
                } else {
                    PathBuf::from("assets")
                };
                out_actions.push(AssetsPanelAction::OpenContextMenu(
                    AssetsContextMenuTarget::Folder(target_path),
                    cursor_pos,
                ));
                return true;
            }
            _ => {}
        }
    }

    // 2. Default right-click in viewport / panel opens folder context menu
    out_actions.push(AssetsPanelAction::OpenContextMenu(
        AssetsContextMenuTarget::Folder(ctx.current_folder.to_path_buf()),
        cursor_pos,
    ));
    true
}

/// Evaluates mouse wheel scrolling against the active Asset Browser panel hit targets.
pub fn handle_assets_scroll(
    cursor_pos: Point,
    scroll_delta: f32,
    panel_rect: Rect,
    sidebar_rect: Option<Rect>,
    content_viewport_rect: Rect,
    out_actions: &mut Vec<AssetsPanelAction>,
) -> bool {
    if !is_point_in_rect(panel_rect, cursor_pos) {
        return false;
    }

    if let Some(sb_rect) = sidebar_rect
        && is_point_in_rect(sb_rect, cursor_pos)
    {
        out_actions.push(AssetsPanelAction::TreeScroll(scroll_delta));
        return true;
    }

    if is_point_in_rect(content_viewport_rect, cursor_pos) {
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
                if let Some(ref hit) = ctx.hit_target
                    && hit.tag == super::types::ASSET_PREVIEW_TAG_ORBIT
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
            return true;
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

        // Quick Asset Preview modal zoom scrolling via semantic hit-target
        if ctx.preview_modal.is_some()
            && let Some(ref hit) = ctx.hit_target
            && (hit.layer == UiLayer::Modal || hit.tag == super::types::ASSET_PREVIEW_TAG_ORBIT)
        {
            let zoom_delta = scroll_delta * 0.002;
            out_actions.push(AssetsPanelAction::InspectZoomDelta(zoom_delta));
            return true;
        }

        return handle_assets_scroll(
            ctx.cursor_pos,
            scroll_delta,
            ctx.panel_rect,
            ctx.sidebar_rect,
            ctx.content_viewport_rect,
            out_actions,
        );
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
            if ctx.preview_modal.is_some() {
                out_actions.push(AssetsPanelAction::CloseInspectModal);
                return true;
            }
            if ctx.context_menu.is_some() {
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
        } else if ctx.preview_modal.is_none() {
            // 5c. Spacebar: Quick Asset Preview modal
            if matches!(logical_key, Key::Named(NamedKey::Space))
                && let Some(sel_path) = ctx.selected_asset
                && let Some(item) = ctx.filtered_items.iter().find(|i| i.path == sel_path)
            {
                out_actions.push(AssetsPanelAction::OpenInspectModal(item.clone()));
                return true;
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