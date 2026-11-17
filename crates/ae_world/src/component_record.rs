// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Length-prefixed binary component and entity serialization streams with forward schema tolerance.
//!
//! Each serialized [`ComponentRecord`] writes its byte length ahead of the `postcard` payload so
//! older or custom readers can inspect `type_hash` and `schema_ver` and skip unknown components
//! without aborting entity loading.

use crate::error::{WorldIoError, WorldResult};
use crate::guid::{EntityGuid, component_type_hash};
use crate::header::ByteReader;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashSet;

/// Canonical binary component type name used to persist entity asset paths (models, sprites, LODs)
/// inside `.aeworld` `EntityRecord` payloads without JSON.
pub const SCENE_ASSET_BINDING_TYPE: &str = "ae_world::SceneAssetBinding";

/// Serialized asset paths for an entity's multi-tier Level of Detail group.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SavedLodPaths {
    /// Virtual path to highest-detail LOD0 mesh asset.
    pub lod_0_path: String,
    /// Optional medium-detail LOD1 mesh path.
    pub lod_1_path: Option<String>,
    /// Optional low-detail LOD2 mesh path.
    pub lod_2_path: Option<String>,
    /// Distance threshold for switching from LOD0 to LOD1.
    pub threshold_1: f32,
    /// Distance threshold for switching from LOD1 to LOD2.
    pub threshold_2: f32,
}

/// Binary component payload storing referenced model, sprite, and LOD paths for one entity.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetBinding {
    /// Virtual path to the entity's primary 3D model asset, if any.
    pub model_path: Option<String>,
    /// Virtual path to the entity's primary 2D sprite texture asset, if any.
    pub sprite_path: Option<String>,
    /// Optional LOD chain asset paths and switch thresholds.
    pub lod_group: Option<SavedLodPaths>,
}

/// Self-framed binary payload for a single ECS component attached to an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRecord {
    /// Deterministic 64-bit FNV-1a type discriminator computed via [`component_type_hash`].
    pub type_hash: u64,
    /// Component schema revision for versioned migration handling.
    pub schema_ver: u16,
    /// Reserved per-record metadata flags.
    pub flags: u16,
    /// `postcard`-encoded binary bytes representing the component fields.
    pub payload: Vec<u8>,
}

impl ComponentRecord {
    /// Serializes a Rust component value into a tagged [`ComponentRecord`] using `postcard`.
    ///
    /// # Errors
    /// Returns [`WorldIoError::Serialization`] if `postcard::to_stdvec` fails.
    pub fn from_value<T: Serialize>(
        type_name: &str,
        schema_ver: u16,
        value: &T,
    ) -> WorldResult<Self> {
        let payload =
            postcard::to_stdvec(value).map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        Ok(Self {
            type_hash: component_type_hash(type_name),
            schema_ver,
            flags: 0,
            payload,
        })
    }

    /// Decodes the stored `postcard` bytes back into concrete Rust type `T`.
    ///
    /// # Errors
    /// Returns [`WorldIoError::Serialization`] if the payload bytes do not match `T`.
    pub fn decode_as<T: DeserializeOwned>(&self) -> WorldResult<T> {
        postcard::from_bytes(&self.payload).map_err(|e| WorldIoError::Serialization(e.to_string()))
    }

    /// Appends the length-prefixed binary representation of this record to `out`.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if payload size exceeds `u32::MAX`.
    pub fn encode_into(&self, out: &mut Vec<u8>) -> WorldResult<()> {
        let payload_len = u32::try_from(self.payload.len()).map_err(|_| {
            WorldIoError::InvalidLayout(format!(
                "Component payload exceeds 4 GiB limit ({} bytes)",
                self.payload.len()
            ))
        })?;
        out.extend_from_slice(&self.type_hash.to_le_bytes());
        out.extend_from_slice(&self.schema_ver.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&self.payload);
        Ok(())
    }

    /// Decodes a length-prefixed record, skipping payloads whose `type_hash` is not present in
    /// `known_hashes` (`None` preserves all records verbatim).
    ///
    /// # Errors
    /// Returns [`WorldIoError::TruncatedData`] when the stream ends mid-record.
    pub fn decode_filtered(
        reader: &mut ByteReader<'_>,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<Option<Self>> {
        let type_hash = reader.read_u64("component type_hash")?;
        let schema_ver = reader.read_u16("component schema_ver")?;
        let flags = reader.read_u16("component flags")?;
        let payload_len = reader.read_u32("component payload_len")? as usize;

        if known_hashes.is_some_and(|set| !set.contains(&type_hash)) {
            reader.skip(payload_len, "skipped unknown component payload")?;
            return Ok(None);
        }

        let slice = reader.read_slice(payload_len, "component payload")?;
        Ok(Some(Self {
            type_hash,
            schema_ver,
            flags,
            payload: slice.to_vec(),
        }))
    }
}

/// Complete binary representation of a persisted entity and its hierarchical parent link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRecord {
    /// Persistent 128-bit identity for this entity across world reloads and delta savegames.
    pub guid: EntityGuid,
    /// Optional persistent parent reference replacing fragile string name matching.
    pub parent_guid: Option<EntityGuid>,
    /// Length-prefixed binary components attached to this entity.
    pub components: Vec<ComponentRecord>,
}

impl EntityRecord {
    /// Creates an empty record for `guid` with optional `parent_guid`.
    #[must_use]
    pub fn new(guid: EntityGuid, parent_guid: Option<EntityGuid>) -> Self {
        Self {
            guid,
            parent_guid,
            components: Vec::new(),
        }
    }

    /// Finds the first component matching `type_name`'s deterministic hash.
    #[must_use]
    pub fn find_component(&self, type_name: &str) -> Option<&ComponentRecord> {
        let wanted = component_type_hash(type_name);
        self.components.iter().find(|c| c.type_hash == wanted)
    }

