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
use super::types::{
    ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId, InspectorTextInputId,
};

/// Semantic tag for the Inspector root scroll panel container.
pub const TAG_INSPECTOR_PANEL_ROOT: u64 = 0xDAFF_0000;

/// Base numeric domain for Inspector action buttons (Add Component, Save Prefab, etc.).
pub const TAG_INSPECTOR_ACTION_BASE: u64 = 0xDA00_0000;

/// Base numeric domain for attachable component items inside the Add Component menu.
pub const TAG_ADD_MENU_COMPONENT_BASE: u64 = 0xDB00_0000;

/// Base numeric domain for selectable items inside the Inspector dropdown popup menus.
pub const TAG_INSPECTOR_DROPDOWN_ITEM_BASE: u64 = 0xDD80_0000;

/// Checks whether a semantic tag belongs to the Inspector subsystem (including the root panel container).
#[inline]
#[must_use]
pub fn is_inspector_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_PANEL_ROOT
        || (0xD000_0000..0xE000_0000).contains(&tag)
        || super::transform::resolve_transform_number_input_tag(tag).is_some()
        || super::transform::resolve_transform_reset_tag(tag).is_some()
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

/// Base numeric domain for dropdown combobox trigger buttons.
pub const TAG_INSPECTOR_DROPDOWN_BASE: u64 = 0xDD00_0000;

/// Base numeric domain for component boolean checkbox toggles.
pub const TAG_INSPECTOR_CHECKBOX_BASE: u64 = 0xDC00_0000;

/// Base numeric domain for scalar numeric input fields.
pub const TAG_INSPECTOR_NUMBER_INPUT_BASE: u64 = 0xD100_0000;

/// Base numeric domain for string text input fields.
pub const TAG_INSPECTOR_TEXT_INPUT_BASE: u64 = 0xDE00_0000;

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

/// Encodes an [`InspectorDropdownId`] into a unique 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_inspector_dropdown_tag(dd: InspectorDropdownId) -> u64 {
    TAG_INSPECTOR_DROPDOWN_BASE + dd as u64
}

/// Resolves a semantic tag into its corresponding [`InspectorDropdownId`].
#[must_use]
pub const fn resolve_inspector_dropdown_tag(tag: u64) -> Option<InspectorDropdownId> {
    if tag >= TAG_INSPECTOR_DROPDOWN_BASE && tag < TAG_INSPECTOR_DROPDOWN_BASE + 8 {
        match tag - TAG_INSPECTOR_DROPDOWN_BASE {
            0 => Some(InspectorDropdownId::RigidBodyType),
            1 => Some(InspectorDropdownId::ColliderShape),
            2 => Some(InspectorDropdownId::SurfaceType),
            3 => Some(InspectorDropdownId::LightType),
            4 => Some(InspectorDropdownId::CameraProjection),
            5 => Some(InspectorDropdownId::ShapeType),
            6 => Some(InspectorDropdownId::UiAnchor),
            7 => Some(InspectorDropdownId::UiTextAlignment),
            _ => None,
        }
    } else {
        None
    }
}

/// Encodes a [`ComponentCheckboxId`] into a unique 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_component_checkbox_tag(cb_id: ComponentCheckboxId) -> u64 {
    TAG_INSPECTOR_CHECKBOX_BASE + cb_id as u64
}

/// Resolves a semantic tag into its corresponding [`ComponentCheckboxId`].
#[must_use]
pub const fn resolve_component_checkbox_tag(tag: u64) -> Option<ComponentCheckboxId> {
    if tag >= TAG_INSPECTOR_CHECKBOX_BASE && tag < TAG_INSPECTOR_CHECKBOX_BASE + 7 {
        match tag - TAG_INSPECTOR_CHECKBOX_BASE {
            0 => Some(ComponentCheckboxId::ColliderIsSensor),
            1 => Some(ComponentCheckboxId::AudioLoop),
            2 => Some(ComponentCheckboxId::AudioSpatial),
            3 => Some(ComponentCheckboxId::AudioPlayOnStart),
            4 => Some(ComponentCheckboxId::LightCastShadows),
            5 => Some(ComponentCheckboxId::UiInteractable),
            6 => Some(ComponentCheckboxId::UiVisible),
            _ => None,
        }
    } else {
        None
    }
}

/// Encodes an [`InspectorNumberInputId`] into a unique 64-bit semantic tag.
///
/// Delegates to [`InspectorNumberInputId::to_tag_index`] as the Single Source of Truth.
#[inline]
#[must_use]
pub const fn encode_inspector_number_input_tag(id: InspectorNumberInputId) -> u64 {
    TAG_INSPECTOR_NUMBER_INPUT_BASE + id.to_tag_index() as u64
}

