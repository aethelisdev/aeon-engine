// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Authoritative catalog and ECS attachment/removal handlers for built-in components.
//!
//! Serves as the single source of truth shared by editor component actions and UI menus
//! without coupling ECS reflection/spawning to UI crates.

/// High-level functional domain grouping for built-in and dynamic ECS components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinComponentCategory {
    /// Skeletal and keyframe animation components.
    Animation,
    /// Positional and ambient audio source components.
    Audio,
    /// Gameplay logic, character controllers, actions, and tags.
    Gameplay,
    /// ECS Scene hierarchy, parenting, and transform components.
    Hierarchy,
    /// Rigidbodies, colliders, and physics materials.
    Physics,
    /// Lights, cameras, meshes, and material renderers.
    Rendering,
    /// In-game 2D UI and HUD designer components.
    UiHud,
    /// Dynamic scripting and custom reflected components.
    CustomDynamic,
}

/// Helper predicate checking if an entity currently owns component `T`.
#[inline]
pub fn has_comp<T: ae_ecs::Component>(world: &ae_ecs::World, entity: ae_ecs::Entity) -> bool {
    world.has::<T>(entity)
}

/// Attaches default component `T` to `entity` and flags `TransformDirty`.
#[inline]
pub fn spawn_comp<T: ae_ecs::Component + Default>(
    world: &mut ae_ecs::World,
    entity: ae_ecs::Entity,
) {
    world
        .insert(entity, (T::default(), ae_plugin_api::TransformDirty))
        .ok();
}

/// Removes component `T` from `entity` and flags `TransformDirty` if removed.
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

/// Attaches [`ae_plugin_api::RigidBody`], ensuring a paired default [`ae_plugin_api::Collider`].
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

/// Attaches [`ae_plugin_api::Collider`], ensuring a paired default [`ae_plugin_api::RigidBody`].
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

/// Attaches [`ae_audio::AudioListener`] (unit struct without `Default`) and marks transform dirty.
#[inline]
pub fn spawn_audio_listener(world: &mut ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .insert(
            entity,
            (ae_audio::AudioListener, ae_plugin_api::TransformDirty),
        )
        .ok();
}

/// Static runtime descriptor for an attachable built-in ECS component.
#[derive(Debug, Clone, Copy)]
pub struct BuiltinComponentSpec {
    /// Canonical component type name used for tag hashing and reflection lookup.
    pub component_name: &'static str,
    /// Human-readable display title shown in component menus.
    pub display_title: &'static str,
    /// Functional domain category.
    pub category: BuiltinComponentCategory,
    /// Function pointer testing whether `entity` already owns this component.
    pub has_component: fn(&ae_ecs::World, ae_ecs::Entity) -> bool,
    /// Function pointer attaching the default component onto `entity`.
    pub spawn_default: fn(&mut ae_ecs::World, ae_ecs::Entity),
    /// Function pointer removing this component from `entity`.
    pub remove_component: fn(&mut ae_ecs::World, ae_ecs::Entity) -> bool,
}

macro_rules! builtin_entry {
    ($name:literal, $title:literal, $cat:ident, $ty:ty) => {
        BuiltinComponentSpec {
            component_name: $name,
            display_title: $title,
            category: BuiltinComponentCategory::$cat,
            has_component: has_comp::<$ty>,
            spawn_default: spawn_comp::<$ty>,
            remove_component: remove_comp::<$ty>,
        }
    };
}

