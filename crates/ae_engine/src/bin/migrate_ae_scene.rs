// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Offline migration CLI converting legacy `.ae3d` / `.ae2d` JSON scene descriptors
//! into partitioned binary `.aeworld` archives (`ae_world`).
//!
//! Usage:
//! `cargo run -p ae_engine --bin migrate_ae_scene -- <input.ae3d|.ae2d> <output.aeworld>`

use ae_ecs::{ComponentHandler, ComponentRegistry};
use ae_plugin_api::EntityGuid;
use ae_world::{
    AssetKind, ComponentRecord, EntityRecord, GuidGenerator, SCENE_ASSET_BINDING_TYPE,
    SavedLodPaths, SceneAssetBinding, WorldDocument, asset_guid_from_virtual_path,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Deserialize)]
struct LegacyLodGroup {
    lod_0_path: String,
    #[serde(default)]
    lod_1_path: Option<String>,
    #[serde(default)]
    lod_2_path: Option<String>,
    threshold_1: f32,
    threshold_2: f32,
}

#[derive(Deserialize)]
struct LegacySavedEntity {
    #[serde(default)]
    dimension: Option<String>,
    #[serde(default)]
    model_path: Option<String>,
    #[serde(default)]
    sprite_path: Option<String>,
    #[serde(default)]
    parent_name: Option<String>,
    #[serde(default)]
    lod_group: Option<LegacyLodGroup>,
    #[serde(flatten)]
    components: HashMap<String, serde_json::Value>,
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn lookup_handler<'a>(
    registry: &'a ComponentRegistry,
    key: &str,
) -> Option<&'a dyn ComponentHandler> {
    if let Some(handler) = registry.get_by_name(key) {
        return Some(handler);
    }
    for handler in registry.handlers() {
        let name = handler.type_name();
        if name.eq_ignore_ascii_case(key)
            || to_snake_case(name) == key
            || (name == "Rotator" && (key == "rotator" || key == "rot"))
            || (name == "MovingPlatform" && (key == "moving_platform" || key == "platform"))
            || (name == "TriggerZone" && (key == "trigger_zone" || key == "trigger"))
            || (name == "DestructibleTarget"
                && (key == "destructible_target" || key == "destructible" || key == "target"))
            || (name == "CharacterAction"
                && (key == "character_action" || key == "combat" || key == "weapon"))
            || (name == "BoundingRadius" && (key == "bounding_radius" || key == "radius"))
            || (name == "BoundingBox" && (key == "bounding_box" || key == "bbox"))
            || (name == "CharacterController"
                && (key == "character_controller" || key == "controller"))
        {
            return Some(&**handler);
        }
    }
    None
}

fn detect_is_3d(entities: &[LegacySavedEntity], input_path: &Path) -> bool {
    if input_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ae2d"))
    {
        return false;
    }
    for se in entities {
        if let Some(ref dim) = se.dimension {
            if dim.eq_ignore_ascii_case("3d") {
                return true;
            }
            if dim.eq_ignore_ascii_case("2d") {
                return false;
            }
        }
        if se.model_path.is_some()
            || se.lod_group.is_some()
            || se.components.contains_key("shape")
            || se.components.contains_key("character_controller")
            || se.components.contains_key("rigid_body")
            || se.components.contains_key("collider")
            || se.components.contains_key("light")
        {
            return true;
        }
    }
    false
}

