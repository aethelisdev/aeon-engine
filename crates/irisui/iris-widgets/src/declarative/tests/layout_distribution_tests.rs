// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Subsystem Unit Tests

use crate::declarative::{
    UiScope, layout_subtree,
    widget::{Button, ScrollAreaView, Toolbar, ToolbarButton},
};
use iris_core::{CornerRadii, Point, Rect, UiTree, WidgetRole};

#[test]
fn test_declarative_modal_hierarchy_and_tags() {
    use crate::modal::{
        MODAL_TAG_CANCEL, MODAL_TAG_CARD, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_SCRIM,
    };
    use iris_core::{Color, UiLayer, WidgetRole};

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let screen_w = 1280.0;
    let screen_h = 720.0;
    let card_w = 400.0;
    let card_h = 200.0;

    let (scrim_resp, scrim_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.6), |scrim| {
            let card_resp = scrim.modal_card(card_w, card_h, |card| {
                card.row(|row| {
                    row.text("Title");
                    row.modal_close_button();
                });
                card.column(|col| {
                    col.text("Content line");
                });
                card.row(|footer| {
                    footer.modal_cancel_button("Cancel", 80.0);
                    footer.modal_confirm_button("OK", 80.0);
                });
            });
            scrim.finish_layout(Rect::new(0.0, 0.0, screen_w, screen_h));
            card_resp.response.id
        });
        let sid = resp.response.id;
        (resp, sid)
    };
    let scrim_node = tree.get(scrim_id).expect("Scrim node exists");
    assert_eq!(scrim_node.tag, MODAL_TAG_SCRIM);
    assert_eq!(scrim_node.role, WidgetRole::ModalWindow);
    assert_eq!(scrim_node.layer, UiLayer::Modal);
    assert_eq!(
        scrim_node.computed_rect,
        Rect::new(0.0, 0.0, screen_w, screen_h)
    );

    let card_id = scrim_resp.inner;
    let card_node = tree.get(card_id).expect("Card node exists");
    assert_eq!(card_node.tag, MODAL_TAG_CARD);
    assert!(card_node.interactive);
    assert_eq!(card_node.role, WidgetRole::ModalWindow);
    assert_eq!(card_node.layer, UiLayer::Modal);
    assert_eq!(card_node.computed_rect.width, card_w);
    assert_eq!(card_node.computed_rect.height, card_h);
    assert_eq!(card_node.computed_rect.x, (screen_w - card_w) * 0.5);
    assert_eq!(card_node.computed_rect.y, (screen_h - card_h) * 0.5);

    // Verify close, cancel, and confirm tags are in the hierarchy
    let mut found_close = false;
    let mut found_confirm = false;
    let mut found_cancel = false;
    tree.traverse_depth_first(card_id, &mut |_id, node| {
        if node.tag == MODAL_TAG_CLOSE {
            found_close = true;
        }
        if node.tag == MODAL_TAG_CONFIRM {
            found_confirm = true;
        }
        if node.tag == MODAL_TAG_CANCEL {
            found_cancel = true;
        }
    });

    assert!(found_close, "Modal close tag must exist");
    assert!(found_confirm, "Modal confirm tag must exist");
    assert!(found_cancel, "Modal cancel tag must exist");
}

#[test]
fn test_toolbar_buttons_child_non_interactive_and_hit_test() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let (icon_btn_id, mode_btn_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let icon_resp = scope.toolbar_icon_button([0.0, 0.0, 1.0, 1.0], 1001, false);
        let mode_resp = scope.toolbar_mode_button(
            Some([0.0, 0.0, 1.0, 1.0]),
            "Perspective",
            1002,
            false,
            120.0,
            CornerRadii::all(4.0),
        );
        (icon_resp.id, mode_resp.id)
    };

    // 1. Verify that all children of ToolbarIconButton have interactive = false
    let icon_btn_node = tree.get(icon_btn_id).expect("Button node exists");
    assert!(
        icon_btn_node.interactive,
        "Button parent must be interactive"
    );
    assert_eq!(icon_btn_node.tag, 1001);
    for &child_id in &icon_btn_node.children {
        let child = tree.get(child_id).expect("Child node exists");
        assert!(
            !child.interactive,
            "Child '{}' of ToolbarIconButton must NOT be interactive",
            child.name.as_deref().unwrap_or("unknown")
        );
    }

    // 2. Verify that all children of ToolbarModeButton have interactive = false
    let mode_btn_node = tree.get(mode_btn_id).expect("Mode button exists");
    assert!(mode_btn_node.interactive, "Mode button must be interactive");
    assert_eq!(mode_btn_node.tag, 1002);
    for &child_id in &mode_btn_node.children {
        let child = tree.get(child_id).expect("Child node exists");
        assert!(
            !child.interactive,
            "Child '{}' of ToolbarModeButton must NOT be interactive",
            child.name.as_deref().unwrap_or("unknown")
        );
    }

    // 3. Test hit-testing directly over the center of the icon
    // Set parent rect to (10, 10, 32, 32) and child icon to (15, 15, 22, 22)
    if let Some(btn) = tree.get_mut(icon_btn_id) {
        btn.computed_rect = Rect::new(10.0, 10.0, 32.0, 32.0);
    }
    for &child_id in &tree.get(icon_btn_id).unwrap().children.clone() {
        if let Some(c) = tree.get_mut(child_id) {
            c.computed_rect = Rect::new(15.0, 15.0, 22.0, 22.0);
        }
    }

    // Center point (26, 26) falls directly on the child icon
    let hit = tree
        .hit_test_target(Point::new(26.0, 26.0))
        .expect("Must hit toolbar button");
    assert_eq!(
        hit.tag, 1001,
        "Click on child icon area must resolve to parent button tag"
    );
    assert_eq!(hit.role, WidgetRole::Button);
}

