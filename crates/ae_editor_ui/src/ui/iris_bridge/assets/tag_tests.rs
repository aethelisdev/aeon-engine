// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Assets Semantic Tag and Action Invariant Unit Tests
//!
//! Validates encoding/decoding roundtrips, domain filters, and typed
//! tag and context menu action resolution invariants.
//!

use super::types::*;

#[test]
fn test_assets_semantic_tags_encoding_roundtrip() {
    // 1. Domain verification
    assert!(is_assets_tag(ASSETS_TAG_PANEL_ROOT));
    assert!(is_assets_tag(ASSETS_TAG_TOGGLE_SIDEBAR));
    assert!(is_assets_tag(ASSETS_TAG_IMPORT));
    assert!(is_assets_tag(ASSETS_TAG_VIEW_GRID));
    assert!(is_assets_tag(ASSETS_TAG_VIEW_LIST));
    assert!(!is_assets_tag(0x0080_0000_0000_0001)); // Preferences domain
    assert!(!is_assets_tag(0x0070_0000_0000_0001)); // UI designer domain
    assert!(!is_assets_tag(0));

    // 2. Category Chips Roundtrip (0..8)
    for cat_idx in 0..=8 {
        let tag = encode_chip_tag(cat_idx);
        assert!(is_assets_tag(tag));
        assert_eq!(parse_chip_tag(tag), Some(cat_idx));
    }
    assert_eq!(parse_chip_tag(ASSETS_TAG_PANEL_ROOT), None);

    // 3. Breadcrumb Navigation Segments Roundtrip (0..255)
    for seg_idx in [0, 1, 5, 128, 255] {
        let tag = encode_breadcrumb_tag(seg_idx);
        assert!(is_assets_tag(tag));
        assert_eq!(parse_breadcrumb_tag(tag), Some(seg_idx));
    }
    assert_eq!(parse_breadcrumb_tag(ASSETS_TAG_PANEL_ROOT), None);

    // 4. Context Menu Actions Roundtrip (0..15)
    for action_idx in 0..=6 {
        let tag = encode_ctx_item_tag(action_idx);
        assert!(is_assets_tag(tag));
        assert_eq!(parse_ctx_item_tag(tag), Some(action_idx));
    }
    assert_eq!(parse_ctx_item_tag(ASSETS_TAG_PANEL_ROOT), None);

    // 5. Folder Tree Rows and Chevrons Roundtrip (0..u32::MAX)
    for node_idx in [0, 1, 42, 1024, 0x00FF_FFFF] {
        let row_tag = encode_tree_row_tag(node_idx);
        let chev_tag = encode_tree_chevron_tag(node_idx);
        assert!(is_assets_tag(row_tag));
        assert!(is_assets_tag(chev_tag));
        assert_eq!(parse_tree_tag(row_tag), Some((node_idx, false)));
        assert_eq!(parse_tree_tag(chev_tag), Some((node_idx, true)));
    }
    assert_eq!(parse_tree_tag(ASSETS_TAG_PANEL_ROOT), None);

    // 6. Asset Items (Select, Spawn, Inspect) Roundtrip
    for item_idx in [0, 1, 99, 10000, 0x00FF_FFFF] {
        let select_tag = encode_item_tag(item_idx);
        let spawn_tag = encode_item_spawn_tag(item_idx);
        let inspect_tag = encode_item_inspect_tag(item_idx);

        assert!(is_assets_tag(select_tag));
        assert!(is_assets_tag(spawn_tag));
        assert!(is_assets_tag(inspect_tag));

        assert_eq!(
            parse_item_tag(select_tag),
            Some((item_idx, AssetItemAction::SelectOrOpen))
        );
        assert_eq!(
            parse_item_tag(spawn_tag),
            Some((item_idx, AssetItemAction::Spawn))
        );
        assert_eq!(
            parse_item_tag(inspect_tag),
            Some((item_idx, AssetItemAction::Inspect))
        );
    }
    assert_eq!(parse_item_tag(ASSETS_TAG_PANEL_ROOT), None);
}

#[test]
fn test_resolve_assets_tag_and_ctx_action_invariants() {
    // Context menu actions resolution
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_INSPECT),
        Some(AssetsContextMenuAction::Inspect)
    );
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_SPAWN),
        Some(AssetsContextMenuAction::Spawn)
    );
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_NEW_FOLDER),
        Some(AssetsContextMenuAction::NewFolder)
    );
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_RENAME),
        Some(AssetsContextMenuAction::Rename)
    );
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_DELETE),
        Some(AssetsContextMenuAction::Delete)
    );
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_COPY_PATH),
        Some(AssetsContextMenuAction::CopyPath)
    );
    assert_eq!(
        resolve_assets_ctx_action(ASSET_CTX_REVEAL),
        Some(AssetsContextMenuAction::Reveal)
    );
    assert_eq!(resolve_assets_ctx_action(999), None);

    // Semantic tag resolution
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_TOGGLE_SIDEBAR),
        Some(AssetsTagTarget::ToggleSidebar)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_IMPORT),
        Some(AssetsTagTarget::Import)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_CLEAN_VRAM),
        Some(AssetsTagTarget::CleanVram)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_VIEW_GRID),
        Some(AssetsTagTarget::ViewGrid)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_VIEW_LIST),
        Some(AssetsTagTarget::ViewList)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_ENGINE_CONTENT),
        Some(AssetsTagTarget::EngineContent)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_SEARCH_INPUT),
        Some(AssetsTagTarget::SearchInput)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_SEARCH_CLEAR),
        Some(AssetsTagTarget::SearchClear)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_NEW_SUBFOLDER),
        Some(AssetsTagTarget::NewSubfolder)
    );
    assert_eq!(
        resolve_assets_tag(ASSETS_TAG_REVEAL),
        Some(AssetsTagTarget::Reveal)
    );
    assert_eq!(
        resolve_assets_tag(encode_chip_tag(3)),
        Some(AssetsTagTarget::Chip(3))
    );
    assert_eq!(
        resolve_assets_tag(encode_breadcrumb_tag(2)),
        Some(AssetsTagTarget::Breadcrumb(2))
    );
    assert_eq!(
        resolve_assets_tag(encode_tree_row_tag(5)),
        Some(AssetsTagTarget::Tree(5, false))
    );
    assert_eq!(
        resolve_assets_tag(encode_tree_chevron_tag(7)),
        Some(AssetsTagTarget::Tree(7, true))
    );
    assert_eq!(
        resolve_assets_tag(encode_item_tag(12)),
        Some(AssetsTagTarget::Item(12, AssetItemAction::SelectOrOpen))
    );
    assert_eq!(resolve_assets_tag(0), None);
    assert_eq!(resolve_assets_tag(0xFFFF_FFFF), None);
}