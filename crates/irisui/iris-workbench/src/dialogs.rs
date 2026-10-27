// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal, Dialog and Floating Window Subsystem
//!
//! Autonomous coordinator providing centered modal dialogue cards, semi-transparent
//! backdrop scrim blockers, Escape/backdrop click dismissals, and detached floating windows.

pub mod controller;
pub mod descriptor;
pub mod response;
pub mod style;

#[cfg(test)]
mod tests;

pub use controller::ModalController;
pub use descriptor::{
    ConfirmDialogDescriptor, FloatingWindowDescriptor, InputDialogDescriptor, ModalDescriptor,
    WorkbenchModal,
};
pub use response::{ModalDismissReason, ModalOutcome};
pub use style::ModalStyle;