// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bridges [`ae_mcp::McpHost`] to [`AeEngine`] without introducing upward crate dependencies.

use crate::engine::AeEngine;
use crate::events::InputArbiter;
use ae_mcp::{
    McpEntityInfo, McpFloatingWindow, McpGpuHandles, McpHealthSnapshot, McpHost, McpLogEntry,
};
use irisui::core::{UiEvent, UiTree};
use serde_json::{Map, Value, json};
use std::sync::atomic::Ordering;
use winit::event::ElementState;
use winit::keyboard::KeyCode as WinitKeyCode;

impl McpHost for AeEngine {
    fn health_snapshot(&self) -> McpHealthSnapshot {
        let win_size = self.render_state.size;
        McpHealthSnapshot {
            physical_size: (win_size.width, win_size.height),
            scale_factor: self.ui.scale_factor(),
            mode: format!("{:?}", self.mode),
            selected_entity_bits: self.ui.selected_entity.map(|e| e.to_bits().get()),
            wants_keyboard_input: self.ui.wants_keyboard_input(),
        }
    }

    fn ui_tree(&self) -> &UiTree {
        self.ui.workbench.tree()
    }

    fn floating_windows(&self) -> Vec<McpFloatingWindow> {
        self.ui
            .workbench
            .dock_state()
            .floating_windows
            .iter()
            .map(|w| McpFloatingWindow {
                id: w.id,
                title: w.title.clone(),
                rect: w.rect,
                content_rect: w.default_content_rect(),
            })
            .collect()
    }

    fn logical_screen_size(&self) -> Option<[f32; 2]> {
        self.ui.workbench.screen_size().map(|s| [s.width, s.height])
    }

    fn dispatch_ui_event(&mut self, event: &UiEvent) -> bool {
        InputArbiter::dispatch_ui_event(self, event)
    }

    fn inject_shortcut_key(&mut self, key: WinitKeyCode, pressed: bool) {
        let state = if pressed {
            ElementState::Pressed
        } else {
            ElementState::Released
        };
        self.input.process_key_event(key, state);
    }

    fn invalidate_ui(&mut self, mark_all: bool) {
        if mark_all {
            self.ui.workbench.notifier.tag_all();
        }
        self.render_state.window.request_redraw();
    }

    fn request_redraw(&self) {
        self.render_state.window.request_redraw();
    }

    fn gpu_handles(&self) -> McpGpuHandles<'_> {
        McpGpuHandles {
            device: &self.render_state.device,
            queue: &self.render_state.queue,
            format: self.render_state.config.format,
            physical_size: (self.render_state.size.width, self.render_state.size.height),
        }
    }

    fn render_ui_to_view(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_size: (u32, u32),
        scale_factor: f32,
    ) {
        ae_editor_ui::ui::iris_bridge::icons::ensure_editor_icon_atlas(
            &mut self.ui.surface,
            &self.render_state.device,
            &self.render_state.queue,
        );
        let hovered_tag = self.ui.workbench.hovered_tag();
        self.ui.surface.render_to_view(
            &mut self.ui.workbench.tree,
            &mut self.ui.workbench.text_system,
            hovered_tag,
            irisui::IrisSurfaceRenderParams {
                device: &self.render_state.device,
                queue: &self.render_state.queue,
                encoder,
                target_view,
                physical_screen_size: physical_size,
                zoom_factor: scale_factor,
                focused_tag_override: self.ui.iris_overlay.focus_manager.focused_tag,
                extra_text_occluders: &[],
            },
        );
    }

    fn list_entities(&self) -> (Option<u64>, Vec<McpEntityInfo>) {
        let entities = self
            .ecs
            .world
            .iter()
            .map(|entity| McpEntityInfo {
                id_bits: entity.entity().to_bits().get(),
                name: entity.get::<&ae_core::ecs::Name>().map(|n| n.0.clone()),
            })
            .collect();
        (self.ui.selected_entity.map(|e| e.to_bits().get()), entities)
    }

    fn select_entity(&mut self, id_bits: Option<u64>, name: Option<&str>) -> Result<u64, String> {
        let entity = resolve_ecs_entity(self, id_bits, name)?;
        self.editor.selected_entities.clear();
        self.editor.selected_entities.push(entity);
        self.editor.selected_entities_set.clear();
        self.editor.selected_entities_set.insert(entity);
        self.ui.selected_entity = Some(entity);
        self.ui
            .workbench
            .notifier
            .tag_redraw(ae_editor_ui::ui::PanelId::Inspector);
        self.ui
            .workbench
            .notifier
            .tag_redraw(ae_editor_ui::ui::PanelId::Hierarchy);
        self.render_state.window.request_redraw();
        Ok(entity.to_bits().get())
    }

    fn inspect_entity(
        &self,
        id_bits: Option<u64>,
        name: Option<&str>,
    ) -> Result<(u64, Map<String, Value>), String> {
        let entity = resolve_ecs_entity(self, id_bits, name)?;
        let registry = ae_core::ecs::ComponentRegistry::global();
        let mut components = Map::new();
        for handler in registry.handlers() {
            if let Some(json_str) = handler.capture_debug_json(&self.ecs.world, entity) {
                let parsed =
                    serde_json::from_str::<Value>(&json_str).unwrap_or_else(|_| json!(json_str));
                components.insert(handler.type_name().to_string(), parsed);
            }
        }
        Ok((entity.to_bits().get(), components))
    }

    fn recent_logs(
        &self,
        limit: usize,
        level: Option<&str>,
    ) -> Result<(u64, Vec<McpLogEntry>), String> {
        let total_count = ae_editor::editor_logger::LOGGER
            .log_count
            .load(Ordering::Relaxed);
        let Ok(logs) = ae_editor::editor_logger::LOGGER.logs.try_lock() else {
            return Err("Logger buffer temporarily contended".to_string());
        };

        let mut entries: Vec<McpLogEntry> = logs
            .iter()
            .rev()
            .filter(|e| level.is_none_or(|lvl| format!("{}", e.level).eq_ignore_ascii_case(lvl)))
            .take(limit)
            .map(|e| McpLogEntry {
                timestamp: e.timestamp.clone(),
                level: format!("{}", e.level),
                target: e.target.clone(),
                message: e.msg.clone(),
            })
            .collect();
        entries.reverse();
        Ok((total_count, entries))
    }
}

fn resolve_ecs_entity(
    engine: &AeEngine,
    target_bits: Option<u64>,
    target_name: Option<&str>,
) -> Result<hecs::Entity, String> {
    if target_bits.is_none() && target_name.is_none() {
        return engine.ui.selected_entity.ok_or_else(|| {
            "No entity selected and neither 'id_bits' nor 'name' given".to_string()
        });
    }

    for entity in engine.ecs.world.iter() {
        let id = entity.entity();
        if target_bits.is_some_and(|b| b == id.to_bits().get()) {
            return Ok(id);
        }
        if let Some(wanted) = target_name
            && entity
                .get::<&ae_core::ecs::Name>()
                .is_some_and(|n| n.0 == wanted)
        {
            return Ok(id);
        }
    }

    Err("No matching entity found in ECS world".to_string())
}