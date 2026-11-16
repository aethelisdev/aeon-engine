// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame-by-frame interpolated pointer drag controller for live [`crate::McpHost`] automation.

use crate::host::McpHost;
use crate::input::{parse_mouse_button, resolve_point_value};
use crate::overlay::InspectVisualOverlay;
use crate::protocol::InspectResponse;
use irisui::core::{MouseButton, Point, UiEvent};
use serde_json::{Value, json};
use std::sync::mpsc::Sender;

/// Multi-frame drag gesture state advancing one interpolated step per host frame.
pub struct ActiveInspectDrag {
    from: Point,
    to: Point,
    button: MouseButton,
    steps: u32,
    current_step: u32,
    responder: Sender<InspectResponse>,
}

/// Initializes a smooth interpolated drag between `params["from"]` and `params["to"]`.
///
/// Returns the initialized [`ActiveInspectDrag`] on success or responds immediately with an
/// error on `responder` when parameters fail validation.
pub fn start_drag<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
    responder: Sender<InspectResponse>,
) -> Option<ActiveInspectDrag> {
    let Some(from_val) = params.get("from") else {
        let _ = responder.send(InspectResponse::err("ui.drag requires 'from' target"));
        return None;
    };
    let Some(to_val) = params.get("to") else {
        let _ = responder.send(InspectResponse::err("ui.drag requires 'to' target"));
        return None;
    };

    let from = match resolve_point_value(host, from_val) {
        Ok(pt) => pt,
        Err(msg) => {
            let _ = responder.send(InspectResponse::err(format!("Invalid 'from': {msg}")));
            return None;
        }
    };
    let to = match resolve_point_value(host, to_val) {
        Ok(pt) => pt,
        Err(msg) => {
            let _ = responder.send(InspectResponse::err(format!("Invalid 'to': {msg}")));
            return None;
        }
    };

    let button = parse_mouse_button(params.get("button").and_then(Value::as_str));
    let steps = params
        .get("steps")
        .and_then(Value::as_u64)
        .unwrap_or(12)
        .clamp(1, 120) as u32;

    let _ = host.dispatch_ui_event(&UiEvent::MouseMove { point: from });
    let _ = host.dispatch_ui_event(&UiEvent::MouseDown {
        point: from,
        button,
    });
    overlay.record_drag_step(from, from);
    host.invalidate_ui(true);

    Some(ActiveInspectDrag {
        from,
        to,
        button,
        steps,
        current_step: 0,
        responder,
    })
}

/// Advances an active drag gesture by one interpolated step, returning `None` when completed.
pub fn tick_drag<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    mut drag: ActiveInspectDrag,
) -> Option<ActiveInspectDrag> {
    drag.current_step += 1;
    let t = (drag.current_step as f32 / drag.steps as f32).clamp(0.0, 1.0);
    let pos = Point::new(
        drag.from.x + (drag.to.x - drag.from.x) * t,
        drag.from.y + (drag.to.y - drag.from.y) * t,
    );

    let _ = host.dispatch_ui_event(&UiEvent::MouseMove { point: pos });
    overlay.record_drag_step(drag.from, pos);
    host.invalidate_ui(true);

    if drag.current_step >= drag.steps {
        let _ = host.dispatch_ui_event(&UiEvent::MouseUp {
            point: drag.to,
            button: drag.button,
        });
        overlay.record_drag_end(drag.from, drag.to);
        host.invalidate_ui(true);
        let _ = drag.responder.send(InspectResponse::ok(json!({
            "from": [drag.from.x, drag.from.y],
            "to": [drag.to.x, drag.to.y],
            "steps": drag.steps,
        })));
        None
    } else {
        Some(drag)
    }
}