/// Resolves an interactive semantic tag into its numeric input field identifier, valid range bounds, and sensitivity.
///
/// Automatically delegates to Transform numeric resolution if appropriate, enabling uniform $O(1)$ dispatch.
/// Safely bounds-checks against at least 64 variants and delegates to [`InspectorNumberInputId::from_tag_index`].
#[must_use]
pub fn resolve_inspector_number_input_tag(
    tag: u64,
) -> Option<(InspectorNumberInputId, f32, f32, f32)> {
    if let Some(res) = super::transform::resolve_transform_number_input_tag(tag) {
        return Some(res);
    }

    if (TAG_INSPECTOR_NUMBER_INPUT_BASE..TAG_INSPECTOR_NUMBER_INPUT_BASE + 64).contains(&tag) {
        let idx = (tag - TAG_INSPECTOR_NUMBER_INPUT_BASE) as usize;
        if let Some(id) = InspectorNumberInputId::from_tag_index(idx) {
            let (min_val, max_val) = id.valid_range();
            let sensitivity = match id {
                InspectorNumberInputId::UiOffsetX
                | InspectorNumberInputId::UiOffsetY
                | InspectorNumberInputId::UiSizeW
                | InspectorNumberInputId::UiSizeH => 1.0,
                InspectorNumberInputId::UiFontSize
                | InspectorNumberInputId::UiBorderWidth
                | InspectorNumberInputId::UiCornerRadius
                | InspectorNumberInputId::UiProgressVal
                | InspectorNumberInputId::UiProgressMin
                | InspectorNumberInputId::UiProgressMax
                | InspectorNumberInputId::UiZIndex => 0.5,
                InspectorNumberInputId::UiAlpha
                | InspectorNumberInputId::UiPivotX
                | InspectorNumberInputId::UiPivotY => 0.01,
                InspectorNumberInputId::RotX
                | InspectorNumberInputId::RotY
                | InspectorNumberInputId::RotZ
                | InspectorNumberInputId::CharacterMaxSlope => 0.5,
                InspectorNumberInputId::ScaleX
                | InspectorNumberInputId::ScaleY
                | InspectorNumberInputId::ScaleZ => 0.01,
                InspectorNumberInputId::PosX
                | InspectorNumberInputId::PosY
                | InspectorNumberInputId::PosZ => 0.1,
                InspectorNumberInputId::RigidBodyMass
                | InspectorNumberInputId::RigidBodyGravity
                | InspectorNumberInputId::ColliderBoxX
                | InspectorNumberInputId::ColliderBoxY
                | InspectorNumberInputId::ColliderBoxZ
                | InspectorNumberInputId::ColliderHalfHeight
                | InspectorNumberInputId::ColliderRadius
                | InspectorNumberInputId::ColliderCenterY => 0.05,
                _ => 0.05,
            };
            return Some((id, min_val, max_val, sensitivity));
        }
    }

    None
}

/// Encodes an [`InspectorTextInputId`] into a unique 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_inspector_text_input_tag(id: InspectorTextInputId) -> u64 {
    TAG_INSPECTOR_TEXT_INPUT_BASE + id as u64
}

/// Resolves a semantic tag into its corresponding [`InspectorTextInputId`].
#[must_use]
pub const fn resolve_inspector_text_input_tag(tag: u64) -> Option<InspectorTextInputId> {
    if tag == TAG_INSPECTOR_TEXT_INPUT_BASE {
        Some(InspectorTextInputId::UiTextContent)
    } else if tag == TAG_INSPECTOR_TEXT_INPUT_BASE + 1 {
        Some(InspectorTextInputId::UiTextInputPlaceholder)
    } else {
        None
    }
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
    fn test_number_input_tag_encoding_and_resolution_coverage() {
        for idx in 0..57 {
            let id = InspectorNumberInputId::from_tag_index(idx)
                .expect("valid index 0..57 must have a variant");
            assert_eq!(id.to_tag_index(), idx);
            let tag = encode_inspector_number_input_tag(id);
            assert!(
                is_inspector_tag(tag),
                "is_inspector_tag must return true for number input {:?}",
                id
            );
            let resolved = resolve_inspector_number_input_tag(tag);
            assert!(
                resolved.is_some(),
                "resolve_inspector_number_input_tag must resolve tag for {:?}",
                id
            );
            assert_eq!(resolved.unwrap().0, id);
        }

        // Out of bounds check: 57..128 must be None
        for out_of_bounds in 57..128 {
            assert_eq!(
                InspectorNumberInputId::from_tag_index(out_of_bounds),
                None,
                "Tag index {} should be out of bounds",
                out_of_bounds
            );
            let invalid_tag = TAG_INSPECTOR_NUMBER_INPUT_BASE + out_of_bounds as u64;
            if out_of_bounds >= 64 {
                assert_eq!(resolve_inspector_number_input_tag(invalid_tag), None);
            }
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