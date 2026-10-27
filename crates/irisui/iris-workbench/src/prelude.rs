// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris Workbench Prelude
//!
//! Convenient re-exports of core workbench primitives, error types, descriptors,
//! and lifecycle structures for  integration with desktop UI applications.

pub use crate::descriptors::{
    MenuBarDescriptor, MenuItemDescriptor, MenuSectionDescriptor, PanelDrawFn, StatusBarDescriptor,
    StatusPillDescriptor, StatusPillStyle, WorkbenchDescriptor, WorkbenchPanelRegistry,
};
pub use crate::error::{WorkbenchError, WorkbenchResult};
pub use crate::hooks::{
    CustomOverlayHook, DragDropPayload, ExternalTextureHook, TypedDragDropPayload,
};
pub use crate::lifecycle::{LifecycleBranch, LifecycleInputs, WorkbenchLifecycle};