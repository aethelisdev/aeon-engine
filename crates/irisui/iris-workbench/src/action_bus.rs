// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Type-Safe Centralized Action Bus
//!
//! Provides a lock-free, zero-reallocation FIFO action queue decoupling
//! UI control interactions (menu bars, toolbars, key bindings, modal dialogs, panels)
//! from backend application execution loops.

pub mod bus;

#[cfg(test)]
mod tests;

pub use bus::ActionBus;
