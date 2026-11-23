// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Menu Catalog & Zero-Allocation Construction Handlers
//!
//! Provides the static catalog and declarative zero-`let` instantiation/removal
//! function pointers for all built-in attachable ECS components.
//!
//! Eliminates hardcoded `match` branches and split-brain architectures across
//! UI views and action dispatchers.

use super::super::types::ComponentCategory;
use crate::ui::iris_bridge::icons::{
    ICON_AUDIO, ICON_CUBE, ICON_EYE_OPEN, ICON_GEAR, ICON_LIGHT, ICON_ROTATE, ICON_WIREFRAME,
};

/// Helper predicate checking if an entity has component `T`.
#[inline]
pub fn has_comp<T: ae_ecs::Component>(world: &ae_ecs::World, entity: ae_ecs::Entity) -> bool {
    world.has::<T>(entity)
}

/// Generic zero-`let` constructor attaching default component `T` and marking transform dirty.
#[inline]
pub fn spawn_comp<T: ae_ecs::Component + Default>(
    world: &mut ae_ecs::World,
    entity: ae_ecs::Entity,
) {
    world
        .insert(entity, (T::default(), ae_plugin_api::TransformDirty))
        .ok();
}

/// Generic zero-`let` destructor removing component `T` and marking transform dirty.
#[inline]
pub fn remove_comp<T: ae_ecs::Component>(
    world: &mut ae_ecs::World,
    entity: ae_ecs::Entity,
) -> bool {
    world
        .remove_one::<T>(entity)
        .is_ok()
        .then(|| {
            world.insert_one(entity, ae_plugin_api::TransformDirty).ok();
        })
        .is_some()
}

/// Zero-`let` constructor for RigidBody, ensuring paired Collider and TransformDirty attachments.
#[inline]
pub fn spawn_rigidbody(world: &mut ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .insert_one(entity, ae_plugin_api::RigidBody::default())
        .ok();
    if !world.has::<ae_plugin_api::Collider>(entity) {
        world
            .insert_one(entity, ae_plugin_api::Collider::default())
            .ok();
    }
    world.insert_one(entity, ae_plugin_api::TransformDirty).ok();
}

/// Zero-`let` constructor for Collider, ensuring paired RigidBody and TransformDirty attachments.
#[inline]
pub fn spawn_collider(world: &mut ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .insert_one(entity, ae_plugin_api::Collider::default())
        .ok();
    if !world.has::<ae_plugin_api::RigidBody>(entity) {
        world
            .insert_one(entity, ae_plugin_api::RigidBody::default())
            .ok();
    }
    world.insert_one(entity, ae_plugin_api::TransformDirty).ok();
}

/// Zero-`let` constructor for AudioListener (which does not implement `Default`).
#[inline]
pub fn spawn_audio_listener(world: &mut ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .insert(
            entity,
            (ae_audio::AudioListener, ae_plugin_api::TransformDirty),
        )
        .ok();
}

/// Static metadata entry for an attachable ECS component in the Add Component cascading menu.
///
/// Serves as the single source of truth for display titles, categories, icons, presence tests,
/// default spawning, and dynamic removal.
#[derive(Debug, Clone, Copy)]
pub struct ComponentMenuEntry {
    /// Canonical component type name (used for tag hashing and entity reflection).
    pub component_name: &'static str,
    /// User-facing display title in the menu item.
    pub display_title: &'static str,
    /// Component category for grouping.
    pub category: ComponentCategory,
    /// Text / emoji fallback icon.
    pub icon: &'static str,
    /// Optional GPU texture atlas UV coordinates.
    pub atlas_icon: Option<[f32; 4]>,
    /// Function pointer checking whether the entity already has this component attached.
    pub has_component: fn(&ae_ecs::World, ae_ecs::Entity) -> bool,
    /// Function pointer instantiating and attaching the default component in `world`.
    pub spawn_default: fn(&mut ae_ecs::World, ae_ecs::Entity),
    /// Function pointer removing the component from `world`.
    pub remove_component: fn(&mut ae_ecs::World, ae_ecs::Entity) -> bool,
}

