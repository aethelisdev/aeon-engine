// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// UI context to encapsulate all referenced fields from the engine shell.
/// This prevents circular dependencies between crates.
pub struct UiContext<'a> {
    pub mode: &'a mut ae_core::modules::EngineMode,
    pub world: &'a mut hecs::World,
    pub editor: &'a mut ae_editor::editor_state::EditorState,
    pub ui: &'a mut crate::ui::EngineUi,
    pub asset_manager: &'a mut ae_renderer::asset::AssetManager,
    pub camera: &'a mut ae_renderer::camera::Camera,
    pub time: &'a mut ae_core::time::Time,
    pub event_bus: &'a mut ae_core::events::DynamicEventBus,
    pub render_state: &'a mut ae_renderer::render::RenderState,
    pub dialog_receivers: &'a mut Vec<std::sync::mpsc::Receiver<std::path::PathBuf>>,
}

/// Dispatches UI action commands from the egui layer into engine state mutations.
/// Each `EngineUiAction` variant maps to a specific engine operation: spawning entities,
/// modifying components, changing modes, updating settings, or triggering undo/redo.
/// All ECS writes, undo history pushes, and selection state updates happen here,
/// keeping the UI layer purely declarative and side-effect-free.
/// **Hierarchical Transform Safety:**
/// When parenting relationships are broken (either via `UnparentEntity` or when a child is
/// reparented via `ParentEntity`), the engine carefully cleans up empty `Children` containers
/// and leftover `GlobalTransform` components on the old parent (if it has no other children and
/// no parent itself). This prevents entities from becoming frozen/motionless in the viewport
/// since the scene graph early-exits on isolated elements and doesn't update their cached
/// `GlobalTransform` matrices.
/// **Borrow Checker Resolution:**
/// We avoid mutable/immutable borrow conflicts on `ctx.world` by copying out target entity
/// handles (e.g., `old_parent_opt`) into temporary stack variables prior to performing any mutating ECS
/// operations (like `remove_one` or `insert_one`), cleanly decoupling lifetimes.
pub fn process_ui_actions(ctx: &mut UiContext, actions: std::vec::Vec<crate::ui::EngineUiAction>) {
    for action in actions {
        match action {
            crate::ui::EngineUiAction::ChangeMode(m) => {
                *ctx.mode = m;
            }
            crate::ui::EngineUiAction::SpawnModel(mid) => {
                let mut base_name = "Model".to_string();
                let mut bbox = ae_core::ecs::BoundingBox {
                    min: [-0.5; 3],
                    max: [0.5; 3],
                };

                if let Some(m) = ctx.asset_manager.models.get(mid) {
                    let path = std::path::Path::new(&m.source_path);
                    base_name = path
                        .file_stem()
                        .unwrap_or(std::ffi::OsStr::new("Model"))
                        .to_string_lossy()
                        .into_owned();
                    bbox = ae_core::ecs::BoundingBox {
                        min: m.min,
                        max: m.max,
                    };
                }

                let new_entity = ctx.world.spawn((
                    ae_core::ecs::Name(base_name),
                    ae_core::ecs::ModelId(mid),
                    ae_core::ecs::Position {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    ae_core::ecs::Rotation::identity(),
                    ae_core::ecs::Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    bbox,
                    ae_core::ecs::Velocity {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
                let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_entity);
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Spawn(new_entity, snap),
                );
                ctx.editor.redo_stack.clear();
                ctx.editor.selected_entities.clear();
                ctx.editor.selected_entities_set.clear();
                ctx.editor.selected_entities.push(new_entity);
                ctx.editor.selected_entities_set.insert(new_entity);
                ctx.ui.selected_entity = Some(new_entity);
            }
            crate::ui::EngineUiAction::SpawnSprite(tid) => {
                let mut base_name = "Sprite".to_string();
                if let Some(t) = ctx.asset_manager.textures.get(tid) {
                    let path = std::path::Path::new(&t.source_path);
                    base_name = path
                        .file_stem()
                        .unwrap_or(std::ffi::OsStr::new("Sprite"))
                        .to_string_lossy()
                        .into_owned();
                }

                let new_entity = ctx.world.spawn((
                    ae_core::ecs::Name(base_name),
                    ae_core::ecs::SpriteId(tid),
                    ae_core::ecs::Position {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    ae_core::ecs::Rotation::identity(),
                    ae_core::ecs::Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    ae_core::ecs::Velocity {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
                let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_entity);
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Spawn(new_entity, snap),
                );
                ctx.editor.redo_stack.clear();
                ctx.editor.selected_entities.clear();
                ctx.editor.selected_entities_set.clear();
                ctx.editor.selected_entities.push(new_entity);
                ctx.editor.selected_entities_set.insert(new_entity);
                ctx.ui.selected_entity = Some(new_entity);
            }
            crate::ui::EngineUiAction::SpawnShape(shape) => {
                let final_name = match shape {
                    ae_core::ecs::Shape::Cube => "New Cube",
                    ae_core::ecs::Shape::Triangle => "New Triangle",
                    ae_core::ecs::Shape::Sphere => "New Sphere",
                    ae_core::ecs::Shape::Cylinder => "New Cylinder",
                    ae_core::ecs::Shape::Capsule => "New Capsule",
                    ae_core::ecs::Shape::Torus => "New Torus",
                }
                .to_string();
                let new_entity = ctx.world.spawn((
                    ae_core::ecs::Name(final_name.clone()),
                    shape,
                    ae_core::ecs::Position {
                        x: 0.0,
                        y: 3.0,
                        z: 0.0,
                    },
                    ae_core::ecs::Rotation::identity(),
                    ae_core::ecs::Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    ae_core::ecs::Color::soft_blue(),
                    ae_core::ecs::Velocity {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    ae_core::ecs::RigidBody {
                        body_type: ae_core::ecs::RigidBodyType::Dynamic,
                        mass: 1.0,
                        gravity_scale: 1.0,
                    },
                    ae_core::ecs::Collider {
                        shape: ae_core::ecs::ColliderShape::Box {
                            half_extents: [0.5, 0.5, 0.5],
                        },
                        friction: 0.7,
                        restitution: 0.0,
                        is_sensor: false,
                    },
                    ae_core::ecs::TransformDirty,
                ));
                let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_entity);
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Spawn(new_entity, snap),
                );
                ctx.editor.redo_stack.clear();
                ctx.editor.selected_entities.clear();
                ctx.editor.selected_entities_set.clear();
                ctx.editor.selected_entities.push(new_entity);
                ctx.editor.selected_entities_set.insert(new_entity);
                ctx.ui.selected_entity = Some(new_entity);
            }
            crate::ui::EngineUiAction::DeleteSelected => {
                ae_editor::actions::delete_selected(
                    ctx.world,
                    ctx.editor,
                    &mut ctx.ui.selected_entity,
                );
            }
            crate::ui::EngineUiAction::SelectEntity(entity_opt) => {
                ctx.editor.selected_entities.clear();
                ctx.editor.selected_entities_set.clear();
                if let Some(e) = entity_opt {
                    ctx.editor.selected_entities.push(e);
                    ctx.editor.selected_entities_set.insert(e);
                }
                ctx.ui.selected_entity = entity_opt;
            }
            crate::ui::EngineUiAction::ParentEntity(child, parent) => {
                let old_parent_opt =
                    if let Ok(old_parent_ref) = ctx.world.get::<&ae_core::ecs::Parent>(child) {
                        Some(old_parent_ref.0)
                    } else {
                        None
                    };

                if let Some(old_parent) = old_parent_opt {
                    let mut remove_parent_children = false;
                    let mut remove_parent_gt = false;

                    if let Ok(mut old_children) =
                        ctx.world.get::<&mut ae_core::ecs::Children>(old_parent)
                    {
                        old_children.0.retain(|&e| e != child);
                        if old_children.0.is_empty() {
                            remove_parent_children = true;
                            if ctx.world.get::<&ae_core::ecs::Parent>(old_parent).is_err() {
                                remove_parent_gt = true;
                            }
                        }
                    }

                    if remove_parent_children {
                        let _ = ctx.world.remove_one::<ae_core::ecs::Children>(old_parent);
                    }
                    if remove_parent_gt {
                        let _ = ctx
                            .world
                            .remove_one::<ae_core::ecs::GlobalTransform>(old_parent);
                    }
                }

                let _ = ctx.world.insert_one(child, ae_core::ecs::Parent(parent));
                if let Ok(mut children) = ctx.world.get::<&mut ae_core::ecs::Children>(parent) {
                    if !children.0.contains(&child) {
                        children.0.push(child);
                    }
                } else {
                    let _ = ctx
                        .world
                        .insert_one(parent, ae_core::ecs::Children(vec![child]));
                }
                let _ = ctx.world.insert_one(child, ae_core::ecs::TransformDirty);
                let _ = ctx.world.insert_one(parent, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::UnparentEntity(child) => {
                let old_parent_opt =
                    if let Ok(old_parent_ref) = ctx.world.get::<&ae_core::ecs::Parent>(child) {
                        Some(old_parent_ref.0)
                    } else {
                        None
                    };

                if let Some(old_parent) = old_parent_opt {
                    let mut remove_parent_children = false;
                    let mut remove_parent_gt = false;

                    if let Ok(mut old_children) =
                        ctx.world.get::<&mut ae_core::ecs::Children>(old_parent)
                    {
                        old_children.0.retain(|&e| e != child);
                        if old_children.0.is_empty() {
                            remove_parent_children = true;
                            if ctx.world.get::<&ae_core::ecs::Parent>(old_parent).is_err() {
                                remove_parent_gt = true;
                            }
                        }
                    }

                    if remove_parent_children {
                        let _ = ctx.world.remove_one::<ae_core::ecs::Children>(old_parent);
                    }
                    if remove_parent_gt {
                        let _ = ctx
                            .world
                            .remove_one::<ae_core::ecs::GlobalTransform>(old_parent);
                    }
                }
                let _ = ctx.world.remove_one::<ae_core::ecs::Parent>(child);
                let _ = ctx.world.remove_one::<ae_core::ecs::GlobalTransform>(child);
                let _ = ctx.world.insert_one(child, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::StressTest(count) => {
                let mut batch = Vec::new();
                let mut seed = 12345u32;
                let mut next_rand = |min: f32, max: f32| {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    let fraction = (seed & 0xFFFFFF) as f32 / 16777216.0;
                    min + fraction * (max - min)
                };

                let bounds = if count >= 1000000 {
                    10000.0
                } else if count >= 100000 {
                    80.0
                } else {
                    40.0
                };
                let height = if count >= 1000000 {
                    200.0
                } else if count >= 100000 {
                    81.0
                } else {
                    41.0
                };

                if count >= 1000000 {
                    // For 1M+ entities, bypass capturing undo history and heap string allocations
                    // to allow instant spawning in under 50 milliseconds!
                    for _ in 0..count {
                        let rx = next_rand(-bounds, bounds);
                        let ry = next_rand(1.0, height);
                        let rz = next_rand(-bounds, bounds);
                        ctx.world.spawn((
                            ae_core::ecs::Shape::Cube,
                            ae_core::ecs::Position {
                                x: rx,
                                y: ry,
                                z: rz,
                            },
                            ae_core::ecs::Rotation::identity(),
                            ae_core::ecs::Scale {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            },
                        ));
                    }
                } else {
                    for _ in 0..count {
                        let rx = next_rand(-bounds, bounds);
                        let ry = next_rand(1.0, height);
                        let rz = next_rand(-bounds, bounds);
                        let new_ent = ctx.world.spawn((
                            ae_core::ecs::Name("Test Cube".to_string()),
                            ae_core::ecs::Shape::Cube,
                            ae_core::ecs::Position {
                                x: rx,
                                y: ry,
                                z: rz,
                            },
                            ae_core::ecs::Rotation::identity(),
                            ae_core::ecs::Scale {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            },
                            ae_core::ecs::Velocity {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        ));
                        let snap =
                            ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_ent);
                        batch.push(ae_editor::undo_redo::Command::Spawn(new_ent, snap));
                    }
                    ae_editor::history::push_undo(
                        ctx.editor,
                        ae_editor::undo_redo::Command::Batch(batch),
                    );
                    ctx.editor.redo_stack.clear();
                }
            }
            crate::ui::EngineUiAction::AaaOpenWorldTest => {
                let mut seed = 54321u32;
                let mut next_rand = |min: f32, max: f32| {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    let fraction = (seed & 0xFFFFFF) as f32 / 16777216.0;
                    min + fraction * (max - min)
                };

                // 1. 10km x 10km Static Ground Plane
                ctx.world.spawn((
                    ae_core::ecs::Name("Static_Ground_10km".to_string()),
                    ae_core::ecs::Shape::Cube,
                    ae_core::ecs::Position {
                        x: 0.0,
                        y: -0.5,
                        z: 0.0,
                    },
                    ae_core::ecs::Rotation::identity(),
                    ae_core::ecs::Scale {
                        x: 10000.0,
                        y: 1.0,
                        z: 10000.0,
                    },
                    ae_core::ecs::Color {
                        r: 0.25,
                        g: 0.35,
                        b: 0.2,
                        a: 1.0,
                    },
                    ae_core::ecs::RigidBody {
                        body_type: ae_core::ecs::RigidBodyType::Static,
                        mass: 0.0,
                        gravity_scale: 0.0,
                    },
                    ae_core::ecs::Collider {
                        shape: ae_core::ecs::ColliderShape::Box {
                            half_extents: [0.5, 0.5, 0.5],
                        },
                        friction: 0.7,
                        restitution: 0.0,
                        is_sensor: false,
                    },
                ));

                // 2. Main Towns & POI Hubs (8 Cities/Fortresses/Villages across 10km map)
                let hubs = [
                    (0.0f32, 0.0f32, 3500),  // Capital City (Center)
                    (0.0, 3500.0, 2000),     // North Fortress
                    (0.0, -3500.0, 2000),    // South Harbor
                    (3500.0, 0.0, 2000),     // East Castle
                    (-3500.0, 0.0, 2000),    // West Ruins
                    (-2500.0, 2500.0, 1500), // NW Outpost
                    (2500.0, -2500.0, 1500), // SE Village
                    (2500.0, 2500.0, 1500),  // NE Sanctuary
                ];

                for &(hx, hz, count) in &hubs {
                    for i in 0..count {
                        let rx = hx + next_rand(-300.0, 300.0);
                        let rz = hz + next_rand(-300.0, 300.0);
                        let sx = next_rand(2.0, 8.0);
                        let sy = next_rand(3.0, 25.0);
                        let sz = next_rand(2.0, 8.0);
                        let ry = sy * 0.5;

                        let shape = match i % 3 {
                            0 => ae_core::ecs::Shape::Cube,
                            1 => ae_core::ecs::Shape::Cylinder,
                            _ => ae_core::ecs::Shape::Torus,
                        };

                        ctx.world.spawn((
                            shape,
                            ae_core::ecs::Position {
                                x: rx,
                                y: ry,
                                z: rz,
                            },
                            ae_core::ecs::Rotation::identity(),
                            ae_core::ecs::Scale {
                                x: sx,
                                y: sy,
                                z: sz,
                            },
                            ae_core::ecs::Color {
                                r: 0.6,
                                g: 0.55,
                                b: 0.5,
                                a: 1.0,
                            },
                        ));
                    }
                }

                // 3. Wilderness & Forest Vegetation Scattering (35,000 trees, rocks, props across 10km)
                for _ in 0..34000 {
                    let rx = next_rand(-4900.0, 4900.0);
                    let rz = next_rand(-4900.0, 4900.0);
                    let sy = next_rand(1.5, 9.0);
                    let sx = next_rand(0.8, 2.5);
                    let sz = sx;
                    let ry = sy * 0.5;

                    let (shape, color) = if next_rand(0.0, 1.0) > 0.4 {
                        (
                            ae_core::ecs::Shape::Cylinder,
                            ae_core::ecs::Color {
                                r: 0.15,
                                g: 0.5,
                                b: 0.2,
                                a: 1.0,
                            },
                        )
                    } else {
                        (
                            ae_core::ecs::Shape::Sphere,
                            ae_core::ecs::Color {
                                r: 0.4,
                                g: 0.4,
                                b: 0.4,
                                a: 1.0,
                            },
                        )
                    };

                    ctx.world.spawn((
                        shape,
                        ae_core::ecs::Position {
                            x: rx,
                            y: ry,
                            z: rz,
                        },
                        ae_core::ecs::Rotation::identity(),
                        ae_core::ecs::Scale {
                            x: sx,
                            y: sy,
                            z: sz,
                        },
                        color,
                    ));
                }

                // 4. Interactive Dynamic Physics Objects in Central City Square
                for i in 0..50 {
                    let rx = next_rand(-15.0, 15.0);
                    let rz = next_rand(-15.0, 15.0);
                    let ry = 2.0 + (i as f32) * 1.2;

                    ctx.world.spawn((
                        ae_core::ecs::Name(format!("Dynamic_Barrel_{}", i)),
                        ae_core::ecs::Shape::Cube,
                        ae_core::ecs::Position {
                            x: rx,
                            y: ry,
                            z: rz,
                        },
                        ae_core::ecs::Rotation::identity(),
                        ae_core::ecs::Scale {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                        },
                        ae_core::ecs::Velocity {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        ae_core::ecs::Color {
                            r: 0.8,
                            g: 0.4,
                            b: 0.1,
                            a: 1.0,
                        },
                        ae_core::ecs::RigidBody {
                            body_type: ae_core::ecs::RigidBodyType::Dynamic,
                            mass: 2.0,
                            gravity_scale: 1.0,
                        },
                        ae_core::ecs::Collider {
                            shape: ae_core::ecs::ColliderShape::Box {
                                half_extents: [0.5, 0.5, 0.5],
                            },
                            friction: 0.6,
                            restitution: 0.4,
                            is_sensor: false,
                        },
                    ));
                }

                // 5. KCC Player Character at City Center (0, 2, 0)
                let player_ent = ctx.world.spawn((
                    ae_core::ecs::Name("Player_Character".to_string()),
                    ae_core::ecs::PlayerTag,
                    ae_core::ecs::Shape::Capsule,
                    ae_core::ecs::Position {
                        x: 0.0,
                        y: 2.0,
                        z: 0.0,
                    },
                    ae_core::ecs::Rotation::identity(),
                    ae_core::ecs::Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    ae_core::ecs::Velocity {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    ae_core::ecs::Color {
                        r: 0.1,
                        g: 0.9,
                        b: 0.5,
                        a: 1.0,
                    },
                    ae_core::ecs::CharacterController {
                        height: 1.8,
                        radius: 0.4,
                        max_slope_climb_angle: 45.0,
                        step_height: 0.3,
                        is_grounded: true,
                    },
                    ae_core::ecs::RigidBody {
                        body_type: ae_core::ecs::RigidBodyType::Kinematic,
                        mass: 1.0,
                        gravity_scale: 1.0,
                    },
                    ae_core::ecs::Collider {
                        shape: ae_core::ecs::ColliderShape::Capsule {
                            half_height: 0.5,
                            radius: 0.4,
                        },
                        friction: 0.5,
                        restitution: 0.0,
                        is_sensor: false,
                    },
                ));

                ctx.editor.selected_entities.clear();
                ctx.editor.selected_entities.push(player_ent);
                ctx.ui.status_message = Some((
                    vec![(
                        "10km Open World map generated with 50,000 objects!".to_string(),
                        egui::Color32::GREEN,
                    )],
                    std::time::Instant::now(),
                ));
            }
            crate::ui::EngineUiAction::Explode => {
                for (pos, vel) in ctx
                    .world
                    .query_mut::<(&ae_core::ecs::Position, &mut ae_core::ecs::Velocity)>()
                {
                    vel.y = pos.y * 1.5 + 20.0;
                    vel.x = pos.x * 2.5;
                    vel.z = pos.z * 2.5;
                }
            }
            crate::ui::EngineUiAction::ModifyName(entity, old, new_name) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Name>(entity) {
                    *c = ae_core::ecs::Name(new_name.clone());
                }
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Modify(
                        entity,
                        ae_editor::undo_redo::Property::Name(old, new_name),
                    ),
                );
                ctx.editor.redo_stack.clear();
            }
            crate::ui::EngineUiAction::LiveUpdatePosition(entity, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Position>(entity) {
                    *c = new_val;
                }
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::ModifyPosition(entity, old, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Position>(entity) {
                    *c = new_val;
                }
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Modify(
                        entity,
                        ae_editor::undo_redo::Property::Position(old, new_val),
                    ),
                );
                ctx.editor.redo_stack.clear();
            }
            crate::ui::EngineUiAction::LiveUpdateRotation(entity, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Rotation>(entity) {
                    *c = new_val;
                }
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::ModifyRotation(entity, old, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Rotation>(entity) {
                    *c = new_val;
                }
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Modify(
                        entity,
                        ae_editor::undo_redo::Property::Rotation(old, new_val),
                    ),
                );
                ctx.editor.redo_stack.clear();
            }
            crate::ui::EngineUiAction::LiveUpdateScale(entity, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Scale>(entity) {
                    *c = new_val;
                }
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::ModifyScale(entity, old, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Scale>(entity) {
                    *c = new_val;
                }
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Modify(
                        entity,
                        ae_editor::undo_redo::Property::Scale(old, new_val),
                    ),
                );
                ctx.editor.redo_stack.clear();
            }
            crate::ui::EngineUiAction::ModifyColor(entity, old, new_val) => {
                let mut updated = false;
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Color>(entity) {
                    *c = new_val;
                    updated = true;
                }
                if !updated {
                    let _ = ctx.world.insert_one(entity, new_val);
                }
                ae_editor::history::push_undo(
                    ctx.editor,
                    ae_editor::undo_redo::Command::Modify(
                        entity,
                        ae_editor::undo_redo::Property::Color(old, new_val),
                    ),
                );
                ctx.editor.redo_stack.clear();
            }
            crate::ui::EngineUiAction::ModifyLightColor(entity, _old, new_val) => {
                if let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Light>(entity) {
                    c.color = new_val;
                }
            }
            // --- Dynamic Component Handlers (Physics) ---
            crate::ui::EngineUiAction::AddRigidBody(entity, rb) => {
                let _ = ctx.world.insert_one(entity, rb);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::RemoveRigidBody(entity) => {
                let _ = ctx.world.remove_one::<ae_core::ecs::RigidBody>(entity);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::ModifyRigidBody(entity, rb) => {
                let _ = ctx.world.insert_one(entity, rb);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::AddCollider(entity, col) => {
                let _ = ctx.world.insert_one(entity, col);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::RemoveCollider(entity) => {
                let _ = ctx.world.remove_one::<ae_core::ecs::Collider>(entity);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::ModifyCollider(entity, col) => {
                let _ = ctx.world.insert_one(entity, col);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::AddCharacterController(entity, ctrl) => {
                let mut old_players = Vec::new();
                for (e, _tag) in ctx
                    .world
                    .query::<(hecs::Entity, &ae_core::ecs::PlayerTag)>()
                    .iter()
                {
                    if e != entity {
                        old_players.push(e);
                    }
                }
                for e in old_players {
                    let _ = ctx.world.remove_one::<ae_core::ecs::PlayerTag>(e);
                }
                let _ = ctx.world.insert_one(entity, ctrl);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::PlayerTag);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::RemoveCharacterController(entity) => {
                let _ = ctx
                    .world
                    .remove_one::<ae_core::ecs::CharacterController>(entity);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::ModifyCharacterController(entity, ctrl) => {
                let _ = ctx.world.insert_one(entity, ctrl);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            crate::ui::EngineUiAction::SetCameraMode(cm) => {
                ctx.camera.mode = cm;
            }
            crate::ui::EngineUiAction::OpenModelDialog => {
                let (tx, rx) = std::sync::mpsc::channel();
                ctx.dialog_receivers.push(rx);

                rayon::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("3D Models", &["glb", "gltf", "fbx"])
                        .pick_file()
                    {
                        let _ = tx.send(path);
                    }
                });
            }
            crate::ui::EngineUiAction::SetCameraTransform {
                pitch,
                yaw,
                position,
            } => {
                ctx.camera.pitch = pitch;
                ctx.camera.yaw = yaw;
                ctx.camera.position = position;
            }
            crate::ui::EngineUiAction::UpdateGraphicsSettings(gs) => {
                ctx.render_state.graphics_settings = gs;
            }
            crate::ui::EngineUiAction::UpdateSnapSettings(ss) => {
                ctx.editor.snapping = ss;
            }
            crate::ui::EngineUiAction::UpdateEditorConfig(cfg) => {
                ctx.time.fixed_time_step = 1.0 / cfg.physics_hz;
                ctx.editor.config = cfg;
            }
            crate::ui::EngineUiAction::SetLiveEditorUpdates(opt) => {
                ctx.editor.enable_live_editor_updates = opt;
            }
            crate::ui::EngineUiAction::Undo => {
                ae_editor::history::undo(ctx.editor, ctx.world);
            }
            crate::ui::EngineUiAction::Redo => {
                ae_editor::history::redo(ctx.editor, ctx.world);
            }
            crate::ui::EngineUiAction::GarbageCollect => {
                ctx.asset_manager.unload_unused_assets(ctx.world);
            }
            crate::ui::EngineUiAction::OpenSaveSceneDialog => {
                let (tx, rx) = std::sync::mpsc::channel();
                ctx.ui.scene_dialog_receivers.push(rx);

                rayon::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Aeon Scene", &["aee"])
                        .set_file_name("scene.aee")
                        .save_file()
                    {
                        let _ = tx.send(crate::ui::SceneDialogAction::SaveTo(path));
                    }
                });
            }
            crate::ui::EngineUiAction::OpenLoadSceneDialog => {
                let (tx, rx) = std::sync::mpsc::channel();
                ctx.ui.scene_dialog_receivers.push(rx);

                rayon::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Aeon Scene", &["aee"])
                        .pick_file()
                    {
                        let _ = tx.send(crate::ui::SceneDialogAction::LoadFrom(path));
                    }
                });
            }
            crate::ui::EngineUiAction::SaveSceneToPath(path) => {
                ctx.editor.active_scene_path = Some(path.clone());
                ctx.ui.active_scene_path = path.to_string_lossy().to_string();
                ctx.ui.pending_save_path = Some(path);
                ctx.ui.should_save_scene = true;
            }
            crate::ui::EngineUiAction::LoadSceneFromPath(path) => {
                ctx.editor.active_scene_path = Some(path.clone());
                ctx.ui.active_scene_path = path.to_string_lossy().to_string();
                ctx.ui.pending_load_path = Some(path);
                ctx.ui.should_load_scene = true;
                ctx.ui.is_loading_assets = true;
            }
            crate::ui::EngineUiAction::SaveEntityAsPrefab(entity, path) => {
                let prefab = ae_editor::prefab::Prefab::create_from_entity(ctx.world, entity);
                if let Err(e) = prefab.save_to_file(&path) {
                    log::error!("Failed to save prefab to {:?}: {}", path, e);
                } else {
                    log::info!(
                        "💾 Successfully saved prefab '{}' to {:?}",
                        prefab.name,
                        path
                    );
                }
            }
            crate::ui::EngineUiAction::InstantiatePrefab(path) => {
                match ae_editor::prefab::Prefab::load_from_file(&path) {
                    Ok(prefab) => {
                        let cam_fwd = ctx.camera.get_forward();
                        let spawn_pos = ae_core::ecs::Position {
                            x: ctx.camera.position.x + cam_fwd.x * 5.0,
                            y: ctx.camera.position.y + cam_fwd.y * 5.0,
                            z: ctx.camera.position.z + cam_fwd.z * 5.0,
                        };
                        let new_ent = prefab.instantiate(ctx.world, Some(spawn_pos));
                        let snap =
                            ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_ent);
                        ae_editor::history::push_undo(
                            ctx.editor,
                            ae_editor::undo_redo::Command::Spawn(new_ent, snap),
                        );
                        ctx.editor.selected_entities = vec![new_ent];
                        ctx.editor.selected_entities_set.clear();
                        ctx.editor.selected_entities_set.insert(new_ent);
                        log::info!(
                            "📦 Successfully instantiated prefab '{}' at {:?}",
                            prefab.name,
                            spawn_pos
                        );
                    }
                    Err(e) => {
                        log::error!("Failed to load prefab from {:?}: {}", path, e);
                    }
                }
            }
            crate::ui::EngineUiAction::LoadScene => {
                ctx.ui.should_load_scene = true;
                ctx.ui.is_loading_assets = true;
            }
            crate::ui::EngineUiAction::SaveScene => {
                ctx.ui.should_save_scene = true;
            }
            crate::ui::EngineUiAction::Exit => {
                ctx.ui.should_exit = true;
            }
            crate::ui::EngineUiAction::AddLodGroup(entity) => {
                let mut lod_0_opt = None;
                if let Ok(m_id) = ctx.world.get::<&ae_core::ecs::ModelId>(entity) {
                    lod_0_opt = Some(m_id.0);
                }

                if lod_0_opt.is_none() {
                    if let Some((h, _)) = ctx.asset_manager.models.iter().next() {
                        lod_0_opt = Some(h);
                    }
                }

                if let Some(l0) = lod_0_opt {
                    let mut other_models = Vec::new();
                    for (h, _) in ctx.asset_manager.models.iter() {
                        if h != l0 {
                            other_models.push(h);
                        }
                    }
                    let lod_1 = if other_models.len() > 0 {
                        Some(other_models[0])
                    } else {
                        None
                    };
                    let lod_2 = if other_models.len() > 1 {
                        Some(other_models[1])
                    } else {
                        None
                    };

                    let new_lod = ae_core::ecs::LodGroup {
                        lod_0: l0,
                        lod_1,
                        lod_2,
                        threshold_1: 40.0,
                        threshold_2: 120.0,
                    };
                    let _ = ctx.world.insert_one(entity, new_lod);
                } else {
                    ctx.ui.status_message = Some((
                        vec![(
                            "Please load at least one model in the scene first!".to_string(),
                            egui::Color32::RED,
                        )],
                        std::time::Instant::now(),
                    ));
                }
            }
            crate::ui::EngineUiAction::RemoveLodGroup(entity) => {
                let _ = ctx.world.remove_one::<ae_core::ecs::LodGroup>(entity);
            }
            crate::ui::EngineUiAction::ModifyLodThresholds(entity, t1, t2) => {
                if let Ok(mut lod) = ctx.world.get::<&mut ae_core::ecs::LodGroup>(entity) {
                    lod.threshold_1 = t1;
                    lod.threshold_2 = t2;
                }
            }
            crate::ui::EngineUiAction::ModifyLodModel(entity, index, handle_opt) => {
                if let Ok(mut lod) = ctx.world.get::<&mut ae_core::ecs::LodGroup>(entity) {
                    match index {
                        0 => {
                            if let Some(h) = handle_opt {
                                lod.lod_0 = h;
                            }
                        }
                        1 => {
                            lod.lod_1 = handle_opt;
                        }
                        2 => {
                            lod.lod_2 = handle_opt;
                        }
                        _ => {}
                    }
                }
            }
            crate::ui::EngineUiAction::ToggleModule(module) => {
                let enabled = ctx.event_bus.is_module_enabled(module);
                ctx.event_bus.set_module_enabled(module, !enabled);
                log::info!("Module {:?} toggled to {}", module, !enabled);
            }
            crate::ui::EngineUiAction::AddAudioSource(entity) => {
                let source = ae_audio::AudioSource::default();
                let _ = ctx.world.insert_one(entity, source);
                log::info!("🔊 Added AudioSource to entity {:?}", entity);
            }
            crate::ui::EngineUiAction::RemoveAudioSource(entity) => {
                let _ = ctx.world.remove_one::<ae_audio::AudioSource>(entity);
                log::info!("🔊 Removed AudioSource from entity {:?}", entity);
            }
            crate::ui::EngineUiAction::AddAudioListener(entity) => {
                let listener = ae_audio::AudioListener;
                let _ = ctx.world.insert_one(entity, listener);
                log::info!("👂 Added AudioListener to entity {:?}", entity);
            }
            crate::ui::EngineUiAction::RemoveAudioListener(entity) => {
                let _ = ctx.world.remove_one::<ae_audio::AudioListener>(entity);
                log::info!("👂 Removed AudioListener from entity {:?}", entity);
            }
            crate::ui::EngineUiAction::ModifyAudioSource(entity, source) => {
                if let Ok(mut existing) = ctx.world.get::<&mut ae_audio::AudioSource>(entity) {
                    *existing = source;
                }
            }
            _ => {}
        }
    }
}