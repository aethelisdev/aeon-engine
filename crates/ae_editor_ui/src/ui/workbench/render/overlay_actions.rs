// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Overlay Action Dispatchers
//!
//! Handles action events triggered by Iris UI overlays including Preferences,
//! Viewport HUD, Scene Hierarchy, and Performance Stats panels.

use crate::ui::iris_bridge;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

/// Context bundle for mutating editor and graphics settings via Preferences actions.
pub struct PreferencesActionContext<'a> {
    pub graphics_settings: &'a mut ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping: &'a mut ae_editor::snapping::SnapSettings,
    pub editor_config: &'a mut ae_editor::editor_state::EditorConfig,
    pub enable_live_updates: &'a mut bool,
    pub gs_changed: &'a mut bool,
    pub snap_changed: &'a mut bool,
    pub cfg_changed: &'a mut bool,
    pub live_changed: &'a mut bool,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
}

impl EngineUi {
    /// Dispatches all pending Preferences modal actions to editor state and settings.
    pub fn process_preferences_actions(&mut self, ctx: PreferencesActionContext<'_>) {
        while let Some(act) = self.pending_preferences_actions.pop() {
            match act {
                iris_bridge::PreferencesAction::Close => {
                    self.show_preferences = false;
                }
                iris_bridge::PreferencesAction::SelectTab(t) => {
                    self.preferences_tab = t;
                    self.iris_overlay.preferences_tab = t;
                }
                iris_bridge::PreferencesAction::ToggleDropdown(dd) => {
                    self.iris_overlay.preferences_dropdown = dd;
                }
                iris_bridge::PreferencesAction::SetUiScale(s) => {
                    ctx.ui_actions.push(EngineUiAction::SetUiScale(s));
                }
                iris_bridge::PreferencesAction::Toggle(toggle_id) => match toggle_id {
                    iris_bridge::PreferencesToggleId::ShadowsEnabled => {
                        ctx.graphics_settings.shadow_enabled =
                            !ctx.graphics_settings.shadow_enabled;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesToggleId::BloomEnabled => {
                        ctx.graphics_settings.bloom_enabled = !ctx.graphics_settings.bloom_enabled;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesToggleId::FogEnabled => {
                        ctx.graphics_settings.fog_enabled = !ctx.graphics_settings.fog_enabled;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesToggleId::LiveUpdatesEnabled => {
                        *ctx.enable_live_updates = !*ctx.enable_live_updates;
                        *ctx.live_changed = true;
                    }
                    iris_bridge::PreferencesToggleId::Module(m) => {
                        ctx.ui_actions.push(EngineUiAction::ToggleModule(m));
                    }
                },
                iris_bridge::PreferencesAction::SetSliderValue(slider_id, val) => match slider_id {
                    iris_bridge::PreferencesSliderId::ShadowBias => {
                        ctx.graphics_settings.shadow_bias = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::BloomIntensity => {
                        ctx.graphics_settings.bloom_intensity = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::SunPitch => {
                        ctx.graphics_settings.sun_pitch = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::SunYaw => {
                        ctx.graphics_settings.sun_yaw = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::AtmosphereDensity => {
                        ctx.graphics_settings.atmosphere_density = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::OzoneDensity => {
                        ctx.graphics_settings.ozone_density = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::SunDiscSize => {
                        ctx.graphics_settings.sun_disc_size = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::SunGlowStrength => {
                        ctx.graphics_settings.sun_glow_strength = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::CloudCoverage => {
                        ctx.graphics_settings.cloud_coverage = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::CloudDensity => {
                        ctx.graphics_settings.cloud_density = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::CloudSpeed => {
                        ctx.graphics_settings.cloud_speed = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::CloudEvolution => {
                        ctx.graphics_settings.cloud_evolution = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::CloudAltitude => {
                        ctx.graphics_settings.cloud_altitude = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::FogDistance => {
                        ctx.graphics_settings.fog_distance = val;
                        *ctx.gs_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::GridSize => {
                        ctx.snapping.grid_size = val;
                        *ctx.snap_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::UndoHistoryLimit => {
                        ctx.editor_config.max_undo_history = val as usize;
                        *ctx.cfg_changed = true;
                    }
                    iris_bridge::PreferencesSliderId::PhysicsFrequency => {
                        ctx.editor_config.physics_hz = val;
                        *ctx.cfg_changed = true;
                    }
                },
                iris_bridge::PreferencesAction::SelectDropdownItem(dd_id, idx) => match dd_id {
                    iris_bridge::PreferencesDropdownId::UiScale => {
                        if let Some(&(scale_val, _)) =
                            iris_bridge::preferences::tabs::general::UI_SCALES.get(idx)
                        {
                            ctx.ui_actions.push(EngineUiAction::SetUiScale(scale_val));
                        }
                    }
                    iris_bridge::PreferencesDropdownId::ShadowResolution => {
                        if let Some(&res) =
                            iris_bridge::preferences::tabs::graphics::SHADOW_RES_OPTIONS.get(idx)
                        {
                            ctx.graphics_settings.shadow_resolution = res;
                            *ctx.gs_changed = true;
                        }
                    }
                    iris_bridge::PreferencesDropdownId::ShadowCascades => {
                        if let Some(&(cascades, _)) =
                            iris_bridge::preferences::tabs::graphics::CASCADE_OPTIONS.get(idx)
                        {
                            ctx.graphics_settings.shadow_cascades = cascades;
                            *ctx.gs_changed = true;
                        }
                    }
                    iris_bridge::PreferencesDropdownId::ShadowPcf => {
                        if let Some(&pcf) =
                            iris_bridge::preferences::tabs::graphics::PCF_OPTIONS.get(idx)
                        {
                            ctx.graphics_settings.shadow_pcf = pcf;
                            *ctx.gs_changed = true;
                        }
                    }
                    iris_bridge::PreferencesDropdownId::FpsLimit => {
                        if let Some(&fps) =
                            iris_bridge::preferences::tabs::graphics::FPS_OPTIONS.get(idx)
                        {
                            ctx.graphics_settings.fps_limit = fps;
                            *ctx.gs_changed = true;
                        }
                    }
                    iris_bridge::PreferencesDropdownId::MsaaSamples => {
                        if let Some(&(samples, _)) =
                            iris_bridge::preferences::tabs::graphics::MSAA_OPTIONS.get(idx)
                        {
                            ctx.graphics_settings.msaa_samples = samples;
                            *ctx.gs_changed = true;
                        }
                    }
                    iris_bridge::PreferencesDropdownId::SkyQuality => {
                        if let Some(&sky) =
                            iris_bridge::preferences::tabs::graphics::SKY_OPTIONS.get(idx)
                        {
                            ctx.graphics_settings.sky_quality = sky;
                            *ctx.gs_changed = true;
                        }
                    }
                    iris_bridge::PreferencesDropdownId::SnapMode => {
                        if let Some(&(mode, _)) =
                            iris_bridge::preferences::tabs::SNAP_MODE_OPTIONS.get(idx)
                        {
                            ctx.snapping.mode = mode;
                            *ctx.snap_changed = true;
                        }
                    }
                },
                iris_bridge::PreferencesAction::Scroll(delta) => {
                    self.iris_overlay.preferences_scroll_y =
                        (self.iris_overlay.preferences_scroll_y + delta).max(0.0);
                }
                iris_bridge::PreferencesAction::ToggleSection(_) => {}
            }
        }
    }

    /// Dispatches all pending Viewport HUD toolbar actions.
    pub fn process_viewport_hud_actions(
        &mut self,
        snapping: &mut ae_editor::snapping::SnapSettings,
        snap_changed: &mut bool,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        for action in self.iris_overlay.take_viewport_hud_actions() {
            match action {
                iris_bridge::ViewportHudAction::SetCameraMode(cmode) => {
                    ui_actions.push(EngineUiAction::SetCameraMode(cmode));
                }
                iris_bridge::ViewportHudAction::SetCameraTransform {
                    pitch,
                    yaw,
                    position,
                    mode,
                } => {
                    ui_actions.push(EngineUiAction::SetCameraTransform {
                        pitch,
                        yaw,
                        position,
                        mode,
                    });
                }
                iris_bridge::ViewportHudAction::ToggleWireframe => {
                    self.wireframe_enabled = !self.wireframe_enabled;
                }
                iris_bridge::ViewportHudAction::SetGizmoMode(gmode) => {
                    self.gizmo_mode = gmode;
                }
                iris_bridge::ViewportHudAction::ToggleGizmoSpace => {
                    self.gizmo_space = self.gizmo_space.toggle();
                }
                iris_bridge::ViewportHudAction::ToggleSnapping => {
                    snapping.mode = match snapping.mode {
                        ae_editor::snapping::SnapMode::Off => ae_editor::snapping::SnapMode::Toggle,
                        _ => ae_editor::snapping::SnapMode::Off,
                    };
                    *snap_changed = true;
                }
                iris_bridge::ViewportHudAction::SelectEntity(ent) => {
                    ui_actions.push(EngineUiAction::SelectEntity(Some(ent)));
                }
                iris_bridge::ViewportHudAction::ToggleDropdown(dd) => {
                    self.iris_overlay.viewport_hud_dropdown = dd;
                }
            }
        }
    }

    /// Dispatches all pending Stats panel toggle actions.
    pub fn process_stats_actions(&mut self) {
        for action in self.iris_overlay.take_stats_actions() {
            match action {
                iris_bridge::StatsPanelAction::ToggleWireframe => {
                    self.wireframe_enabled = !self.wireframe_enabled;
                }
                iris_bridge::StatsPanelAction::ToggleGrid => {
                    self.grid_enabled = !self.grid_enabled;
                }
                iris_bridge::StatsPanelAction::Scroll(_) => {}
            }
        }
    }

    /// Dispatches all pending Scene Hierarchy panel action events.
    pub fn process_hierarchy_actions(&mut self, ui_actions: &mut Vec<EngineUiAction>) {
        for action in self.iris_overlay.take_hierarchy_actions() {
            match action {
                iris_bridge::HierarchyAction::SelectEntity(ent) => {
                    ui_actions.push(EngineUiAction::SelectEntity(ent));
                }
                iris_bridge::HierarchyAction::ToggleVisibility(ent) => {
                    ui_actions.push(EngineUiAction::ToggleVisibility(ent));
                }
                iris_bridge::HierarchyAction::DeleteSelected => {
                    ui_actions.push(EngineUiAction::DeleteSelected);
                }
                iris_bridge::HierarchyAction::SpawnShape(shape) => {
                    ui_actions.push(EngineUiAction::SpawnShape(shape));
                }
                iris_bridge::HierarchyAction::SpawnUiElement(elem) => {
                    ui_actions.push(EngineUiAction::SpawnUiElement(elem));
                }
                iris_bridge::HierarchyAction::OpenModelDialog => {
                    ui_actions.push(EngineUiAction::OpenModelDialog);
                }
                iris_bridge::HierarchyAction::OpenLoadPrefabDialog => {
                    ui_actions.push(EngineUiAction::OpenLoadPrefabDialog);
                }
                iris_bridge::HierarchyAction::InstantiatePrefab(path) => {
                    ui_actions.push(EngineUiAction::InstantiatePrefab(path));
                }
                iris_bridge::HierarchyAction::SpawnPhase1TestSandbox => {
                    ui_actions.push(EngineUiAction::SpawnPhase1TestSandbox);
                }
                iris_bridge::HierarchyAction::StressTest(n) => {
                    ui_actions.push(EngineUiAction::StressTest(n));
                }
                iris_bridge::HierarchyAction::AaaOpenWorldTest => {
                    ui_actions.push(EngineUiAction::AaaOpenWorldTest);
                }
                iris_bridge::HierarchyAction::Explode => {
                    ui_actions.push(EngineUiAction::Explode);
                }
                _ => {}
            }
        }
    }

    /// Drains and processes queued actions from the Content / Asset Browser panel.
    pub fn process_assets_actions(&mut self, ui_actions: &mut Vec<EngineUiAction>) {
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
                    let _ = crate::ui::panels::assets::file_ops::open_in_file_explorer(&path);
                }
                iris_bridge::AssetsPanelAction::CleanVram => {
                    ui_actions.push(EngineUiAction::GarbageCollect);
                }
                iris_bridge::AssetsPanelAction::OpenCreateSubfolder(parent) => {
                    self.asset_browser.new_folder_parent = Some(parent);
                    self.asset_browser.new_folder_name.clear();
                }
                iris_bridge::AssetsPanelAction::SpawnAsset(path, cat) => match cat {
                    crate::ui::panels::assets::types::AssetCategory::Models3D => {
                        ui_actions.push(EngineUiAction::SpawnModelPathAt(path, [0.0, 0.0, 0.0]));
                    }
                    crate::ui::panels::assets::types::AssetCategory::Textures2D => {
                        ui_actions.push(EngineUiAction::SpawnSpritePathAt(path, [0.0, 0.0, 0.0]));
                    }
                    crate::ui::panels::assets::types::AssetCategory::Scenes => {
                        ui_actions.push(EngineUiAction::LoadSceneFromPath(path));
                    }
                    _ => {}
                },
                iris_bridge::AssetsPanelAction::InspectAsset(item) => {
                    self.asset_browser.preview_modal =
                        Some(crate::ui::panels::assets::types::PreviewModalState {
                            item,
                            orbit_yaw: 0.0,
                            orbit_pitch: 0.3,
                            zoom_distance: 1.0,
                            show_wireframe: true,
                            channel_mask: [true, true, true, true],
                            wgsl_source: None,
                        });
                }
                iris_bridge::AssetsPanelAction::OpenRename(path, name, is_folder) => {
                    self.asset_browser.rename_state =
                        Some(crate::ui::panels::assets::types::RenamingState {
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
                        Some(crate::ui::panels::assets::types::AssetDragPayload {
                            path: item.path,
                            name: item.name,
                            category: item.category,
                            model_handle: item.model_handle,
                            texture_handle: item.texture_handle,
                        });
                }
                iris_bridge::AssetsPanelAction::EndAssetDrag => {
                    self.asset_browser.drag_payload = None;
                }
                _ => {}
            }
        }
    }

    /// Drains and processes all queued actions from the Animation Timeline Studio panel.
    pub fn process_timeline_actions(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let Some(entity) = self.selected_entity else {
            self.iris_overlay.timeline_actions.clear();
            return;
        };

        for action in self.iris_overlay.take_timeline_actions() {
            match action {
                iris_bridge::TimelineAction::TogglePlayPause => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.state = if updated.state == ae_animation::AnimationState::Playing {
                            ae_animation::AnimationState::Paused
                        } else {
                            ae_animation::AnimationState::Playing
                        };
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::Stop => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.state = ae_animation::AnimationState::Stopped;
                        updated.current_time = 0.0;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::StepFrame(delta_frames) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        let duration = updated
                            .current_clip
                            .as_ref()
                            .map_or(1.0, |c| c.duration.max(0.1));
                        let frame_step = 1.0 / 30.0;
                        updated.current_time = (updated.current_time
                            + delta_frames as f32 * frame_step)
                            .clamp(0.0, duration);
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::ToggleLoop => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.looping = !updated.looping;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::SetSpeed(speed) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.speed = speed;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::ScrubTo(time) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        let duration = updated
                            .current_clip
                            .as_ref()
                            .map_or(1.0, |c| c.duration.max(0.1));
                        updated.current_time = time.clamp(0.0, duration);
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::AddAnimationPlayer(ent) => {
                    ui_actions.push(EngineUiAction::AddComponent(ent, "AnimationPlayer"));
                }
            }
        }
    }

    /// Dispatches all pending Material & Surface Studio panel actions.
    pub fn process_material_actions(&mut self, ui_actions: &mut Vec<EngineUiAction>) {
        for action in self.iris_overlay.take_material_actions() {
            match action {
                iris_bridge::MaterialAction::AssignTextureToEntity(ent, path) => {
                    ui_actions.push(EngineUiAction::AssignTextureToEntity(ent, path));
                }
                iris_bridge::MaterialAction::RemoveTextureFromEntity(ent) => {
                    ui_actions.push(EngineUiAction::RemoveTextureFromEntity(ent));
                }
                iris_bridge::MaterialAction::SetModelSubmeshAlphaMode(model_id, idx, mode) => {
                    ui_actions.push(EngineUiAction::SetModelSubmeshAlphaMode(
                        model_id, idx, mode,
                    ));
                }
                iris_bridge::MaterialAction::SetModelSubmeshTexture(model_id, idx, path) => {
                    ui_actions.push(EngineUiAction::SetModelSubmeshTexture(model_id, idx, path));
                }
                iris_bridge::MaterialAction::PickAndAssignEntityTexture(ent) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                        .pick_file()
                    {
                        ui_actions.push(EngineUiAction::AssignTextureToEntity(
                            ent,
                            path.to_string_lossy().to_string(),
                        ));
                    }
                }
                iris_bridge::MaterialAction::PickAndSetSubmeshTexture(model_id, idx) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                        .pick_file()
                    {
                        ui_actions.push(EngineUiAction::SetModelSubmeshTexture(
                            model_id,
                            idx,
                            path.to_string_lossy().to_string(),
                        ));
                    }
                }
                iris_bridge::MaterialAction::AddColorComponent(ent) => {
                    ui_actions.push(EngineUiAction::AddComponent(ent, "Color"));
                }
                iris_bridge::MaterialAction::Scroll(_) => {}
            }
        }
    }
}