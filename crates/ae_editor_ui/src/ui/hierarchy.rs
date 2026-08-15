// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::EngineUi;

/// A pre-built flat snapshot of one entity's hierarchy display data.
/// Built once per frame during sync to render the outliner tree with zero ECS world queries during draw.
#[derive(Clone)]
pub struct HierarchyRow {
    /// The ECS entity this row represents.
    pub entity: hecs::Entity,
    /// Display name.
    pub name: String,
    /// Indentation depth in the parent-child tree (0 = root).
    pub depth: usize,
    /// Whether this entity has at least one valid child.
    pub has_children: bool,
    /// Icon prefix derived from data-driven component presence.
    pub icon: &'static str,
    /// Whether the entity is currently visible (does not have the `Hidden` component).
    pub is_visible: bool,
}

/// Cached, pre-flattened scene hierarchy list with virtual scrolling support.
pub struct HierarchyCache {
    /// Flat ordered list of all entities (DFS pre-order: parent before children).
    pub rows: Vec<HierarchyRow>,
    /// Accumulates entities during recursive DFS without re-allocating per call.
    scratch: Vec<(hecs::Entity, usize)>,
}

impl HierarchyCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            scratch: Vec::new(),
        }
    }

    /// Rebuilds the flat row list from the ECS world using a single-pass DFS traversal.
    /// Uses 100% data-driven component inspection (Rule 21) to determine entity icons,
    /// parent-child tree nesting, and viewport visibility.
    pub fn sync(&mut self, world: &hecs::World) {
        let entity_count = world.len() as usize;

        // --- Single O(N) query: collect component data for display ---
        use std::collections::HashMap;

        // Pre-size maps up to a safety cap (25,000 max) to prevent memory allocation spikes
        let max_display_entities = 25_000;
        let cap = entity_count.min(max_display_entities);
        let mut name_map: HashMap<hecs::Entity, String> = HashMap::with_capacity(cap);
        let mut parent_map: HashMap<hecs::Entity, hecs::Entity> = HashMap::with_capacity(cap);
        let mut children_map: HashMap<hecs::Entity, Vec<hecs::Entity>> =
            HashMap::with_capacity(cap);
        let mut icon_map: HashMap<hecs::Entity, &'static str> = HashMap::with_capacity(cap);
        let mut visibility_map: HashMap<hecs::Entity, bool> = HashMap::with_capacity(cap);

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
                if world.contains(p.0) {
                    parent_map.insert(ent, p.0);
                }
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

            // 100% Data-Driven Component Icon Assignment (Strict Rule 21 Compliance)
            let icon = if ent_ref.get::<&ae_core::ecs::Light>().is_some() {
                "💡 "
            } else if ent_ref.get::<&ae_audio::AudioSource>().is_some() {
                "🔊 "
            } else if ent_ref.get::<&ae_core::ecs::PlayerTag>().is_some() {
                "🎮 "
            } else if ent_ref.get::<&ae_core::ecs::ModelId>().is_some() {
                "📦 "
            } else if ent_ref.get::<&ae_core::ecs::Shape>().is_some() {
                "🧊 "
            } else if ent_ref.get::<&ae_core::ecs::SpriteId>().is_some() {
                "🖼 "
            } else {
                "📁 "
            };
            icon_map.insert(ent, icon);

            // Visibility (Hidden component check)
            let is_visible = ent_ref.get::<&ae_core::ecs::Hidden>().is_none();
            visibility_map.insert(ent, is_visible);
        }

        // Two-way synchronization: ensure all parent_map links are in children_map
        for (&child, &parent) in &parent_map {
            let list = children_map.entry(parent).or_default();
            if !list.contains(&child) {
                list.push(child);
            }
        }

        // --- Collect root entities (no parent, or parent is dead) ---
        let mut roots: Vec<hecs::Entity> = name_map
            .keys()
            .copied()
            .filter(|ent| !parent_map.contains_key(ent))
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
            let has_children = children_map
                .get(&ent)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            let icon = icon_map.get(&ent).copied().unwrap_or("📁 ");
            let is_visible = visibility_map.get(&ent).copied().unwrap_or(true);

            self.rows.push(HierarchyRow {
                entity: ent,
                name,
                depth,
                has_children,
                icon,
                is_visible,
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

/// Draws one pre-built `HierarchyRow` in the hierarchy panel with selection, tree branch lines, and working eye toggle.
fn draw_row(
    ui: &mut egui::Ui,
    row: &HierarchyRow,
    is_selected: bool,
    ui_actions: &mut Vec<crate::ui::EngineUiAction>,
) {
    let indent = row.depth as f32 * 16.0;
    let text_color = if is_selected {
        egui::Color32::from_rgb(0, 229, 255) // Aeon Cyan selection accent
    } else {
        egui::Color32::from_gray(215)
    };

    ui.horizontal(|ui| {
        // 1. Indentation Space
        if indent > 0.0 {
            ui.add_space(indent);
        }

        // 2. Tree branch connector
        if row.depth > 0 {
            ui.label(
                egui::RichText::new("└──")
                    .color(egui::Color32::from_rgb(0, 180, 216))
                    .monospace(),
            );
        } else if row.has_children {
            ui.label(
                egui::RichText::new("▼")
                    .size(10.0)
                    .color(egui::Color32::from_gray(160)),
            );
        }

        // 3. Component Icon
        ui.label(egui::RichText::new(row.icon).size(13.0));

        // 4. Entity Selectable Label (fills available space minus eye button)
        let label_width = (ui.available_width() - 28.0).max(30.0);
        let name_resp = ui.add_sized(
            [label_width, 18.0],
            egui::Button::new(egui::RichText::new(&row.name).color(text_color).strong())
                .selected(is_selected)
                .fill(if is_selected {
                    egui::Color32::from_rgb(0, 70, 90)
                } else {
                    egui::Color32::TRANSPARENT
                })
                .frame(is_selected),
        );
        if name_resp.clicked() {
            ui_actions.push(crate::ui::EngineUiAction::SelectEntity(Some(row.entity)));
        }

        // 5. Eye Visibility Toggle Button (Dedicated, isolated hitbox)
        let (eye_icon, eye_color, tooltip) = if row.is_visible {
            (
                "👁",
                egui::Color32::from_gray(220),
                "Visible in Viewport (Click to hide)",
            )
        } else {
            (
                "🚫",
                egui::Color32::from_rgb(230, 80, 80),
                "Hidden in Viewport (Click to show)",
            )
        };

        let eye_btn = ui.add_sized(
            [20.0, 18.0],
            egui::Button::new(egui::RichText::new(eye_icon).size(12.0).color(eye_color))
                .fill(egui::Color32::TRANSPARENT)
                .frame(false),
        );
        if eye_btn.clicked() {
            ui_actions.push(crate::ui::EngineUiAction::ToggleVisibility(row.entity));
        }
        if eye_btn.hovered() {
            eye_btn.show_tooltip_text(tooltip);
        }
    });
}

// ─── EngineUi impl ──────────────────────────────────────────────────────────

impl EngineUi {
    /// Renders the modern docked Left Panel (Hierarchy Outliner & Stats Tabs).
    /// Features:
    /// - Live search & filter bar with clear button
    /// - Collapsible `▼ ➕ Spawn 3D Shapes & Assets` section
    /// - Collapsible `▼ ⚡ Stress Benchmarks` section
    /// - Virtual scrolling DFS entity tree with component badges, tree branch lines, and working eye toggle
    pub(super) fn draw_left_panel(
        show_left_panel: &mut bool,
        left_panel_tab: &mut usize,
        hierarchy_search_query: &mut String,
        selected_entity: &mut Option<hecs::Entity>,
        ui: &mut egui::Ui,
        world: &hecs::World,
        is_editing: bool,
        ui_actions: &mut Vec<crate::ui::EngineUiAction>,
        cache: &mut HierarchyCache,
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
    ) -> Option<egui::Rect> {
        if !*show_left_panel {
            return None;
        }

        let resp = egui::Panel::left("left_docked_panel")
            .default_size(280.0)
            .min_size(220.0)
            .max_size(500.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(20, 20, 25))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60))),
            )
            .show(ui, |ui| {
                // 1. Tab Bar on top of Left Panel + Close Button
                ui.horizontal(|ui| {
                    crate::ui::tab_bar::draw_tab_bar(
                        ui,
                        left_panel_tab,
                        &[
                            crate::ui::tab_bar::EditorTab::new(0, "🏗️", "Hierarchy"),
                            crate::ui::tab_bar::EditorTab::new(1, "📊", "Stats"),
                        ],
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✖")
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(160)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false),
                            )
                            .on_hover_text("Close Left Panel")
                            .clicked()
                        {
                            *show_left_panel = false;
                        }
                    });
                });

                ui.add_space(6.0);

                if *left_panel_tab == 0 {
                    // ─────────────────────────────────────────────────────────────
                    // TAB 0: Modern  Scene Hierarchy (Outliner)
                    // ─────────────────────────────────────────────────────────────
                    cache.sync(world);

                    ui.add_enabled_ui(is_editing, |ui| {
                        // 1. Live Search Bar
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("🔍")
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(140)),
                            );
                            let search_width = if !hierarchy_search_query.is_empty() {
                                (ui.available_width() - 24.0).max(80.0)
                            } else {
                                ui.available_width()
                            };
                            ui.add(
                                egui::TextEdit::singleline(hierarchy_search_query)
                                    .hint_text("Search entities...")
                                    .desired_width(search_width),
                            );
                            if !hierarchy_search_query.is_empty() {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("✖")
                                                .size(10.0)
                                                .color(egui::Color32::from_gray(160)),
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .frame(false),
                                    )
                                    .on_hover_text("Clear search query")
                                    .clicked()
                                {
                                    hierarchy_search_query.clear();
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // 2. Collapsible: ➕ Spawn 3D Shapes & Assets
                        egui::CollapsingHeader::new("➕  Spawn Shapes & Assets")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button("📦 Cube").clicked() {
                                        ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                            ae_core::ecs::Shape::Cube,
                                        ));
                                    }
                                    if ui.button("🔮 Sphere").clicked() {
                                        ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                            ae_core::ecs::Shape::Sphere,
                                        ));
                                    }
                                    if ui.button("🧪 Cylinder").clicked() {
                                        ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                            ae_core::ecs::Shape::Cylinder,
                                        ));
                                    }
                                    if ui.button("💊 Capsule").clicked() {
                                        ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                            ae_core::ecs::Shape::Capsule,
                                        ));
                                    }
                                    if ui.button("🍩 Torus").clicked() {
                                        ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                            ae_core::ecs::Shape::Torus,
                                        ));
                                    }
                                    if ui.button("📐 Triangle").clicked() {
                                        ui_actions.push(crate::ui::EngineUiAction::SpawnShape(
                                            ae_core::ecs::Shape::Triangle,
                                        ));
                                    }
                                });

                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .button("📁 3D Model")
                                        .on_hover_text("Load 3D Model file (glTF, GLB, OBJ)")
                                        .clicked()
                                    {
                                        ui_actions.push(crate::ui::EngineUiAction::OpenModelDialog);
                                    }

                                    if ui
                                        .button("📦 Load Prefab")
                                        .on_hover_text("Load and instantiate a .aeprefab template")
                                        .clicked()
                                    {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Aeon Prefab", &["aeprefab"])
                                            .pick_file()
                                        {
                                            ui_actions.push(
                                                crate::ui::EngineUiAction::InstantiatePrefab(path),
                                            );
                                        }
                                    }

                                    if ui
                                        .add_enabled(
                                            selected_entity.is_some(),
                                            egui::Button::new("🗑 Delete"),
                                        )
                                        .on_hover_text("Delete selected entity from scene")
                                        .clicked()
                                    {
                                        ui_actions.push(crate::ui::EngineUiAction::DeleteSelected);
                                    }
                                });
                            });

                        // 3. Collapsible: ⚡ Stress Benchmarks
                        egui::CollapsingHeader::new("⚡  Stress Benchmarks")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    if ui
                                        .button("OpenWorld (10km)")
                                        .on_hover_text("Spawn vast 10km open-world grid terrain")
                                        .clicked()
                                    {
                                        ui_actions
                                            .push(crate::ui::EngineUiAction::AaaOpenWorldTest);
                                    }
                                    if ui
                                        .button("10k")
                                        .on_hover_text("Spawn 10,000 dynamic physics cubes")
                                        .clicked()
                                    {
                                        ui_actions
                                            .push(crate::ui::EngineUiAction::StressTest(10000));
                                    }
                                    if ui
                                        .button("100k")
                                        .on_hover_text("Spawn 100,000 instanced ECS entities")
                                        .clicked()
                                    {
                                        ui_actions
                                            .push(crate::ui::EngineUiAction::StressTest(100000));
                                    }
                                    if ui
                                        .button("10M")
                                        .on_hover_text("Spawn 10,000,000 GPU batch stress test")
                                        .clicked()
                                    {
                                        ui_actions
                                            .push(crate::ui::EngineUiAction::StressTest(10000000));
                                    }
                                    if ui
                                        .button("💥 BOOM!")
                                        .on_hover_text("Trigger physics shockwave explosion")
                                        .clicked()
                                    {
                                        ui_actions.push(crate::ui::EngineUiAction::ChangeMode(
                                            ae_core::modules::EngineMode::Play,
                                        ));
                                        ui_actions.push(crate::ui::EngineUiAction::Explode);
                                    }
                                });
                            });

                        ui.separator();

                        // 4. Virtual Scrolling Tree Header & Rows
                        ui.label(
                            egui::RichText::new("🌍  Scene Hierarchy Tree")
                                .strong()
                                .color(egui::Color32::from_gray(180)),
                        );
                        ui.add_space(2.0);

                        let row_height = 22.0_f32;
                        let search_query_lower = hierarchy_search_query.trim().to_lowercase();
                        let filtered_indices: Vec<usize> = if search_query_lower.is_empty() {
                            (0..cache.rows.len()).collect()
                        } else {
                            cache
                                .rows
                                .iter()
                                .enumerate()
                                .filter(|(_, r)| {
                                    r.name.to_lowercase().contains(&search_query_lower)
                                })
                                .map(|(i, _)| i)
                                .collect()
                        };

                        let total_visible = filtered_indices.len();

                        if total_visible == 0 {
                            if !search_query_lower.is_empty() {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(15.0);
                                    ui.label(
                                        egui::RichText::new("No matching entities found")
                                            .color(egui::Color32::from_gray(140))
                                            .italics(),
                                    );
                                });
                            } else {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(15.0);
                                    ui.label(
                                        egui::RichText::new("Scene is empty")
                                            .color(egui::Color32::from_gray(140))
                                            .italics(),
                                    );
                                });
                            }
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_height(ui.available_height() - 28.0)
                                .show_rows(ui, row_height, total_visible, |ui, visible_range| {
                                    for idx in visible_range {
                                        if let Some(&row_idx) = filtered_indices.get(idx) {
                                            let row = &cache.rows[row_idx];
                                            let is_selected = *selected_entity == Some(row.entity);
                                            draw_row(ui, row, is_selected, ui_actions);
                                        }
                                    }
                                });
                        }

                        // 5. Scene Summary Footer
                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                            ui.separator();
                            ui.horizontal(|ui| {
                                let total = world.len();
                                let sel_str = if selected_entity.is_some() {
                                    " • 1 Selected"
                                } else {
                                    ""
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} Object{}{}",
                                        total,
                                        if total == 1 { "" } else { "s" },
                                        sel_str
                                    ))
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(140)),
                                );
                            });
                        });
                    });
                } else {
                    // ─────────────────────────────────────────────────────────────
                    // TAB 1: Stats & Profiler
                    // ─────────────────────────────────────────────────────────────
                    egui::ScrollArea::vertical().show(ui, |ui| {
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
                }
            });

        Some(resp.response.rect)
    }
}