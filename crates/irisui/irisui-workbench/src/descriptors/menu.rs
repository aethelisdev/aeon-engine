// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Type-Safe Menu Bar and Action Descriptors
//!
//! Pure module root re-exporting menu item, section, branding, and menu bar descriptors.

mod bar;
mod items;

pub use bar::MenuBarDescriptor;
pub use items::{
    MenuBarBrandingDescriptor, MenuItemDescriptor, MenuRightItemDescriptor, MenuSectionDescriptor,
};