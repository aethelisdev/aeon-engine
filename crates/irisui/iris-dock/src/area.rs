// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Docking Area Runner (`area`)
//!
//! Orchestrates dock hierarchy rendering, layout computation, splitter interactions,
//! and tab dragging workflows.

pub mod runner;
pub mod splitter;
pub mod splitter_drag;

pub use runner::*;
pub use splitter::*;
pub use splitter_drag::*;