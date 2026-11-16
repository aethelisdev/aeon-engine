// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Application Orchestrator
//!
//! Provides the primary [`Workbench`] struct and [`WorkbenchOutcome`] results combining
//! retained-mode docking, frameless window management, modal dialogs, and decoupled
//! action dispatching into a unified desktop IDE framework component.

pub mod escape;
pub mod outcome;
pub mod outside_click;
pub mod pointer;
pub mod runner;
pub mod shell;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_escape;
#[cfg(test)]
mod tests_events;
#[cfg(test)]
mod tests_floating;
#[cfg(test)]
mod tests_hover;
#[cfg(test)]
mod tests_outside_click;
#[cfg(test)]
mod tests_theme_screen;

pub use outcome::WorkbenchOutcome;
pub use pointer::{HoverTargetResolution, PointerState, resolve_hover_targets};
pub use runner::Workbench;
pub use shell::{WorkbenchFrameSync, WorkbenchFrameSyncParams};