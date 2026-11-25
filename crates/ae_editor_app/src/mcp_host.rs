// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bridges [`ae_mcp::McpHost`] to [`EditorSession`] without introducing upward dependencies.

use crate::events::InputArbiter;
use crate::session::EditorSession;
use ae_mcp::{
    McpEntityInfo, McpFloatingWindow, McpGpuHandles, McpHealthSnapshot, McpHost, McpLogEntry,
};
use irisui::core::{Rect, UiEvent};
use irisui::workbench::{UiInspectionSummary, UiTreeDump};
use serde_json::{Map, Value, json};
use std::sync::atomic::Ordering;
use winit::event::ElementState;
use winit::keyboard::KeyCode as WinitKeyCode;

impl McpHost for EditorSession {
    fn health_snapshot(&self) -> McpHealthSnapshot {
        let win_size = self.render.render_state.size;
        McpHealthSnapshot {
            physical_size: (win_size.width, win_size.height),
            scale_factor: self.ui.scale_factor(),
            mode: format!("{:?}", self.window.mode),
            selected_entity_bits: self.editor.primary_selected().map(|e| e.to_bits().get()),
            wants_keyboard_input: self.ui.wants_keyboard_input(),
        }
    }

    fn ui_inspection_summary(&self) -> UiInspectionSummary {
        self.ui.ui_inspection_summary()
    }

    fn inspect_ui_nodes(
        &self,
        tag_filter: Option<u64>,
        name_filter: Option<&str>,
        only_popups: bool,
    ) -> UiTreeDump {
        self.ui
            .inspect_ui_nodes(tag_filter, name_filter, only_popups)
    }

    fn resolve_tag_rect(&self, tag: u64) -> Option<Rect> {
        self.ui.resolve_tag_rect(tag)
    }

    fn resolve_widget_rect(&self, name: &str, index: usize) -> Option<Rect> {
        self.ui.resolve_widget_rect(name, index)
    }

    fn floating_windows(&self) -> Vec<McpFloatingWindow> {
        self.ui
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
        self.ui.logical_screen_size()
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
        self.core.input.process_key_event(key, state);
    }

    fn invalidate_ui(&mut self, mark_all: bool) {
        if mark_all {
            self.ui.invalidate_all();
        }
        self.render.render_state.window.request_redraw();
    }

    fn request_redraw(&self) {
        self.render.render_state.window.request_redraw();
    }

    fn gpu_handles(&self) -> McpGpuHandles<'_> {
        McpGpuHandles {
            device: &self.render.render_state.device,
            queue: &self.render.render_state.queue,
            format: self.render.render_state.config.format,
            physical_size: (
                self.render.render_state.size.width,
                self.render.render_state.size.height,
            ),
        }
    }

    fn render_ui_to_view(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_size: (u32, u32),
        scale_factor: f32,
    ) {
        self.ui.render_offscreen_snapshot(
            &self.render.render_state.device,
            &self.render.render_state.queue,
            encoder,
            target_view,
            physical_size,
            scale_factor,
        );
    }

    fn list_entities(&self) -> (Option<u64>, Vec<McpEntityInfo>) {
        let entities = self
            .core
            .world
            .iter()
            .map(|entity| McpEntityInfo {
                id_bits: entity.entity().to_bits().get(),
                name: entity.get_cloned::<ae_plugin_api::Name>().map(|n| n.0),
            })
            .collect();
        (
            self.editor.primary_selected().map(|e| e.to_bits().get()),
            entities,
        )
    }

    fn select_entity(&mut self, id_bits: Option<u64>, name: Option<&str>) -> Result<u64, String> {
        let entity = resolve_ecs_entity(self, id_bits, name)?;
        self.editor.select_single(Some(entity));
        self.ui.refresh_selection_panels();
        self.render.render_state.window.request_redraw();
        Ok(entity.to_bits().get())
    }

    fn inspect_entity(
        &self,
        id_bits: Option<u64>,
        name: Option<&str>,
    ) -> Result<(u64, Map<String, Value>), String> {
        let entity = resolve_ecs_entity(self, id_bits, name)?;
        let registry = ae_plugin_api::engine_component_registry();
        let mut components = Map::new();
        for handler in registry.handlers() {
            if let Some(json_str) = handler.capture_debug_json(&self.core.world, entity) {
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
    session: &EditorSession,
    target_bits: Option<u64>,
    target_name: Option<&str>,
) -> Result<ae_ecs::Entity, String> {
    if target_bits.is_none() && target_name.is_none() {
        return session.editor.primary_selected().ok_or_else(|| {
            "No entity selected and neither 'id_bits' nor 'name' given".to_string()
        });
    }

    for entity in session.core.world.iter() {
        let id = entity.entity();
        if target_bits.is_some_and(|b| b == id.to_bits().get()) {
            return Ok(id);
        }
        if let Some(wanted) = target_name
            && entity.inspect::<ae_plugin_api::Name, _>(|n| n.0 == wanted) == Some(true)
        {
            return Ok(id);
        }
    }

    Err("No matching entity found in ECS world".to_string())
}