#[test]
fn test_panel_tagged_declarative_layout_and_flex_grow() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let panel_rect = Rect::new(50.0, 100.0, 800.0, 400.0);

    let (panel_id, (tb_id, vp_id, btn_id)) = {
        let mut scope = UiScope::new(&mut tree, root);
        let panel_resp = scope
            .panel("TestPanelRoot")
            .tagged(0x5000)
            .bounds(panel_rect)
            .background(iris_core::Color::BLACK)
            .show(|panel_scope| {
                let tb_resp = Toolbar::new("TestToolbar")
                    .height(34.0)
                    .tag(0x500a)
                    .background(iris_core::Color::BLACK)
                    .show(panel_scope, |tb| {
                        let btn_resp = tb.add(ToolbarButton::new("Action").width(80.0).tag(0x5001));
                        btn_resp.id
                    });

                let vp_resp = ScrollAreaView::new("TestViewport")
                    .tag(0x500b)
                    .background(iris_core::Color::BLACK)
                    .show(panel_scope, |vp| {
                        vp.label("No entries");
                    });

                (tb_resp.response.id, vp_resp.response.id, tb_resp.inner)
            });
        let pid = panel_resp.response.id;
        (pid, panel_resp.inner)
    };

    let panel_node = tree.get(panel_id).expect("Panel node exists");
    assert_eq!(panel_node.computed_rect, panel_rect);

    let tb_node = tree.get(tb_id).expect("Toolbar node exists");
    assert_eq!(tb_node.computed_rect.x, panel_rect.x);
    assert_eq!(tb_node.computed_rect.y, panel_rect.y);
    assert_eq!(tb_node.computed_rect.width, panel_rect.width);
    assert_eq!(tb_node.computed_rect.height, 34.0);

    let vp_node = tree.get(vp_id).expect("Viewport node exists");
    assert_eq!(vp_node.computed_rect.x, panel_rect.x);
    assert_eq!(vp_node.computed_rect.y, panel_rect.y + 34.0);
    assert_eq!(vp_node.computed_rect.width, panel_rect.width);
    assert_eq!(vp_node.computed_rect.height, panel_rect.height - 34.0);

    // Verify child button received valid geometry
    let btn_node = tree.get(btn_id).expect("Button node exists");
    assert_eq!(btn_node.computed_rect.width, 80.0);
    assert_eq!(btn_node.computed_rect.height, 24.0);
    assert!(btn_node.computed_rect.x >= panel_rect.x);
    assert!(btn_node.computed_rect.y >= panel_rect.y);

    // Verify hit testing on button works
    let hit = tree
        .hit_test_target(Point::new(
            btn_node.computed_rect.x + 10.0,
            btn_node.computed_rect.y + 10.0,
        ))
        .expect("Hit test should hit button");
    assert_eq!(hit.tag, 0x5001);
}

#[test]
fn test_declarative_toggle_pill_flex_distribution_fits_bounds() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let row_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.row(|row| {
            row.add(
                Button::new("🟫 Opaque")
                    .small()
                    .active(true)
                    .flex(true)
                    .tag(101),
            );
            row.add(
                Button::new("✂️ Cutout")
                    .small()
                    .active(false)
                    .flex(true)
                    .tag(102),
            );
            row.add(
                Button::new("💧 Blend")
                    .small()
                    .active(false)
                    .flex(true)
                    .tag(103),
            );
        });
        resp.response.id
    };

    let bounds = Rect::new(10.0, 10.0, 260.0, 40.0);
    layout_subtree(&mut tree, root, bounds);

    let row_node = tree.get(row_id).expect("Row node exists");
    assert_eq!(row_node.children.len(), 3);

    let p1 = tree.get(row_node.children[0]).expect("Pill 1");
    let p2 = tree.get(row_node.children[1]).expect("Pill 2");
    let p3 = tree.get(row_node.children[2]).expect("Pill 3");

    // All three pills must have identical flex-grow width
    assert!(p1.computed_rect.width > 50.0);
    assert_eq!(p1.computed_rect.width, p2.computed_rect.width);
    assert_eq!(p2.computed_rect.width, p3.computed_rect.width);

    // Pill 3 ("💧 Blend") must stay strictly within bounds without overflowing
    assert!(p3.computed_rect.right() <= bounds.right());
}

