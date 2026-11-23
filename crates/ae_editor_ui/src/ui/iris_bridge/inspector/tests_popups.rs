// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Floating Popups Unit Tests
//!
//! Verifies 100% declarative [`UiScope`] floating popup construction, layout isolation,
//! and $O(1)$ semantic tag hit-testing for ComboBox dropdowns, Cascading Add Component menus,
//! and 2D HSV Color Pickers.

use super::*;
use ae_plugin_api::Position;
use irisui::prelude::*;

#[test]
fn test_inspector_add_menu_cascading_declarative_scope_and_hit_testing() {
    use irisui::widgets::declarative::hash_label;
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = ae_ecs::World::new();
    let entity = world.spawn((Position::default(),));

    // Build Add Component button anchor via declarative UiScope directly in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "AddComponentButton",
            Style::new()
                .position_absolute()
                .left(50.0)
                .top(500.0)
                .width(220.0)
                .height(28.0),
            WidgetRole::Button,
            TAG_INSPECTOR_ADD_COMPONENT,
            |btn| {
                btn.label_styled_passive(
                    "AddButtonLabel",
                    "➕ Add Component",
                    12.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().height(24.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    // Build the cascading add menu via declarative UiScope
    tree.open_popup(TAG_INSPECTOR_ADD_COMPONENT);
    tree.set_active_submenu(
        hash_label("AddComponentMenu"),
        ComponentCategory::Physics.to_tag(),
    );
    add_menu::draw_add_component_menu(&mut UiScope::new(&mut tree, root), &world, Some(entity));

    // Find Physics category card item
    let mut physics_cat_node = None;
    for (_id, node) in tree.iter() {
        if node.layer == UiLayer::Popup
            && node.role == WidgetRole::DropdownItem
            && node.tag == ComponentCategory::Physics.to_tag()
        {
            physics_cat_node = Some(node.clone());
            break;
        }
    }
    assert!(
        physics_cat_node.is_some(),
        "Physics category item must be constructed in Level 0 menu"
    );

    // Hit-test an item inside the Submenu (Level 1)
    let item_node = tree.iter().find(|(_, n)| {
        n.role == WidgetRole::DropdownItem
            && add_menu::resolve_component_name_from_tag(n.tag).is_some()
    });
    let (_, hit_node) = item_node.expect("Level 1 submenu item must exist");
    let hit_center = Point::new(
        hit_node.computed_rect.x + hit_node.computed_rect.width * 0.5,
        hit_node.computed_rect.y + hit_node.computed_rect.height * 0.5,
    );

    let hit = tree
        .hit_test_target(hit_center)
        .expect("Must hit component item in submenu");
    assert_eq!(hit.layer, UiLayer::Popup);
    assert_eq!(hit.role, WidgetRole::DropdownItem);

    let resolved_name = add_menu::resolve_component_name_from_tag(hit.tag);
    assert!(
        resolved_name.is_some(),
        "Hit tag must resolve to a valid component name"
    );
}

#[test]
fn test_inspector_color_picker_popup_declarative_scope_and_hit_testing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = ae_ecs::World::new();
    let entity = world.spawn((Position::default(),));

    // Build Swatch trigger anchor via declarative UiScope directly in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "ColorSwatchAnchor",
            Style::new()
                .position_absolute()
                .left(100.0)
                .top(200.0)
                .width(48.0)
                .height(20.0),
            WidgetRole::ColorSwatch,
            appearance_color_swatch_tag(),
            |btn| {
                btn.label_styled_passive(
                    "SwatchLabel",
                    "■",
                    11.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().height(18.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    // Build color picker popup via declarative UiScope
    tree.open_popup(appearance_color_swatch_tag());
    let mut scope = UiScope::new(&mut tree, root);
    color_picker_popup::build_color_picker_popup(&mut scope, &world, Some(entity));

    // Verify popup elements exist with UiLayer::Popup
    let mut sv_box_found = false;
    let mut hue_bar_found = false;
    let mut close_btn_found = false;

    for (id, node) in tree.iter() {
        if tree.effective_layer(id) == UiLayer::Popup {
            if node.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX {
                sv_box_found = true;
                assert!(node.computed_rect.width >= 150.0);
                assert!(node.computed_rect.height >= 120.0);
            } else if node.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_HUE_BAR {
                hue_bar_found = true;
                assert!(node.computed_rect.width >= 15.0);
                assert!(node.computed_rect.height >= 120.0);
            } else if node.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_CLOSE {
                close_btn_found = true;
            }
        }
    }

    assert!(sv_box_found, "2D SV Box must exist in Popup layer");
    assert!(hue_bar_found, "Hue Bar must exist in Popup layer");
    assert!(close_btn_found, "Close Button must exist in Popup layer");

    // Verify hardware hit-testing hits the SV box and returns the full container's bounding rectangle
    let (_, sv_node) = tree
        .iter()
        .find(|(_, n)| n.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX)
        .expect("SV box must exist");
    let test_point = Point::new(
        sv_node.computed_rect.x + 20.0,
        sv_node.computed_rect.y + 20.0,
    );
    let hit = tree
        .hit_test_target(test_point)
        .expect("SV box point must hit a target");
    assert_eq!(
        hit.tag,
        color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
        "Clicking inside SV box must hit TAG_INSPECTOR_COLOR_PICKER_SV_BOX"
    );
    assert_eq!(
        hit.rect, sv_node.computed_rect,
        "Hit rect must match the full SV box container, not an internal cell"
    );
}

#[test]
fn test_clicking_appearance_color_swatch_does_not_double_toggle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = ae_ecs::World::new();
    let entity = world.spawn((Position::default(),));

    let mut name = String::new();
    let _ = panel::build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &world,
        Some(entity),
        &mut name,
    );
    let _ = LayoutEngine::new().compute_layout(&mut tree, Size::new(320.0, 900.0));

    let swatch_rect = tree
        .node_rect_tagged(appearance_color_swatch_tag(), Point::ZERO)
        .expect("Color swatch must exist in Inspector panel");
    let click_pt = Point::new(swatch_rect.x + 4.0, swatch_rect.y + 4.0);

    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pt,
        button: MouseButton::Left,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pt,
        button: MouseButton::Left,
    });

    assert_eq!(
        tree.event_state.frame_clicked_tag,
        Some(appearance_color_swatch_tag()),
        "Swatch click must record semantic appearance_color_swatch_tag in event state"
    );

    tree.clear();
    let root2 = tree.create_root().expect("root2");
    let _ = panel::build_inspector_panel(
        &mut UiScope::new(&mut tree, root2),
        &world,
        Some(entity),
        &mut name,
    );

    assert!(
        tree.is_popup_tag_active(appearance_color_swatch_tag()),
        "Clicking Object Color swatch must directly activate appearance_color_swatch_tag in UiTree"
    );
}

/// Verifies that continuous Color Picker dragging mutates the entity in real time
/// for smooth viewport rendering without flooding the undo stack, and commits
/// exactly one atomic undo command on release.
#[test]
fn test_color_picker_drag_and_undo_restoration() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = ae_ecs::World::new();
    let initial_color = ae_plugin_api::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
    let entity = world.spawn((initial_color,));

    tree.open_popup(appearance_color_swatch_tag());
    let mut ui_actions = Vec::new();
    // 0. Initial popup build to register SV box bounds
    let (_, init_act) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut UiScope::new(&mut tree, root),
            &world,
            Some(entity),
        );
    assert!(init_act.is_none());

    let sv_rect = tree
        .iter()
        .find(|(_, n)| n.tag == super::color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX)
        .map(|(_, n)| n.computed_rect)
        .expect("SV box must exist");

    // 1. Mouse down on SV box begins color editing
    let start_pt = Point::new(sv_rect.x + 15.0, sv_rect.y + 15.0);
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: start_pt,
    });
    let _ = tree.clear_children(root);
    if let (_, Some(act)) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut UiScope::new(&mut tree, root),
            &world,
            Some(entity),
        )
    {
        ui_actions.push(act);
    }
    assert!(
        ui_actions.is_empty(),
        "MouseDown must not produce undo actions prematurely"
    );

    // 2. Simulate 20 continuous drag frames (mouse movement)
    for i in 1..=20 {
        let drag_pt = Point::new(
            sv_rect.x + 15.0 + (i as f32) * 5.0,
            sv_rect.y + 15.0 + (i as f32) * 4.0,
        );
        let _ = tree.dispatch_event(&UiEvent::MouseMove { point: drag_pt });
        let _ = tree.clear_children(root);
        if let (_, Some(act)) =
            crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
                &mut UiScope::new(&mut tree, root),
                &world,
                Some(entity),
            )
        {
            ui_actions.push(act);
        }
        let cur = world.get_copied::<ae_plugin_api::Color>(entity).unwrap();
        assert_ne!(
            cur, initial_color,
            "Entity color in world must update live during drag"
        );
    }
    assert!(
        ui_actions.is_empty(),
        "Live dragging must not spam undo actions during dragging"
    );

    // 3. Mouse released -> commits atomic ModifyColor
    let end_pt = Point::new(sv_rect.x + 115.0, sv_rect.y + 95.0);
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: end_pt,
    });
    let _ = tree.clear_children(root);
    if let (_, Some(act)) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut UiScope::new(&mut tree, root),
            &world,
            Some(entity),
        )
    {
        ui_actions.push(act);
    }
    assert_eq!(
        ui_actions.len(),
        1,
        "Drag release must emit exactly one atomic ModifyColor action"
    );

    let final_color = world.get_copied::<ae_plugin_api::Color>(entity).unwrap();
    if let crate::ui::EngineUiAction::ModifyColor(ent, old_c, new_c) = ui_actions[0] {
        assert_eq!(ent, entity);
        assert_eq!(old_c, initial_color);
        assert_eq!(new_c, final_color);

        // 4. Verify Undo restoration
        let mut undo_cmd = ae_editor::undo_redo::Command::Modify(
            ent,
            ae_editor::undo_redo::Property::Color(old_c, new_c),
        );
        undo_cmd.undo(&mut world);
        let restored_color = world.get_copied::<ae_plugin_api::Color>(entity).unwrap();
        assert_eq!(
            restored_color, initial_color,
            "Undo must restore the exact pre-edit color"
        );
    } else {
        panic!("Expected EngineUiAction::ModifyColor");
    }
}

