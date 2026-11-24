// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Menu Presentation Icons
//!
//! Maps authoritative [`ae_scene::BUILTIN_COMPONENTS`] entries to UI text fallback icons
//! and GPU texture atlas UVs for the cascading Add Component menu without duplicating ECS logic.

use crate::ui::iris_bridge::icons::{
    ICON_AUDIO, ICON_CUBE, ICON_EYE_OPEN, ICON_GEAR, ICON_LIGHT, ICON_ROTATE, ICON_WIREFRAME,
};
pub use ae_scene::{BUILTIN_COMPONENTS, BuiltinComponentSpec, find_builtin_component};
use irisui::widgets::cascading_menu::CascadingMenuIcon;

/// Resolves the display icon for a built-in component `type_name` inside the Add Component menu.
#[inline]
#[must_use]
pub const fn menu_icon_for_component(type_name: &str) -> CascadingMenuIcon {
    match type_name.as_bytes() {
        b"RigidBody" => CascadingMenuIcon::Texture(ICON_GEAR),
        b"Collider" => CascadingMenuIcon::Texture(ICON_WIREFRAME),
        b"PhysicsMaterial" => CascadingMenuIcon::Text("🧪"),
        b"CharacterController" => CascadingMenuIcon::Text("🚶"),
        b"CharacterAction" => CascadingMenuIcon::Text("💨"),
        b"PlayerTag" => CascadingMenuIcon::Text("👤"),
        b"Velocity" => CascadingMenuIcon::Text("⚡"),
        b"Rotator" => CascadingMenuIcon::Texture(ICON_ROTATE),
        b"MovingPlatform" => CascadingMenuIcon::Text("🚡"),
        b"TriggerZone" => CascadingMenuIcon::Text("📡"),
        b"DestructibleTarget" => CascadingMenuIcon::Text("🎯"),
        b"Light" => CascadingMenuIcon::Texture(ICON_LIGHT),
        b"ModelId" => CascadingMenuIcon::Texture(ICON_CUBE),
        b"Shape" => CascadingMenuIcon::Texture(ICON_CUBE),
        b"LodGroup" => CascadingMenuIcon::Texture(ICON_EYE_OPEN),
        b"AnimationPlayer" => CascadingMenuIcon::Text("🎬"),
        b"AudioSource" => CascadingMenuIcon::Texture(ICON_AUDIO),
        b"AudioListener" => CascadingMenuIcon::Text("👂"),
        b"UiElement" => CascadingMenuIcon::Text("📐"),
        b"UiPanel" => CascadingMenuIcon::Text("🔲"),
        b"UiText" => CascadingMenuIcon::Text("🔤"),
        b"UiProgressBar" => CascadingMenuIcon::Text("📊"),
        b"UiButton" => CascadingMenuIcon::Text("🔘"),
        b"UiImage" => CascadingMenuIcon::Text("🖼️"),
        b"UiSlider" => CascadingMenuIcon::Text("🎚️"),
        b"UiCheckbox" => CascadingMenuIcon::Text("☑️"),
        b"UiTextInput" => CascadingMenuIcon::Text("📝"),
        b"UiLayoutGroup" => CascadingMenuIcon::Text("🗂️"),
        b"PlayerHealthBarTag" => CascadingMenuIcon::Text("❤️"),
        b"ScoreDisplayTag" => CascadingMenuIcon::Text("🏆"),
        b"ReticleTag" => CascadingMenuIcon::Text("🎯"),
        _ => CascadingMenuIcon::Text("🧩"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_builtin_component_has_explicit_menu_icon_and_no_fallback_text_puzzle() {
        for spec in BUILTIN_COMPONENTS {
            let icon = menu_icon_for_component(spec.component_name);
            assert!(
                !matches!(icon, CascadingMenuIcon::Text("🧩")),
                "Built-in component '{}' must define an explicit menu icon",
                spec.component_name
            );
        }
        assert!(matches!(
            menu_icon_for_component("UnknownCustomComponent"),
            CascadingMenuIcon::Text("🧩")
        ));
    }
}