    /// Encodes a slice of [`EntityRecord`]s into a self-framed binary stream with length-prefixed components.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if entity/component counts overflow `u32`.
    pub fn encode_stream(records: &[EntityRecord]) -> WorldResult<Vec<u8>> {
        let count = u32::try_from(records.len()).map_err(|_| {
            WorldIoError::InvalidLayout("Entity stream exceeds u32 entity limit".to_string())
        })?;
        let mut out = Vec::with_capacity(4 + records.len() * 64);
        out.extend_from_slice(&count.to_le_bytes());

        for entity in records {
            out.extend_from_slice(&entity.guid.to_le_bytes());
            let parent_raw = entity.parent_guid.unwrap_or(EntityGuid::NIL);
            out.extend_from_slice(&parent_raw.to_le_bytes());

            let comp_count = u32::try_from(entity.components.len()).map_err(|_| {
                WorldIoError::InvalidLayout("Entity has more than u32::MAX components".to_string())
            })?;
            out.extend_from_slice(&comp_count.to_le_bytes());
            for comp in &entity.components {
                comp.encode_into(&mut out)?;
            }
        }
        Ok(out)
    }

    /// Decodes a binary entity stream produced by [`EntityRecord::encode_stream`].
    ///
    /// When `known_hashes` is `Some`, components whose `type_hash` is absent from the set
    /// are skipped cleanly via their `payload_len` prefix without failing entity loading.
    ///
    /// # Errors
    /// Returns [`WorldIoError::TruncatedData`] on short or malformed input buffers.
    pub fn decode_stream(
        bytes: &[u8],
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<Vec<Self>> {
        let mut reader = ByteReader::new(bytes);
        let entity_count = reader.read_u32("entity stream count")? as usize;
        // Guard against absurd claimed counts on tiny buffers before preallocating.
        let min_bytes_per_entity = 16 + 16 + 4;
        if entity_count.saturating_mul(min_bytes_per_entity) > reader.remaining() {
            return Err(WorldIoError::TruncatedData {
                context: "entity stream records",
                expected: entity_count.saturating_mul(min_bytes_per_entity),
                actual: reader.remaining(),
            });
        }

        let mut out = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            let guid = EntityGuid::from_le_bytes(reader.read_u128("entity guid")?.to_le_bytes());
            let parent_val =
                EntityGuid::from_le_bytes(reader.read_u128("entity parent_guid")?.to_le_bytes());
            let parent_guid = if parent_val.is_nil() {
                None
            } else {
                Some(parent_val)
            };

            let comp_count = reader.read_u32("entity component_count")? as usize;
            let min_comp_bytes = comp_count.saturating_mul(16);
            if min_comp_bytes > reader.remaining() {
                return Err(WorldIoError::TruncatedData {
                    context: "entity component table",
                    expected: min_comp_bytes,
                    actual: reader.remaining(),
                });
            }

            let mut components = Vec::with_capacity(comp_count);
            for _ in 0..comp_count {
                if let Some(record) = ComponentRecord::decode_filtered(&mut reader, known_hashes)? {
                    components.push(record);
                }
            }

            out.push(Self {
                guid,
                parent_guid,
                components,
            });
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guid::GuidGenerator;
    use ae_plugin_api::{Name, Position};

    #[test]
    fn test_component_record_roundtrip_and_unknown_skip_forward_compat() {
        let mut ids = GuidGenerator::from_seed(77);
        let root_guid = ids.next_entity_guid();
        let child_guid = ids.next_entity_guid();

        #[derive(Serialize, Deserialize)]
        struct FutureCustomComponent {
            custom_flags: u64,
            extra_blob: Vec<u8>,
        }

        let mut child = EntityRecord::new(child_guid, Some(root_guid));
        child.components.push(
            ComponentRecord::from_value("Name", 1, &Name("Hero".to_string()))
                .expect("serialize Name"),
        );
        child.components.push(
            ComponentRecord::from_value(
                "FutureUnknownV9",
                9,
                &FutureCustomComponent {
                    custom_flags: 0xdead_beef,
                    extra_blob: vec![1, 2, 3, 4, 5, 6],
                },
            )
            .expect("serialize FutureUnknownV9"),
        );
        child.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(3.5, 4.0, -9.25))
                .expect("serialize Position"),
        );

        let stream = EntityRecord::encode_stream(&[child]).expect("encode entity stream");

        // Older reader only knows "Name" and "Position": must cleanly skip FutureUnknownV9 and still read Position.
        let known: HashSet<u64> = [component_type_hash("Name"), component_type_hash("Position")]
            .into_iter()
            .collect();

        let decoded =
            EntityRecord::decode_stream(&stream, Some(&known)).expect("filtered decode succeeds");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].guid, child_guid);
        assert_eq!(decoded[0].parent_guid, Some(root_guid));
        assert_eq!(decoded[0].components.len(), 2);

        let restored_name: Name = decoded[0]
            .find_component("Name")
            .expect("Name present")
            .decode_as()
            .expect("decode Name");
        let restored_pos: Position = decoded[0]
            .find_component("Position")
            .expect("Position present")
            .decode_as()
            .expect("decode Position");

        assert_eq!(restored_name.0, "Hero");
        assert_eq!(restored_pos, Position::new(3.5, 4.0, -9.25));
    }

    #[test]
    fn test_entity_stream_rejects_corrupted_truncated_counts() {
        let mut fake = Vec::new();
        fake.extend_from_slice(&1_000_000u32.to_le_bytes());
        fake.extend_from_slice(&[0u8; 8]);
        assert!(matches!(
            EntityRecord::decode_stream(&fake, None),
            Err(WorldIoError::TruncatedData { .. })
        ));
    }
}