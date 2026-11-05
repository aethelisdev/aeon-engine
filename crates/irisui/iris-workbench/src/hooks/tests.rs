// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use iris_core::{Color, Rect, UiTree};
use iris_widgets::canvas::ImmediateCanvas;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestAssetPayload {
    asset_id: u32,
    relative_path: String,
}

#[test]
fn test_viewport_surface_declarative_texture_view() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");
    let parent = tree.create_node();
    if let Some(node) = tree.get_mut(parent) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }
    let _ = tree.add_child(root, parent);

    let mut scope = iris_widgets::declarative::UiScope::new(&mut tree, parent);
    let view =
        ExternalTextureView::new(101, 1920.0, 1080.0).with_aspect_ratio(AspectRatio::Stretch);

    let mut content_invoked = false;
    scope.viewport_surface(Some(view), Some("Placeholder"), |_vp_scope| {
        content_invoked = true;
    });

    assert!(
        content_invoked,
        "Viewport surface content closure must execute"
    );

    let parent_node = tree.get(parent).expect("Parent exists");
    assert_eq!(parent_node.children.len(), 1);
    let container_id = parent_node.children[0];
    let container_node = tree.get(container_id).expect("Container exists");
    assert_eq!(container_node.children.len(), 1);
    let quad_id = container_node.children[0];
    let quad_node = tree.get(quad_id).expect("Quad exists");
    assert_eq!(
        quad_node.external_texture,
        Some(iris_core::ExternalTextureId(101))
    );
}

#[test]
fn test_drag_drop_payload_dynamic_and_typed() {
    let original_data = TestAssetPayload {
        asset_id: 42,
        relative_path: "models/player.gltf".to_string(),
    };

    // 1. Create erased payload
    let mut payload = DragDropPayload::new(original_data.clone(), "Player Model", "AssetBrowser");

    assert_eq!(payload.preview_label(), "Player Model");
    assert_eq!(payload.source_panel(), "AssetBrowser");
    assert!(payload.is::<TestAssetPayload>());
    assert!(!payload.is::<String>());
    assert!(!payload.is::<u32>());

    // 2. Downcast reference
    let borrowed = payload
        .downcast_ref::<TestAssetPayload>()
        .expect("Must borrow");
    assert_eq!(borrowed.asset_id, 42);
    assert_eq!(borrowed.relative_path, "models/player.gltf");

    // 3. Mutable downcast reference
    if let Some(mut_borrowed) = payload.downcast_mut::<TestAssetPayload>() {
        mut_borrowed.asset_id = 99;
    }
    assert_eq!(
        payload.downcast_ref::<TestAssetPayload>().unwrap().asset_id,
        99
    );

    // 4. Downcast value
    let unboxed = payload
        .downcast::<TestAssetPayload>()
        .expect("Downcast must succeed");
    assert_eq!(unboxed.asset_id, 99);

    // 5. TypedDragDropPayload roundtrip
    let typed = TypedDragDropPayload::new(
        TestAssetPayload {
            asset_id: 777,
            relative_path: "textures/albedo.png".to_string(),
        },
        "Albedo Texture",
        "Inspector",
    );
    let erased = typed.into_erased();
    assert!(erased.is::<TestAssetPayload>());

    let restored =
        TypedDragDropPayload::<TestAssetPayload>::try_from_erased(erased).expect("Must restore");
    assert_eq!(restored.data.asset_id, 777);
    assert_eq!(restored.preview_label, "Albedo Texture");
    assert_eq!(restored.source_panel, "Inspector");
}

#[test]
fn test_drag_drop_payload_downcast_failure_preserves_carrier() {
    let payload = DragDropPayload::new(12345_u32, "Count", "Metrics");
    let failed = payload.downcast::<String>();
    assert!(failed.is_err());
    let recovered = failed.unwrap_err();
    assert_eq!(recovered.preview_label(), "Count");
    assert_eq!(*recovered.downcast_ref::<u32>().unwrap(), 12345);
}

#[test]
fn test_custom_overlay_hook_execution() {
    let mut tree = UiTree::new();
    let node_id = tree.create_node();
    let bounds = Rect::new(10.0, 10.0, 200.0, 100.0);
    let mut canvas = ImmediateCanvas::new(&mut tree, node_id, bounds);

    // 1. Closure hook
    let mut invoked = false;
    let mut closure_hook = |c: &mut ImmediateCanvas<'_>, b: Rect| {
        c.fill_rect(b, Color::WHITE, 0.0);
        invoked = true;
    };

    closure_hook.render_overlay(&mut canvas, bounds);
    assert!(invoked);

    // 2. Boxed trait object hook
    struct TelemetryOverlay {
        sample_count: usize,
    }

    impl CustomOverlayHook for TelemetryOverlay {
        fn render_overlay(&mut self, canvas: &mut ImmediateCanvas<'_>, bounds: Rect) {
            self.sample_count += 1;
            canvas.draw_line(
                iris_core::Point::new(bounds.x, bounds.y),
                iris_core::Point::new(bounds.x + 50.0, bounds.y + 50.0),
                Color::BLACK,
                2.0,
            );
        }
    }

    let mut boxed_hook: Box<dyn CustomOverlayHook> = Box::new(TelemetryOverlay { sample_count: 0 });
    boxed_hook.render_overlay(&mut canvas, bounds);
    // Tree has recorded volatile canvas primitives
    assert!(tree.volatile_canvas_commands.contains_key(&node_id));
}

