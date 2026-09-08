// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native workspace geometry and external viewport resource preparation.

use crate::ui::panel_layout::PanelId;
use crate::ui::workbench::state::EngineUi;
use irisui::prelude::*;
use super::EditorUiRenderParams;

/// Stable host identity reused when the renderer replaces its resolved viewport color target.
pub(crate) const VIEWPORT_TEXTURE: ExternalTextureId = ExternalTextureId(0);

impl EngineUi {
    /// Updates native docking before any panel reads its geometry, then registers the D2 view.
    pub(crate) fn prepare_native_frame(&mut self, params: &EditorUiRenderParams<'_>) {
        self.platform.native_scale = params.window.scale_factor() as f32;
        let size = params.window.inner_size();
        let scale = self.pixels_per_point();
        let bounds = Rect::new(0.0, 26.0, size.width as f32 / scale, (size.height as f32 / scale - 48.0).max(0.0));
        self.dock_frame = crate::ui::docking::update_docking(
            &mut self.layout_state, &mut self.dock_session, &self.platform.pointer,
            bounds, self.iris_overlay.has_popup_or_modal(),
        );
        self.last_viewport_rect = self.dock_frame.panel(PanelId::Viewport).filter(|rect| rect.is_valid()).unwrap_or(Rect::ZERO);
        let renderer = &mut self.iris_overlay.renderer;
        if let Some(view) = params.viewport_texture_view {
            renderer.external_textures.set(params.device, &renderer.external_pipeline, VIEWPORT_TEXTURE, view);
        } else {
            renderer.external_textures.remove(VIEWPORT_TEXTURE);
        }
        if self.status_message.as_ref().is_some_and(|(_, time)| time.elapsed().as_secs_f32() > 5.0) {
            self.status_message = None;
        }
    }
}