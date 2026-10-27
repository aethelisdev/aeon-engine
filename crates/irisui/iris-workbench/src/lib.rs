// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris Workbench (`iris-workbench`)
//!
//! Desktop IDE and Professional Workbench Application Shell Framework for Iris UI.
//!
//! Provides high-level abstractions for building professional desktop creative software:
//! - Declarative application shell layouts (menubar, statusbar, dock tree, floating dialogs).
//! - Autonomous 3-branch lifecycle management (`Full Shell`, `Subtree Isolated Rebuild`, `In-place Dynamic Overlay`).
//! - Hardware-accelerated viewport and external texture hook integration points.
//! - Type-safe drag-and-drop payload management across panels and floating windows.
//! - Dual-layout synchronization eliminating visual jitter between retained node graphs and flexbox solvers.
//!
//! ## Design Guarantees
//!
//! - **100% Safe Rust**: Strictly forbids `unsafe` code (`#![forbid(unsafe_code)]`).
//! - **Engine Independence**: Strictly decoupled from game engine internals or ECS structures (`ae_core`, `hecs`).
//! - **Zero Frame Allocation**: Hot path runtime loops avoid dynamic heap reallocations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod descriptors;
pub mod error;
pub mod hooks;
pub mod prelude;

pub use descriptors::{
    MenuBarDescriptor, MenuItemDescriptor, MenuSectionDescriptor, PanelDrawFn, PanelRegistry,
    StatusBarDescriptor, StatusPillDescriptor, StatusPillStyle, WorkbenchDescriptor,
    WorkbenchPanelRegistry,
};
pub use error::{WorkbenchError, WorkbenchResult};
pub use hooks::{CustomOverlayHook, DragDropPayload, ExternalTextureHook, TypedDragDropPayload};