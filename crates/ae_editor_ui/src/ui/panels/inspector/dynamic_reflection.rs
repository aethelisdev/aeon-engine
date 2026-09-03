// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generic Reflection-Driven Component Inspector & Dynamic Fallback UI.
//!
//! Provides automatic UI generation for any ECS component registered in `ComponentRegistry`
//! that lacks a specialized manual `ComponentUiHandler`. Automatically inspects, formats,
//! and edits numbers, booleans, strings, colors, vectors, arrays, and nested structures.
//!

use crate::ui::types::EngineUiAction;

/// Checks if a component is an internal system marker or handled by top-level inspector headers.
pub fn is_internal_or_specialized(type_name: &str) -> bool {
    matches!(
        type_name,
        "Position"
            | "Rotation"
            | "Scale"
            | "Name"
            | "TransformDirty"
            | "GlobalTransform"
            | "Hidden"
            | "BoundingBox"
            | "BoundingRadius"
            | "Shape"
            | "Color"
            | "Light"
            | "Parent"
            | "Children"
            | "ModelId"
            | "SpriteId"
    )
}

/// Formats a raw identifier name (e.g. `max_health`, `is_invincible`) into human-readable label.
pub fn format_field_label(field_name: &str) -> String {
    let clean_name = field_name.strip_prefix("is_").unwrap_or(field_name);
    let mut words = Vec::new();
    let mut current_word = String::new();

    for ch in clean_name.chars() {
        if ch == '_' || ch == '-' {
            if !current_word.is_empty() {
                words.push(capitalize_first(&current_word));
                current_word.clear();
            }
        } else if ch.is_uppercase() && !current_word.is_empty() {
            words.push(capitalize_first(&current_word));
            current_word.clear();
            current_word.push(ch);
        } else {
            current_word.push(ch);
        }
    }
    if !current_word.is_empty() {
        words.push(capitalize_first(&current_word));
    }

    if words.is_empty() {
        format!("{}:", field_name)
    } else {
        format!("{}:", words.join(" "))
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Renders a dynamic reflection inspector card for a component using its type-erased handler.
pub fn draw_dynamic_component_card(
    ui: &mut egui::Ui,
    ctx: &mut super::registry::InspectorContext,
    handler: &dyn ae_core::registry::ComponentHandler,
) {
    let type_name = handler.type_name();
    if is_internal_or_specialized(type_name) {
        return;
    }

    if let Some(bytes) = handler.capture(ctx.world, ctx.entity)
        && let Ok(mut json_val) = serde_json::from_slice::<serde_json::Value>(&bytes)
    {
        let mut changed = false;

        let (_, remove_clicked) = super::widgets::draw_inspector_card(
            ui,
            type_name,
            "🧩",
            egui::Color32::from_rgb(140, 200, 255),
            true,
            |ui| match &mut json_val {
                serde_json::Value::Object(map) => {
                    if map.is_empty() {
                        ui.label(
                            egui::RichText::new("Marker tag (no fields)")
                                .small()
                                .color(egui::Color32::from_gray(160)),
                        );
                    } else {
                        for (key, val) in map.iter_mut() {
                            if draw_json_field(ui, key, val) {
                                changed = true;
                            }
                        }
                    }
                }
                serde_json::Value::Array(arr) => {
                    for (i, val) in arr.iter_mut().enumerate() {
                        if draw_json_field(ui, &format!("[{}]", i), val) {
                            changed = true;
                        }
                    }
                }
                other => {
                    if draw_json_field(ui, "Value", other) {
                        changed = true;
                    }
                }
            },
        );

        if changed && let Ok(updated_bytes) = serde_json::to_vec(&json_val) {
            ctx.ui_actions.push(EngineUiAction::CommitComponentModify(
                ctx.entity,
                type_name,
                bytes,
                updated_bytes,
            ));
        }

        if remove_clicked {
            ctx.ui_actions
                .push(EngineUiAction::RemoveComponent(ctx.entity, type_name));
        }
    }
}

/// Recursively renders a JSON field value with appropriate egui widgets.
/// Returns `true` if the user modified the field value.
pub fn draw_json_field(ui: &mut egui::Ui, field_name: &str, value: &mut serde_json::Value) -> bool {
    let mut changed = false;

    match value {
        serde_json::Value::Bool(b) => {
            ui.horizontal(|ui| {
                if ui.checkbox(b, format_field_label(field_name)).changed() {
                    changed = true;
                }
            });
        }
        serde_json::Value::Number(num) => {
            if let Some(mut f) = num.as_f64() {
                ui.horizontal(|ui| {
                    ui.label(format_field_label(field_name));
                    if ui.add(egui::DragValue::new(&mut f).speed(0.05)).changed()
                        && let Some(new_num) = serde_json::Number::from_f64(f)
                    {
                        *num = new_num;
                        changed = true;
                    }
                });
            } else if let Some(mut i) = num.as_i64() {
                ui.horizontal(|ui| {
                    ui.label(format_field_label(field_name));
                    if ui.add(egui::DragValue::new(&mut i).speed(1)).changed() {
                        *num = serde_json::Number::from(i);
                        changed = true;
                    }
                });
            } else if let Some(mut u) = num.as_u64() {
                ui.horizontal(|ui| {
                    ui.label(format_field_label(field_name));
                    if ui.add(egui::DragValue::new(&mut u).speed(1)).changed() {
                        *num = serde_json::Number::from(u);
                        changed = true;
                    }
                });
            }
        }
        serde_json::Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(format_field_label(field_name));
                if ui.text_edit_singleline(s).changed() {
                    changed = true;
                }
            });
        }
        serde_json::Value::Array(arr) => {
            let lower_name = field_name.to_lowercase();
            // Color Detection: 3 or 4 floating point numbers named "color"
            if lower_name.contains("color")
                && (arr.len() == 3 || arr.len() == 4)
                && arr.iter().all(|v| v.is_number())
            {
                if arr.len() == 3 {
                    let mut col = [
                        arr[0].as_f64().unwrap_or(1.0) as f32,
                        arr[1].as_f64().unwrap_or(1.0) as f32,
                        arr[2].as_f64().unwrap_or(1.0) as f32,
                    ];
                    ui.horizontal(|ui| {
                        ui.label(format_field_label(field_name));
                        if ui.color_edit_button_rgb(&mut col).changed() {
                            if let Some(n0) = serde_json::Number::from_f64(col[0] as f64) {
                                arr[0] = serde_json::Value::Number(n0);
                            }
                            if let Some(n1) = serde_json::Number::from_f64(col[1] as f64) {
                                arr[1] = serde_json::Value::Number(n1);
                            }
                            if let Some(n2) = serde_json::Number::from_f64(col[2] as f64) {
                                arr[2] = serde_json::Value::Number(n2);
                            }
                            changed = true;
                        }
                    });
                } else {
                    let mut col = [
                        arr[0].as_f64().unwrap_or(1.0) as f32,
                        arr[1].as_f64().unwrap_or(1.0) as f32,
                        arr[2].as_f64().unwrap_or(1.0) as f32,
                        arr[3].as_f64().unwrap_or(1.0) as f32,
                    ];
                    ui.horizontal(|ui| {
                        ui.label(format_field_label(field_name));
                        if ui.color_edit_button_rgba_unmultiplied(&mut col).changed() {
                            if let Some(n0) = serde_json::Number::from_f64(col[0] as f64) {
                                arr[0] = serde_json::Value::Number(n0);
                            }
                            if let Some(n1) = serde_json::Number::from_f64(col[1] as f64) {
                                arr[1] = serde_json::Value::Number(n1);
                            }
                            if let Some(n2) = serde_json::Number::from_f64(col[2] as f64) {
                                arr[2] = serde_json::Value::Number(n2);
                            }
                            if let Some(n3) = serde_json::Number::from_f64(col[3] as f64) {
                                arr[3] = serde_json::Value::Number(n3);
                            }
                            changed = true;
                        }
                    });
                }
            } else if (arr.len() == 2 || arr.len() == 3 || arr.len() == 4)
                && arr.iter().all(|v| v.is_number())
            {
                // Compact Numeric Vector Row [X, Y, Z, W]
                let labels = ["X:", "Y:", "Z:", "W:"];
                ui.horizontal(|ui| {
                    ui.label(format_field_label(field_name));
                    for (i, elem) in arr.iter_mut().enumerate() {
                        if i < labels.len() {
                            ui.label(labels[i]);
                        }
                        if let Some(mut f) = elem.as_f64()
                            && ui.add(egui::DragValue::new(&mut f).speed(0.05)).changed()
                            && let Some(new_num) = serde_json::Number::from_f64(f)
                        {
                            *elem = serde_json::Value::Number(new_num);
                            changed = true;
                        }
                    }
                });
            } else {
                // General Array / Collection
                egui::CollapsingHeader::new(format!(
                    "{} ({} items)",
                    format_field_label(field_name),
                    arr.len()
                ))
                .default_open(true)
                .show(ui, |ui| {
                    ui.indent("array_indent", |ui| {
                        for (i, item) in arr.iter_mut().enumerate() {
                            if draw_json_field(ui, &format!("[{}]", i), item) {
                                changed = true;
                            }
                        }
                    });
                });
            }
        }
        serde_json::Value::Object(map) => {
            egui::CollapsingHeader::new(format_field_label(field_name))
                .default_open(true)
                .show(ui, |ui| {
                    ui.indent("object_indent", |ui| {
                        for (k, v) in map.iter_mut() {
                            if draw_json_field(ui, k, v) {
                                changed = true;
                            }
                        }
                    });
                });
        }
        serde_json::Value::Null => {
            ui.horizontal(|ui| {
                ui.label(format_field_label(field_name));
                ui.colored_label(egui::Color32::from_gray(130), "null");
            });
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::registry::ComponentHandler;

    #[test]
    fn test_format_field_label() {
        assert_eq!(format_field_label("health"), "Health:");
        assert_eq!(format_field_label("max_health"), "Max Health:");
        assert_eq!(format_field_label("is_invincible"), "Invincible:");
        assert_eq!(format_field_label("moveSpeed"), "Move Speed:");
        assert_eq!(
            format_field_label("target_position_y"),
            "Target Position Y:"
        );
    }

    #[test]
    fn test_is_internal_or_specialized() {
        assert!(is_internal_or_specialized("TransformDirty"));
        assert!(is_internal_or_specialized("Position"));
        assert!(!is_internal_or_specialized("CustomStats"));
        assert!(!is_internal_or_specialized("Inventory"));
    }

    #[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
    struct CustomStats {
        pub health: f32,
        pub is_alive: bool,
        pub name: String,
        pub color: [f32; 3],
    }

    #[test]
    fn test_dynamic_component_card_lifecycle() {
        let handler = ae_core::registry::TypedComponentHandler::<CustomStats>::with_default(
            "CustomStats",
            CustomStats::default,
        );

        let mut world = hecs::World::new();
        let entity = world.spawn(());

        // Capture before adding
        assert!(handler.capture(&world, entity).is_none());

        // Add default
        handler
            .add_default(&mut world, entity)
            .expect("Added default CustomStats");
        assert!(handler.has_component(&world, entity));

        let captured = handler
            .capture(&world, entity)
            .expect("Captured CustomStats");
        let val: serde_json::Value = serde_json::from_slice(&captured).expect("Parsed JSON");
        assert!(val.is_object());
        let map = val.as_object().unwrap();
        assert!(map.contains_key("health"));
        assert!(map.contains_key("is_alive"));
        assert!(map.contains_key("name"));
        assert!(map.contains_key("color"));

        // Modify via JSON
        let modified_json = serde_json::json!({
            "health": 99.5,
            "is_alive": true,
            "name": "Hero",
            "color": [1.0, 0.5, 0.0]
        });
        let modified_bytes = serde_json::to_vec(&modified_json).unwrap();
        handler
            .apply(&mut world, entity, &modified_bytes)
            .expect("Applied modification");

        let stats = world
            .get::<&CustomStats>(entity)
            .expect("Retrieved CustomStats");
        assert_eq!(stats.health, 99.5);
        assert!(stats.is_alive);
        assert_eq!(stats.name, "Hero");
        assert_eq!(stats.color, [1.0, 0.5, 0.0]);
    }
}