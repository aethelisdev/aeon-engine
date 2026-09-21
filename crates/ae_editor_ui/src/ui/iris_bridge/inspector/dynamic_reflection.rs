// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generic reflection utilities and component inspector helpers.

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

/// Formats a raw identifier name (e.g. `max_health`, `is_invincible`) into a human-readable label.
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