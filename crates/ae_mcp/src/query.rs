// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Read-only inspection and scene/log query handlers operating against [`crate::McpHost`].

use crate::host::McpHost;
use crate::overlay::InspectVisualOverlay;
use crate::protocol::{InspectResponse, parse_u64_json_value};
use crate::screenshot::capture_ui_screenshot;
use irisui::core::{Rect, UiLayer};
use serde_json::{Value, json};
use std::path::Path;

fn rect_json(r: Rect) -> Value {
    json!({
        "x": r.x,
        "y": r.y,
        "width": r.width,
        "height": r.height,
        "right": r.right(),
        "bottom": r.bottom(),
    })
}

/// Builds the live health telemetry JSON response.
pub fn handle_health<H: McpHost + ?Sized>(host: &H, drag_in_progress: bool) -> InspectResponse {
    let snap = host.health_snapshot();
    let zoom = if snap.scale_factor > 0.0 {
        snap.scale_factor
    } else {
        1.0
    };
    let tree = host.ui_tree();
    InspectResponse::ok(json!({
        "status": "running",
        "physical_size": [snap.physical_size.0, snap.physical_size.1],
        "logical_size": [
            snap.physical_size.0 as f32 / zoom,
            snap.physical_size.1 as f32 / zoom,
        ],
        "scale_factor": snap.scale_factor,
        "mode": snap.mode,
        "selected_entity_bits": snap.selected_entity_bits,
        "node_count": tree.len(),
        "popup_active": tree.has_active_popup_nodes(),
        "active_combobox_tag": tree.active_combobox_tag.map(|t| format!("0x{t:016x}")),
        "wants_keyboard_input": snap.wants_keyboard_input,
        "drag_in_progress": drag_in_progress,
    }))
}

/// Serializes active `UiTree` nodes, tag bounding caches, popups, and floating dock windows.
pub fn handle_ui_tree_dump<H: McpHost + ?Sized>(host: &H, params: &Value) -> InspectResponse {
    let name_filter = params.get("name_filter").and_then(Value::as_str);
    let only_popups = params
        .get("only_popups")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let tag_filter = params.get("tag").and_then(parse_u64_json_value);

    let tree = host.ui_tree();
    let mut nodes = Vec::new();
    for (id, node) in tree.iter() {
        if only_popups && node.layer < UiLayer::Floating {
            continue;
        }
        if let Some(target_tag) = tag_filter
            && node.tag != target_tag
        {
            continue;
        }
        if let Some(substr) = name_filter {
            let name_match = node.name.as_deref().is_some_and(|n| n.contains(substr));
            let text_match = node.text.as_deref().is_some_and(|t| t.contains(substr));
            if !name_match && !text_match {
                continue;
            }
        }
        let parent_name = node
            .parent
            .and_then(|pid| tree.get(pid))
            .and_then(|p| p.name.as_deref());
        nodes.push(json!({
            "id": format!("{id:?}"),
            "name": node.name,
            "parent_name": parent_name,
            "tag": node.tag,
            "tag_hex": format!("0x{:016x}", node.tag),
            "role": format!("{:?}", node.role),
            "layer": format!("{:?}", node.layer),
            "computed_rect": rect_json(node.computed_rect),
            "position": format!("{:?}", node.style.position),
            "inset_left": node.style.inset_left,
            "inset_top": node.style.inset_top,
            "interactive": node.interactive,
            "text": node.text,
        }));
    }

    let tag_rects: Vec<Value> = tree
        .tag_rects
        .iter()
        .filter(|(tag, _)| tag_filter.is_none_or(|t| t == **tag))
        .map(|(tag, rect)| {
            json!({
                "tag": tag,
                "tag_hex": format!("0x{tag:016x}"),
                "rect": rect_json(*rect),
            })
        })
        .collect();

    let active_popup_tags: Vec<String> = tree
        .active_popup_tags
        .iter()
        .map(|t| format!("0x{t:016x}"))
        .collect();

    let floating_windows: Vec<Value> = host
        .floating_windows()
        .into_iter()
        .map(|w| {
            json!({
                "id": w.id,
                "title": w.title,
                "rect": rect_json(w.rect),
                "content_rect": rect_json(w.content_rect),
            })
        })
        .collect();

    InspectResponse::ok(json!({
        "screen_size": host.logical_screen_size(),
        "active_popup_tags": active_popup_tags,
        "active_combobox_tag": tree.active_combobox_tag.map(|t| format!("0x{t:016x}")),
        "floating_windows": floating_windows,
        "node_count": nodes.len(),
        "nodes": nodes,
        "tag_rects": tag_rects,
    }))
}

/// Lists all entities reported by `host`.
pub fn handle_list_entities<H: McpHost + ?Sized>(host: &H) -> InspectResponse {
    let (selected, list) = host.list_entities();
    let entities: Vec<Value> = list
        .into_iter()
        .map(|e| json!({ "id_bits": e.id_bits, "name": e.name }))
        .collect();
    InspectResponse::ok(json!({
        "selected_entity_bits": selected,
        "entities": entities,
    }))
}

/// Selects a target entity via `host` and notifies UI panels.
pub fn handle_select_entity<H: McpHost + ?Sized>(host: &mut H, params: &Value) -> InspectResponse {
    let id_bits = params.get("id_bits").and_then(Value::as_u64);
    let name = params.get("name").and_then(Value::as_str);
    match host.select_entity(id_bits, name) {
        Ok(selected) => InspectResponse::ok(json!({ "selected_entity_bits": selected })),
        Err(msg) => InspectResponse::err(msg),
    }
}

/// Returns serialized components for an entity via `host`.
pub fn handle_inspect_entity<H: McpHost + ?Sized>(host: &H, params: &Value) -> InspectResponse {
    let id_bits = params.get("id_bits").and_then(Value::as_u64);
    let name = params.get("name").and_then(Value::as_str);
    match host.inspect_entity(id_bits, name) {
        Ok((bits, components)) => InspectResponse::ok(json!({
            "id_bits": bits,
            "component_count": components.len(),
            "components": components,
        })),
        Err(msg) => InspectResponse::err(msg),
    }
}

/// Returns recent filtered log entries via `host`.
pub fn handle_engine_logs<H: McpHost + ?Sized>(host: &H, params: &Value) -> InspectResponse {
    let limit = params
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, 200) as usize;
    let level = params.get("level").and_then(Value::as_str);

    match host.recent_logs(limit, level) {
        Ok((total_count, entries)) => {
            let logs: Vec<Value> = entries
                .into_iter()
                .map(|e| {
                    json!({
                        "timestamp": e.timestamp,
                        "level": e.level,
                        "target": e.target,
                        "message": e.message,
                    })
                })
                .collect();
            InspectResponse::ok(json!({
                "total_log_count": total_count,
                "returned": logs.len(),
                "logs": logs,
            }))
        }
        Err(msg) => InspectResponse::err(msg),
    }
}

/// Captures a live WGPU screenshot PNG file at `params["path"]`.
pub fn handle_screenshot<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
) -> InspectResponse {
    let path_str = params
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("target/aeon_inspect_shot.png");
    match capture_ui_screenshot(host, overlay, Path::new(path_str)) {
        Ok((w, h)) => InspectResponse::ok(json!({
            "path": path_str,
            "width": w,
            "height": h,
        })),
        Err(e) => InspectResponse::err(e),
    }
}