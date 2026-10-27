// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Drag-and-Drop Extension Payloads & Controllers
//!
//! Provides type-safe dynamic carriers, state controllers, and configurable preview
//! styling for cross-panel, cross-dock-tree, and cross-window drag-and-drop operations
//! within the workbench shell.

use std::any::Any;
use std::fmt;

use iris_core::Color;
use iris_core::geometry::{Insets, Point};

/// Function pointer type for cloning type-erased heap payloads.
type ErasedCloneFn = fn(&(dyn Any + Send + Sync)) -> Box<dyn Any + Send + Sync>;

#[inline]
fn clone_erased_box<T: Any + Send + Sync + Clone>(
    any: &(dyn Any + Send + Sync),
) -> Box<dyn Any + Send + Sync> {
    let concrete = any.downcast_ref::<T>().expect("type invariant violated");
    Box::new(concrete.clone())
}

/// Metadata descriptor encapsulating visual badges, source panels, and preview labels.
#[derive(Debug, Clone, PartialEq)]
struct DragDropMetadata {
    preview_label: String,
    source_panel: String,
    badge_text: Option<String>,
    badge_color: Option<Color>,
}

/// Type-safe dynamic drag-and-drop payload carrier.
///
/// Wraps an arbitrary payload object in a type-erased container while preserving
/// reflection metadata for safe runtime downcasting, visual badge indicators,
/// and automated cloning across UI panels and dock viewports.
pub struct DragDropPayload {
    payload: Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    meta: Box<DragDropMetadata>,
    clone_fn: Option<ErasedCloneFn>,
}

impl fmt::Debug for DragDropPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DragDropPayload")
            .field("type_name", &self.type_name)
            .field("preview_label", &self.meta.preview_label)
            .field("source_panel", &self.meta.source_panel)
            .field("badge_text", &self.meta.badge_text)
            .field("badge_color", &self.meta.badge_color)
            .finish()
    }
}

impl Clone for DragDropPayload {
    fn clone(&self) -> Self {
        let payload = self
            .clone_fn
            .expect("DragDropPayload must be created with a Clone type to support cloning")(
            &*self.payload,
        );
        Self {
            payload,
            type_name: self.type_name,
            meta: self.meta.clone(),
            clone_fn: self.clone_fn,
        }
    }
}

impl PartialEq for DragDropPayload {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name && self.meta == other.meta
    }
}

impl DragDropPayload {
    /// Creates a new type-erased payload with automated cloning support.
    ///
    /// # Arguments
    /// * `data` - The typed payload object to transfer (must implement `Clone`).
    /// * `preview_label` - Text shown next to the drag cursor (e.g., `"Cube.gltf"`).
    /// * `source_panel` - Identifier of the panel originating the drag operation (e.g., `"Assets"`).
    pub fn new<T: Any + Send + Sync + Clone>(
        data: T,
        preview_label: impl Into<String>,
        source_panel: impl Into<String>,
    ) -> Self {
        Self {
            payload: Box::new(data),
            type_name: std::any::type_name::<T>(),
            meta: Box::new(DragDropMetadata {
                preview_label: preview_label.into(),
                source_panel: source_panel.into(),
                badge_text: None,
                badge_color: None,
            }),
            clone_fn: Some(clone_erased_box::<T>),
        }
    }

    /// Creates a new type-erased payload for types that do not implement `Clone`.
    ///
    /// Calling [`Clone::clone`] on a payload constructed with this method will panic.
    pub fn new_unclonable<T: Any + Send + Sync>(
        data: T,
        preview_label: impl Into<String>,
        source_panel: impl Into<String>,
    ) -> Self {
        Self {
            payload: Box::new(data),
            type_name: std::any::type_name::<T>(),
            meta: Box::new(DragDropMetadata {
                preview_label: preview_label.into(),
                source_panel: source_panel.into(),
                badge_text: None,
                badge_color: None,
            }),
            clone_fn: None,
        }
    }

