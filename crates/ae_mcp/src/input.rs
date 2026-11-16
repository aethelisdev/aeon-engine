// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Pointer, scroll, and key/target resolution helpers for live [`crate::McpHost`] control.

use crate::host::McpHost;
use crate::overlay::InspectVisualOverlay;
use crate::protocol::{InspectResponse, parse_u64_json_value};
use irisui::core::{KeyCode, MouseButton, Point, UiEvent};
use serde_json::{Value, json};
use winit::keyboard::KeyCode as WinitKeyCode;

/// Injects click (`left`/`right`/`middle`, single or double) events through `host`.
pub fn handle_ui_click<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
) -> InspectResponse {
    let point = match resolve_click_target(host, params) {
        Ok(pt) => pt,
        Err(msg) => return InspectResponse::err(msg),
    };

    let button = parse_mouse_button(params.get("button").and_then(Value::as_str));
    let double_click = params
        .get("double_click")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let moved = host.dispatch_ui_event(&UiEvent::MouseMove { point });
    let mut down = host.dispatch_ui_event(&UiEvent::MouseDown { point, button });
    let mut up = host.dispatch_ui_event(&UiEvent::MouseUp { point, button });

    if double_click {
        down |= host.dispatch_ui_event(&UiEvent::MouseDown { point, button });
        up |= host.dispatch_ui_event(&UiEvent::MouseUp { point, button });
    }

    overlay.record_click(point, button, double_click);
    host.invalidate_ui(true);

    let tree = host.ui_tree();
    InspectResponse::ok(json!({
        "clicked_point": [point.x, point.y],
        "double_click": double_click,
        "consumed": { "move": moved, "down": down, "up": up },
        "active_combobox_tag": tree.active_combobox_tag.map(|t| format!("0x{t:016x}")),
        "active_popup_tags": tree.active_popup_tags.iter().map(|t| format!("0x{t:016x}")).collect::<Vec<_>>(),
    }))
}

/// Injects a pointer movement event through `host` and updates `overlay`.
pub fn handle_ui_mouse_move<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
) -> InspectResponse {
    let Some(x) = params.get("x").and_then(Value::as_f64) else {
        return InspectResponse::err("Missing float param 'x'");
    };
    let Some(y) = params.get("y").and_then(Value::as_f64) else {
        return InspectResponse::err("Missing float param 'y'");
    };
    let point = Point::new(x as f32, y as f32);
    let consumed = host.dispatch_ui_event(&UiEvent::MouseMove { point });
    overlay.record_move(point);
    host.request_redraw();
    InspectResponse::ok(json!({ "point": [x, y], "consumed": consumed }))
}

/// Injects a vertical mouse scroll delta at `(x, y)` with ripple visual feedback.
pub fn handle_ui_scroll<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
) -> InspectResponse {
    let x = params.get("x").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let y = params.get("y").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let delta_y = params.get("delta_y").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let point = Point::new(x, y);
    let _ = host.dispatch_ui_event(&UiEvent::MouseMove { point });
    let consumed = host.dispatch_ui_event(&UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y,
        point,
    });
    overlay.record_scroll(point);
    host.request_redraw();
    InspectResponse::ok(json!({ "point": [x, y], "delta_y": delta_y, "consumed": consumed }))
}

/// Parses an optional mouse button string (`"left"`, `"right"`, `"middle"`).
#[must_use]
pub fn parse_mouse_button(raw: Option<&str>) -> MouseButton {
    match raw {
        Some("right") => MouseButton::Right,
        Some("middle") => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

/// Resolves a coordinate target from `[x, y]`, a widget name string, or `{point, tag, name}` object.
///
/// # Errors
/// Returns a descriptive error string when the target descriptor is malformed or missing.
pub fn resolve_point_value<H: McpHost + ?Sized>(host: &H, val: &Value) -> Result<Point, String> {
    if let Some(arr) = val.as_array()
        && arr.len() >= 2
        && let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64())
    {
        return Ok(Point::new(x as f32, y as f32));
    }
    if val.is_object() {
        return resolve_click_target(host, val);
    }
    if let Some(name) = val.as_str() {
        if let Some(tag) = parse_u64_json_value(val)
            && name.starts_with("0x")
        {
            return resolve_tag_center(host, tag);
        }
        return resolve_name_center(host, name, 0);
    }
    Err("Expected [x, y], widget name string, or {point/tag/name} object".to_string())
}

/// Resolves a target center point from `point`, `tag`, or `name`/`index` JSON fields.
///
/// # Errors
/// Returns an error string when no valid matching node or coordinate exists.
pub fn resolve_click_target<H: McpHost + ?Sized>(
    host: &H,
    params: &Value,
) -> Result<Point, String> {
    if let Some(arr) = params.get("point").and_then(Value::as_array)
        && arr.len() >= 2
        && let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64())
    {
        return Ok(Point::new(x as f32, y as f32));
    }

    if let Some(tag) = params.get("tag").and_then(parse_u64_json_value) {
        return resolve_tag_center(host, tag);
    }

    if let Some(name) = params.get("name").and_then(Value::as_str) {
        let idx = params.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
        return resolve_name_center(host, name, idx);
    }

    Err("Target requires 'point': [x, y], 'tag', or 'name'".to_string())
}

fn resolve_tag_center<H: McpHost + ?Sized>(host: &H, tag: u64) -> Result<Point, String> {
    let rect = host
        .ui_tree()
        .resolve_anchor_rect(tag)
        .ok_or_else(|| format!("Tag 0x{tag:016x} has no resolved bounding rect"))?;
    Ok(Point::new(
        rect.x + rect.width * 0.5,
        rect.y + rect.height * 0.5,
    ))
}

