// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Browser Overlay Action Dispatcher
//!
//! Handles file navigation, asset drag-and-drop instantiation, category selection,
//! and asset import dialog events.

use crate::ui::iris_bridge;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

impl EngineUi {
    /// Drains and processes queued actions from the Content / Asset Browser panel.
    ///
    /// When an asset drag finishes over the active 3D/2D Viewport canvas, computes
    /// the target world coordinates and dispatches the entity spawn action.
    pub fn process_assets_actions(
        &mut self,
        ui_actions: &mut Vec<EngineUiAction>,
        camera: &ae_renderer::camera::Camera,
        is_2d_mode: bool,
    ) {
        for action in self.iris_overlay.take_assets_actions() {
            match action {
                iris_bridge::AssetsPanelAction::NavigateFolder(path) => {
                    self.asset_browser.current_folder = path;
                }
                iris_bridge::AssetsPanelAction::SelectAsset(opt) => {
                    self.asset_browser.selected_asset = opt;
                }
                iris_bridge::AssetsPanelAction::SelectCategory(cat) => {
                    self.asset_browser.active_category = cat;
                }
                iris_bridge::AssetsPanelAction::SetViewMode(mode) => {
                    self.asset_browser.view_mode = mode;
                }
                iris_bridge::AssetsPanelAction::ToggleSidebar => {
                    self.asset_browser.sidebar_collapsed = !self.asset_browser.sidebar_collapsed;
                }
                iris_bridge::AssetsPanelAction::SearchInput(query) => {
                    self.asset_browser.search_query = query;
                }
                iris_bridge::AssetsPanelAction::ClearSearch => {
                    self.asset_browser.search_query.clear();
                }
                iris_bridge::AssetsPanelAction::OpenImportDialog => {
                    ui_actions.push(EngineUiAction::OpenModelDialog);
                }
                iris_bridge::AssetsPanelAction::RevealFolder(path) => {
                    let _ = crate::assets::file_ops::open_in_file_explorer(&path);
                }
                iris_bridge::AssetsPanelAction::CleanVram => {
                    ui_actions.push(EngineUiAction::GarbageCollect);
                }
                iris_bridge::AssetsPanelAction::OpenCreateSubfolder(parent) => {
                    self.asset_browser.new_folder_parent = Some(parent);
                    self.asset_browser.new_folder_name.clear();
                }
                iris_bridge::AssetsPanelAction::SpawnAsset(path, cat) => match cat {
                    crate::assets::types::AssetCategory::Models3D => {
                        ui_actions.push(EngineUiAction::SpawnModelPathAt(path, [0.0, 0.0, 0.0]));
                    }
                    crate::assets::types::AssetCategory::Textures2D => {
                        ui_actions.push(EngineUiAction::SpawnSpritePathAt(path, [0.0, 0.0, 0.0]));
                    }
                    crate::assets::types::AssetCategory::Scenes => {
                        if is_2d_mode && crate::assets::scanner::is_scene_file_3d(&path) {
                            log::warn!("Cannot load 3D scene in 2D mode: {:?}", path);
                        } else {
                            ui_actions.push(EngineUiAction::LoadSceneFromPath(path));
                        }
                    }
                    _ => {}
                },
                iris_bridge::AssetsPanelAction::InspectAsset(item) => {
                    self.iris_overlay.assets.preview_modal = Some(
                        crate::ui::iris_bridge::assets::types::AssetPreviewModalState {
                            item,
                            orbit_yaw: 0.0,
                            orbit_pitch: 0.3,
                            zoom_distance: 1.0,
                            show_wireframe: true,
                        },
                    );
                }
                iris_bridge::AssetsPanelAction::OpenRename(path, name, is_folder) => {
                    self.asset_browser.rename_state = Some(crate::assets::types::RenamingState {
                        target_path: path,
                        current_name: name,
                        is_folder,
                    });
                }
                iris_bridge::AssetsPanelAction::OpenDelete(path) => {
                    self.asset_browser.delete_confirmation = Some(path);
                }
                iris_bridge::AssetsPanelAction::CopyPath(path) => {
                    log::info!("Asset file path copied: {}", path.display());
                }
                iris_bridge::AssetsPanelAction::StartAssetDrag(item) => {
                    self.asset_browser.drag_payload =
                        Some(crate::assets::types::AssetDragPayload {
                            path: item.path,
                            name: item.name,
                            category: item.category,
                            model_handle: item.model_handle,
                            texture_handle: item.texture_handle,
                        });
                }
                iris_bridge::AssetsPanelAction::EndAssetDrag => {
                    if let Some(payload) = self.asset_browser.drag_payload.take() {
                        let cursor_pos = self.iris_overlay.cursor_pos();
                        if self.last_viewport_rect.contains_point(cursor_pos)
                            && self.last_viewport_rect.width > 20.0
                            && self.last_viewport_rect.height > 20.0
                        {
                            let world_pos = if !is_2d_mode {
                                if let Some(hit) =
                                    crate::assets::drag_drop::compute_ground_intersection(
                                        [cursor_pos.x, cursor_pos.y],
                                        self.last_viewport_rect,
                                        camera,
                                    )
                                {
                                    hit
                                } else {
                                    let forward = camera.target - camera.position;
                                    let len = (forward.x * forward.x + forward.z * forward.z)
                                        .sqrt()
                                        .max(0.001);
                                    [
                                        camera.position.x + (forward.x / len) * 3.0,
                                        0.0,
                                        camera.position.z + (forward.z / len) * 3.0,
                                    ]
                                }
                            } else {
                                let rel_x = cursor_pos.x
                                    - self.last_viewport_rect.x
                                    - self.last_viewport_rect.width * 0.5;
                                let rel_y = cursor_pos.y
                                    - self.last_viewport_rect.y
                                    - self.last_viewport_rect.height * 0.5;
                                [rel_x, -rel_y, 0.0]
                            };

                            log::info!(
                                "Asset '{}' dropped onto viewport at {:?}",
                                payload.name,
                                world_pos
                            );

                            match payload.category {
                                crate::assets::types::AssetCategory::Models3D => {
                                    if let Some(handle) = payload.model_handle {
                                        ui_actions
                                            .push(EngineUiAction::SpawnModelAt(handle, world_pos));
                                    } else {
                                        ui_actions.push(EngineUiAction::SpawnModelPathAt(
                                            payload.path,
                                            world_pos,
                                        ));
                                    }
                                }
                                crate::assets::types::AssetCategory::Textures2D => {
                                    if let Some(handle) = payload.texture_handle {
                                        ui_actions
                                            .push(EngineUiAction::SpawnSpriteAt(handle, world_pos));
                                    } else {
                                        ui_actions.push(EngineUiAction::SpawnSpritePathAt(
                                            payload.path,
                                            world_pos,
                                        ));
                                    }
                                }
                                crate::assets::types::AssetCategory::Scenes => {
                                    if is_2d_mode
                                        && crate::assets::scanner::is_scene_file_3d(&payload.path)
                                    {
                                        log::warn!(
                                            "Cannot load 3D scene in 2D mode: {:?}",
                                            payload.path
                                        );
                                    } else {
                                        ui_actions
                                            .push(EngineUiAction::LoadSceneFromPath(payload.path));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                iris_bridge::AssetsPanelAction::ToggleEngineContent => {
                    self.asset_browser.show_engine_content =
                        !self.asset_browser.show_engine_content;
                    self.iris_overlay.notifier.tag_all();
                }
                _ => {}
            }
        }
    }
}