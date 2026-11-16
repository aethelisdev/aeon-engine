// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained-mode docking chrome rendering, tab strip generation, and splitter management.

pub mod builder;
pub mod frame;
pub mod tags;

pub use builder::build_dock_chrome;
pub use frame::{
    DEFAULT_SPLITTER_HIT_MARGIN, DEFAULT_SPLITTER_THICKNESS, DEFAULT_SPLITTER_TOTAL_HIT_ZONE,
    DockChevronTarget, DockChromeFrame, DockChromeParams, DockChromeStyle, DockCloseTarget,
    DockSplitterTarget, DockTabTarget, hit_splitter,
};
pub use tags::{
    TAG_DOCK_TAB_BASE, TAG_DOCK_TAB_CHEVRON_BASE, TAG_DOCK_TAB_CLOSE_BASE,
    encode_dock_tab_chevron_tag, encode_dock_tab_close_tag, encode_dock_tab_tag,
    is_dock_chrome_tag, is_dock_tab_chevron_tag, is_dock_tab_close_tag, is_dock_tab_tag,
};

#[cfg(test)]
#[path = "chrome_tests.rs"]
mod tests;