#[test]
fn test_external_texture_view_render() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(10.0, 20.0, 800.0, 600.0);
    }
    let mut scope = iris_widgets::declarative::UiScope::new(&mut tree, root);

    let view = ExternalTextureView::new(42, 800.0, 600.0);
    view.render(&mut scope);

    assert_eq!(tree.get(root).unwrap().children.len(), 1);
    let child_id = tree.get(root).unwrap().children[0];
    let child = tree.get(child_id).unwrap();
    assert_eq!(
        child.external_texture,
        Some(iris_core::ExternalTextureId(42))
    );
}

#[test]
fn test_workbench_ui_scope_ext_viewport_surface_and_overlay() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);
    }
    let mut scope = iris_widgets::declarative::UiScope::new(&mut tree, root);

    let view = ExternalTextureView::new(88, 1280.0, 720.0).with_aspect_ratio(AspectRatio::Stretch);
    let mut overlay_rendered = false;

    // Test declarative viewport surface with child volatile overlay
    scope.viewport_surface(Some(view), Some("Fallback"), |vp| {
        vp.volatile_overlay(
            "TestOverlay",
            1001,
            Rect::new(10.0, 10.0, 100.0, 100.0),
            |canvas| {
                overlay_rendered = true;
                canvas.draw_line(
                    iris_core::Point::new(0.0, 0.0),
                    iris_core::Point::new(10.0, 10.0),
                    Color::WHITE,
                    1.0,
                );
            },
        );
    });

    assert!(overlay_rendered, "Volatile overlay closure must execute");
    assert_eq!(tree.get(root).unwrap().children.len(), 1);
}

#[test]
fn test_drag_drop_controller_workflow_and_payload_cloning() {
    let mut controller = DragDropController::new();
    assert!(!controller.is_dragging());
    assert!(controller.active_payload().is_none());

    let payload = TypedDragDropPayload::new(
        TestAssetPayload {
            asset_id: 99,
            relative_path: "models/robot.gltf".to_string(),
        },
        "robot.gltf",
        "Assets",
    )
    .with_badge("3D", Color::rgba(0.0, 0.9, 1.0, 1.0));

    let erased = payload.clone().into_erased();
    assert_eq!(erased.badge_text(), Some("3D"));
    assert_eq!(erased.badge_color(), Some(Color::rgba(0.0, 0.9, 1.0, 1.0)));
    assert_eq!(erased.preview_label(), "robot.gltf");
    assert_eq!(erased.source_panel(), "Assets");

    // Test cloning of erased payload
    let erased_cloned = erased.clone();
    assert_eq!(erased, erased_cloned);

    // Start drag
    let start_point = iris_core::Point::new(100.0, 200.0);
    controller.start_drag(erased, start_point);
    assert!(controller.is_dragging());
    assert_eq!(controller.start_pos(), Some(start_point));
    assert_eq!(controller.current_pos(), Some(start_point));

    // Update cursor
    let move_point = iris_core::Point::new(150.0, 250.0);
    controller.update_cursor(move_point);
    assert_eq!(controller.current_pos(), Some(move_point));

    // Type query and borrowing
    assert!(controller.has_payload::<TestAssetPayload>());
    assert!(!controller.has_payload::<u64>());
    assert_eq!(
        controller.payload::<TestAssetPayload>().map(|p| p.asset_id),
        Some(99)
    );

    // Consume typed payload
    let extracted = controller.take_typed::<TestAssetPayload>().unwrap();
    assert_eq!(extracted.data.asset_id, 99);
    assert_eq!(extracted.preview_label, "robot.gltf");
    assert_eq!(extracted.badge_text.as_deref(), Some("3D"));

    // Drag is now completed
    assert!(!controller.is_dragging());
}

#[test]
fn test_drag_drop_preview_scope_extension() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = iris_widgets::declarative::UiScope::new(&mut tree, root);

    let mut controller = DragDropController::new();
    let payload = DragDropPayload::new(42u32, "GoldMaterial.mat", "MaterialPanel")
        .with_badge("MAT", Color::rgba(0.2, 0.8, 0.3, 1.0));
    controller.start_drag(payload, iris_core::Point::new(50.0, 50.0));

    // Render preview capsule declaratively
    let cursor = iris_core::Point::new(300.0, 400.0);
    let custom_style = DragDropPreviewStyle::new()
        .with_offset(iris_core::Point::new(20.0, 20.0))
        .with_colors(Color::BLACK, Color::WHITE);

    scope.drag_drop_preview(&controller, cursor, Some(&custom_style));

    // Verify container and children were emitted
    assert_eq!(tree.get(root).unwrap().children.len(), 1);
    let capsule_id = tree.get(root).unwrap().children[0];
    let capsule_node = tree.get(capsule_id).unwrap();
    assert_eq!(capsule_node.style.inset_left, Some(320.0));
    assert_eq!(capsule_node.style.inset_top, Some(420.0));
}