/// Canonical catalog of all 31 built-in attachable ECS components.
pub const BUILTIN_COMPONENTS: &[BuiltinComponentSpec] = &[
    // 1. Physics Components
    BuiltinComponentSpec {
        component_name: "RigidBody",
        display_title: "RigidBody",
        category: BuiltinComponentCategory::Physics,
        has_component: has_comp::<ae_plugin_api::RigidBody>,
        spawn_default: spawn_rigidbody,
        remove_component: remove_comp::<ae_plugin_api::RigidBody>,
    },
    BuiltinComponentSpec {
        component_name: "Collider",
        display_title: "3D Collider",
        category: BuiltinComponentCategory::Physics,
        has_component: has_comp::<ae_plugin_api::Collider>,
        spawn_default: spawn_collider,
        remove_component: remove_comp::<ae_plugin_api::Collider>,
    },
    builtin_entry!(
        "PhysicsMaterial",
        "Physics Material",
        Physics,
        ae_plugin_api::PhysicsMaterial
    ),
    // 2. Character & Gameplay Components
    builtin_entry!(
        "CharacterController",
        "Character Controller",
        Gameplay,
        ae_plugin_api::CharacterController
    ),
    builtin_entry!(
        "CharacterAction",
        "CharacterAction",
        Gameplay,
        ae_plugin_api::CharacterAction
    ),
    builtin_entry!(
        "PlayerTag",
        "Player Tag",
        Gameplay,
        ae_plugin_api::PlayerTag
    ),
    builtin_entry!("Velocity", "Velocity", Gameplay, ae_plugin_api::Velocity),
    builtin_entry!("Rotator", "Rotator", Gameplay, ae_plugin_api::Rotator),
    builtin_entry!(
        "MovingPlatform",
        "Moving Platform",
        Gameplay,
        ae_plugin_api::MovingPlatform
    ),
    builtin_entry!(
        "TriggerZone",
        "Trigger Zone",
        Gameplay,
        ae_plugin_api::TriggerZone
    ),
    builtin_entry!(
        "DestructibleTarget",
        "Destructible Target",
        Gameplay,
        ae_plugin_api::DestructibleTarget
    ),
    // 3. Rendering, LOD & Illumination Components
    builtin_entry!("Light", "Light", Rendering, ae_light::Light),
    builtin_entry!(
        "ModelId",
        "3D Model / Mesh",
        Rendering,
        ae_plugin_api::ModelId
    ),
    builtin_entry!(
        "Shape",
        "3D Shape Primitive",
        Rendering,
        ae_plugin_api::Shape
    ),
    builtin_entry!("LodGroup", "LOD Group", Rendering, ae_plugin_api::LodGroup),
    // 4. Skeletal Animation Components
    builtin_entry!(
        "AnimationPlayer",
        "Animation Player",
        Animation,
        ae_animation::AnimationPlayer
    ),
    // 5. Audio Components
    builtin_entry!("AudioSource", "Audio Source", Audio, ae_audio::AudioSource),
    BuiltinComponentSpec {
        component_name: "AudioListener",
        display_title: "Audio Listener",
        category: BuiltinComponentCategory::Audio,
        has_component: has_comp::<ae_audio::AudioListener>,
        spawn_default: spawn_audio_listener,
        remove_component: remove_comp::<ae_audio::AudioListener>,
    },
    // 6. In-Game 2D UI & HUD Components
    builtin_entry!(
        "UiElement",
        "2D Screen UI Element",
        UiHud,
        ae_plugin_api::UiElement
    ),
    builtin_entry!(
        "UiPanel",
        "UI Background Panel",
        UiHud,
        ae_plugin_api::UiPanel
    ),
    builtin_entry!("UiText", "UI Text Label", UiHud, ae_plugin_api::UiText),
    builtin_entry!(
        "UiProgressBar",
        "UI Progress Bar",
        UiHud,
        ae_plugin_api::UiProgressBar
    ),
    builtin_entry!(
        "UiButton",
        "Interactive Button",
        UiHud,
        ae_plugin_api::UiButton
    ),
    builtin_entry!("UiImage", "UI Sprite Image", UiHud, ae_plugin_api::UiImage),
    builtin_entry!(
        "UiSlider",
        "UI Numeric Slider",
        UiHud,
        ae_plugin_api::UiSlider
    ),
    builtin_entry!(
        "UiCheckbox",
        "UI Checkbox",
        UiHud,
        ae_plugin_api::UiCheckbox
    ),
    builtin_entry!(
        "UiTextInput",
        "UI Text Input Field",
        UiHud,
        ae_plugin_api::UiTextInput
    ),
    builtin_entry!(
        "UiLayoutGroup",
        "UI Layout Group",
        UiHud,
        ae_plugin_api::UiLayoutGroup
    ),
    builtin_entry!(
        "PlayerHealthBarTag",
        "Health Bar Tag",
        UiHud,
        ae_plugin_api::PlayerHealthBarTag
    ),
    builtin_entry!(
        "ScoreDisplayTag",
        "Score Display Tag",
        UiHud,
        ae_plugin_api::ScoreDisplayTag
    ),
    builtin_entry!(
        "ReticleTag",
        "Reticle Tag",
        UiHud,
        ae_plugin_api::ReticleTag
    ),
];

/// Searches [`BUILTIN_COMPONENTS`] for an entry matching `type_name`.
#[inline]
#[must_use]
pub fn find_builtin_component(type_name: &str) -> Option<&'static BuiltinComponentSpec> {
    BUILTIN_COMPONENTS
        .iter()
        .find(|e| e.component_name == type_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_builtin_components_spawn_detect_and_remove() {
        assert_eq!(
            BUILTIN_COMPONENTS.len(),
            31,
            "Built-in catalog must contain exactly 31 attachable ECS components"
        );

        for entry in BUILTIN_COMPONENTS {
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
        assert!(world.has::<ae_plugin_api::RigidBody>(entity));
        assert!(world.has::<ae_plugin_api::Collider>(entity));

        let entity2 = world.spawn(());
        spawn_collider(&mut world, entity2);
        assert!(world.has::<ae_plugin_api::Collider>(entity2));
        assert!(world.has::<ae_plugin_api::RigidBody>(entity2));
    }

    #[test]
    fn test_find_builtin_component_lookup_and_negative_cases() {
        assert_eq!(
            find_builtin_component("RigidBody").map(|e| e.category),
            Some(BuiltinComponentCategory::Physics)
        );
        assert!(find_builtin_component("").is_none());
        assert!(find_builtin_component("DoesNotExist").is_none());
    }
}