fn resolve_name_center<H: McpHost + ?Sized>(
    host: &H,
    name: &str,
    idx: usize,
) -> Result<Point, String> {
    let rect = host
        .ui_tree()
        .iter()
        .filter(|(_, n)| {
            n.name.as_deref() == Some(name) || n.text.as_deref().is_some_and(|t| t == name)
        })
        .nth(idx)
        .map(|(_, n)| n.computed_rect)
        .filter(|r| r.width > 0.0 && r.height > 0.0)
        .ok_or_else(|| format!("Widget '{name}' (index {idx}) not found with non-zero rect"))?;
    Ok(Point::new(
        rect.x + rect.width * 0.5,
        rect.y + rect.height * 0.5,
    ))
}

/// Parses named keys (`"Escape"`, `"Enter"`, `"F5"`, etc.) or single characters into Iris/Winit codes.
#[must_use]
pub fn parse_key_spec(raw: &str) -> Option<(KeyCode, Option<char>, Option<WinitKeyCode>)> {
    match raw {
        "Escape" | "Esc" => Some((KeyCode::Escape, None, Some(WinitKeyCode::Escape))),
        "Enter" | "Return" => Some((KeyCode::Enter, None, Some(WinitKeyCode::Enter))),
        "Tab" => Some((KeyCode::Tab, None, Some(WinitKeyCode::Tab))),
        "Backspace" => Some((KeyCode::Backspace, None, Some(WinitKeyCode::Backspace))),
        "Delete" | "Del" => Some((KeyCode::Delete, None, Some(WinitKeyCode::Delete))),
        "Space" | " " => Some((KeyCode::Space, Some(' '), Some(WinitKeyCode::Space))),
        "ArrowLeft" | "Left" => Some((KeyCode::ArrowLeft, None, Some(WinitKeyCode::ArrowLeft))),
        "ArrowRight" | "Right" => Some((KeyCode::ArrowRight, None, Some(WinitKeyCode::ArrowRight))),
        "ArrowUp" | "Up" => Some((KeyCode::ArrowUp, None, Some(WinitKeyCode::ArrowUp))),
        "ArrowDown" | "Down" => Some((KeyCode::ArrowDown, None, Some(WinitKeyCode::ArrowDown))),
        "Home" => Some((KeyCode::Home, None, Some(WinitKeyCode::Home))),
        "End" => Some((KeyCode::End, None, Some(WinitKeyCode::End))),
        "F5" => Some((KeyCode::Other, None, Some(WinitKeyCode::F5))),
        "F2" => Some((KeyCode::Other, None, Some(WinitKeyCode::F2))),
        other => {
            let mut chars = other.chars();
            let ch = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            Some((KeyCode::Character(ch), Some(ch), char_to_winit_key(ch)))
        }
    }
}

const fn char_to_winit_key(ch: char) -> Option<WinitKeyCode> {
    match ch.to_ascii_lowercase() {
        'a' => Some(WinitKeyCode::KeyA),
        'b' => Some(WinitKeyCode::KeyB),
        'c' => Some(WinitKeyCode::KeyC),
        'd' => Some(WinitKeyCode::KeyD),
        'e' => Some(WinitKeyCode::KeyE),
        'f' => Some(WinitKeyCode::KeyF),
        'g' => Some(WinitKeyCode::KeyG),
        'h' => Some(WinitKeyCode::KeyH),
        'i' => Some(WinitKeyCode::KeyI),
        'j' => Some(WinitKeyCode::KeyJ),
        'k' => Some(WinitKeyCode::KeyK),
        'l' => Some(WinitKeyCode::KeyL),
        'm' => Some(WinitKeyCode::KeyM),
        'n' => Some(WinitKeyCode::KeyN),
        'o' => Some(WinitKeyCode::KeyO),
        'p' => Some(WinitKeyCode::KeyP),
        'q' => Some(WinitKeyCode::KeyQ),
        'r' => Some(WinitKeyCode::KeyR),
        's' => Some(WinitKeyCode::KeyS),
        't' => Some(WinitKeyCode::KeyT),
        'u' => Some(WinitKeyCode::KeyU),
        'v' => Some(WinitKeyCode::KeyV),
        'w' => Some(WinitKeyCode::KeyW),
        'x' => Some(WinitKeyCode::KeyX),
        'y' => Some(WinitKeyCode::KeyY),
        'z' => Some(WinitKeyCode::KeyZ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mouse_button_variants() {
        assert_eq!(parse_mouse_button(None), MouseButton::Left);
        assert_eq!(parse_mouse_button(Some("left")), MouseButton::Left);
        assert_eq!(parse_mouse_button(Some("right")), MouseButton::Right);
        assert_eq!(parse_mouse_button(Some("middle")), MouseButton::Middle);
        assert_eq!(parse_mouse_button(Some("unknown")), MouseButton::Left);
    }

    #[test]
    fn test_parse_key_spec_named_and_char() {
        let (esc, txt, wk) = parse_key_spec("Escape").expect("Escape valid");
        assert_eq!(esc, KeyCode::Escape);
        assert_eq!(txt, None);
        assert_eq!(wk, Some(WinitKeyCode::Escape));

        let (ch, c_txt, c_wk) = parse_key_spec("z").expect("char z valid");
        assert_eq!(ch, KeyCode::Character('z'));
        assert_eq!(c_txt, Some('z'));
        assert_eq!(c_wk, Some(WinitKeyCode::KeyZ));

        assert!(parse_key_spec("").is_none());
        assert!(parse_key_spec("InvalidMultiCharKey").is_none());
    }
}