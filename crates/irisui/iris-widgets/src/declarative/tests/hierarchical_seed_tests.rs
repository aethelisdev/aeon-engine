// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical Seed & Tag Isolation Subsystem Tests
//!
//! Validates deterministic seed isolation across named scopes (`panel`, `collapsing`,
//! `dropdown_menu_card`, `with_id`), anonymous layout seed preservation (`row`, `column`, `container`),
//! and autonomous menu bar item tag generation conforming to FAZ 4.2.6 standards.

use crate::declarative::{UiScope, widget::Button};
use iris_core::{Color, UiTree};

#[test]
fn test_hierarchical_seed_isolation_between_panels() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");
    let mut scope = UiScope::new(&mut tree, root);

    let mut panel_a_btn_tag = 0;
    let mut panel_b_btn_tag = 0;

    scope.panel("PanelA").show(|panel_a| {
        panel_a_btn_tag = panel_a.tag_for("Delete");
        let _ = panel_a.button("Delete");
    });

    scope.panel("PanelB").show(|panel_b| {
        panel_b_btn_tag = panel_b.tag_for("Delete");
        let _ = panel_b.button("Delete");
    });

    assert_ne!(
        panel_a_btn_tag, panel_b_btn_tag,
        "Buttons with identical labels across different named panels must receive distinct tags"
    );
}

#[test]
fn test_hierarchical_seed_isolation_between_collapsing_headers() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");
    let mut scope = UiScope::new(&mut tree, root);

    let mut header_1_btn_tag = 0;
    let mut header_2_btn_tag = 0;

    scope.collapsing_default_open("FirstSection", |s1| {
        header_1_btn_tag = s1.tag_for("Apply");
        let _ = s1.button("Apply");
    });

    scope.collapsing_default_open("SecondSection", |s2| {
        header_2_btn_tag = s2.tag_for("Apply");
        let _ = s2.button("Apply");
    });

    assert_ne!(
        header_1_btn_tag, header_2_btn_tag,
        "Buttons with identical labels across different collapsing scopes must receive distinct tags"
    );
}

#[test]
fn test_anonymous_layout_containers_preserve_seed() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");
    let mut scope = UiScope::new(&mut tree, root);

    let root_tag = scope.tag_for("Submit");
    let mut row_tag = 0;
    let mut col_tag = 0;

    scope.row(|row_scope| {
        row_tag = row_scope.tag_for("Submit");
        row_scope.column(|col_scope| {
            col_tag = col_scope.tag_for("Submit");
        });
    });

    assert_eq!(
        root_tag, row_tag,
        "Row container must preserve parent seed for layout stability"
    );
    assert_eq!(
        row_tag, col_tag,
        "Column container must preserve parent seed for layout stability"
    );
}

#[test]
fn test_menu_bar_items_and_actions_derive_tags_autonomously() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");
    let (file_id, edit_id, play_id, expected_file_tag, expected_edit_tag, expected_play_tag) = {
        let mut scope = UiScope::new(&mut tree, root);
        let file_resp = scope.menu_bar_item("File", false);
        let edit_resp = scope.menu_bar_item("Edit", false);
        let play_resp = scope.add(
            Button::new("▶ Play")
                .fill(Color::rgba(0.2, 0.6, 0.3, 1.0))
                .text_color(Color::WHITE),
        );

        (
            file_resp.id,
            edit_resp.id,
            play_resp.id,
            scope.tag_for("File"),
            scope.tag_for("Edit"),
            scope.tag_for("▶ Play"),
        )
    };

    let file_node = tree.get(file_id).expect("File item exists");
    let edit_node = tree.get(edit_id).expect("Edit item exists");
    let play_node = tree.get(play_id).expect("Play button exists");

    assert_ne!(
        file_node.tag, 0,
        "Menu item must have non-zero autonomous tag"
    );
    assert_ne!(
        edit_node.tag, 0,
        "Menu item must have non-zero autonomous tag"
    );
    assert_ne!(
        file_node.tag, edit_node.tag,
        "Distinct menu items must have distinct tags"
    );
    assert_eq!(file_node.tag, expected_file_tag);
    assert_eq!(edit_node.tag, expected_edit_tag);
    assert_eq!(play_node.tag, expected_play_tag);
}