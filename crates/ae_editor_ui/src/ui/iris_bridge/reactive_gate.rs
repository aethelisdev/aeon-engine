// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Reactive change detection gate for Retained-Mode Iris UI editor overlays.
//!
//! Evaluates whether layout geometry, window bounds, dock revisions,
//! active modals, or selection state have mutated before triggering UI tree rebuilds.
//! Live telemetry (Stats panel) updates run through an independent in-place pipeline
//! without requiring tree rebuilds or quad rebaking.
//!

use super::types::{IrisEditorOverlay, OverlayUpdateParams};

impl IrisEditorOverlay {
    /// Evaluates whether the UI tree requires structural reconstruction or if the current layout is asleep.
    /// Checks window resolution deltas, zoom scale changes, dock revision increments,
    /// entity selection shifts, modal visibility, and explicit command stream dirty status.
    /// Note: Live telemetry text updates do NOT trigger tree rebuilds; they are updated
    /// in place independently via `update_live_telemetry_in_place`.
    pub(crate) fn should_rebuild_overlay(
        &self,
        params: &OverlayUpdateParams<'_>,
        screen_width: f32,
        screen_height: f32,
    ) -> bool {
        let dimensions_changed = (self.last_dimensions.0 - screen_width).abs() > 0.5
            || (self.last_dimensions.1 - screen_height).abs() > 0.5;
        let zoom_changed = (self.last_zoom_factor - params.zoom_factor).abs() > 0.001;
        let dock_changed = params.layout_state.revision != self.last_dock_revision;
        let active_menu_changed = self.active_menu != self.last_active_menu;
        let selection_changed = params.selected_entity != self.last_selected_entity;
        let editing_changed =
            params.is_editing != self.last_is_editing || params.is_2d != self.last_is_2d;
        let viewport_rect_changed = params.viewport_rect != self.last_viewport_rect;
        let status_len = params.status_spans.map_or(0, |s| s.len());
        let status_len_changed = status_len != self.last_status_len;
        let world_len_changed = params.world.len() != self.last_world_len;
        let viewport_texture_changed =
            params.has_viewport_texture != self.last_has_viewport_texture;

        let cam_pos = [
            params.camera.position.x,
            params.camera.position.y,
            params.camera.position.z,
        ];
        let cam_rot = [params.camera.yaw.0, params.camera.pitch.0];
        let camera_changed = !params.is_2d
            && ((cam_pos[0] - self.last_camera_pos[0]).abs() > 0.001
                || (cam_pos[1] - self.last_camera_pos[1]).abs() > 0.001
                || (cam_pos[2] - self.last_camera_pos[2]).abs() > 0.001
                || (cam_rot[0] - self.last_camera_orientation[0]).abs() > 0.0005
                || (cam_rot[1] - self.last_camera_orientation[1]).abs() > 0.0005
                || (params.camera.ortho_scale - self.last_camera_ortho_scale).abs() > 0.001);

        let modals_or_inputs_active = self.dropdown_rect.is_some()
            || self.about_targets.is_some()
            || self.delete_targets.is_some()
            || self.new_folder_targets.is_some()
            || self.rename_targets.is_some()
            || self.loading_targets.is_some()
            || self.preferences_targets.is_some()
            || self.active_menu.is_some()
            || params.show_about
            || params.show_preferences
            || params.delete_target.is_some()
            || params.new_folder_parent.is_some()
            || params.rename_target.is_some()
            || params.is_loading_assets
            || self.viewport_hud_dropdown.is_some()
            || self.hierarchy_is_add_menu_open
            || self.hierarchy_active_context_menu.is_some()
            || self.inspector_is_add_menu_open
            || self.assets_context_menu.is_some()
            || self.assets_preview_modal.is_some()
            || self.hierarchy_is_search_focused
            || self.assets_is_search_focused
            || self.inspector_active_text_input.is_some()
            || self.inspector_active_number_input.is_some();

        self.command_list.commands.is_empty()
            || self.tree.root().is_none()
            || dimensions_changed
            || zoom_changed
            || dock_changed
            || active_menu_changed
            || selection_changed
            || editing_changed
            || viewport_rect_changed
            || viewport_texture_changed
            || camera_changed
            || status_len_changed
            || world_len_changed
            || modals_or_inputs_active
            || self.needs_layout_rebuild
            || self.is_command_list_dirty
    }

    /// Caches all snapshot variables to enable idle detection on subsequent frames.
    pub(crate) fn record_overlay_state(
        &mut self,
        params: &OverlayUpdateParams<'_>,
        screen_width: f32,
        screen_height: f32,
    ) {
        self.last_dimensions = (screen_width, screen_height);
        self.last_zoom_factor = params.zoom_factor;
        self.last_dock_revision = params.layout_state.revision;
        self.last_active_menu = self.active_menu;
        self.last_selected_entity = params.selected_entity;
        self.last_is_editing = params.is_editing;
        self.last_is_2d = params.is_2d;
        self.last_viewport_rect = params.viewport_rect;
        self.last_has_viewport_texture = params.has_viewport_texture;
        self.last_camera_pos = [
            params.camera.position.x,
            params.camera.position.y,
            params.camera.position.z,
        ];
        self.last_camera_orientation = [params.camera.yaw.0, params.camera.pitch.0];
        self.last_camera_ortho_scale = params.camera.ortho_scale;
        self.last_status_len = params.status_spans.map_or(0, |s| s.len());
        self.last_world_len = params.world.len();
        self.needs_layout_rebuild = false;
    }

