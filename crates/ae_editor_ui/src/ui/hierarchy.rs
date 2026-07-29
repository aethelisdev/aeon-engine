// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::EngineUi;

/// A pre-built flat snapshot of one entity's hierarchy display data.
/// Built once when the world changes (dirty flag), not every frame.
/// Stores all data needed to render one row, so zero `world.get` calls during draw.
#[derive(Clone)]
pub struct HierarchyRow {
    /// The ECS entity this row represents.
    pub entity: hecs::Entity,
    /// Display name (owned, so no world access needed during draw).
    pub name: String,
    /// Indentation depth in the parent-child tree (0 = root).
    pub depth: usize,
    /// Whether this entity has at least one valid child.
    pub has_children: bool,
    /// Icon prefix derived from component presence (Light vs. generic mesh).
    pub icon: &'static str,
}

/// Cached, pre-flattened scene hierarchy list.
/// Rebuilt only when the world changes (`entity_count` differs from last frame).
/// Drawing is O(visible_rows) instead of O(total_entities) thanks to virtual scrolling.
pub struct HierarchyCache {
    /// Flat ordered list of all entities (DFS pre-order: parent before children).
    pub rows: Vec<HierarchyRow>,
    /// The world entity count at the time of the last rebuild.
    last_entity_count: usize,
    /// Accumulates entities during recursive DFS without re-allocating per call.
    scratch: Vec<(hecs::Entity, usize)>,
}

impl HierarchyCache {
    /// Creates an empty cache. The first call to `sync` will populate it.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            last_entity_count: usize::MAX, // Force rebuild on first frame
            scratch: Vec::new(),
        }
    }

    /// Rebuilds the flat row list from the ECS world using a single-pass DFS traversal.
    /// This is called at most once per frame, and only when entity count changed.
    /// A single `world.query` call collects all component data upfront; no per-entity
    /// random `world.get` lookups happen during the traversal.
    /// # Algorithm
    /// 1. One `world.query` → build an `EntityMap` of all entity data in one O(N) pass.
    /// 2. Collect root entities (entities with no valid parent) into a Vec.
    /// 3. DFS from each root, pushing `HierarchyRow` entries in display order.
    pub fn sync(&mut self, world: &hecs::World) {
        let entity_count = world.len() as usize;
        if entity_count == self.last_entity_count {
            return; // Cache is fresh — zero work this frame
        }
        self.last_entity_count = entity_count;

        // --- Single O(N) query: collect component data for display ---
        // Using a temporary HashMap keyed by Entity to allow fast parent lookups.
        use std::collections::HashMap;

        // Pre-size maps up to a safety cap (25,000 max) to prevent multi-gigabyte RAM allocation
        // spikes and 14+ second CPU freezes in 10M entity stress tests.
        let max_display_entities = 25_000;
        let cap = entity_count.min(max_display_entities);
        let mut name_map: HashMap<hecs::Entity, String> = HashMap::with_capacity(cap);
        let mut parent_map: HashMap<hecs::Entity, hecs::Entity> = HashMap::with_capacity(cap);
        let mut children_map: HashMap<hecs::Entity, Vec<hecs::Entity>> =
            HashMap::with_capacity(cap);
        let mut is_light: HashMap<hecs::Entity, bool> = HashMap::with_capacity(cap);

        // One pass over entities — capped at max_display_entities for large scenes
        let mut processed_count = 0;
        for ent_ref in world.iter() {
            if processed_count >= max_display_entities {
                break;
            }
            processed_count += 1;
            let ent = ent_ref.entity();

            // Name component
            let name = ent_ref
                .get::<&ae_core::ecs::Name>()
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("Entity {:?}", ent));
            name_map.insert(ent, name);

            // Parent component
            if let Some(p) = ent_ref.get::<&ae_core::ecs::Parent>() {
                parent_map.insert(ent, p.0);
            }

            // Children component
            if let Some(c) = ent_ref.get::<&ae_core::ecs::Children>() {
                let valid_children: Vec<hecs::Entity> =
                    c.0.iter()
                        .copied()
                        .filter(|&ch| world.contains(ch))
                        .collect();
                if !valid_children.is_empty() {
                    children_map.insert(ent, valid_children);
                }
            }

            // Light component (for icon)
            if ent_ref.get::<&ae_core::ecs::Light>().is_some() {
                is_light.insert(ent, true);
            }
        }

        // --- Collect root entities (no parent, or parent is dead) ---
        let mut roots: Vec<hecs::Entity> = name_map
            .keys()
            .copied()
            .filter(|ent| {
                match parent_map.get(ent) {
                    Some(&parent) => !world.contains(parent), // orphan → treat as root
                    None => true,                             // no parent → root
                }
            })
            .collect();

        // Deterministic ordering: sort by entity id for a stable display
        roots.sort_by_key(|e| e.id());

        // --- DFS traversal: build flat ordered row list ---
        self.rows.clear();
        self.scratch.clear();

        // Push roots in reverse so the first root comes off the stack first
        for &root in roots.iter().rev() {
            self.scratch.push((root, 0));
        }

        while let Some((ent, depth)) = self.scratch.pop() {
            let name = name_map.remove(&ent).unwrap_or_default();
            let has_children = children_map.contains_key(&ent);
            let icon = if is_light.contains_key(&ent) {
                "☀ "
            } else {
                "📦 "
            };

            self.rows.push(HierarchyRow {
                entity: ent,
                name,
                depth,
                has_children,
                icon,
            });

            // Push children in reverse DFS order
            if let Some(children) = children_map.get(&ent) {
                for &child in children.iter().rev() {
                    self.scratch.push((child, depth + 1));
                }
            }
        }
    }
}

