// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris Workbench Prelude
//!
//! Convenient re-exports of core workbench primitives, error types, descriptors,
//! and lifecycle structures for  integration with desktop UI applications.

pub use crate::action_bus::ActionBus;
pub use crate::descriptors::{
    LiveRefreshInterval, MenuBarBrandingDescriptor, MenuBarDescriptor, MenuItemDescriptor,
    MenuRightItemDescriptor, MenuSectionDescriptor, PanelBackground, PanelBorder, PanelCatalog,
    PanelDrawFn, PanelMetadata, PanelRegistryTarget, PanelSurfaceStyle, RegistryActiveTabViewer,
    RegistryTabViewer, ScopedPanelContext, ScopedPanelDrawFn, ScopedPanelRegistry,
    ScopedWorkbenchPanel, StatefulPanel, StatefulPanelDrawFn, StatefulPanelLiveFn,
    StatusBarDescriptor, StatusPillDescriptor, StatusPillStyle, WorkbenchAppData,
    WorkbenchAppScope, WorkbenchDescriptor, WorkbenchPanel, WorkbenchPanelContext,
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
    AspectRatio, ClosureOverlayHook, ClosureVolatileHook, CustomOverlayHook, DragDropController,
    DragDropPayload, DragDropPreviewStyle, ExternalTextureView, OverlayAnchor, OverlayHook,
    OverlayHost, OverlayId, TypedDragDropPayload, VolatileCanvasHook, VolatileHookRegistry,
    VolatileRefreshResult, WorkbenchUiScopeExt,
};
pub use crate::layout_sync::{
    DualLayoutEliminator, LayoutSyncMetrics, SubtreeEngines, synchronize_dock_frame,
    synchronize_dock_leaves,
};
pub use crate::lifecycle::{
    EventInvalidationRoute, LifecycleBranch, LifecycleInputs, WorkbenchLifecycle,
    route_event_invalidation, route_event_invalidation_with_fallback,
};
pub use crate::workbench::{
    PointerState, UiInspectionSummary, UiNodeDump, UiTreeDump, Workbench, WorkbenchChromeBuilderFn,
    WorkbenchExternalContext, WorkbenchFrameOutcome, WorkbenchFramePassResult,
    WorkbenchFrameRunner, WorkbenchFrameServices, WorkbenchFrameUpdateParams, WorkbenchHost,
    WorkbenchHostEventResult, WorkbenchHostFrameParams, WorkbenchHostPass, WorkbenchOutcome,
    WorkbenchOwnedFrameParams, WorkbenchSurfaceFrameParams, WorkbenchSurfaceRender,
};