/// Canonical catalog of all built-in attachable ECS components in the Add Component menu.
///
/// Eliminates global registry locks and runtime trait object iteration for engine built-ins.
pub const BUILTIN_COMPONENT_MENU_CATALOG: &[ComponentMenuEntry] = &[
    // 1. Physics Components
    ComponentMenuEntry {
        component_name: "RigidBody",
        display_title: "RigidBody",
        category: ComponentCategory::Physics,
        icon: "⚙️",
        atlas_icon: Some(ICON_GEAR),
        has_component: has_comp::<ae_plugin_api::RigidBody>,
        spawn_default: spawn_rigidbody,
        remove_component: remove_comp::<ae_plugin_api::RigidBody>,
    },
    ComponentMenuEntry {
        component_name: "Collider",
        display_title: "3D Collider",
        category: ComponentCategory::Physics,
        icon: "📦",
        atlas_icon: Some(ICON_WIREFRAME),
        has_component: has_comp::<ae_plugin_api::Collider>,
        spawn_default: spawn_collider,
        remove_component: remove_comp::<ae_plugin_api::Collider>,
    },
    ComponentMenuEntry {
        component_name: "PhysicsMaterial",
        display_title: "Physics Material",
        category: ComponentCategory::Physics,
        icon: "🧪",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::PhysicsMaterial>,
        spawn_default: spawn_comp::<ae_plugin_api::PhysicsMaterial>,
        remove_component: remove_comp::<ae_plugin_api::PhysicsMaterial>,
    },
    // 2. Character & Gameplay Components
    ComponentMenuEntry {
        component_name: "CharacterController",
        display_title: "Character Controller",
        category: ComponentCategory::Gameplay,
        icon: "🚶",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::CharacterController>,
        spawn_default: spawn_comp::<ae_plugin_api::CharacterController>,
        remove_component: remove_comp::<ae_plugin_api::CharacterController>,
    },
    ComponentMenuEntry {
        component_name: "CharacterAction",
        display_title: "CharacterAction",
        category: ComponentCategory::Gameplay,
        icon: "💨",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::CharacterAction>,
        spawn_default: spawn_comp::<ae_plugin_api::CharacterAction>,
        remove_component: remove_comp::<ae_plugin_api::CharacterAction>,
    },
    ComponentMenuEntry {
        component_name: "PlayerTag",
        display_title: "Player Tag",
        category: ComponentCategory::Gameplay,
        icon: "👤",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::PlayerTag>,
        spawn_default: spawn_comp::<ae_plugin_api::PlayerTag>,
        remove_component: remove_comp::<ae_plugin_api::PlayerTag>,
    },
    ComponentMenuEntry {
        component_name: "Velocity",
        display_title: "Velocity",
        category: ComponentCategory::Gameplay,
        icon: "⚡",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::Velocity>,
        spawn_default: spawn_comp::<ae_plugin_api::Velocity>,
        remove_component: remove_comp::<ae_plugin_api::Velocity>,
    },
    ComponentMenuEntry {
        component_name: "Rotator",
        display_title: "Rotator",
        category: ComponentCategory::Gameplay,
        icon: "🔄",
        atlas_icon: Some(ICON_ROTATE),
        has_component: has_comp::<ae_plugin_api::Rotator>,
        spawn_default: spawn_comp::<ae_plugin_api::Rotator>,
        remove_component: remove_comp::<ae_plugin_api::Rotator>,
    },
    ComponentMenuEntry {
        component_name: "MovingPlatform",
        display_title: "Moving Platform",
        category: ComponentCategory::Gameplay,
        icon: "🚡",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::MovingPlatform>,
        spawn_default: spawn_comp::<ae_plugin_api::MovingPlatform>,
        remove_component: remove_comp::<ae_plugin_api::MovingPlatform>,
    },
    ComponentMenuEntry {
        component_name: "TriggerZone",
        display_title: "Trigger Zone",
        category: ComponentCategory::Gameplay,
        icon: "📡",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::TriggerZone>,
        spawn_default: spawn_comp::<ae_plugin_api::TriggerZone>,
        remove_component: remove_comp::<ae_plugin_api::TriggerZone>,
    },
    ComponentMenuEntry {
        component_name: "DestructibleTarget",
        display_title: "Destructible Target",
        category: ComponentCategory::Gameplay,
        icon: "🎯",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::DestructibleTarget>,
        spawn_default: spawn_comp::<ae_plugin_api::DestructibleTarget>,
        remove_component: remove_comp::<ae_plugin_api::DestructibleTarget>,
    },
    // 3. Rendering, LOD & Illumination Components
    ComponentMenuEntry {
        component_name: "Light",
        display_title: "Light",
        category: ComponentCategory::Rendering,
        icon: "💡",
        atlas_icon: Some(ICON_LIGHT),
        has_component: has_comp::<ae_plugin_api::Light>,
        spawn_default: spawn_comp::<ae_plugin_api::Light>,
        remove_component: remove_comp::<ae_plugin_api::Light>,
    },
    ComponentMenuEntry {
        component_name: "ModelId",
        display_title: "3D Model / Mesh",
        category: ComponentCategory::Rendering,
        icon: "📦",
        atlas_icon: Some(ICON_CUBE),
        has_component: has_comp::<ae_plugin_api::ModelId>,
        spawn_default: spawn_comp::<ae_plugin_api::ModelId>,
        remove_component: remove_comp::<ae_plugin_api::ModelId>,
    },
    ComponentMenuEntry {
        component_name: "Shape",
        display_title: "3D Shape Primitive",
        category: ComponentCategory::Rendering,
        icon: "🎲",
        atlas_icon: Some(ICON_CUBE),
        has_component: has_comp::<ae_plugin_api::Shape>,
        spawn_default: spawn_comp::<ae_plugin_api::Shape>,
        remove_component: remove_comp::<ae_plugin_api::Shape>,
    },
    ComponentMenuEntry {
        component_name: "LodGroup",
        display_title: "LOD Group",
        category: ComponentCategory::Rendering,
        icon: "👁️",
        atlas_icon: Some(ICON_EYE_OPEN),
        has_component: has_comp::<ae_plugin_api::LodGroup>,
        spawn_default: spawn_comp::<ae_plugin_api::LodGroup>,
        remove_component: remove_comp::<ae_plugin_api::LodGroup>,
    },
    // 4. Skeletal Animation Components
    ComponentMenuEntry {
        component_name: "AnimationPlayer",
        display_title: "Animation Player",
        category: ComponentCategory::Animation,
        icon: "🎬",
        atlas_icon: None,
        has_component: has_comp::<ae_animation::AnimationPlayer>,
        spawn_default: spawn_comp::<ae_animation::AnimationPlayer>,
        remove_component: remove_comp::<ae_animation::AnimationPlayer>,
    },
    // 5. Audio Components
    ComponentMenuEntry {
        component_name: "AudioSource",
        display_title: "Audio Source",
        category: ComponentCategory::Audio,
        icon: "🔊",
        atlas_icon: Some(ICON_AUDIO),
        has_component: has_comp::<ae_audio::AudioSource>,
        spawn_default: spawn_comp::<ae_audio::AudioSource>,
        remove_component: remove_comp::<ae_audio::AudioSource>,
    },
    ComponentMenuEntry {
        component_name: "AudioListener",
        display_title: "Audio Listener",
        category: ComponentCategory::Audio,
        icon: "👂",
        atlas_icon: None,
        has_component: has_comp::<ae_audio::AudioListener>,
        spawn_default: spawn_audio_listener,
        remove_component: remove_comp::<ae_audio::AudioListener>,
    },
    // 6. In-Game 2D UI & HUD Components
    ComponentMenuEntry {
        component_name: "UiElement",
        display_title: "2D Screen UI Element",
        category: ComponentCategory::UiHud,
        icon: "📐",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiElement>,
        spawn_default: spawn_comp::<ae_plugin_api::UiElement>,
        remove_component: remove_comp::<ae_plugin_api::UiElement>,
    },
    ComponentMenuEntry {
        component_name: "UiPanel",
        display_title: "UI Background Panel",
        category: ComponentCategory::UiHud,
        icon: "🔲",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiPanel>,
        spawn_default: spawn_comp::<ae_plugin_api::UiPanel>,
        remove_component: remove_comp::<ae_plugin_api::UiPanel>,
    },
    ComponentMenuEntry {
        component_name: "UiText",
        display_title: "UI Text Label",
        category: ComponentCategory::UiHud,
        icon: "🔤",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiText>,
        spawn_default: spawn_comp::<ae_plugin_api::UiText>,
        remove_component: remove_comp::<ae_plugin_api::UiText>,
    },
    ComponentMenuEntry {
        component_name: "UiProgressBar",
        display_title: "UI Progress Bar",
        category: ComponentCategory::UiHud,
        icon: "📊",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiProgressBar>,
        spawn_default: spawn_comp::<ae_plugin_api::UiProgressBar>,
        remove_component: remove_comp::<ae_plugin_api::UiProgressBar>,
    },
    ComponentMenuEntry {
        component_name: "UiButton",
        display_title: "Interactive Button",
        category: ComponentCategory::UiHud,
        icon: "🔘",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiButton>,
        spawn_default: spawn_comp::<ae_plugin_api::UiButton>,
        remove_component: remove_comp::<ae_plugin_api::UiButton>,
    },
    ComponentMenuEntry {
        component_name: "UiImage",
        display_title: "UI Sprite Image",
        category: ComponentCategory::UiHud,
        icon: "🖼️",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiImage>,
        spawn_default: spawn_comp::<ae_plugin_api::UiImage>,
        remove_component: remove_comp::<ae_plugin_api::UiImage>,
    },
    ComponentMenuEntry {
        component_name: "UiSlider",
        display_title: "UI Numeric Slider",
        category: ComponentCategory::UiHud,
        icon: "🎚️",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiSlider>,
        spawn_default: spawn_comp::<ae_plugin_api::UiSlider>,
        remove_component: remove_comp::<ae_plugin_api::UiSlider>,
    },
    ComponentMenuEntry {
        component_name: "UiCheckbox",
        display_title: "UI Checkbox",
        category: ComponentCategory::UiHud,
        icon: "☑️",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiCheckbox>,
        spawn_default: spawn_comp::<ae_plugin_api::UiCheckbox>,
        remove_component: remove_comp::<ae_plugin_api::UiCheckbox>,
    },
    ComponentMenuEntry {
        component_name: "UiTextInput",
        display_title: "UI Text Input Field",
        category: ComponentCategory::UiHud,
        icon: "📝",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiTextInput>,
        spawn_default: spawn_comp::<ae_plugin_api::UiTextInput>,
        remove_component: remove_comp::<ae_plugin_api::UiTextInput>,
    },
    ComponentMenuEntry {
        component_name: "UiLayoutGroup",
        display_title: "UI Layout Group",
        category: ComponentCategory::UiHud,
        icon: "🗂️",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::UiLayoutGroup>,
        spawn_default: spawn_comp::<ae_plugin_api::UiLayoutGroup>,
        remove_component: remove_comp::<ae_plugin_api::UiLayoutGroup>,
    },
    ComponentMenuEntry {
        component_name: "PlayerHealthBarTag",
        display_title: "Health Bar Tag",
        category: ComponentCategory::UiHud,
        icon: "❤️",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::PlayerHealthBarTag>,
        spawn_default: spawn_comp::<ae_plugin_api::PlayerHealthBarTag>,
        remove_component: remove_comp::<ae_plugin_api::PlayerHealthBarTag>,
    },
    ComponentMenuEntry {
        component_name: "ScoreDisplayTag",
        display_title: "Score Display Tag",
        category: ComponentCategory::UiHud,
        icon: "🏆",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::ScoreDisplayTag>,
        spawn_default: spawn_comp::<ae_plugin_api::ScoreDisplayTag>,
        remove_component: remove_comp::<ae_plugin_api::ScoreDisplayTag>,
    },
    ComponentMenuEntry {
        component_name: "ReticleTag",
        display_title: "Reticle Tag",
        category: ComponentCategory::UiHud,
        icon: "🎯",
        atlas_icon: None,
        has_component: has_comp::<ae_plugin_api::ReticleTag>,
        spawn_default: spawn_comp::<ae_plugin_api::ReticleTag>,
        remove_component: remove_comp::<ae_plugin_api::ReticleTag>,
    },
];