impl Default for HierarchyCache {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Single row draw helper ─────────────────────────────────────────────────

/// Draws one pre-built `HierarchyRow` in the hierarchy panel.
/// All display data is already in the row — zero ECS lookups during draw.
/// Indentation is achieved with `ui.add_space` before the selectable label.
fn draw_row(
    ui: &mut egui::Ui,
    row: &HierarchyRow,
    is_selected: bool,
    ui_actions: &mut Vec<crate::ui::EngineUiAction>,
) {
    let indent = row.depth as f32 * 16.0;
    let text_color = if is_selected {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(200)
    };
    let text = format!(
        "{}{}{}",
        row.icon,
        if row.has_children { "▾ " } else { "" },
        row.name
    );

    ui.horizontal(|ui| {
        if indent > 0.0 {
            ui.add_space(indent);
        }
        let response =
            ui.selectable_label(is_selected, egui::RichText::new(text).color(text_color));
        if response.clicked() {
            ui_actions.push(crate::ui::EngineUiAction::SelectEntity(Some(row.entity)));
        }
    });
}

// ─── EngineUi impl ──────────────────────────────────────────────────────────

impl EngineUi {
    /// Renders the scene hierarchy tree using a cached flat list and virtual scrolling.
    /// # Performance contract
    /// - **Cache hit** (entity count unchanged): zero ECS lookups, O(visible_rows) draw only.
    /// - **Cache miss** (entity spawned/deleted): one O(N) single-pass world iteration to rebuild,
    ///   then O(visible_rows) draw. For 35 000 entities this is ~35k struct field reads per rebuild,
    ///   which happens once and amortizes across all subsequent frames until the scene changes.
    pub(super) fn draw_hierarchy_panel(
        selected_entity: &mut Option<hecs::Entity>,
        ctx: &egui::Context,
        world: &hecs::World,
        is_editing: bool,
        ui_actions: &mut Vec<crate::ui::EngineUiAction>,
        cache: &mut HierarchyCache,
    ) -> Option<egui::Rect> {
        // Sync cache first (no-op when entity count is unchanged)
        let hierarchy_resp = egui::Window::new("Scene Hierarchy")
            .default_pos(egui::pos2(250.0, 35.0))
            .default_size([240.0, 320.0])
            .default_open(false)
            .show(ctx, |ui| {
                // Sync cache ONLY when the window is actually open and visible!
                cache.sync(world);

                ui.add_enabled_ui(is_editing, |ui| {
                    ui.label("Scene Controls:");
                    ui.horizontal(|ui| {
                        #[allow(deprecated)]
                        ui.menu_button("➕ Add Shape", |ui| {
                            if ui.button("📦 Cube").clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                    ae_core::ecs::Shape::Cube,
                                ));
                                ui.close();
                            }
                            if ui.button("📐 Triangle").clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                    ae_core::ecs::Shape::Triangle,
                                ));
                                ui.close();
                            }
                            if ui.button("🔮 Sphere").clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                    ae_core::ecs::Shape::Sphere,
                                ));
                                ui.close();
                            }
                            if ui.button("🧪 Cylinder").clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                    ae_core::ecs::Shape::Cylinder,
                                ));
                                ui.close();
                            }
                            if ui.button("💊 Capsule").clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                    ae_core::ecs::Shape::Capsule,
                                ));
                                ui.close();
                            }
                            if ui.button("🍩 Torus").clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                    ae_core::ecs::Shape::Torus,
                                ));
                                ui.close();
                            }
                        });

                        if ui.button("📁 Load 3D Model").clicked() {
                            ui_actions.push(crate::ui::EngineUiAction::OpenModelDialog);
                        }

                        if ui
                            .button("📦 Load Prefab")
                            .on_hover_text(
                                "Load and instantiate a .aeprefab template into the scene",
                            )
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Aeon Prefab", &["aeprefab"])
                                .pick_file()
                            {
                                ui_actions.push(crate::ui::EngineUiAction::InstantiatePrefab(path));
                            }
                        }

                        if ui
                            .add_enabled(selected_entity.is_some(), egui::Button::new("🗑 Delete"))
                            .clicked()
                        {
                            ui_actions.push(crate::ui::EngineUiAction::DeleteSelected);
                        }
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        if ui.button("OpenWorld Test (10km)").clicked() {
                            ui_actions.push(crate::ui::EngineUiAction::AaaOpenWorldTest);
                        }

                        if ui.button("10k Objects").clicked() {
                            ui_actions.push(crate::ui::EngineUiAction::StressTest(10000));
                        }

                        if ui.button("100k Objects").clicked() {
                            ui_actions.push(crate::ui::EngineUiAction::StressTest(100000));
                        }

                        if ui.button("10M Objects").clicked() {
                            ui_actions.push(crate::ui::EngineUiAction::StressTest(10000000));
                        }

                        if ui.button("💥 BOOM!").clicked() {
                            ui_actions.push(crate::ui::EngineUiAction::ChangeMode(
                                ae_core::modules::EngineMode::Play,
                            ));
                            ui_actions.push(crate::ui::EngineUiAction::Explode);
                        }
                    });

                    ui.separator();
                    ui.label(format!("All Objects ({}):", world.len()));
                    ui.separator();

                    // Virtual scrolling: only visible rows are drawn
                    // Row height is a fixed estimate — keeps layout stable for large scenes
                    let row_height = 20.0_f32;
                    let total_rows = cache.rows.len();

                    if total_rows == 0 {
                        ui.label("No entities in scene.");
                    } else {
                        egui::ScrollArea::vertical().show_rows(
                            ui,
                            row_height,
                            total_rows,
                            |ui, visible_range| {
                                for idx in visible_range {
                                    let row = &cache.rows[idx];
                                    let is_selected = *selected_entity == Some(row.entity);
                                    draw_row(ui, row, is_selected, ui_actions);
                                }
                            },
                        );
                    }
                });
            });

        hierarchy_resp.map(|r| r.response.rect)
    }

    /// Renders the performance stats and diagnostics window.
    pub(super) fn draw_stats_panel(
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        ctx: &egui::Context,
        fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
    ) -> Option<egui::Rect> {
        let stats_resp = egui::Window::new("Engine Stats")
            .default_pos(egui::pos2(10.0, 35.0))
            .default_width(220.0)
            .default_open(false)
            .show(ctx, |ui| {
                ui.heading("Performance");
                ui.label(format!("FPS: {:.0}", fps));
                ui.label(format!("Frame Time: {:.2} ms", 1000.0 / fps));
                ui.separator();

                ui.heading("⏱ CPU Profiler");
                let bar_max = profiler_frame_ms.max(1.0);
                ui.horizontal(|ui| {
                    ui.label("ECS/Logic:");
                    ui.add(
                        egui::ProgressBar::new(profiler_ecs_ms / bar_max)
                            .text(format!("{:.2} ms", profiler_ecs_ms)),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Render:   ");
                    ui.add(
                        egui::ProgressBar::new(profiler_render_ms / bar_max)
                            .text(format!("{:.2} ms", profiler_render_ms)),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Present:  ");
                    ui.add(
                        egui::ProgressBar::new(profiler_present_ms / bar_max)
                            .fill(egui::Color32::from_rgb(80, 80, 140))
                            .text(format!("{:.2} ms", profiler_present_ms)),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("UI:       ");
                    ui.add(
                        egui::ProgressBar::new(profiler_ui_ms / bar_max)
                            .text(format!("{:.2} ms", profiler_ui_ms)),
                    );
                });
                ui.label(format!("Total Frame: {:.2} ms", profiler_frame_ms));
                ui.separator();

                ui.heading("💾 Memory");
                let total_mb = memory_models_mb + memory_textures_mb;
                ui.label(format!("Models (RAM+VRAM): {:.2} MB", memory_models_mb));
                ui.label(format!("Textures (VRAM):   {:.2} MB", memory_textures_mb));
                ui.label(format!("Total (Estimate):  {:.2} MB", total_mb));
                ui.separator();

                ui.checkbox(wireframe_enabled, "🕸 Wireframe Mode (Edges)");
                ui.checkbox(grid_enabled, "🔲 Show Grid");
            });

        stats_resp.map(|r| r.response.rect)
    }
}