/// Verifies that clicking the close button on the color picker popup clears `appearance_color_swatch_tag` in `UiTree`.
#[test]
fn test_color_picker_popup_close_button_interaction() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let world = ae_ecs::World::new();

    tree.open_popup(appearance_color_swatch_tag());
    {
        let mut scope = UiScope::new(&mut tree, root);
        color_picker_popup::build_color_picker_popup(&mut scope, &world, None);
    }
    let _ = LayoutEngine::new().compute_layout(&mut tree, Size::new(800.0, 600.0));

    let close_rect = tree
        .node_rect_tagged(
            color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_CLOSE,
            Point::ZERO,
        )
        .expect("Close button must carry TAG_INSPECTOR_COLOR_PICKER_CLOSE");
    let click_pt = Point::new(close_rect.x + 2.0, close_rect.y + 2.0);

    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pt,
        button: MouseButton::Left,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pt,
        button: MouseButton::Left,
    });

    assert_eq!(
        tree.event_state.frame_clicked_tag,
        Some(color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_CLOSE),
        "Close button click must record tag in event state"
    );

    tree.clear();
    let root2 = tree.create_root().expect("root2");
    {
        let mut scope = UiScope::new(&mut tree, root2);
        color_picker_popup::build_color_picker_popup(&mut scope, &world, None);
    }

    assert!(
        !tree.is_popup_tag_active(appearance_color_swatch_tag()),
        "Clicking close button must clear appearance_color_swatch_tag in UiTree"
    );
}

