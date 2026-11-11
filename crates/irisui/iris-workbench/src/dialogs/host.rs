// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Dialog and Modal Host State Machine
//!
//! Autonomous coordinator managing declarative modal dialog state transitions,
//! multi-modal hierarchical stacking, semi-transparent backdrop scrim occlusion,
//! keyboard focus traps (Escape to dismiss, Enter to confirm), and automatic routing
//! of confirm/submit payloads into the [`ActionBus`].

pub mod active;
pub mod convert;
pub mod focus;
pub mod render;
pub mod stack;
pub mod trap;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_focus;
#[cfg(test)]
mod tests_stacking;
#[cfg(test)]
mod tests_trap;

pub use active::{ActiveDialog, CustomModalRenderFn, CustomSheetRenderFn, DialogKind};
pub use focus::{DialogFocusCycle, DialogFocusItem};
pub use stack::DialogHost;
pub use trap::DialogFocusTrap;