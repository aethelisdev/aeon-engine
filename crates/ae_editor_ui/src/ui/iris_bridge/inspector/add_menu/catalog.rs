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
pub fn has_comp<T: hecs::Component>(world: &hecs::World, entity: hecs::Entity) -> bool {
    world.get::<&T>(entity).is_ok()
}

/// Generic zero-`let` constructor attaching default component `T` and marking transform dirty.
#[inline]
pub fn spawn_comp<T: hecs::Component + Default>(world: &mut hecs::World, entity: hecs::Entity) {
    drop(world.insert(entity, (T::default(), ae_core::ecs::TransformDirty)));
}

/// Generic zero-`let` destructor removing component `T` and marking transform dirty.
#[inline]
pub fn remove_comp<T: hecs::Component>(world: &mut hecs::World, entity: hecs::Entity) -> bool {
    world
        .remove_one::<T>(entity)
        .is_ok()
        .then(|| drop(world.insert_one(entity, ae_core::ecs::TransformDirty)))
        .is_some()
}

/// Zero-`let` constructor for RigidBody, ensuring paired Collider and TransformDirty attachments.
#[inline]
pub fn spawn_rigidbody(world: &mut hecs::World, entity: hecs::Entity) {
    drop(world.insert_one(entity, ae_core::ecs::RigidBody::default()));
    if world.get::<&ae_core::ecs::Collider>(entity).is_err() {
        drop(world.insert_one(entity, ae_core::ecs::Collider::default()));
    }
    drop(world.insert_one(entity, ae_core::ecs::TransformDirty));
}

/// Zero-`let` constructor for Collider, ensuring paired RigidBody and TransformDirty attachments.
#[inline]
pub fn spawn_collider(world: &mut hecs::World, entity: hecs::Entity) {
    drop(world.insert_one(entity, ae_core::ecs::Collider::default()));
    if world.get::<&ae_core::ecs::RigidBody>(entity).is_err() {
        drop(world.insert_one(entity, ae_core::ecs::RigidBody::default()));
    }
    drop(world.insert_one(entity, ae_core::ecs::TransformDirty));
}

