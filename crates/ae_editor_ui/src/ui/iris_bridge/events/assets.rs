// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Content / Asset Browser Window Event Routing
//!
//! Dispatches mouse clicks, double clicks, right-click context menus, quick preview modal,
//! wheel scrolling, and search query typing for the Iris UI Asset Browser panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::WindowEvent;

impl IrisEditorOverlay {
    /// Routes window events to the Content / Asset Browser panel when active.
    /// Returns `Some(IrisOverlayEventResult)` if the event was intercepted and consumed.
    pub(crate) fn handle_assets_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let targets = self.assets_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();
        let mut actions = Vec::new();

        let current_folder = self.assets_current_folder.clone();
        let search_query = self.assets_search_query.clone();

        let ctx = super::super::assets::AssetsEventContext {
            cursor_pos: self.cursor_pos,
            targets,
            current_folder: &current_folder,
            search_query: &search_query,
            is_search_focused: self.assets_is_search_focused,
            selected_asset: self.assets_selected_asset.as_deref(),
        };

        let consumed = super::super::assets::handle_assets_panel_event(
            event,
            &ctx,
            &mut self.assets_click_tracker,
            &mut actions,
        );

        if !consumed && actions.is_empty() {
            return None;
        }

        for action in actions {
            match action {
                super::super::assets::AssetsPanelAction::Scroll(delta) => {
                    self.assets_scroll_y = (self.assets_scroll_y - delta).max(0.0);
                }
                super::super::assets::AssetsPanelAction::TreeScroll(delta) => {
                    self.assets_tree_scroll_y = (self.assets_tree_scroll_y - delta).max(0.0);
                }
                super::super::assets::AssetsPanelAction::FocusSearch(focused) => {
                    self.assets_is_search_focused = focused;
                }
                super::super::assets::AssetsPanelAction::SearchInput(ref query) => {
                    self.assets_search_query = query.clone();
                    self.assets_actions.push(action);
                }
                super::super::assets::AssetsPanelAction::ClearSearch => {
                    self.assets_search_query.clear();
                    self.assets_actions.push(action);
                }
                super::super::assets::AssetsPanelAction::NavigateFolder(ref path) => {
                    self.assets_current_folder = path.clone();
                    self.assets_actions.push(action);
                }
                super::super::assets::AssetsPanelAction::OpenContextMenu(target, pos) => {
                    self.assets_context_menu = Some((target, pos));
                }
                super::super::assets::AssetsPanelAction::CloseContextMenu => {
                    self.assets_context_menu = None;
                }
                super::super::assets::AssetsPanelAction::OpenInspectModal(item) => {
                    self.assets_preview_modal =
                        Some(super::super::assets::AssetPreviewModalState {
                            item,
                            orbit_yaw: 0.0,
                            orbit_pitch: 0.3,
                            zoom_distance: 1.0,
                            show_wireframe: true,
                        });
                }
                super::super::assets::AssetsPanelAction::CloseInspectModal => {
                    self.assets_preview_modal = None;
                }
                super::super::assets::AssetsPanelAction::InspectOrbitDelta(dx, dy) => {
                    if let Some(ref mut pm) = self.assets_preview_modal {
                        pm.orbit_yaw += dx;
                        pm.orbit_pitch = (pm.orbit_pitch + dy).clamp(-1.5, 1.5);
                    }
                }
                super::super::assets::AssetsPanelAction::InspectZoomDelta(dz) => {
                    if let Some(ref mut pm) = self.assets_preview_modal {
                        pm.zoom_distance = (pm.zoom_distance - dz).clamp(0.4, 3.0);
                    }
                }
                super::super::assets::AssetsPanelAction::SelectAsset(ref opt) => {
                    self.assets_selected_asset = opt.clone();
                    self.assets_actions.push(action);
                }
                other => {
                    self.assets_actions.push(other);
                }
            }
        }

        result.consumed = consumed;
        Some(result)
    }
}