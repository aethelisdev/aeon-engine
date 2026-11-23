// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Component and bundle contracts for [`crate::World`] without runtime borrow-guard wrappers.

/// Marker trait implemented by all types storable as components inside [`crate::World`].
///
/// Decouples engine subsystems from third-party ECS trait bounds (`Send + Sync + 'static`).
pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}

/// Contract for heterogeneous component collections spawned onto or inserted into an entity.
///
/// Implemented automatically for all tuple component bundles as well as [`crate::BuiltEntity`].
pub trait Bundle: Sized {
    /// Spawns a new entity populated with this bundle inside `world`.
    #[doc(hidden)]
    fn spawn_into(self, world: &mut hecs::World) -> hecs::Entity;

    /// Inserts all components in this bundle onto an existing `entity` in `world`.
    #[doc(hidden)]
    fn insert_into(
        self,
        world: &mut hecs::World,
        entity: hecs::Entity,
    ) -> Result<(), hecs::NoSuchEntity>;

    /// Appends all components in this bundle onto `builder`.
    #[doc(hidden)]
    fn add_to_builder(self, builder: &mut hecs::EntityBuilder);
}

impl<B: hecs::DynamicBundle> Bundle for B {
    #[inline]
    fn spawn_into(self, world: &mut hecs::World) -> hecs::Entity {
        world.spawn(self)
    }

    #[inline]
    fn insert_into(
        self,
        world: &mut hecs::World,
        entity: hecs::Entity,
    ) -> Result<(), hecs::NoSuchEntity> {
        world.insert(entity, self)
    }

    #[inline]
    fn add_to_builder(self, builder: &mut hecs::EntityBuilder) {
        builder.add_bundle(self);
    }
}

/// Contract for fixed-schema tuple component bundles eligible for batched archetype spawning.
pub trait BatchBundle: Sized {
    /// Backend bundle representation consumed during batched spawning.
    #[doc(hidden)]
    type Raw: hecs::Bundle + 'static;

    /// Converts this bundle into the backend batch representation.
    #[doc(hidden)]
    fn into_raw(self) -> Self::Raw;
}

impl<B: hecs::Bundle + 'static> BatchBundle for B {
    type Raw = Self;

    #[inline]
    fn into_raw(self) -> Self::Raw {
        self
    }
}

/// Contract for fixed-schema component bundles removed atomically from an entity.
pub trait ComponentBundle: Sized {
    /// Backend bundle type removed from storage.
    #[doc(hidden)]
    type Raw: hecs::Bundle + 'static;

    /// Converts the backend bundle output into `Self`.
    #[doc(hidden)]
    fn from_raw(raw: Self::Raw) -> Self;
}

impl<B: hecs::Bundle + 'static> ComponentBundle for B {
    type Raw = Self;

    #[inline]
    fn from_raw(raw: Self::Raw) -> Self {
        raw
    }
}