// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Entity Name and Identity Header Builder
//!
//! Renders the top entity rename input box and label using high-level
//! declarative [`UiScope`] property primitives and flexbox layout.
//!

use super::types::InspectorPanelParams;
use irisui::prelude::*;

/// Returns the persistent 64-bit semantic tag identifying the entity rename text input box for $O(1)$ hit-testing.
#[inline]
pub fn entity_name_input_tag() -> u64 {
    hash_label_with_seed(0, "InspectorEntityName")
}

/// Resolves whether a semantic tag corresponds to the entity name rename input box.
#[inline]
pub fn resolve_entity_name_input_tag(tag: u64) -> bool {
    tag == hash_label_with_seed(0, "InspectorEntityName")
}

/// Output node handles created during Inspector header construction.
pub struct HeaderNodes {
    /// Name input container node ID.
    pub name_box_id: WidgetId,
    /// Name text node ID.
    pub name_text_id: WidgetId,
}

/// Builds the top Entity Name header declaratively into the parent [`UiScope`].
///
/// Encapsulates the entity name editor within a pure declarative [`UiScope`] property row,
/// managing flex layout, hover states, caret blinking, and focus rings without manual
/// coordinate arithmetic or imperative target buffers.
pub fn build_entity_header(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    params: &InspectorPanelParams<'_>,
) -> HeaderNodes {
    let padding_x = 6.0;
    let header_w = params.panel_rect.width - padding_x * 2.0;

    let is_editing = params.active_rename_buffer.is_some();
    let entity_name = if let Some(buf) = params.active_rename_buffer {
        buf.to_string()
    } else {
        params
            .world
            .get::<&ae_core::ecs::Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| format!("Entity {:?}", entity))
    };

    let mut captured_box_id = WidgetId::default();
    let mut captured_text_id = WidgetId::default();

    let header_style = Style::new()
        .width(header_w)
        .height(30.0)
        .margin_insets(Insets::new(4.0, padding_x, 4.0, padding_x))
        .flex_col();

    scope.container_named("EntityHeaderCard", header_style, |header_scope| {
        let opts = PropertyTextOptions::new()
            .with_editing(is_editing, params.blink_caret)
            .with_all_selected(params.is_rename_all_selected)
            .with_label_width(60.0)
            .with_custom_tag(entity_name_input_tag());

        let resp = header_scope.property_text_with_options("🏷 Name:", &entity_name, opts);
        captured_box_id = resp.box_id;
        captured_text_id = resp.text_id;
    });

    HeaderNodes {
        name_box_id: captured_box_id,
        name_text_id: captured_text_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::Name;

    #[test]
    fn test_declarative_entity_header_structure_and_layout() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((Name("Dynamic Cube".to_string()),));

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
            hovered_tag: None,
        };

        let mut scope = UiScope::new(&mut tree, root);
        let nodes = build_entity_header(&mut scope, entity, &params);
        scope.finish_layout(params.panel_rect);

        assert!(nodes.name_box_id != WidgetId::default());
        assert!(nodes.name_text_id != WidgetId::default());

        let box_node = tree.get(nodes.name_box_id).expect("Box node must exist");
        assert_eq!(box_node.tag, entity_name_input_tag());
        assert_eq!(box_node.role, WidgetRole::TextInput);
        assert_eq!(box_node.cursor, Some(WidgetCursor::Text));
        assert!(box_node.computed_rect.width > 100.0);
        assert!(box_node.computed_rect.height >= 24.0);

        let text_node = tree.get(nodes.name_text_id).expect("Text node must exist");
        assert_eq!(text_node.text.as_deref(), Some("Dynamic Cube"));
    }
}