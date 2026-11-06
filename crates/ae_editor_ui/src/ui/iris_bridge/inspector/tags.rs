// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Semantic Interaction Tags and O(1) Resolvers for Scene Inspector
//!
//! Provides unique 64-bit persistent semantic tags and hit-testing resolvers for
//! all interactive Inspector widgets (component deletion buttons, dropdown triggers,
//! boolean checkboxes, numeric scrubbers, string inputs, and action buttons).
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use super::registry::InspectorRegistry;

/// Semantic tag for the Inspector root scroll panel container.
pub const TAG_INSPECTOR_PANEL_ROOT: u64 = 0xDAFF_0000;

/// Semantic tag for the Inspector scrollable cards container viewport.
pub const TAG_INSPECTOR_CARDS_CONTAINER: u64 = 0xDAFF_0001;

/// Base numeric domain for Inspector action buttons (Add Component, Save Prefab, etc.).
pub const TAG_INSPECTOR_ACTION_BASE: u64 = 0xDA00_0000;

/// Base numeric domain for attachable component items inside the Add Component menu.
pub const TAG_ADD_MENU_COMPONENT_BASE: u64 = 0xDB00_0000;

/// Checks whether a semantic tag belongs to the Inspector subsystem (including the root panel container).
#[inline]
#[must_use]
pub fn is_inspector_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_PANEL_ROOT
        || tag == TAG_INSPECTOR_CARDS_CONTAINER
        || (0xD000_0000..0xE000_0000).contains(&tag)
}

/// Semantic tag for the `➕ Add Component` button in the Inspector footer.
pub const TAG_INSPECTOR_ADD_COMPONENT: u64 = TAG_INSPECTOR_ACTION_BASE + 1;

/// Semantic tag for the `💾 Save as Prefab` button in the Inspector footer.
pub const TAG_INSPECTOR_SAVE_PREFAB: u64 = TAG_INSPECTOR_ACTION_BASE + 2;

/// Semantic tag for the `↺ Preset` reset button in the Physics Material card.
pub const TAG_INSPECTOR_PRESET_RESET: u64 = TAG_INSPECTOR_ACTION_BASE + 3;

/// Semantic tag for the audio file picker `📁` button in the AudioSource card.
pub const TAG_INSPECTOR_AUDIO_PICK: u64 = TAG_INSPECTOR_ACTION_BASE + 4;

/// Semantic tag for the audio play/stop preview toggle button in the AudioSource card.
pub const TAG_INSPECTOR_AUDIO_PLAY: u64 = TAG_INSPECTOR_ACTION_BASE + 5;

/// Semantic tag for the `❌ Unparent` button in the Parenting card.
pub const TAG_INSPECTOR_UNPARENT: u64 = TAG_INSPECTOR_ACTION_BASE + 6;

/// Semantic tag for the interactive object color swatch button in the Appearance card.
pub const TAG_INSPECTOR_COLOR_SWATCH: u64 = TAG_INSPECTOR_ACTION_BASE + 7;

/// Semantic tag for the hexadecimal color text input box in the Appearance card.
pub const TAG_INSPECTOR_COLOR_HEX: u64 = TAG_INSPECTOR_ACTION_BASE + 8;

/// Semantic tag for the `[+] Add` color palette swatch button in the Appearance card.
pub const TAG_INSPECTOR_ADD_PALETTE: u64 = TAG_INSPECTOR_ACTION_BASE + 9;

/// Semantic tag for the `[Clr] Clear` color palette button in the Appearance card.
pub const TAG_INSPECTOR_CLEAR_PALETTE: u64 = TAG_INSPECTOR_ACTION_BASE + 10;

/// Semantic tag for the floating 2D HSV Color Picker popup card container.
pub const TAG_INSPECTOR_COLOR_PICKER_CARD: u64 = TAG_INSPECTOR_ACTION_BASE + 11;

/// Semantic tag for the floating 2D HSV Color Picker close `✖` button.
pub const TAG_INSPECTOR_COLOR_PICKER_CLOSE: u64 = TAG_INSPECTOR_ACTION_BASE + 12;

