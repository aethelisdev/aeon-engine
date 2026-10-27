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
pub use crate::dialogs::{
    FloatingWindowDescriptor, ModalController, ModalDescriptor, ModalDismissReason, ModalOutcome,
};
pub use crate::error::{WorkbenchError, WorkbenchResult};
pub use crate::hooks::{
    CustomOverlayHook, DragDropController, DragDropPayload, DragDropPreviewStyle,
    ExternalTextureHook, TypedDragDropPayload, WorkbenchUiScopeExt,
};
pub use crate::layout_sync::{
    DualLayoutEliminator, LayoutSyncMetrics, synchronize_dock_frame, synchronize_dock_leaves,
};
pub use crate::lifecycle::{LifecycleBranch, LifecycleInputs, WorkbenchLifecycle};