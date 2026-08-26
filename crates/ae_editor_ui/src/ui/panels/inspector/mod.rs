// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Sub-module — Handles component editing, transform drag inputs, appearance, audio, and physics inspection.
//!

pub mod animation;
pub mod appearance;
pub mod audio;
pub mod behavior;
pub mod dynamic_reflection;
pub mod lod;
pub mod panel;
pub mod parenting;
pub mod physics;
pub mod registry;
pub mod ui_components;
pub mod widgets;

pub use registry::{ComponentUiHandler, InspectorContext, InspectorUiRegistry};