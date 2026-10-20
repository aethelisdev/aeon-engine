// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toolbar & ToolbarGroup Container Builders
//!
//! Provides the configurable [`Toolbar`] floating pill container builder and [`ToolbarGroup`]
//! modular section builders for assembling desktop application tool strips, segmented mode
//! selectors, and interactive utility toolbars without engine-specific coupling.
//!

pub mod container;
pub mod group;
pub mod group_scope;
pub mod scope;

pub use container::Toolbar;
pub use group::ToolbarGroup;
pub use group_scope::ToolbarGroupScope;
pub use scope::ToolbarScope;