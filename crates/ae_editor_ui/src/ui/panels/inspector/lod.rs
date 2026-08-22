// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::{EngineUi, EngineUiAction};
use cgmath::InnerSpace;

impl EngineUi {
    /// Renders the LodGroup inspector panel section.
    pub(super) fn draw_lod_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(lod) = world.get::<&ae_core::ecs::LodGroup>(entity) {
            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "LOD Group",
                "📊",
                egui::Color32::from_rgb(180, 150, 255),
                true,
                |ui| {
                    let cam_pos = cgmath::Vector3::new(
                        camera.position.x,
                        camera.position.y,
                        camera.position.z,
                    );
                    let p_world =
                        if let Ok(gt) = world.get::<&ae_core::ecs::GlobalTransform>(entity) {
                            cgmath::Vector3::new(gt.0.w.x, gt.0.w.y, gt.0.w.z)
                        } else if let Ok(pos) = world.get::<&ae_core::ecs::Position>(entity) {
                            cgmath::Vector3::new(pos.x, pos.y, pos.z)
                        } else {
                            cgmath::Vector3::new(0.0, 0.0, 0.0)
                        };
                    let dist = (p_world - cam_pos).magnitude();

                    let active_lod = if dist < lod.threshold_1 {
                        "LOD 0 (High Detail)"
                    } else if dist < lod.threshold_2 {
                        "LOD 1 (Medium Detail)"
                    } else {
                        "LOD 2 (Low Detail)"
                    };

                    ui.colored_label(
                        egui::Color32::from_rgb(77, 163, 255),
                        format!("Distance: {:.1} units", dist),
                    );
                    ui.colored_label(egui::Color32::GREEN, format!("Active Mesh: {}", active_lod));

                    ui.separator();

                    let mut get_model_name = |handle: ae_renderer::asset::AssetHandle| -> String {
                        if let Some(asset) = models.get(handle) {
                            std::path::Path::new(&asset.source_path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&asset.source_path)
                                .to_string()
                        } else {
                            format!("Unknown Model ({:?})", handle)
                        }
                    };

                    // LOD 0
                    ui.horizontal(|ui| {
                        ui.label("LOD 0 (High):");
                        let current_name = get_model_name(lod.lod_0);
                        #[allow(deprecated)]
                        egui::ComboBox::from_id_salt(egui::Id::new(("lod0_combo", entity)))
                            .selected_text(current_name)
                            .show_ui(ui, |ui| {
                                for (handle, _) in models.iter() {
                                    let name = get_model_name(handle);
                                    if ui.selectable_label(handle == lod.lod_0, name).clicked() {
                                        ui_actions.push(EngineUiAction::ModifyLodModel(
                                            entity,
                                            0,
                                            Some(handle),
                                        ));
                                    }
                                }
                            });
                    });

                    // LOD 1
                    ui.horizontal(|ui| {
                        ui.label("LOD 1 (Med):");
                        let current_name = lod
                            .lod_1
                            .map(&mut get_model_name)
                            .unwrap_or_else(|| "None".to_string());
                        #[allow(deprecated)]
                        egui::ComboBox::from_id_salt(egui::Id::new(("lod1_combo", entity)))
                            .selected_text(current_name)
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(lod.lod_1.is_none(), "None").clicked() {
                                    ui_actions
                                        .push(EngineUiAction::ModifyLodModel(entity, 1, None));
                                }
                                for (handle, _) in models.iter() {
                                    let name = get_model_name(handle);
                                    if ui
                                        .selectable_label(Some(handle) == lod.lod_1, name)
                                        .clicked()
                                    {
                                        ui_actions.push(EngineUiAction::ModifyLodModel(
                                            entity,
                                            1,
                                            Some(handle),
                                        ));
                                    }
                                }
                            });
                    });

                    // LOD 2
                    ui.horizontal(|ui| {
                        ui.label("LOD 2 (Low):");
                        let current_name = lod
                            .lod_2
                            .map(&mut get_model_name)
                            .unwrap_or_else(|| "None".to_string());
                        #[allow(deprecated)]
                        egui::ComboBox::from_id_salt(egui::Id::new(("lod2_combo", entity)))
                            .selected_text(current_name)
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(lod.lod_2.is_none(), "None").clicked() {
                                    ui_actions
                                        .push(EngineUiAction::ModifyLodModel(entity, 2, None));
                                }
                                for (handle, _) in models.iter() {
                                    let name = get_model_name(handle);
                                    if ui
                                        .selectable_label(Some(handle) == lod.lod_2, name)
                                        .clicked()
                                    {
                                        ui_actions.push(EngineUiAction::ModifyLodModel(
                                            entity,
                                            2,
                                            Some(handle),
                                        ));
                                    }
                                }
                            });
                    });

                    ui.separator();

                    let mut t1 = lod.threshold_1;
                    let mut t2 = lod.threshold_2;
                    ui.horizontal(|ui| {
                        ui.label("LOD 0->1 Dist:");
                        if ui
                            .add(egui::DragValue::new(&mut t1).speed(0.5).range(0.1..=t2))
                            .changed()
                        {
                            ui_actions.push(EngineUiAction::ModifyLodThresholds(entity, t1, t2));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("LOD 1->2 Dist:");
                        if ui
                            .add(egui::DragValue::new(&mut t2).speed(0.5).range(t1..=2000.0))
                            .changed()
                        {
                            ui_actions.push(EngineUiAction::ModifyLodThresholds(entity, t1, t2));
                        }
                    });
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveComponent(entity, "LodGroup"));
            }
        }
    }
}

pub struct LodGroupUiHandler;

impl super::registry::ComponentUiHandler for LodGroupUiHandler {
    fn component_name(&self) -> &'static str {
        "LodGroup"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "LOD Group (Distance Switching)",
            "👁",
            egui::Color32::from_rgb(120, 230, 200),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Rendering", "LOD Group")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::LodGroup>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_lod_section(
            ui,
            ctx.world,
            ctx.entity,
            ctx.camera,
            ctx.models,
            ctx.ui_actions,
        );
    }
}