fn convert_legacy_scene(input: &Path, output: &Path) -> Result<usize, String> {
    let file = File::open(input).map_err(|e| format!("Failed to open {input:?}: {e}"))?;
    let legacy_entities: Vec<LegacySavedEntity> = serde_json::from_reader(BufReader::new(file))
        .map_err(|e| format!("Failed to parse legacy JSON {input:?}: {e}"))?;

    let is_2d = !detect_is_3d(&legacy_entities, input);
    let registry = ae_plugin_api::engine_component_registry();
    let mut guid_gen = GuidGenerator::new();
    let mut doc = WorldDocument::new(guid_gen.next_entity_guid(), is_2d, 128.0);

    let mut assigned_guids: Vec<EntityGuid> = Vec::with_capacity(legacy_entities.len());
    let mut name_to_guid: HashMap<String, EntityGuid> = HashMap::new();
    for ent in &legacy_entities {
        let g = guid_gen.next_entity_guid();
        assigned_guids.push(g);
        if let Some(name) = ent.components.get("name").and_then(|v| v.as_str()) {
            name_to_guid.insert(name.to_string(), g);
        }
    }

    for (idx, ent) in legacy_entities.into_iter().enumerate() {
        let guid = assigned_guids[idx];
        let parent_guid = ent
            .parent_name
            .as_ref()
            .and_then(|pn| name_to_guid.get(pn).copied());
        let mut record = EntityRecord::new(guid, parent_guid);
        let mut pos_opt: Option<[f32; 3]> = None;
        let mut is_global = false;

        for (key, val) in &ent.components {
            if val.is_null() {
                continue;
            }
            if key == "is_player" {
                if val.as_bool().unwrap_or(false)
                    && let Some(handler) = registry.get_by_name("PlayerTag")
                {
                    let bytes = handler.transcode_json_to_binary("null")?;
                    record.components.push(ComponentRecord {
                        type_hash: handler.type_hash(),
                        schema_ver: handler.schema_version(),
                        flags: 0,
                        payload: bytes,
                    });
                    is_global = true;
                }
                continue;
            }
            if key == "position"
                && let (Some(x), Some(y), Some(z)) = (
                    val.get("x").and_then(|v| v.as_f64()),
                    val.get("y").and_then(|v| v.as_f64()),
                    val.get("z").and_then(|v| v.as_f64()),
                )
            {
                pos_opt = Some([x as f32, y as f32, z as f32]);
            }
            if key == "light" {
                is_global = true;
            }
            if let Some(handler) = lookup_handler(registry, key) {
                let json_str = serde_json::to_string(val).map_err(|e| e.to_string())?;
                if let Ok(payload) = handler.transcode_json_to_binary(&json_str) {
                    record.components.push(ComponentRecord {
                        type_hash: handler.type_hash(),
                        schema_ver: handler.schema_version(),
                        flags: 0,
                        payload,
                    });
                }
            }
        }

        if ent.model_path.is_some() || ent.sprite_path.is_some() || ent.lod_group.is_some() {
            if let Some(ref mp) = ent.model_path {
                doc.register_asset_ref(asset_guid_from_virtual_path(mp), AssetKind::Model, mp)
                    .map_err(|e| e.to_string())?;
            }
            if let Some(ref sp) = ent.sprite_path {
                doc.register_asset_ref(asset_guid_from_virtual_path(sp), AssetKind::Texture, sp)
                    .map_err(|e| e.to_string())?;
            }
            let lod_group = ent.lod_group.map(|lg| SavedLodPaths {
                lod_0_path: lg.lod_0_path,
                lod_1_path: lg.lod_1_path,
                lod_2_path: lg.lod_2_path,
                threshold_1: lg.threshold_1,
                threshold_2: lg.threshold_2,
            });
            let binding = SceneAssetBinding {
                model_path: ent.model_path,
                sprite_path: ent.sprite_path,
                lod_group,
            };
            let comp_rec = ComponentRecord::from_value(SCENE_ASSET_BINDING_TYPE, 1, &binding)
                .map_err(|e| e.to_string())?;
            record.components.push(comp_rec);
        }

        match (is_global, pos_opt) {
            (false, Some(pos)) => doc.insert_entity_at(pos, record),
            _ => doc.persistent_entities.push(record),
        }
    }

    doc.save_to_file(output).map_err(|e| e.to_string())?;
    let total =
        doc.persistent_entities.len() + doc.cells.values().map(|c| c.entities.len()).sum::<usize>();
    Ok(total)
}

fn main() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let Some(in_arg) = args.next() else {
        return Err("Usage: migrate_ae_scene <input.ae3d|.ae2d> <output.aeworld>".to_string());
    };
    let Some(out_arg) = args.next() else {
        return Err("Usage: migrate_ae_scene <input.ae3d|.ae2d> <output.aeworld>".to_string());
    };

    let input = Path::new(&in_arg);
    let output = Path::new(&out_arg);
    let count = convert_legacy_scene(input, output)?;
    let _ = writeln!(
        std::io::stdout(),
        "Converted {count} entities from {} -> {}",
        input.display(),
        output.display()
    );
    Ok(())
}