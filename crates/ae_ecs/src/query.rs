// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type-safe zero-allocation query traits and iteration adapters over [`crate::World`].

use crate::component::Component;
use crate::entity::Entity;
use crate::error::QueryOneError;

/// Trait implemented by component borrow patterns queryable via [`crate::World::query`]
/// and [`crate::World::query_mut`].
///
/// Supports shared references (`&T`), mutable references (`&mut T`), optional borrows
/// (`Option<&T>`, `Option<&mut T>`), [`Entity`] handles, and arbitrary tuples up to 12 elements.
pub trait EcsQuery {
    /// Underlying backend query descriptor.
    #[doc(hidden)]
    type Raw: hecs::Query;

    /// Item type yielded for each matched entity.
    type Item<'a>;

    /// Converts a backend query item into [`Self::Item`] with zero heap allocation.
    #[doc(hidden)]
    fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a>;
}

impl<'q, T: Component> EcsQuery for &'q T {
    type Raw = &'q T;
    type Item<'a> = &'a T;

    #[inline]
    fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a> {
        raw
    }
}

impl<'q, T: Component> EcsQuery for &'q mut T {
    type Raw = &'q mut T;
    type Item<'a> = &'a mut T;

    #[inline]
    fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a> {
        raw
    }
}

impl<'q, T: Component> EcsQuery for Option<&'q T> {
    type Raw = Option<&'q T>;
    type Item<'a> = Option<&'a T>;

    #[inline]
    fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a> {
        raw
    }
}

impl<'q, T: Component> EcsQuery for Option<&'q mut T> {
    type Raw = Option<&'q mut T>;
    type Item<'a> = Option<&'a mut T>;

    #[inline]
    fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a> {
        raw
    }
}

impl EcsQuery for Entity {
    type Raw = hecs::Entity;
    type Item<'a> = Entity;

    #[inline]
    fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a> {
        Entity::from_raw(raw)
    }
}

macro_rules! impl_ecs_query_tuple {
    ($($idx:tt : $name:ident),+) => {
        impl<$($name: EcsQuery),+> EcsQuery for ($($name,)+) {
            type Raw = ($($name::Raw,)+);
            type Item<'a> = ($($name::Item<'a>,)+);

            #[inline]
            fn map_item<'a>(raw: <Self::Raw as hecs::Query>::Item<'a>) -> Self::Item<'a> {
                ($($name::map_item(raw.$idx),)+)
            }
        }
    };
}

impl_ecs_query_tuple!(0: A);
impl_ecs_query_tuple!(0: A, 1: B);
impl_ecs_query_tuple!(0: A, 1: B, 2: C);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_ecs_query_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);

/// Active dynamically-checked borrow guard returned by [`crate::World::query`].
pub struct QueryBorrow<'w, Q: EcsQuery>(pub(crate) hecs::QueryBorrow<'w, Q::Raw>);

impl<Q: EcsQuery> QueryBorrow<'_, Q> {
    /// Iterates sequentially over all matching archetypes without allocating.
    #[inline]
    pub fn iter(&mut self) -> QueryIter<'_, Q> {
        QueryIter(self.0.iter())
    }
}

/// Iterator yielded by [`QueryBorrow::iter`].
pub struct QueryIter<'q, Q: EcsQuery>(hecs::QueryIter<'q, Q::Raw>);

impl<'q, Q: EcsQuery> Iterator for QueryIter<'q, Q> {
    type Item = Q::Item<'q>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Q::map_item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// Lock-free exclusive mutable query guard returned by [`crate::World::query_mut`].
pub struct QueryMut<'w, Q: EcsQuery>(pub(crate) hecs::QueryMut<'w, Q::Raw>);

impl<'w, Q: EcsQuery> IntoIterator for QueryMut<'w, Q> {
    type Item = Q::Item<'w>;
    type IntoIter = QueryIter<'w, Q>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        QueryIter(self.0.into_iter())
    }
}

/// Single-entity query borrow returned by [`crate::World::query_one`].
pub struct QueryOne<'w, Q: EcsQuery>(pub(crate) hecs::QueryOne<'w, Q::Raw>);

impl<Q: EcsQuery> QueryOne<'_, Q> {
    /// Resolves the queried components for the target entity, returning `Err` if absent or unsatisfied.
    #[inline]
    pub fn get(&mut self) -> Result<Q::Item<'_>, QueryOneError> {
        self.0.get().map(Q::map_item).map_err(QueryOneError::from)
    }
}