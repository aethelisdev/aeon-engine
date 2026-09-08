// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! AE Core Math — Unified conversion traits and helpers for cgmath <-> glam interop.
//!
//! Exposes conversion traits and utilities across geometric libraries used
//! within the engine, maintaining strict decoupling between subsystems.
//!

pub mod conversions;

pub use conversions::*;