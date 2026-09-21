// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Overlay Action Dispatcher
//!
//! Handles configuration mutations, graphics settings updates, and snapping toggles
//! emitted by the Preferences modal dialog panel.

use crate::ui::iris_bridge;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

/// Context bundle for mutating editor and graphics settings via Preferences actions.
pub struct PreferencesActionContext<'a> {
    /// Mutable reference to current engine graphics settings.
    pub graphics_settings: &'a mut ae_renderer::graphics_settings::GraphicsSettings,
    /// Mutable reference to viewport snapping settings.
    pub snapping: &'a mut ae_editor::snapping::SnapSettings,
    /// Mutable reference to editor session configuration.
    pub editor_config: &'a mut ae_editor::editor_state::EditorConfig,
    /// Mutable flag indicating whether live shader/environment updates are enabled.
    pub enable_live_updates: &'a mut bool,
    /// Flag set to true if graphics settings were modified.
    pub gs_changed: &'a mut bool,
    /// Flag set to true if snapping settings were modified.
    pub snap_changed: &'a mut bool,
    /// Flag set to true if editor config was modified.
    pub cfg_changed: &'a mut bool,
    /// Flag set to true if live updates state changed.
    pub live_changed: &'a mut bool,
    /// Outgoing UI action event queue.
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
                    self.iris_overlay.preferences.tab = t;
                }
                iris_bridge::PreferencesAction::ToggleDropdown(dd) => {
                    self.iris_overlay.preferences.dropdown = dd;
                }
                iris_bridge::PreferencesAction::SetUiScale(s) => {
                    self.ui_zoom_factor = s.clamp(0.6, 2.0);
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
                            self.ui_zoom_factor = scale_val.clamp(0.6, 2.0);
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
                    self.iris_overlay.preferences.scroll_y =
                        (self.iris_overlay.preferences.scroll_y + delta).max(0.0);
                }
                iris_bridge::PreferencesAction::ToggleSection(_) => {}
            }
        }
    }
}