// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(clippy::allow_attributes)]

//! AE Editor UI — Pure declarative Iris UI panels, docking layout, and [`ui::EngineUiAction`] emitter.

pub mod ui;