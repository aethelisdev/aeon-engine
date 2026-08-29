// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Widgets (`iris-widgets`)
//!
//! Standard widget set and game-engine editor components for Iris UI.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod asset_card;
pub mod button;
pub mod input;
pub mod inspector;
pub mod menubar;
pub mod panel;
pub mod typography;

pub use asset_card::{AssetCardBuilder, TreeItemBuilder};
pub use button::{ButtonBuilder, TabBuilder};
pub use input::{
    CheckboxBuilder, DragValueBuilder, SliderBuilder, TextInputBuilder, TextInputState,
};
pub use inspector::{ColorPickerBuilder, DropdownBuilder, PropertyRowBuilder};
pub use menubar::{DropdownMenuBuilder, MenuBarBuilder};
pub use panel::PanelBuilder;
pub use typography::{LabelBuilder, SectionHeaderBuilder};

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Color, TextAlign, UiTree};

    #[test]
    fn test_text_input_state_utf8_safety() {
        let mut state = TextInputState::new("Ağaç");
        assert_eq!(state.buffer, "Ağaç");
        assert_eq!(state.cursor_byte_idx, "Ağaç".len());

        // Backspace 'ç' (2 bytes in UTF-8)
        state.backspace();
        assert_eq!(state.buffer, "Ağa");

        // Insert Turkish special character 'ı'
        state.insert_str("ı");
        assert_eq!(state.buffer, "Ağaı");

        // Move left across multi-byte character
        state.move_left();
        state.backspace();
        assert_eq!(state.buffer, "Ağı");
    }

    #[test]
    fn test_text_input_state_ime_composition() {
        let mut state = TextInputState::new("Tokyo: ");
        assert_eq!(state.display_text(), "Tokyo: ");

        // Set Japanese IME preedit
        state.set_ime_preedit("とうきょう", Some((0, 5)));
        assert_eq!(state.display_text(), "Tokyo: [とうきょう]");

        // Commit Japanese IME finalized string
        state.commit_ime("東京");
        assert_eq!(state.buffer, "Tokyo: 東京");
        assert_eq!(state.display_text(), "Tokyo: 東京");
        assert_eq!(state.ime_preedit, None);
    }

    #[test]
    fn test_panel_builder_dark_theme() {
        let mut tree = UiTree::new();
        let panel_id = PanelBuilder::new(&mut tree).dark_theme().build();
        let node = tree.get(panel_id).expect("Panel node must exist");

        assert_eq!(node.style.background_color, Color::hex("#101016"));
        assert_eq!(node.style.corner_radii.top_left, 4.0);
        assert!(node.style.box_shadow.is_some());
    }

    #[test]
    fn test_label_and_button_builder() {
        let mut tree = UiTree::new();
        let label_id = LabelBuilder::new(&mut tree, "Test Label")
            .font_size(16.0, 20.0)
            .color(Color::RED)
            .build();

        let node = tree.get(label_id).expect("Label node must exist");
        assert_eq!(node.text.as_deref(), Some("Test Label"));
        assert_eq!(node.font_size, 16.0);
        assert_eq!(node.text_color, Color::RED);

        let button_id = ButtonBuilder::new(&mut tree, "Submit").build();
        let btn_node = tree.get(button_id).expect("Button node must exist");
        assert_eq!(btn_node.text.as_deref(), Some("Submit"));
        assert_eq!(btn_node.text_align, TextAlign::Center);
    }

    #[test]
    fn test_game_engine_builders() {
        let mut tree = UiTree::new();
        let tab_id = TabBuilder::new(&mut tree, "Hierarchy", true).build();
        assert!(tree.get(tab_id).is_some());

        let section_id = SectionHeaderBuilder::new(&mut tree, "Transform", Color::GREEN).build();
        assert!(tree.get(section_id).is_some());

        let prop = PropertyRowBuilder::new_xyz(&mut tree, "Position", 0.0, 2.0, 8.0);
        assert!(tree.get(prop.build()).is_some());

        let input_id = TextInputBuilder::new(&mut tree, "player", "Search...", true).build();
        assert!(tree.get(input_id).is_some());

        let drag_id = DragValueBuilder::new(&mut tree, "X", 1.23, Color::RED, false).build();
        assert!(tree.get(drag_id).is_some());

        let tree_item_id = TreeItemBuilder::new(&mut tree, "Player Character", true).build();
        assert!(tree.get(tree_item_id).is_some());

        let asset_id =
            AssetCardBuilder::new(&mut tree, "shader.wgsl", "WGSL", Color::YELLOW).build();
        assert!(tree.get(asset_id).is_some());

        let checkbox_id = CheckboxBuilder::new(&mut tree, "Cast Shadows", true).build();
        assert!(tree.get(checkbox_id).is_some());

        let slider_id = SliderBuilder::new(&mut tree, "Roughness", 0.75, 0.0, 1.0).build();
        assert!(tree.get(slider_id).is_some());

        let color_id = ColorPickerBuilder::new(&mut tree, "Albedo", Color::BLUE).build();
        assert!(tree.get(color_id).is_some());

        let dropdown_id = DropdownBuilder::new(&mut tree, "Shading", "PBR Lit", false).build();
        assert!(tree.get(dropdown_id).is_some());

        let mut menu_bar = MenuBarBuilder::new(&mut tree, 1920.0);
        menu_bar.add_menu_button("File", false, false);
        menu_bar.add_action_button("▶ Play", Color::GREEN, Color::WHITE, false);
        let menu_id = menu_bar.build();
        assert!(tree.get(menu_id).is_some());
    }
}