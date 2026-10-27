// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3-Branch Lifecycle and Dynamic Frame Evaluation
//!
//! Autonomous state machine coordinating structural shell reconstructions (Branch 1),
//! isolated granular leaf panel rebuilds (Branch 2), and in-place volatile draw refreshes (Branch 3).

pub mod branch;
pub mod inputs;
pub mod state_machine;

#[cfg(test)]
mod tests;

pub use branch::LifecycleBranch;
pub use inputs::LifecycleInputs;
pub use state_machine::WorkbenchLifecycle;