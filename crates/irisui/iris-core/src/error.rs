// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Error types for Iris UI core operations.

use crate::id::WidgetId;
use thiserror::Error;

/// Comprehensive error enumeration for UI tree, hierarchy, and arena operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IrisCoreError {
    /// The specified widget ID was not found in the arena (e.g. stale key or already removed).
    #[error("Widget node not found: {0:?}")]
    NodeNotFound(WidgetId),

    /// An attempt was made to set a node as a child of itself or create a circular parent-child loop.
    #[error("Circular hierarchy detected when attaching widget {child:?} to parent {parent:?}")]
    CircularHierarchy {
        /// The child widget ID being attached.
        child: WidgetId,
        /// The target parent widget ID.
        parent: WidgetId,
    },

    /// The root widget is already set and cannot be replaced without explicit teardown.
    #[error("Root widget already exists: {0:?}")]
    RootAlreadyExists(WidgetId),
}