    /// Attaches an optional category badge text and indicator color via fluent builder.
    ///
    /// # Arguments
    /// * `text` - Short badge string (e.g., `"3D"`, `"TEX"`, `"MAT"`).
    /// * `color` - Visual theme accent color for the badge pill.
    #[must_use]
    pub fn with_badge(mut self, text: impl Into<String>, color: Color) -> Self {
        self.meta.badge_text = Some(text.into());
        self.meta.badge_color = Some(color);
        self
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
            let meta = self.meta;
            let clone_fn = self.clone_fn;
            match self.payload.downcast::<T>() {
                Ok(boxed) => Ok(boxed),
                Err(original) => Err(Self {
                    payload: original,
                    type_name,
                    meta,
                    clone_fn,
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
        &self.meta.preview_label
    }

    /// Returns the source panel identifier that initiated the drag.
    #[inline]
    #[must_use]
    pub fn source_panel(&self) -> &str {
        &self.meta.source_panel
    }

    /// Returns the optional category badge text, if configured.
    #[inline]
    #[must_use]
    pub fn badge_text(&self) -> Option<&str> {
        self.meta.badge_text.as_deref()
    }

    /// Returns the optional category badge color, if configured.
    #[inline]
    #[must_use]
    pub const fn badge_color(&self) -> Option<Color> {
        self.meta.badge_color
    }

    /// Returns the Rust type name of the enclosed payload.
    #[inline]
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }
}

/// Strongly-typed drag-and-drop payload descriptor with compile-time known payload type `T`.
///
/// Provides ergonomic creation and direct consumption without manual downcasting
/// when interacting with known panel data contracts.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedDragDropPayload<T> {
    /// The concrete payload data being dragged.
    pub data: T,

    /// Human-readable label displayed next to the drag cursor.
    pub preview_label: String,

    /// Identifier of the panel initiating the drag operation.
    pub source_panel: String,

    /// Optional visual category badge text (e.g. `"3D"`).
    pub badge_text: Option<String>,

    /// Optional visual theme color for the badge.
    pub badge_color: Option<Color>,
}

impl<T: Any + Send + Sync + Clone> TypedDragDropPayload<T> {
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
            badge_text: None,
            badge_color: None,
        }
    }

    /// Attaches an optional category badge text and indicator color via fluent builder.
    #[must_use]
    pub fn with_badge(mut self, text: impl Into<String>, color: Color) -> Self {
        self.badge_text = Some(text.into());
        self.badge_color = Some(color);
        self
    }

    /// Erases the concrete type into a dynamic [`DragDropPayload`].
    pub fn into_erased(self) -> DragDropPayload {
        let mut erased = DragDropPayload::new(self.data, self.preview_label, self.source_panel);
        if let (Some(badge), Some(color)) = (self.badge_text, self.badge_color) {
            erased = erased.with_badge(badge, color);
        }
        erased
    }

    /// Attempts to extract a typed descriptor from a dynamic [`DragDropPayload`].
    ///
    /// Returns `Ok(TypedDragDropPayload<T>)` if the underlying type matches `T`,
    /// or returns `Err(DragDropPayload)` if types differ.
    pub fn try_from_erased(erased: DragDropPayload) -> Result<Self, DragDropPayload> {
        if erased.is::<T>() {
            let preview_label = erased.meta.preview_label.clone();
            let source_panel = erased.meta.source_panel.clone();
            let badge_text = erased.meta.badge_text.clone();
            let badge_color = erased.meta.badge_color;
            match erased.downcast::<T>() {
                Ok(boxed) => Ok(Self {
                    data: *boxed,
                    preview_label,
                    source_panel,
                    badge_text,
                    badge_color,
                }),
                Err(original) => Err(original),
            }
        } else {
            Err(erased)
        }
    }
}

/// Visual configuration descriptor for the floating drag-and-drop cursor preview.
///
/// Allows consuming applications to customize the appearance of the floating drag capsule
/// (colors, borders, insets, offsets, typography) without hardcoded values.
#[derive(Debug, Clone, PartialEq)]
pub struct DragDropPreviewStyle {
    /// Offset from the mouse cursor in logical pixels (default: `(14.0, 14.0)`).
    pub offset: Point,
    /// Background fill color of the floating capsule.
    pub background: Color,
    /// Border stroke color.
    pub border_color: Color,
    /// Border width in logical pixels.
    pub border_width: f32,
    /// Corner rounding radius in logical pixels.
    pub corner_radius: f32,
    /// Padding insets inside the capsule.
    pub padding: Insets,
    /// Gap between category badge and preview label.
    pub gap: f32,
    /// Primary text label color.
    pub text_color: Color,
    /// Font size for the preview label.
    pub font_size: f32,
}

impl Default for DragDropPreviewStyle {
    fn default() -> Self {
        Self {
            offset: Point::new(14.0, 14.0),
            background: Color::rgba(0.07, 0.09, 0.13, 0.90),
            border_color: Color::rgba(0.0, 0.90, 1.0, 0.86),
            border_width: 1.0,
            corner_radius: 6.0,
            padding: Insets::new(4.0, 8.0, 4.0, 8.0),
            gap: 6.0,
            text_color: Color::WHITE,
            font_size: 12.0,
        }
    }
}