/// Searches the built-in component catalog for an entry matching `type_name`.
#[inline]
#[must_use]
pub fn find_builtin_component_entry(type_name: &str) -> Option<&'static ComponentMenuEntry> {
    BUILTIN_COMPONENT_MENU_CATALOG
        .iter()
        .find(|e| e.component_name == type_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_catalog_entries_spawn_detect_and_remove() {
        assert_eq!(
            BUILTIN_COMPONENT_MENU_CATALOG.len(),
            31,
            "Built-in catalog must contain exactly 31 attachable ECS components"
        );

        for entry in BUILTIN_COMPONENT_MENU_CATALOG {
            let mut world = ae_ecs::World::new();
            let entity = world.spawn(());

            assert!(
                !(entry.has_component)(&world, entity),
                "Fresh entity must not have component '{}'",
                entry.component_name
            );

            (entry.spawn_default)(&mut world, entity);

            assert!(
                (entry.has_component)(&world, entity),
                "Component '{}' must be detected on entity after spawn_default",
                entry.component_name
            );

            assert!(
                world.has::<ae_plugin_api::TransformDirty>(entity),
                "Component '{}' must flag TransformDirty upon attachment",
                entry.component_name
            );

            let removed = (entry.remove_component)(&mut world, entity);
            assert!(
                removed,
                "Component '{}' must be successfully removed via remove_component",
                entry.component_name
            );

            assert!(
                !(entry.has_component)(&world, entity),
                "Component '{}' must no longer be present on entity after removal",
                entry.component_name
            );
        }
    }

    #[test]
    fn test_rigidbody_and_collider_paired_spawning() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn(());

        spawn_rigidbody(&mut world, entity);
        assert!(
            world.has::<ae_plugin_api::RigidBody>(entity),
            "RigidBody must be attached"
        );
        assert!(
            world.has::<ae_plugin_api::Collider>(entity),
            "Paired default Collider must be attached when spawning RigidBody"
        );

        let entity2 = world.spawn(());
        spawn_collider(&mut world, entity2);
        assert!(
            world.has::<ae_plugin_api::Collider>(entity2),
            "Collider must be attached"
        );
        assert!(
            world.has::<ae_plugin_api::RigidBody>(entity2),
            "Paired default RigidBody must be attached when spawning Collider"
        );
    }
}