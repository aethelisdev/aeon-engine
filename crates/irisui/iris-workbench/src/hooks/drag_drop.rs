// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Drag-and-Drop Extension Payloads
//!
//! Provides type-safe dynamic carriers for cross-panel, cross-dock-tree, and
//! cross-window drag-and-drop operations within the workbench shell.

use std::any::Any;
use std::fmt;

/// Type-safe dynamic drag-and-drop payload carrier.
///
/// Wraps an arbitrary payload object implementing `Any + Send + Sync` in a type-erased
/// heap container, while preserving reflection metadata for safe downcasting across
/// disparate UI panel controllers and dock viewports.
pub struct DragDropPayload {
    payload: Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    preview_label: String,
    source_panel: String,
}

impl fmt::Debug for DragDropPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DragDropPayload")
            .field("type_name", &self.type_name)
            .field("preview_label", &self.preview_label)
            .field("source_panel", &self.source_panel)
            .finish()
    }
}

impl DragDropPayload {
    /// Creates a new type-erased payload with the given data and visual descriptors.
    ///
    /// # Arguments
    /// * `data` - The typed payload object to transfer.
    /// * `preview_label` - Text shown next to the drag cursor (e.g., `"Cube.gltf"`).
    /// * `source_panel` - Identifier of the panel originating the drag operation (e.g., `"Assets"`).
    pub fn new<T: Any + Send + Sync>(
        data: T,
        preview_label: impl Into<String>,
        source_panel: impl Into<String>,
    ) -> Self {
        Self {
            payload: Box::new(data),
            type_name: std::any::type_name::<T>(),
            preview_label: preview_label.into(),
            source_panel: source_panel.into(),
        }
    }

    /// Returns `true` if the underlying payload matches the requested concrete type `T`.
    #[inline]
    #[must_use]
    pub fn is<T: Any + Send + Sync>(&self) -> bool {
        self.payload.is::<T>()
    }

    /// Attempts to borrow the payload as a reference to concrete type `T`.
    ///
    /// Returns `Some(&T)` if the runtime type matches `T`, or `None` otherwise.
    #[inline]
    #[must_use]
    pub fn downcast_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    /// Attempts to mutably borrow the payload as a reference to concrete type `T`.
    ///
    /// Returns `Some(&mut T)` if the runtime type matches `T`, or `None` otherwise.
    #[inline]
    pub fn downcast_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.payload.downcast_mut::<T>()
    }

    /// Attempts to consume the carrier and downcast the boxed payload to concrete type `T`.
    ///
    /// Returns `Ok(Box<T>)` on success, or `Err(self)` if the concrete type does not match.
    pub fn downcast<T: Any + Send + Sync>(self) -> Result<Box<T>, Self> {
        if self.payload.is::<T>() {
            let type_name = self.type_name;
            let preview_label = self.preview_label;
            let source_panel = self.source_panel;
            match self.payload.downcast::<T>() {
                Ok(boxed) => Ok(boxed),
                Err(original) => Err(Self {
                    payload: original,
                    type_name,
                    preview_label,
                    source_panel,
                }),
            }
        } else {
            Err(self)
        }
    }

    /// Returns the human-readable preview label describing the transferred item.
    #[inline]
    #[must_use]
    pub fn preview_label(&self) -> &str {
        &self.preview_label
    }

    /// Returns the source panel identifier that initiated the drag.
    #[inline]
    #[must_use]
    pub fn source_panel(&self) -> &str {
        &self.source_panel
    }

    /// Returns the Rust type name of the enclosed payload.
    #[inline]
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
}

/// Strongly-typed drag-and-drop payload descriptor with compile-time known payload type `T`.
///
/// Provides ergonomic creation and direct consumption without manual downcasting
/// when interacting with known panel data contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDragDropPayload<T> {
    /// The concrete payload data being dragged.
    pub data: T,

    /// Human-readable label displayed next to the drag cursor.
    pub preview_label: String,

    /// Identifier of the panel initiating the drag operation.
    pub source_panel: String,
}

impl<T: Any + Send + Sync> TypedDragDropPayload<T> {
    /// Creates a new typed drag-and-drop payload descriptor.
    ///
    /// # Arguments
    /// * `data` - Concrete payload value.
    /// * `preview_label` - Text shown next to cursor during drag.
    /// * `source_panel` - Originating panel identifier.
    pub fn new(data: T, preview_label: impl Into<String>, source_panel: impl Into<String>) -> Self {
        Self {
            data,
            preview_label: preview_label.into(),
            source_panel: source_panel.into(),
        }
    }

    /// Erases the concrete type into a dynamic [`DragDropPayload`].
    pub fn into_erased(self) -> DragDropPayload {
        DragDropPayload::new(self.data, self.preview_label, self.source_panel)
    }

    /// Attempts to extract a typed descriptor from a dynamic [`DragDropPayload`].
    ///
    /// Returns `Ok(TypedDragDropPayload<T>)` if the underlying type matches `T`,
    /// or returns `Err(DragDropPayload)` if types differ.
    pub fn try_from_erased(erased: DragDropPayload) -> Result<Self, DragDropPayload> {
        if erased.is::<T>() {
            let preview_label = erased.preview_label.clone();
            let source_panel = erased.source_panel.clone();
            match erased.downcast::<T>() {
                Ok(boxed) => Ok(Self {
                    data: *boxed,
                    preview_label,
                    source_panel,
                }),
                Err(original) => Err(original),
            }
        } else {
            Err(erased)
        }
    }
}
