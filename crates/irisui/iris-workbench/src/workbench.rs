// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Application Orchestrator
//!
//! Provides the primary [`Workbench`] struct and [`WorkbenchOutcome`] results combining
//! retained-mode docking, frameless window management, modal dialogs, and decoupled
//! action dispatching into a unified desktop IDE framework component.

pub mod outcome;
pub mod pointer;
pub mod runner;

#[cfg(test)]
mod tests;

pub use outcome::WorkbenchOutcome;
pub use runner::Workbench;