/// Zero-`let` constructor for AudioListener (which does not implement `Default`).
#[inline]
pub fn spawn_audio_listener(world: &mut hecs::World, entity: hecs::Entity) {
    drop(world.insert(
        entity,
        (ae_audio::AudioListener, ae_core::ecs::TransformDirty),
    ));
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
    pub has_component: fn(&hecs::World, hecs::Entity) -> bool,
    /// Function pointer instantiating and attaching the default component in `world`.
    pub spawn_default: fn(&mut hecs::World, hecs::Entity),
    /// Function pointer removing the component from `world`.
    pub remove_component: fn(&mut hecs::World, hecs::Entity) -> bool,
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
        has_component: has_comp::<ae_core::ecs::RigidBody>,
        spawn_default: spawn_rigidbody,
        remove_component: remove_comp::<ae_core::ecs::RigidBody>,
    },
    ComponentMenuEntry {
        component_name: "Collider",
        display_title: "3D Collider",
        category: ComponentCategory::Physics,
        icon: "📦",
        atlas_icon: Some(ICON_WIREFRAME),
        has_component: has_comp::<ae_core::ecs::Collider>,
        spawn_default: spawn_collider,
        remove_component: remove_comp::<ae_core::ecs::Collider>,
    },
    ComponentMenuEntry {
        component_name: "PhysicsMaterial",
        display_title: "Physics Material",
        category: ComponentCategory::Physics,
        icon: "🧪",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::PhysicsMaterial>,
        spawn_default: spawn_comp::<ae_core::ecs::PhysicsMaterial>,
        remove_component: remove_comp::<ae_core::ecs::PhysicsMaterial>,
    },
    // 2. Character & Gameplay Components
    ComponentMenuEntry {
        component_name: "CharacterController",
        display_title: "Character Controller",
        category: ComponentCategory::Gameplay,
        icon: "🚶",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::CharacterController>,
        spawn_default: spawn_comp::<ae_core::ecs::CharacterController>,
        remove_component: remove_comp::<ae_core::ecs::CharacterController>,
    },
    ComponentMenuEntry {
        component_name: "CharacterAction",
        display_title: "CharacterAction",
        category: ComponentCategory::Gameplay,
        icon: "💨",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::CharacterAction>,
        spawn_default: spawn_comp::<ae_core::ecs::CharacterAction>,
        remove_component: remove_comp::<ae_core::ecs::CharacterAction>,
    },
    ComponentMenuEntry {
        component_name: "PlayerTag",
        display_title: "Player Tag",
        category: ComponentCategory::Gameplay,
        icon: "👤",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::PlayerTag>,
        spawn_default: spawn_comp::<ae_core::ecs::PlayerTag>,
        remove_component: remove_comp::<ae_core::ecs::PlayerTag>,
    },
    ComponentMenuEntry {
        component_name: "Velocity",
        display_title: "Velocity",
        category: ComponentCategory::Gameplay,
        icon: "⚡",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::Velocity>,
        spawn_default: spawn_comp::<ae_core::ecs::Velocity>,
        remove_component: remove_comp::<ae_core::ecs::Velocity>,
    },
    ComponentMenuEntry {
        component_name: "Rotator",
        display_title: "Rotator",
        category: ComponentCategory::Gameplay,
        icon: "🔄",
        atlas_icon: Some(ICON_ROTATE),
        has_component: has_comp::<ae_core::ecs::Rotator>,
        spawn_default: spawn_comp::<ae_core::ecs::Rotator>,
        remove_component: remove_comp::<ae_core::ecs::Rotator>,
    },
    ComponentMenuEntry {
        component_name: "MovingPlatform",
        display_title: "Moving Platform",
        category: ComponentCategory::Gameplay,
        icon: "🚡",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::MovingPlatform>,
        spawn_default: spawn_comp::<ae_core::ecs::MovingPlatform>,
        remove_component: remove_comp::<ae_core::ecs::MovingPlatform>,
    },
    ComponentMenuEntry {
        component_name: "TriggerZone",
        display_title: "Trigger Zone",
        category: ComponentCategory::Gameplay,
        icon: "📡",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::TriggerZone>,
        spawn_default: spawn_comp::<ae_core::ecs::TriggerZone>,
        remove_component: remove_comp::<ae_core::ecs::TriggerZone>,
    },
    ComponentMenuEntry {
        component_name: "DestructibleTarget",
        display_title: "Destructible Target",
        category: ComponentCategory::Gameplay,
        icon: "🎯",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::DestructibleTarget>,
        spawn_default: spawn_comp::<ae_core::ecs::DestructibleTarget>,
        remove_component: remove_comp::<ae_core::ecs::DestructibleTarget>,
    },
    // 3. Rendering, LOD & Illumination Components
    ComponentMenuEntry {
        component_name: "Light",
        display_title: "Light",
        category: ComponentCategory::Rendering,
        icon: "💡",
        atlas_icon: Some(ICON_LIGHT),
        has_component: has_comp::<ae_core::ecs::Light>,
        spawn_default: spawn_comp::<ae_core::ecs::Light>,
        remove_component: remove_comp::<ae_core::ecs::Light>,
    },
    ComponentMenuEntry {
        component_name: "ModelId",
        display_title: "3D Model / Mesh",
        category: ComponentCategory::Rendering,
        icon: "📦",
        atlas_icon: Some(ICON_CUBE),
        has_component: has_comp::<ae_core::ecs::ModelId>,
        spawn_default: spawn_comp::<ae_core::ecs::ModelId>,
        remove_component: remove_comp::<ae_core::ecs::ModelId>,
    },
    ComponentMenuEntry {
        component_name: "Shape",
        display_title: "3D Shape Primitive",
        category: ComponentCategory::Rendering,
        icon: "🎲",
        atlas_icon: Some(ICON_CUBE),
        has_component: has_comp::<ae_core::ecs::Shape>,
        spawn_default: spawn_comp::<ae_core::ecs::Shape>,
        remove_component: remove_comp::<ae_core::ecs::Shape>,
    },
    ComponentMenuEntry {
        component_name: "LodGroup",
        display_title: "LOD Group",
        category: ComponentCategory::Rendering,
        icon: "👁️",
        atlas_icon: Some(ICON_EYE_OPEN),
        has_component: has_comp::<ae_core::ecs::LodGroup>,
        spawn_default: spawn_comp::<ae_core::ecs::LodGroup>,
        remove_component: remove_comp::<ae_core::ecs::LodGroup>,
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
        has_component: has_comp::<ae_core::ecs::UiElement>,
        spawn_default: spawn_comp::<ae_core::ecs::UiElement>,
        remove_component: remove_comp::<ae_core::ecs::UiElement>,
    },
    ComponentMenuEntry {
        component_name: "UiPanel",
        display_title: "UI Background Panel",
        category: ComponentCategory::UiHud,
        icon: "🔲",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiPanel>,
        spawn_default: spawn_comp::<ae_core::ecs::UiPanel>,
        remove_component: remove_comp::<ae_core::ecs::UiPanel>,
    },
    ComponentMenuEntry {
        component_name: "UiText",
        display_title: "UI Text Label",
        category: ComponentCategory::UiHud,
        icon: "🔤",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiText>,
        spawn_default: spawn_comp::<ae_core::ecs::UiText>,
        remove_component: remove_comp::<ae_core::ecs::UiText>,
    },
    ComponentMenuEntry {
        component_name: "UiProgressBar",
        display_title: "UI Progress Bar",
        category: ComponentCategory::UiHud,
        icon: "📊",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiProgressBar>,
        spawn_default: spawn_comp::<ae_core::ecs::UiProgressBar>,
        remove_component: remove_comp::<ae_core::ecs::UiProgressBar>,
    },
    ComponentMenuEntry {
        component_name: "UiButton",
        display_title: "Interactive Button",
        category: ComponentCategory::UiHud,
        icon: "🔘",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiButton>,
        spawn_default: spawn_comp::<ae_core::ecs::UiButton>,
        remove_component: remove_comp::<ae_core::ecs::UiButton>,
    },
    ComponentMenuEntry {
        component_name: "UiImage",
        display_title: "UI Sprite Image",
        category: ComponentCategory::UiHud,
        icon: "🖼️",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiImage>,
        spawn_default: spawn_comp::<ae_core::ecs::UiImage>,
        remove_component: remove_comp::<ae_core::ecs::UiImage>,
    },
    ComponentMenuEntry {
        component_name: "UiSlider",
        display_title: "UI Numeric Slider",
        category: ComponentCategory::UiHud,
        icon: "🎚️",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiSlider>,
        spawn_default: spawn_comp::<ae_core::ecs::UiSlider>,
        remove_component: remove_comp::<ae_core::ecs::UiSlider>,
    },
    ComponentMenuEntry {
        component_name: "UiCheckbox",
        display_title: "UI Checkbox",
        category: ComponentCategory::UiHud,
        icon: "☑️",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiCheckbox>,
        spawn_default: spawn_comp::<ae_core::ecs::UiCheckbox>,
        remove_component: remove_comp::<ae_core::ecs::UiCheckbox>,
    },
    ComponentMenuEntry {
        component_name: "UiTextInput",
        display_title: "UI Text Input Field",
        category: ComponentCategory::UiHud,
        icon: "📝",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiTextInput>,
        spawn_default: spawn_comp::<ae_core::ecs::UiTextInput>,
        remove_component: remove_comp::<ae_core::ecs::UiTextInput>,
    },
    ComponentMenuEntry {
        component_name: "UiLayoutGroup",
        display_title: "UI Layout Group",
        category: ComponentCategory::UiHud,
        icon: "🗂️",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::UiLayoutGroup>,
        spawn_default: spawn_comp::<ae_core::ecs::UiLayoutGroup>,
        remove_component: remove_comp::<ae_core::ecs::UiLayoutGroup>,
    },
    ComponentMenuEntry {
        component_name: "PlayerHealthBarTag",
        display_title: "Health Bar Tag",
        category: ComponentCategory::UiHud,
        icon: "❤️",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::PlayerHealthBarTag>,
        spawn_default: spawn_comp::<ae_core::ecs::PlayerHealthBarTag>,
        remove_component: remove_comp::<ae_core::ecs::PlayerHealthBarTag>,
    },
    ComponentMenuEntry {
        component_name: "ScoreDisplayTag",
        display_title: "Score Display Tag",
        category: ComponentCategory::UiHud,
        icon: "🏆",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::ScoreDisplayTag>,
        spawn_default: spawn_comp::<ae_core::ecs::ScoreDisplayTag>,
        remove_component: remove_comp::<ae_core::ecs::ScoreDisplayTag>,
    },
    ComponentMenuEntry {
        component_name: "ReticleTag",
        display_title: "Reticle Tag",
        category: ComponentCategory::UiHud,
        icon: "🎯",
        atlas_icon: None,
        has_component: has_comp::<ae_core::ecs::ReticleTag>,
        spawn_default: spawn_comp::<ae_core::ecs::ReticleTag>,
        remove_component: remove_comp::<ae_core::ecs::ReticleTag>,
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
            let mut world = hecs::World::new();
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
                world.get::<&ae_core::ecs::TransformDirty>(entity).is_ok(),
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
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        spawn_rigidbody(&mut world, entity);
        assert!(
            world.get::<&ae_core::ecs::RigidBody>(entity).is_ok(),
            "RigidBody must be attached"
        );
        assert!(
            world.get::<&ae_core::ecs::Collider>(entity).is_ok(),
            "Paired default Collider must be attached when spawning RigidBody"
        );

        let entity2 = world.spawn(());
        spawn_collider(&mut world, entity2);
        assert!(
            world.get::<&ae_core::ecs::Collider>(entity2).is_ok(),
            "Collider must be attached"
        );
        assert!(
            world.get::<&ae_core::ecs::RigidBody>(entity2).is_ok(),
            "Paired default RigidBody must be attached when spawning Collider"
        );
    }
}