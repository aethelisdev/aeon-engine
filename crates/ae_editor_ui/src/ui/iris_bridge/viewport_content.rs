// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport image, game UI, and editor HUD composition within native Iris panel layers.

use super::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::docking_render::{rect_node, text_node};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds viewport content under the owning docked or floating panel, preserving z order.
    pub(crate) fn build_viewport_content(&mut self, parent: WidgetId, params: &OverlayUpdateParams<'_>) {
        let rect = params.viewport_rect;
        if !rect.is_valid() { return; }
        let root = rect_node(&mut self.tree, parent, rect, Style::new().background(Color::BLACK));
        if let Some(node) = self.tree.get_mut(root) { node.style.clip_children = true; }
        if params.has_viewport_texture {
            let image = rect_node(&mut self.tree, root, rect, Style::new());
            if let Some(node) = self.tree.get_mut(image) {
                node.external_texture = Some(crate::ui::workbench::render::native_pass::VIEWPORT_TEXTURE);
            }
        } else {
            text_node(&mut self.tree, root, rect, "Rendering viewport...", 14.0, Color::WHITE);
        }
        if !params.is_editing && params.enabled_modules.contains(&ae_core::modules::EngineModule::Render) {
            let mouse = rect.contains_point(self.cursor_pos).then_some([self.cursor_pos.x - rect.x, self.cursor_pos.y - rect.y]);
            let commands = ae_core::ui::UiLayoutResolver::resolve_draw_commands(params.world, rect.width, rect.height, mouse, params.pointer_clicked);
            crate::ui::viewport_hud::render_ui_draw_commands(&mut self.tree, root, rect, &commands);
        }
        let mut targets = super::viewport_hud::ViewportHudTargets::default();
        super::viewport_hud::build_viewport_hud(&mut self.tree, root, &super::viewport_hud::ViewportHudParams {
            viewport_rect: rect, camera: params.camera, wireframe_enabled: params.wireframe_enabled,
            gizmo_mode: params.gizmo_mode, gizmo_space: params.gizmo_space, snapping: params.snapping_settings,
            cursor_pos: self.cursor_pos, active_dropdown: self.viewport_hud_dropdown,
            selected_entity: params.selected_entity, world: params.world, is_editing: params.is_editing,
        }, &mut targets);
        self.viewport_hud_targets = Some(targets);
    }
}