impl DragDropPreviewStyle {
    /// Creates a new preview style descriptor with default dark-studio values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the cursor offset in logical pixels.
    #[must_use]
    pub const fn with_offset(mut self, offset: Point) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the background fill and border outline colors.
    #[must_use]
    pub const fn with_colors(mut self, background: Color, border_color: Color) -> Self {
        self.background = background;
        self.border_color = border_color;
        self
    }

    /// Sets the padding insets and item gap.
    #[must_use]
    pub const fn with_spacing(mut self, padding: Insets, gap: f32) -> Self {
        self.padding = padding;
        self.gap = gap;
        self
    }
}

/// Application-wide drag-and-drop controller managing active payload state,
/// cursor positions, and visual transition states.
///
/// Designed to be integrated directly into the workbench lifecycle and panel controllers
/// without engine coupling or global state bags.
#[derive(Debug, Default)]
pub struct DragDropController {
    active_payload: Option<DragDropPayload>,
    start_pos: Option<Point>,
    current_pos: Option<Point>,
}

impl DragDropController {
    /// Creates a new inactive drag-and-drop controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_payload: None,
            start_pos: None,
            current_pos: None,
        }
    }

    /// Initiates an active drag operation with the given type-erased payload and cursor start position.
    pub fn start_drag(&mut self, payload: DragDropPayload, start_pos: Point) {
        self.start_pos = Some(start_pos);
        self.current_pos = Some(start_pos);
        self.active_payload = Some(payload);
    }

    /// Initiates an active drag operation with a typed payload descriptor.
    pub fn start_typed<T: Any + Send + Sync + Clone>(
        &mut self,
        payload: TypedDragDropPayload<T>,
        start_pos: Point,
    ) {
        self.start_drag(payload.into_erased(), start_pos);
    }

    /// Updates the current mouse cursor location during drag motion.
    pub fn update_cursor(&mut self, cursor: Point) {
        self.current_pos = Some(cursor);
    }

    /// Returns `true` if a drag operation is currently active.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.active_payload.is_some()
    }

    /// Returns an immutable reference to the currently dragged payload, if active.
    #[must_use]
    pub const fn active_payload(&self) -> Option<&DragDropPayload> {
        self.active_payload.as_ref()
    }

    /// Returns a mutable reference to the currently dragged payload, if active.
    pub fn active_payload_mut(&mut self) -> Option<&mut DragDropPayload> {
        self.active_payload.as_mut()
    }

    /// Returns `true` if the active payload contains concrete type `T`.
    #[must_use]
    pub fn has_payload<T: Any + Send + Sync>(&self) -> bool {
        self.active_payload.as_ref().is_some_and(|p| p.is::<T>())
    }

    /// Attempts to borrow the active payload as concrete type `T`.
    #[must_use]
    pub fn payload<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.active_payload
            .as_ref()
            .and_then(|p| p.downcast_ref::<T>())
    }

    /// Attempts to mutably borrow the active payload as concrete type `T`.
    pub fn payload_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.active_payload
            .as_mut()
            .and_then(|p| p.downcast_mut::<T>())
    }

    /// Completes the drag operation, returning the active payload and resetting position state.
    pub fn end_drag(&mut self) -> Option<DragDropPayload> {
        self.start_pos = None;
        self.current_pos = None;
        self.active_payload.take()
    }

    /// Cancels the drag operation without applying drop mutations.
    pub fn cancel_drag(&mut self) -> Option<DragDropPayload> {
        self.end_drag()
    }

    /// Attempts to consume the active payload and downcast it to a [`TypedDragDropPayload<T>`].
    ///
    /// If the concrete type matches `T`, completes the drag session and returns `Some(TypedDragDropPayload<T>)`.
    /// Otherwise, preserves the active drag state and returns `None`.
    pub fn take_typed<T: Any + Send + Sync + Clone>(&mut self) -> Option<TypedDragDropPayload<T>> {
        if self.has_payload::<T>() {
            let erased = self.end_drag()?;
            TypedDragDropPayload::try_from_erased(erased).ok()
        } else {
            None
        }
    }

    /// Returns the initial cursor location where the drag was started.
    #[must_use]
    pub const fn start_pos(&self) -> Option<Point> {
        self.start_pos
    }

    /// Returns the latest recorded cursor position during the drag.
    #[must_use]
    pub const fn current_pos(&self) -> Option<Point> {
        self.current_pos
    }

    /// Clears any active drag session and position tracking.
    pub fn clear(&mut self) {
        self.active_payload = None;
        self.start_pos = None;
        self.current_pos = None;
    }
}