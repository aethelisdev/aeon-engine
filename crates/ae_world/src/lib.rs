// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Binary world archive (`.aeworld`), spatial partition streaming (`.aechunk`),
//! block-compressed asset containers (`.aepak`), Virtual File System (`VfsProvider`),
//! and atomic runtime delta savegames (`.aesave`).

pub mod aligned_buffer;
pub mod component_record;
pub mod cooked;
pub mod error;
pub mod guid;
pub mod guid_resolver;
pub mod header;
pub mod pak;
pub mod partition;
pub mod project_layout;
pub mod save_game;
pub mod vfs;
pub mod world_archive;
pub mod world_dir;

pub use aligned_buffer::{AlignedBuffer, BUFFER_ALIGNMENT_BYTES};
pub use component_record::{
    ComponentRecord, EntityRecord, SCENE_ASSET_BINDING_TYPE, SavedLodPaths, SceneAssetBinding,
};
pub use cooked::{
    COOKED_FORMAT_VERSION, CookedAlphaMode, CookedAudio, CookedMesh, CookedMipLevel, CookedSubmesh,
    CookedTexture, CookedTextureFormat, TargetTextureProfile, compute_asset_content_hash,
    intermediate_cook_path, read_cached_cook_bytes, write_cached_cook_bytes,
};
pub use error::{WorldIoError, WorldResult};
pub use guid::{
    AssetGuid, EntityGuid, GuidGenerator, asset_guid_from_virtual_path, component_type_hash,
};
pub use guid_resolver::{PendingGuidLink, PendingGuidResolver, ResolvedGuidLink};
pub use header::{
    BinaryEnvelopeHeader, ByteReader, CHUNK_MAGIC, COOKED_AUDIO_MAGIC, COOKED_MESH_MAGIC,
    COOKED_TEXTURE_MAGIC, DecodedEnvelope, ENVELOPE_HEADER_BYTES, PAK_MAGIC, SAVE_MAGIC,
    WORLD_MAGIC, compute_crc32, decode_checksummed_envelope, encode_checksummed_envelope,
    verify_crc32,
};
pub use pak::{
    DEFAULT_PAK_BLOCK_SIZE, PAK_BLOCK_ALIGNMENT, PAK_FORMAT_VERSION, PAK_HEADER_BYTES,
    PakBlockMeta, PakBuilder, PakCodec, PakIndex, PakReader, PakTocEntry, open_pak_file,
};
pub use partition::{
    CHUNK_FORMAT_VERSION, CellStreamingPlanner, PartitionCell, SpatialCellCoord, StreamingPlan,
};
pub use project_layout::{
    AEMETA_FORMAT_VERSION, AEMETA_MAGIC, AssetMeta, ProjectWorkspaceLayout, sync_content_metadata,
};
pub use save_game::{
    CellDelta, GameplayValue, SAVE_FORMAT_VERSION, SaveSlotMetadata, WorldDeltaSnapshot,
    atomic_write_save, backup_path_for, load_save_with_fallback, temp_path_for,
};
pub use vfs::{
    CompositeVfs, DiskVfsProvider, PakFileVfsProvider, PakMemoryVfsProvider, VfsProvider,
    sanitize_virtual_path,
};
pub use world_archive::{
    AssetKind, AssetRefEntry, PartitionIndexEntry, WORLD_FLAG_2D, WORLD_FORMAT_VERSION,
    WorldDocument, WorldIndex,
};
pub use world_dir::{
    PartitionedSaveStats, WORLD_FLAG_PARTITIONED_DIR, WORLD_MANIFEST_FILENAME,
    resolve_partitioned_world_paths,
};