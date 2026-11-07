// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Bottom Action Bar Builder
//!
//! Renders `➕ Add Component` and `💾 Save as Prefab` action buttons using declarative [`UiScope::dual_action_bar`].

use super::tags::TAG_INSPECTOR_ADD_COMPONENT;
use super::types::{InspectorAction, InspectorPanelParams};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;

/// Resolves the sanitized prefab file path for the specified entity from its ECS [`ae_core::ecs::Name`] component.
///
/// Replaces non-alphanumeric characters (other than `_` and `-`) with `_` and falls back to `"prefab"`
/// when the entity has no name or its sanitized name is empty.
#[must_use]
pub fn prefab_file_path_for_entity(
    world: &hecs::World,
    entity: hecs::Entity,
) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(
        "assets/prefabs/{}.prefab.json",
        world
            .get::<&ae_core::ecs::Name>(entity)
            .ok()
            .map(|n| {
                n.0.trim()
                    .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_")
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "prefab".to_string())
    ))
}

/// Builds the Inspector bottom action bar declaratively into the parent [`UiScope`],
/// returning [`EngineUiAction::SaveEntityAsPrefab`] directly when the `💾 Save as Prefab`
/// button is clicked.
pub fn build_inspector_footer(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    actions: &mut Vec<InspectorAction>,
) -> Option<EngineUiAction> {
    scope
        .dual_action_bar(
            "InspectorFooter",
            params.panel_rect.width,
            ActionBarButton::new("AddComponentBtn", "Add Component")
                .with_icon(
                    "AddComponentPlusIcon",
                    crate::ui::iris_bridge::icons::ICON_PLUS,
                )
                .with_text_name("AddComponentBtnText")
                .with_text_align(TextAlign::Left)
                .with_padding(Insets::new(0.0, 6.0, 0.0, 6.0))
                .with_active(params.is_add_menu_open)
                .with_tag(TAG_INSPECTOR_ADD_COMPONENT),
            ActionBarButton::new("SavePrefabBtn", "💾 Save as Prefab")
                .with_text_name("SavePrefabBtnText")
                .with_text_align(TextAlign::Center),
        )
        .on_primary_click(|| {
            if params.is_add_menu_open {
                actions.push(InspectorAction::CloseAddComponentMenu);
            } else {
                actions.push(InspectorAction::OpenAddComponentMenu(Point::ZERO));
            }
        })
        .then_secondary(|| {
            params.selected_entity.map(|entity| {
                EngineUiAction::SaveEntityAsPrefab(
                    entity,
                    prefab_file_path_for_entity(params.world, entity),
                )
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_inspector_footer_layout_and_direct_prefab_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Name("Hero Guard".to_string()),));

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(12.0, 775.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.configure_container("TestRoot", Style::new().flex_col(), false);
            let ui_action = build_inspector_footer(&mut scope, &params, &mut actions);
            assert!(ui_action.is_none());
            scope.finish_layout(params.panel_rect);
        }

        let add_btn = tree.assert_node("AddComponentBtn", WidgetRole::Button);
        let save_btn = tree.assert_node("SavePrefabBtn", WidgetRole::Button);

        assert_eq!(add_btn.tag, TAG_INSPECTOR_ADD_COMPONENT);
        assert_ne!(save_btn.tag, 0);

        assert_eq!(add_btn.computed_rect.width, 148.0);
        assert_eq!(add_btn.computed_rect.height, 24.0);
        assert_eq!(save_btn.computed_rect.width, 148.0);
        assert_eq!(save_btn.computed_rect.height, 24.0);

        assert_eq!(
            add_btn.style.hover_background,
            Some(Color::rgba(0.200, 0.208, 0.235, 1.0))
        );
        assert_eq!(
            save_btn.style.hover_background,
            Some(Color::rgba(0.200, 0.208, 0.235, 1.0))
        );

        // Simulate clicking SavePrefabBtn and verify direct EngineUiAction::SaveEntityAsPrefab emission
        let save_rect = tree.assert_node_rect("SavePrefabBtn");
        let click_pt = Point::new(save_rect.x + 10.0, save_rect.y + 10.0);
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        let emitted = {
            let mut scope = UiScope::new(&mut tree, root);
            build_inspector_footer(&mut scope, &params, &mut actions)
        };
        let Some(EngineUiAction::SaveEntityAsPrefab(ent, path)) = emitted else {
            panic!("Expected EngineUiAction::SaveEntityAsPrefab");
        };
        assert_eq!(ent, entity);
        assert_eq!(
            path,
            std::path::PathBuf::from("assets/prefabs/Hero_Guard.prefab.json")
        );
        assert!(actions.is_empty());
    }
}