/// Semantic tag for the floating 2D HSV Color Picker 2D Saturation-Value box.
pub const TAG_INSPECTOR_COLOR_PICKER_SV_BOX: u64 = TAG_INSPECTOR_ACTION_BASE + 13;

/// Semantic tag for the floating 2D HSV Color Picker vertical rainbow Hue spectrum bar.
pub const TAG_INSPECTOR_COLOR_PICKER_HUE_BAR: u64 = TAG_INSPECTOR_ACTION_BASE + 14;

/// Base numeric domain for color palette swatch buttons in the Appearance card.
pub const TAG_INSPECTOR_PALETTE_SWATCH_BASE: u64 = 0xDF20_0000;

/// Encodes a palette swatch index into a unique 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_palette_swatch_tag(idx: usize) -> u64 {
    TAG_INSPECTOR_PALETTE_SWATCH_BASE + idx as u64
}

/// Resolves a semantic tag into its palette swatch index if it represents a swatch pill.
#[inline]
#[must_use]
pub const fn resolve_palette_swatch_tag(tag: u64) -> Option<usize> {
    if tag >= TAG_INSPECTOR_PALETTE_SWATCH_BASE && tag < TAG_INSPECTOR_PALETTE_SWATCH_BASE + 1024 {
        Some((tag - TAG_INSPECTOR_PALETTE_SWATCH_BASE) as usize)
    } else {
        None
    }
}

/// Base numeric domain for component delete/trash action buttons.
pub const TAG_COMPONENT_DELETE_BASE: u64 = 0xDF00_0000;

/// Encodes a component type name into a stable 64-bit component deletion tag.
///
/// Hashes the component name and masks it to 24 bits (`0x00FF_FFFF`), ensuring
/// that the tag stays strictly within the `TAG_COMPONENT_DELETE_BASE` (`0xDF00_0000`)
/// domain and within `is_inspector_tag` range (`0xD000_0000..0xE000_0000`).
#[inline]
#[must_use]
pub fn encode_component_delete_tag(component_name: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in component_name.as_bytes() {
        hash = (hash ^ (*byte as u64)).wrapping_mul(0x100000001b3);
    }
    TAG_COMPONENT_DELETE_BASE | (hash & 0x00FF_FFFF)
}

/// Resolves a semantic tag into its canonical component type name if it represents a deletion button.
#[must_use]
pub fn resolve_component_delete_tag(tag: u64) -> Option<&'static str> {
    let registry = InspectorRegistry::global();
    for handler in registry.handlers() {
        let name = handler.component_name();
        if encode_component_delete_tag(name) == tag {
            return Some(name);
        }
    }

    let comp_registry = ae_core::registry::ComponentRegistry::global();
    for handler in comp_registry.handlers() {
        let name = handler.type_name();
        if encode_component_delete_tag(name) == tag {
            return Some(name);
        }
    }

    if encode_component_delete_tag("UiElement") == tag {
        return Some("UiElement");
    }

    None
}

/// Resolves whether a semantic tag corresponds to the `➕ Add Component` footer button.
#[inline]
#[must_use]
pub const fn resolve_add_component_btn_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_ADD_COMPONENT
}

/// Resolves whether a semantic tag corresponds to the `💾 Save as Prefab` footer button.
#[inline]
#[must_use]
pub const fn resolve_save_prefab_btn_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_SAVE_PREFAB
}

/// Resolves whether a semantic tag corresponds to the Physics Material preset reset button.
#[inline]
#[must_use]
pub const fn resolve_preset_reset_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_PRESET_RESET
}

/// Resolves whether a semantic tag corresponds to the audio file picker button.
#[inline]
#[must_use]
pub const fn resolve_audio_pick_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_AUDIO_PICK
}

/// Resolves whether a semantic tag corresponds to the audio playback preview toggle button.
#[inline]
#[must_use]
pub const fn resolve_audio_play_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_AUDIO_PLAY
}

