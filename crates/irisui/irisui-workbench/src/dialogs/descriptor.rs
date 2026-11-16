// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal, Sheet, and Floating Window Descriptors
//!
//! Declarative configuration structures specifying geometry, styling, and dismissal
//! behaviors for centered modal dialogues, sliding edge sheets, confirmation prompts,
//! text inputs, and detached floating windows.

pub mod confirm;
pub mod custom;
pub mod floating;
pub mod info;
pub mod input;
pub mod modal;
pub mod progress;
pub mod sheet;
pub mod unified;

pub use confirm::{ConfirmDialogDescriptor, ConfirmationDialogDescriptor};
pub use custom::{CustomDialogDescriptor, CustomModalRenderFn};
pub use floating::FloatingWindowDescriptor;
pub use info::{InfoCardDialogDescriptor, InfoEntry, InfoLink};
pub use input::{InputDialogDescriptor, PromptDialogDescriptor, PromptSubmitFn};
pub use modal::{ModalDescriptor, WorkbenchModal};
pub use progress::ProgressModalDescriptor;
pub use sheet::{CustomSheetRenderFn, SheetAnchor, SheetDialogDescriptor};
pub use unified::DialogDescriptor;