#[test]
fn test_declarative_row_flex_badge_and_button_fits_bounds() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");
    if let Some(node) = tree.get_mut(root) {
        node.set_style(iris_core::Style::new().flex_col());
    }

    let row_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.row(|row| {
            row.label("Texture:");
            let badge_style = iris_core::Style::new()
                .flex_row()
                .flex_grow(1.0)
                .clip_children(true);
            row.container(badge_style, |badge| {
                badge.label("stylized_ww1_plane.glb_tex_0");
            });
            let _ = row.button_with_icon_styled_tagged(
                [0.0, 0.0, 1.0, 1.0],
                iris_core::Color::WHITE,
                "Change",
                None,
                999,
            );
        });
        resp.response.id
    };

    let bounds = Rect::new(10.0, 10.0, 260.0, 40.0);
    layout_subtree(&mut tree, root, bounds);

    let row_node = tree.get(row_id).expect("Row node exists");
    assert_eq!(row_node.children.len(), 3);

    let label_node = tree.get(row_node.children[0]).expect("Label");
    let badge_node = tree.get(row_node.children[1]).expect("Badge");
    let btn_node = tree.get(row_node.children[2]).expect("Button");

    // All elements have positive width
    assert!(label_node.computed_rect.width > 30.0);
    assert!(badge_node.computed_rect.width > 30.0);
    assert!(btn_node.computed_rect.width > 50.0);

    // Button must fit inside row bounds without being pushed off
    assert!(btn_node.computed_rect.right() <= bounds.right());
}

#[test]
fn test_space_between_does_not_stretch_unconstrained_children() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let tb_resp = scope.container_named(
        "Toolbar",
        iris_core::Style::new()
            .flex_row()
            .justify_content(iris_core::JustifyContent::SpaceBetween)
            .width(1000.0)
            .height(30.0),
        |tb| {
            let l_resp = tb.container_named(
                "LeftGroup",
                iris_core::Style::new().flex_row().height(24.0),
                |left| {
                    left.label_styled_passive(
                        "BreadcrumbText",
                        "assets",
                        11.5,
                        iris_core::Color::WHITE,
                        iris_core::TextAlign::Left,
                        iris_core::Style::new(),
                    );
                },
            );

            let r_resp = tb.container_named(
                "RightGroup",
                iris_core::Style::new().flex_row().width(500.0).height(24.0),
                |_right| {},
            );

            (l_resp.response.id, r_resp.response.id)
        },
    );

    let (left_id, right_id) = tb_resp.inner;

    let bounds = Rect::new(0.0, 0.0, 1000.0, 30.0);
    scope.finish_layout(bounds);

    let left_node = tree.get(left_id).expect("Left node exists");
    let right_node = tree.get(right_id).expect("Right node exists");

    // Left child should take its intrinsic content width (<100px), NOT stretch to fill 500px!
    assert!(left_node.computed_rect.width < 100.0);
    assert_eq!(left_node.computed_rect.x, 0.0);

    // Right child should have width 500px and be pinned to the right edge (x + w == 1000.0)
    assert_eq!(right_node.computed_rect.width, 500.0);
    assert_eq!(right_node.computed_rect.right(), 1000.0);
}

#[test]
fn test_declarative_text_wrap_multi_line_height_measurement() {
    use crate::declarative::measure_height_constrained;
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let long_desc = "Direct hardware execution with low-overhead command recording, multi-draw indirect, and compute passes.";

    scope.container_named(
        "ColContainer",
        iris_core::Style::new().flex_col().width(150.0),
        |col| {
            col.label_styled_passive(
                "LongDesc",
                long_desc,
                11.0,
                iris_core::Color::WHITE,
                iris_core::TextAlign::Left,
                iris_core::Style::new(),
            );
        },
    );

    let lid = scope
        .tree()
        .find_node_by_name("LongDesc")
        .expect("LongDesc exists");

    // Unconstrained should measure at least single line height (~14px..20px)
    let h_unconstrained = measure_height_constrained(scope.tree(), lid, None);
    assert!((14.0..=20.0).contains(&h_unconstrained));

    // In a narrow width constraint of 150px, this 104-character string should wrap to multiple lines (>30px)
    let h_constrained = measure_height_constrained(scope.tree(), lid, Some(150.0));
    assert!(
        h_constrained >= 30.0,
        "Expected wrapped height >= 30.0, got {h_constrained}"
    );

    // Finish layout in a column container with width 150.0
    let bounds = Rect::new(0.0, 0.0, 150.0, 300.0);
    scope.finish_layout(bounds);

    let node = tree.get(lid).expect("Node exists");
    assert!(
        node.computed_rect.height >= 30.0,
        "Expected computed_rect.height >= 30.0, got {}",
        node.computed_rect.height
    );
    assert_eq!(node.computed_rect.width, 150.0);
}