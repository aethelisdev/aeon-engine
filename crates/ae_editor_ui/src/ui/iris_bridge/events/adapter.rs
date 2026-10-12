// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Winit WindowEvent to Iris UI Event Adapter
//!
//! Re-exports the hardware-independent platform event translation routines from [`iris_winit`].
//!

pub use iris_winit::adapter::winit_to_ui_event;