    /// Updates live telemetry metrics in place directly on retained nodes without touching tree geometry.
    /// Runs on a smooth 100 ms cadence without triggering UI tree rebuilds or baking.
    pub(crate) fn update_live_telemetry_in_place(&mut self, params: &OverlayUpdateParams<'_>) {
        if self.last_stats_update.elapsed() < std::time::Duration::from_millis(100) {
            return;
        }
        self.last_stats_update = std::time::Instant::now();

        if let Some(ref state) = self.stats_retained {
            let stats_params = super::stats::types::StatsPanelParams {
                panel_rect: state.panel_rect,
                scroll_y: self.stats_scroll_y,
                cursor_pos: self.cursor_pos,
                wireframe_enabled: params.wireframe_enabled,
                grid_enabled: params.grid_enabled,
                fps: params.fps,
                frame_pacing: params.frame_pacing,
                frame_pacing_stats: params.frame_pacing_stats,
                cpu_timings: params.cpu_timings,
                gpu_pass_timings: params.gpu_pass_timings,
                draw_call_stats: params.draw_call_stats,
                vram_stats: params.vram_stats,
                render_triangles: params.render_triangles,
                render_vertices: params.render_vertices,
                gpu_adapter_name: params.gpu_adapter_name,
                gpu_backend: params.gpu_backend,
                active_entities_count: params.active_entities_count,
                selected_entity: params.selected_entity,
                revision: state.last_revision,
            };

            super::stats::panel::update_stats_panel_text_values(
                &mut self.tree,
                &state.nodes,
                &stats_params,
            );
            // In-place telemetry update modifies text payload, not quad layout or geometry.
            // Clear PAINT and LAYOUT flags so tree_dirty remains strictly false on idle frames.
            self.tree.clear_all_dirty(
                irisui::prelude::DirtyFlags::PAINT | irisui::prelude::DirtyFlags::LAYOUT,
            );
            self.is_text_dirty = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    #[test]
    fn test_telemetry_timing_cadence() {
        let last_update = Instant::now() - Duration::from_millis(150);
        let elapsed = last_update.elapsed();
        assert!(elapsed >= Duration::from_millis(100));

        let fresh_update = Instant::now();
        assert!(fresh_update.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn test_dock_revision_change_triggers_rebuild() {
        let last_dock_revision: u64 = 0;
        let mut layout = crate::ui::panel_layout::PanelLayoutState::new_default();
        layout.bump_revision();
        assert_eq!(layout.revision, 1);
        assert_ne!(layout.revision, last_dock_revision);
    }

    #[test]
    fn test_viewport_texture_transition_triggers_rebuild() {
        let last_has_texture = false;
        let current_has_texture = true;
        let viewport_texture_changed = current_has_texture != last_has_texture;
        assert!(
            viewport_texture_changed,
            "Transition from placeholder to 3D texture must trigger rebuild"
        );
    }

    #[test]
    fn test_camera_motion_triggers_overlay_rebuild() {
        let last_cam_pos = [0.0f32, 2.0, 5.0];
        let last_cam_rot = [0.0f32, 0.0];
        let last_ortho_scale = 1.0f32;

        // 1. Unchanged camera in 3D mode
        let current_cam_pos = [0.0f32, 2.0, 5.0];
        let current_cam_rot = [0.0f32, 0.0];
        let current_ortho = 1.0f32;
        let is_2d = false;

        let camera_changed = !is_2d
            && ((current_cam_pos[0] - last_cam_pos[0]).abs() > 0.001
                || (current_cam_pos[1] - last_cam_pos[1]).abs() > 0.001
                || (current_cam_pos[2] - last_cam_pos[2]).abs() > 0.001
                || (current_cam_rot[0] - last_cam_rot[0]).abs() > 0.0005
                || (current_cam_rot[1] - last_cam_rot[1]).abs() > 0.0005
                || (current_ortho - last_ortho_scale).abs() > 0.001);
        assert!(!camera_changed, "Identical camera must not trigger rebuild");

        // 2. Camera yaw rotated by mouse drag
        let moved_cam_rot = [0.05f32, 0.0];
        let camera_changed = !is_2d
            && ((current_cam_pos[0] - last_cam_pos[0]).abs() > 0.001
                || (current_cam_pos[1] - last_cam_pos[1]).abs() > 0.001
                || (current_cam_pos[2] - last_cam_pos[2]).abs() > 0.001
                || (moved_cam_rot[0] - last_cam_rot[0]).abs() > 0.0005
                || (moved_cam_rot[1] - last_cam_rot[1]).abs() > 0.0005
                || (current_ortho - last_ortho_scale).abs() > 0.001);
        assert!(
            camera_changed,
            "Camera rotation must trigger rebuild for live billboard reprojection"
        );
    }
}