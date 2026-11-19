// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Application Orchestrator Module
//!
//! Provides the primary [`Workbench`] struct combining retained-mode docking,
//! frameless window management, modal dialogs, and decoupled action dispatching
//! into a unified, crates.io-grade desktop IDE framework component.
//!
//! Decomposed into cohesive submodules:
//! - [`action_types`]: Type aliases for external file drop, zoom, and preferences action closures.
//! - [`coordinator`]: Core [`Workbench`] struct definition, constructor, and fluent builders.
//! - [`events`]: Shell and content event handling, frameless window resizing, and invalidation routing.
//! - [`layout`]: Workspace rectangle computation, text preparation queries, and partial leaf reconstruction.
//! - [`navigation`]: UI zoom factor computation, preset stepping, and zoom reset controls.

pub mod action_types;
pub mod coordinator;
pub mod events;
pub mod layout;
pub mod navigation;
pub mod surface;

pub use action_types::{FileDropActionBuilder, PreferencesActionBuilder, ZoomActionBuilder};
pub use coordinator::Workbench;
pub use surface::WorkbenchSurfaceRender;