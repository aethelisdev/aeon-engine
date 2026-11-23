// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Bottom Action Bar Builder
//!
//! Renders `➕ Add Component` and `💾 Save as Prefab` action buttons using declarative [`UiScope::dual_action_bar`].

use crate::ui::types::EngineUiAction;
use irisui::prelude::*;

/// Semantic tag for the `➕ Add Component` button in the Inspector footer.
pub const TAG_INSPECTOR_ADD_COMPONENT: u64 = UiTag::from_str("ae.inspector.add_component").raw();

/// Resolves the sanitized prefab file path for the specified entity from its ECS [`ae_plugin_api::Name`] component.
///
/// Replaces non-alphanumeric characters (other than `_` and `-`) with `_` and falls back to `"prefab"`
/// when the entity has no name or its sanitized name is empty.
#[must_use]
pub fn prefab_file_path_for_entity(
    world: &ae_ecs::World,
    entity: ae_ecs::Entity,
) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(
        "{}/{}.aeprefab",
        if std::path::Path::new("Content").is_dir() {
            "Content/Prefabs"
        } else {
            "assets/prefabs"
        },
        world
            .inspect::<ae_plugin_api::Name, _>(entity, |n| {
                n.0.trim()
                    .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_")
            })
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "prefab".to_string())
    ))
}

/// Builds the Inspector bottom action bar declaratively into the parent [`UiScope`],
/// returning [`EngineUiAction::SaveEntityAsPrefab`] directly when the `💾 Save as Prefab`
/// button is clicked.
pub fn build_inspector_footer(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    selected_entity: Option<ae_ecs::Entity>,
) -> Option<EngineUiAction> {
    scope
        .dual_action_bar_auto(
            "InspectorFooter",
            ActionBarButton::new("AddComponentBtn", "Add Component")
                .with_icon(
                    "AddComponentPlusIcon",
                    crate::ui::iris_bridge::icons::ICON_PLUS,
                )
                .with_text_name("AddComponentBtnText")
                .with_text_align(TextAlign::Left)
                .with_padding(Insets::new(0.0, 6.0, 0.0, 6.0))
                .with_tag(TAG_INSPECTOR_ADD_COMPONENT)
                .with_popup_toggle(TAG_INSPECTOR_ADD_COMPONENT),
            ActionBarButton::new("SavePrefabBtn", "💾 Save as Prefab")
                .with_text_name("SavePrefabBtnText")
                .with_text_align(TextAlign::Center),
        )
        .then_secondary(|| {
            selected_entity.map(|entity| {
                EngineUiAction::SaveEntityAsPrefab(
                    entity,
                    prefab_file_path_for_entity(world, entity),
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
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Name("Hero Guard".to_string()),));
        let panel_rect = Rect::new(0.0, 0.0, 320.0, 800.0);
        tree.set_root_bounds(panel_rect);

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.configure_container("TestRoot", Style::new().flex_col(), false);
            let ui_action = build_inspector_footer(&mut scope, &world, Some(entity));
            assert!(ui_action.is_none());
            scope.finish_layout(panel_rect);
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

        // Simulate clicking AddComponentBtn and verify scoped popup tag activation in UiTree
        let add_rect = tree.assert_node_rect("AddComponentBtn");
        let click_add_pt = Point::new(add_rect.x + 10.0, add_rect.y + 10.0);
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_add_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_add_pt,
        });
        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = build_inspector_footer(&mut scope, &world, Some(entity));
        }
        assert!(
            tree.is_popup_tag_active(TAG_INSPECTOR_ADD_COMPONENT),
            "Clicking Add Component button must activate TAG_INSPECTOR_ADD_COMPONENT in UiTree"
        );

        // Toggle AddComponentBtn off cleanly before clicking SavePrefabBtn
        tree.event_state.clear_frame_clicks();
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_add_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_add_pt,
        });
        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = build_inspector_footer(&mut scope, &world, Some(entity));
        }
        assert!(
            !tree.is_popup_tag_active(TAG_INSPECTOR_ADD_COMPONENT),
            "Clicking Add Component button a second time must toggle TAG_INSPECTOR_ADD_COMPONENT off"
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
            build_inspector_footer(&mut scope, &world, Some(entity))
        };
        let Some(EngineUiAction::SaveEntityAsPrefab(ent, path)) = emitted else {
            panic!("Expected EngineUiAction::SaveEntityAsPrefab");
        };
        assert_eq!(ent, entity);
        assert_eq!(
            path,
            std::path::PathBuf::from("assets/prefabs/Hero_Guard.aeprefab")
        );
    }
}