// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Container & Hierarchical Layout Primitives
//!
//! Provides structural layout primitives on [`super::core::UiScope`]: generic containers, flex rows,
//! flex columns, themed cards, and fluent builders.

pub mod builder;
pub mod cards;
pub mod flex;
pub mod generic;
pub mod scroll;

pub use builder::{ColumnBuilder, RowBuilder};
pub use generic::{ContainerResponseExt, InnerResponse};