#[test]
fn test_inspector_cascading_add_component_menu_outside_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let mut world = ae_ecs::World::new();
    let entity = world.spawn((Position::default(),));

    // 1. Create Add Component button anchor in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "AddComponentButton",
            Style::new()
                .position_absolute()
                .left(50.0)
                .top(400.0)
                .width(200.0)
                .height(28.0),
            WidgetRole::Button,
            TAG_INSPECTOR_ADD_COMPONENT,
            |btn| {
                btn.label_styled_passive(
                    "AddButtonLabel",
                    "➕ Add Component",
                    12.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().height(24.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    tree.open_popup(TAG_INSPECTOR_ADD_COMPONENT);

    // 2. Build cascading add menu
    add_menu::draw_add_component_menu(&mut UiScope::new(&mut tree, root), &world, Some(entity));

    // 3. Verify TAG_INSPECTOR_ADD_COMPONENT is registered as active popup tag
    assert!(
        tree.is_popup_tag_active(TAG_INSPECTOR_ADD_COMPONENT),
        "Add component trigger tag must be registered as active popup tag"
    );

    // 4. Click on trigger button: inside popup check
    let btn_click = Point::new(60.0, 410.0);
    assert!(
        tree.is_point_inside_popup(btn_click),
        "Click on Add Component button must be recognized as inside popup"
    );
    assert!(
        !tree.dismiss_popups_on_outside_click(btn_click),
        "Click on Add Component button must not dismiss popup"
    );
    assert!(tree.is_popup_tag_active(TAG_INSPECTOR_ADD_COMPONENT));

    // 5. Click outside: dismiss popups
    let outside_point = Point::new(800.0, 800.0);
    assert!(
        !tree.is_point_inside_popup(outside_point),
        "Point (800, 800) must be outside popup"
    );
    assert!(
        tree.dismiss_popups_on_outside_click(outside_point),
        "Outside click must dismiss popup"
    );
    assert!(
        !tree.is_popup_tag_active(TAG_INSPECTOR_ADD_COMPONENT),
        "Active popup tag must be cleared on dismissal"
    );
    assert!(tree.popups_dismissed());
}

#[test]
fn test_inspector_color_picker_popup_outside_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let mut world = ae_ecs::World::new();
    let entity = world.spawn((Position::default(),));

    let swatch_tag = super::appearance::appearance_color_swatch_tag();

    // 1. Create color swatch trigger button in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "ColorSwatchButton",
            Style::new()
                .position_absolute()
                .left(50.0)
                .top(200.0)
                .width(40.0)
                .height(20.0),
            WidgetRole::Button,
            swatch_tag,
            |_btn| {},
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    tree.open_popup(swatch_tag);

    // 2. Build color picker popup
    {
        let mut scope = UiScope::new(&mut tree, root);
        color_picker_popup::build_color_picker_popup(&mut scope, &world, Some(entity));
    }

    // 3. Verify swatch tag is registered as active popup tag
    assert!(
        tree.is_popup_tag_active(swatch_tag),
        "Color swatch trigger tag must be registered as active popup tag"
    );

    // 4. Click on swatch button: inside popup check
    let swatch_click = Point::new(60.0, 210.0);
    assert!(
        tree.is_point_inside_popup(swatch_click),
        "Click on color swatch button must be recognized as inside popup"
    );
    assert!(
        !tree.dismiss_popups_on_outside_click(swatch_click),
        "Click on color swatch button must not dismiss popup"
    );
    assert!(tree.is_popup_tag_active(swatch_tag));

    // 5. Click outside: dismiss popups
    let outside_point = Point::new(800.0, 800.0);
    assert!(
        !tree.is_point_inside_popup(outside_point),
        "Point (800, 800) must be outside popup"
    );
    assert!(
        tree.dismiss_popups_on_outside_click(outside_point),
        "Outside click must dismiss popup"
    );
    assert!(
        !tree.is_popup_tag_active(swatch_tag),
        "Active popup tag must be cleared on dismissal"
    );
    assert!(tree.popups_dismissed());
}

#[test]
fn test_inspector_real_panel_popups_open_on_right_side() {
    use crate::ui::panel_layout::PanelId;
    use irisui::workbench::{
        Workbench, WorkbenchDescriptor, WorkbenchExternalContext, WorkbenchFrameUpdateParams,
        WorkbenchPanelRegistry,
    };

    let mut world = ae_ecs::World::new();
    let entity = world.spawn((
        ae_plugin_api::Name("Cube".to_string()),
        ae_plugin_api::Position::default(),
        ae_plugin_api::Rotation::default(),
        ae_plugin_api::Scale::default(),
        ae_plugin_api::RigidBody::default(),
        ae_plugin_api::Collider::default(),
        ae_plugin_api::Velocity::default(),
    ));

    let dock = crate::ui::panel_layout::create_default_dock_state();

    let mut wb = Workbench::<PanelId, (), ()>::new(WorkbenchDescriptor::new(
        "RealInspectorTest",
        dock.clone(),
    ))
    .with_menubar_height(36.0)
    .with_statusbar_height(22.0)
    .with_screen_size((1920.0, 1080.0));

    let mut name = String::new();

    struct Ctx<'a> {
        world: &'a ae_ecs::World,
        entity: ae_ecs::Entity,
        name: &'a mut String,
    }

    let mut registry = WorkbenchPanelRegistry::<PanelId, Ctx<'_>>::new();
    for p in PanelId::all() {
        registry.register_domain_panel(*p, |s, _| {
            s.label("Stub");
        });
    }
    registry.register_domain_panel(PanelId::Inspector, |scope, ctx| {
        let _ = panel::build_inspector_panel(scope, ctx.world, Some(ctx.entity), ctx.name);
    });

    let mut ctx = Ctx {
        world: &world,
        entity,
        name: &mut name,
    };

    fn step<'a>(
        wb: &mut Workbench<PanelId, (), ()>,
        d: &irisui::dock::DockState<PanelId>,
        reg: &mut WorkbenchPanelRegistry<PanelId, Ctx<'a>>,
        c: &mut Ctx<'a>,
        popup: Option<bool>,
        label: &str,
    ) {
        wb.update_frame_live_with_registry(
            WorkbenchFrameUpdateParams {
                dock: d,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: reg,
                context: c,
            },
            |i| popup.map_or(i, |p| i.with_popup_active(p)),
            |_s, _c, _sz| {},
            |_s, _c| {},
        )
        .expect(label);
    }

    // Frame 0: initial build
    step(&mut wb, &dock, &mut registry, &mut ctx, None, "frame 0");

    // Frame 1: open Add Component menu
    wb.tree.open_popup(footer::TAG_INSPECTOR_ADD_COMPONENT);
    wb.notifier.tag_redraw(PanelId::Inspector);
    step(
        &mut wb,
        &dock,
        &mut registry,
        &mut ctx,
        Some(true),
        "frame 1",
    );

    let add_menu_rect = wb
        .tree
        .node_rect("AddComponentMenu")
        .expect("AddComponentMenu must exist");
    assert!(
        add_menu_rect.x > 1300.0 && add_menu_rect.y > 500.0,
        "AddComponentMenu must open on right Inspector footer, got {:?}",
        add_menu_rect
    );

    // Frame 2: close AddComponentMenu, open RigidBody Type ComboBox via GranularPanels
    wb.tree.close_all_popups();
    let rb_combo_tag = wb
        .tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
        .map(|(_, n)| n.tag)
        .expect("RigidBody ComboPill must exist");
    wb.tree.active_combobox_tag = Some(rb_combo_tag);
    wb.notifier.tag_redraw(PanelId::Inspector);
    step(
        &mut wb,
        &dock,
        &mut registry,
        &mut ctx,
        Some(false),
        "frame 2",
    );

    let combo_rect = wb
        .tree
        .node_rect("ComboBoxPopup")
        .expect("ComboBoxPopup must exist");
    assert!(
        combo_rect.x > 900.0,
        "RigidBody ComboBoxPopup must open on right Inspector panel, got {combo_rect:?}"
    );

    // Frame 3: clear active combobox with a clean rebuild, then click the second ComboPill (Collider Shape)
    wb.tree.active_combobox_tag = None;
    wb.notifier.tag_redraw(PanelId::Inspector);
    step(
        &mut wb,
        &dock,
        &mut registry,
        &mut ctx,
        Some(false),
        "frame 2.5 clear",
    );
    let collider_pill_rect = wb
        .tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("ComboPill"))
        .nth(1)
        .map(|(_, n)| n.computed_rect)
        .expect("Collider ComboPill must exist");
    assert!(
        collider_pill_rect.x > 1300.0,
        "Collider ComboPill itself must be on right panel, got {:?}",
        collider_pill_rect
    );

    struct NoopWindow;
    impl irisui::winit::WindowInterface for NoopWindow {
        fn set_cursor(&mut self, _icon: winit::window::CursorIcon) {}
        fn drag_resize_window(
            &mut self,
            _direction: irisui::winit::ResizeDirection,
        ) -> Result<(), irisui::winit::WindowOpError> {
            Ok(())
        }
        fn drag_window(&mut self) -> Result<(), irisui::winit::WindowOpError> {
            Ok(())
        }
        fn toggle_maximize(&mut self) {}
        fn is_maximized(&self) -> bool {
            false
        }
        fn minimize(&mut self) {}
        fn close(&mut self) {}
        fn logical_size(&self) -> Size {
            Size::new(1920.0, 1080.0)
        }
        fn scale_factor(&self) -> f64 {
            1.0
        }
    }
    let mut win = NoopWindow;
    let mut dock_mut = dock.clone();
    let click_pt = Point::new(collider_pill_rect.x + 8.0, collider_pill_rect.y + 8.0);
    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove { point: click_pt },
        &mut dock_mut,
    );
    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseDown {
            point: click_pt,
            button: MouseButton::Left,
        },
        &mut dock_mut,
    );
    let _ = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseUp {
            point: click_pt,
            button: MouseButton::Left,
        },
        &mut dock_mut,
    );
    wb.notifier.tag_redraw(PanelId::Inspector);
    step(
        &mut wb,
        &dock_mut,
        &mut registry,
        &mut ctx,
        Some(false),
        "frame 3",
    );

    let collider_popup_rect = wb
        .tree
        .node_rect("ComboBoxPopup")
        .expect("Collider ComboBoxPopup must exist after click");
    assert!(
        collider_popup_rect.x > 1300.0,
        "Collider ComboBoxPopup after real mouse click must open on right Inspector panel, got {:?}",
        collider_popup_rect
    );
}