/// Resolves whether a semantic tag corresponds to the unparenting action button.
#[inline]
#[must_use]
pub const fn resolve_unparent_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_UNPARENT
}

/// Checks whether a semantic tag corresponds to the 2D HSV Color Picker card container.
#[inline]
#[must_use]
pub const fn resolve_color_picker_card_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_COLOR_PICKER_CARD
}

/// Checks whether a semantic tag corresponds to the 2D HSV Color Picker close button.
#[inline]
#[must_use]
pub const fn resolve_color_picker_close_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_COLOR_PICKER_CLOSE
}

/// Checks whether a semantic tag corresponds to the 2D HSV Color Picker Saturation-Value box.
#[inline]
#[must_use]
pub const fn resolve_color_picker_sv_box_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_COLOR_PICKER_SV_BOX
}

/// Checks whether a semantic tag corresponds to the 2D HSV Color Picker vertical Hue spectrum bar.
#[inline]
#[must_use]
pub const fn resolve_color_picker_hue_bar_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_COLOR_PICKER_HUE_BAR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_delete_tag_domain_preservation() {
        let components = [
            "Light",
            "ModelId",
            "LodGroup",
            "Parent",
            "UiElement",
            "UiPanel",
            "UiText",
            "UiProgressBar",
            "UiButton",
            "UiImage",
            "UiSlider",
            "UiCheckbox",
            "UiTextInput",
            "PlayerHealthBarTag",
            "ScoreDisplayTag",
            "ReticleTag",
            "RigidBody",
            "Collider",
            "PhysicsMaterial",
            "PlayerTag",
            "Velocity",
            "CharacterController",
            "AudioSource",
            "AudioListener",
        ];

        for name in components {
            let tag = encode_component_delete_tag(name);
            assert!(
                (TAG_COMPONENT_DELETE_BASE..=(TAG_COMPONENT_DELETE_BASE | 0x00FF_FFFF))
                    .contains(&tag),
                "Delete tag for {} must stay within 0xDF00_0000..=0xDFFF_FFFF domain (got 0x{:016X})",
                name,
                tag
            );
            assert!(
                is_inspector_tag(tag),
                "is_inspector_tag must return true for delete tag of {} (0x{:016X})",
                name,
                tag
            );
            assert_eq!(
                resolve_component_delete_tag(tag),
                Some(name),
                "Delete tag must resolve back to {}",
                name
            );
        }
    }

    #[test]
    fn test_color_picker_tag_resolution() {
        assert!(is_inspector_tag(TAG_INSPECTOR_COLOR_PICKER_CARD));
        assert!(is_inspector_tag(TAG_INSPECTOR_COLOR_PICKER_CLOSE));
        assert!(is_inspector_tag(TAG_INSPECTOR_COLOR_PICKER_SV_BOX));
        assert!(is_inspector_tag(TAG_INSPECTOR_COLOR_PICKER_HUE_BAR));

        assert!(resolve_color_picker_card_tag(
            TAG_INSPECTOR_COLOR_PICKER_CARD
        ));
        assert!(resolve_color_picker_close_tag(
            TAG_INSPECTOR_COLOR_PICKER_CLOSE
        ));
        assert!(resolve_color_picker_sv_box_tag(
            TAG_INSPECTOR_COLOR_PICKER_SV_BOX
        ));
        assert!(resolve_color_picker_hue_bar_tag(
            TAG_INSPECTOR_COLOR_PICKER_HUE_BAR
        ));

        assert!(!resolve_color_picker_card_tag(
            TAG_INSPECTOR_COLOR_PICKER_CLOSE
        ));
        assert!(!resolve_color_picker_close_tag(
            TAG_INSPECTOR_COLOR_PICKER_SV_BOX
        ));
        assert!(!resolve_color_picker_sv_box_tag(
            TAG_INSPECTOR_COLOR_PICKER_HUE_BAR
        ));
        assert!(!resolve_color_picker_hue_bar_tag(
            TAG_INSPECTOR_COLOR_PICKER_CARD
        ));
    }
}