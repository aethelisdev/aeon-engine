// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Workbench Descriptors
//!
//! Provides declarative configuration descriptors for application menu bars,
//! bottom status bars, telemetry pills, generic panel registries, and the
//! composite application shell workbench descriptor.

pub mod menu;
pub mod panel_registry;
pub mod status;
pub mod workbench;

#[cfg(test)]
mod tests;

pub use menu::{MenuBarDescriptor, MenuItemDescriptor, MenuSectionDescriptor};
pub use panel_registry::{PanelDrawFn, PanelRegistry, WorkbenchPanelRegistry};
pub use status::{StatusBarDescriptor, StatusPillDescriptor, StatusPillStyle};
pub use workbench::WorkbenchDescriptor;