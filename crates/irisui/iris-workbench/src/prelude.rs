// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris Workbench Prelude
//!
//! Convenient re-exports of core workbench primitives, error types, descriptors,
//! and lifecycle structures for  integration with desktop UI applications.

pub use crate::action_bus::ActionBus;
pub use crate::descriptors::{
    MenuBarBrandingDescriptor, MenuBarDescriptor, MenuItemDescriptor, MenuRightItemDescriptor,
    MenuSectionDescriptor, PanelDrawFn, PanelMetadata, RegistryTabViewer, StatusBarDescriptor,
    StatusPillDescriptor, StatusPillStyle, WorkbenchDescriptor, WorkbenchPanel,
    WorkbenchPanelRegistry,
};
pub use crate::dialogs::{
    ActiveDialog, ConfirmDialogDescriptor, ConfirmationDialogDescriptor, CustomDialogDescriptor,
    CustomModalRenderFn, CustomSheetRenderFn, DialogDescriptor, DialogFocusTrap, DialogHost,
    DialogKind, FloatingWindowDescriptor, InfoCardDialogDescriptor, InfoEntry,
    InputDialogDescriptor, ModalController, ModalDescriptor, ModalDismissReason, ModalOutcome,
    ModalStyle, ProgressModalDescriptor, PromptDialogDescriptor, SheetAnchor,
    SheetDialogDescriptor, WorkbenchModal,
};
pub use crate::error::{WorkbenchError, WorkbenchResult};
pub use crate::hooks::{
    AspectRatio, ClosureOverlayHook, CustomOverlayHook, DragDropController, DragDropPayload,
    DragDropPreviewStyle, ExternalTextureView, OverlayAnchor, OverlayHook, OverlayHost, OverlayId,
    TypedDragDropPayload, WorkbenchUiScopeExt,
};
pub use crate::layout_sync::{
    DualLayoutEliminator, LayoutSyncMetrics, SubtreeEngines, synchronize_dock_frame,
    synchronize_dock_leaves,
};
pub use crate::lifecycle::{LifecycleBranch, LifecycleInputs, WorkbenchLifecycle};
pub use crate::workbench::{Workbench, WorkbenchOutcome};