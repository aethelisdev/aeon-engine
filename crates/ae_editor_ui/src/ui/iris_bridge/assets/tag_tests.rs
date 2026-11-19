// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Assets Semantic Tag Invariant Unit Tests
//!
//! Validates deterministic `UiTag` derivation and collision-freedom across
//! all static and indexed Asset Browser interaction tags.

use super::types::*;
use std::collections::HashSet;

#[test]
fn test_assets_semantic_tags_uniqueness_and_determinism() {
    let static_tags = [
        ASSETS_TAG_PANEL_ROOT,
        ASSETS_TAG_TOGGLE_SIDEBAR,
        ASSETS_TAG_IMPORT,
        ASSETS_TAG_CLEAN_VRAM,
        ASSETS_TAG_VIEW_GRID,
        ASSETS_TAG_VIEW_LIST,
        ASSETS_TAG_ENGINE_CONTENT,
        ASSETS_TAG_SEARCH_INPUT,
        ASSETS_TAG_SEARCH_CLEAR,
        ASSETS_TAG_TREE_VIEWPORT,
        ASSETS_TAG_CONTENT_VIEWPORT,
        ASSETS_TAG_SCROLLBAR_TRACK,
        ASSETS_TAG_SCROLLBAR_THUMB,
        ASSETS_TAG_TREE_SCROLLBAR_TRACK,
        ASSETS_TAG_TREE_SCROLLBAR_THUMB,
        ASSETS_TAG_NEW_SUBFOLDER,
        ASSETS_TAG_REVEAL,
        ASSETS_TAG_PREVIEW_MODAL,
        ASSET_PREVIEW_TAG_REVEAL,
        ASSET_PREVIEW_TAG_ORBIT,
    ];

    let mut seen = HashSet::new();
    for tag in static_tags {
        assert_ne!(tag, 0, "Semantic tag must never be zero");
        assert!(
            seen.insert(tag),
            "Static asset semantic tag collision detected: {tag}"
        );
    }

    for cat_idx in 0..=8 {
        let tag = encode_chip_tag(cat_idx);
        assert_eq!(tag, encode_chip_tag(cat_idx));
        assert!(seen.insert(tag), "Chip tag collision at index {cat_idx}");
    }

    for seg_idx in [0, 1, 5, 128, 255] {
        let tag = encode_breadcrumb_tag(seg_idx);
        assert_eq!(tag, encode_breadcrumb_tag(seg_idx));
        assert!(
            seen.insert(tag),
            "Breadcrumb tag collision at index {seg_idx}"
        );
    }

    for node_idx in [0, 1, 42, 1024, 65535] {
        let row_tag = encode_tree_row_tag(node_idx);
        let chev_tag = encode_tree_chevron_tag(node_idx);
        assert_ne!(row_tag, chev_tag);
        assert!(seen.insert(row_tag));
        assert!(seen.insert(chev_tag));
    }

    for item_idx in [0, 1, 99, 10000, 65535] {
        let select_tag = encode_item_tag(item_idx);
        let spawn_tag = encode_item_spawn_tag(item_idx);
        let inspect_tag = encode_item_inspect_tag(item_idx);
        assert!(seen.insert(select_tag));
        assert!(seen.insert(spawn_tag));
        assert!(seen.insert(inspect_tag));
    }
}