// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Spatial world partition cells (`.aechunk`), parallel compression, and hysteresis streaming planner.

use crate::component_record::EntityRecord;
use crate::error::{WorldIoError, WorldResult};
use crate::header::{CHUNK_MAGIC, decode_checksummed_envelope, encode_checksummed_envelope};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Current format version written to compressed `.aechunk` envelopes.
pub const CHUNK_FORMAT_VERSION: u32 = 1;

/// Discrete 3D grid cell coordinates partitioning an unbounded world space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpatialCellCoord {
    /// Grid index along the world X axis.
    pub x: i32,
    /// Grid index along the world Y axis.
    pub y: i32,
    /// Grid index along the world Z axis.
    pub z: i32,
}

impl SpatialCellCoord {
    /// Origin cell `(0, 0, 0)`.
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    /// Constructs cell coordinates directly from integer grid indices.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Maps a world-space `[x, y, z]` position onto its containing grid cell.
    ///
    /// Non-positive or non-finite `cell_size` falls back safely to `1.0` and non-finite coordinates
    /// clamp to `0` so malformed transforms never panic.
    #[must_use]
    pub fn from_world_pos(pos: [f32; 3], cell_size: f32) -> Self {
        let size = if cell_size.is_finite() && cell_size > 0.0 {
            cell_size
        } else {
            1.0
        };
        let to_idx = |coord: f32| -> i32 {
            if !coord.is_finite() {
                return 0;
            }
            let floored = (coord / size).floor();
            if floored >= i32::MAX as f32 {
                i32::MAX
            } else if floored <= i32::MIN as f32 {
                i32::MIN
            } else {
                floored as i32
            }
        };
        Self {
            x: to_idx(pos[0]),
            y: to_idx(pos[1]),
            z: to_idx(pos[2]),
        }
    }

    /// Returns the world-space `(min_corner, max_corner)` AABB bounds of this cell.
    #[must_use]
    pub fn world_bounds(self, cell_size: f32) -> ([f32; 3], [f32; 3]) {
        let size = if cell_size.is_finite() && cell_size > 0.0 {
            cell_size
        } else {
            1.0
        };
        let min_x = self.x as f32 * size;
        let min_y = self.y as f32 * size;
        let min_z = self.z as f32 * size;
        (
            [min_x, min_y, min_z],
            [min_x + size, min_y + size, min_z + size],
        )
    }

    /// Computes Chebyshev (`L-infinity`) grid distance between two cells for cubic streaming rings.
    #[must_use]
    pub fn chebyshev_distance(self, other: Self) -> i32 {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        let dz = self.z.abs_diff(other.z);
        let max_d = dx.max(dy).max(dz);
        i32::try_from(max_d).unwrap_or(i32::MAX)
    }

    /// Returns the canonical chunk filename relative to a partitioned world directory (`cells/x_y_z.aechunk`).
    #[must_use]
    pub fn relative_chunk_path(self) -> String {
        format!("cells/{}_{}_{}.aechunk", self.x, self.y, self.z)
    }

    /// Returns this cell's minimum world-space corner in double precision (`f64`) to prevent
    /// multi-kilometer floating-point drift when offsetting camera-relative coordinates.
    #[must_use]
    pub fn world_origin_f64(self, cell_size: f32) -> [f64; 3] {
        let size = if cell_size.is_finite() && cell_size > 0.0 {
            f64::from(cell_size)
        } else {
            1.0
        };
        [
            f64::from(self.x) * size,
            f64::from(self.y) * size,
            f64::from(self.z) * size,
        ]
    }

    /// Converts a double-precision world position `[x, y, z]` into `(SpatialCellCoord, cell_local_f32)`.
    ///
    /// Keeps per-entity offsets bounded inside `[0.0, cell_size)` so large open worlds retain
    /// full single-precision mantissa accuracy far from world origin `(0, 0, 0)`.
    #[must_use]
    pub fn split_world_pos_f64(pos: [f64; 3], cell_size: f32) -> (Self, [f32; 3]) {
        let size = if cell_size.is_finite() && cell_size > 0.0 {
            f64::from(cell_size)
        } else {
            1.0
        };
        let to_idx_and_local = |v: f64| -> (i32, f32) {
            if !v.is_finite() {
                return (0, 0.0);
            }
            let floored = (v / size).floor();
            let idx = if floored >= f64::from(i32::MAX) {
                i32::MAX
            } else if floored <= f64::from(i32::MIN) {
                i32::MIN
            } else {
                floored as i32
            };
            let local = (v - f64::from(idx) * size) as f32;
            (idx, local)
        };
        let (x, lx) = to_idx_and_local(pos[0]);
        let (y, ly) = to_idx_and_local(pos[1]);
        let (z, lz) = to_idx_and_local(pos[2]);
        (Self { x, y, z }, [lx, ly, lz])
    }

    /// Converts a cell-local `f32` offset back into double-precision world coordinates.
    #[must_use]
    pub fn local_to_world_f64(self, local: [f32; 3], cell_size: f32) -> [f64; 3] {
        let origin = self.world_origin_f64(cell_size);
        [
            origin[0] + f64::from(local[0]),
            origin[1] + f64::from(local[1]),
            origin[2] + f64::from(local[2]),
        ]
    }

    /// Evaluates a cell-local coordinate relative to `origin_cell` using intermediate `f64` math
    /// before narrowing to `f32` for camera-relative GPU uniform and model matrix submission.
    #[must_use]
    pub fn relative_pos_to_origin(
        self,
        local: [f32; 3],
        origin_cell: Self,
        cell_size: f32,
    ) -> [f32; 3] {
        let size = if cell_size.is_finite() && cell_size > 0.0 {
            f64::from(cell_size)
        } else {
            1.0
        };
        let dx = i64::from(self.x) - i64::from(origin_cell.x);
        let dy = i64::from(self.y) - i64::from(origin_cell.y);
        let dz = i64::from(self.z) - i64::from(origin_cell.z);
        [
            ((dx as f64) * size + f64::from(local[0])) as f32,
            ((dy as f64) * size + f64::from(local[1])) as f32,
            ((dz as f64) * size + f64::from(local[2])) as f32,
        ]
    }
}

/// Self-contained spatial partition chunk storing all persistent entities inside one grid cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionCell {
    /// Grid coordinates covered by this chunk.
    pub coord: SpatialCellCoord,
    /// Monotonic edit revision incremented when entities inside this cell change.
    pub revision: u64,
    /// Entities residing inside this cell.
    pub entities: Vec<EntityRecord>,
}

impl PartitionCell {
    /// Creates a new empty cell at `coord`.
    #[must_use]
    pub fn new(coord: SpatialCellCoord) -> Self {
        Self {
            coord,
            revision: 1,
            entities: Vec::new(),
        }
    }

    /// Encodes and LZ4-compresses this chunk inside a CRC32-verified [`CHUNK_MAGIC`] envelope.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] or [`WorldIoError::Serialization`] on encoding failure.
    pub fn encode_chunk(&self) -> WorldResult<Vec<u8>> {
        let entity_stream = EntityRecord::encode_stream(&self.entities)?;
        let mut raw = Vec::with_capacity(20 + entity_stream.len());
        raw.extend_from_slice(&self.coord.x.to_le_bytes());
        raw.extend_from_slice(&self.coord.y.to_le_bytes());
        raw.extend_from_slice(&self.coord.z.to_le_bytes());
        raw.extend_from_slice(&self.revision.to_le_bytes());
        raw.extend_from_slice(&entity_stream);

        let compressed = lz4_flex::compress_prepend_size(&raw);
        Ok(encode_checksummed_envelope(
            CHUNK_MAGIC,
            CHUNK_FORMAT_VERSION,
            0,
            &compressed,
        ))
    }

    /// Validates, decompresses, and decodes a `.aechunk` buffer with optional component filtering.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on bad magic, CRC32 mismatch, truncated input, or LZ4 corruption.
    pub fn decode_chunk(bytes: &[u8], known_hashes: Option<&HashSet<u64>>) -> WorldResult<Self> {
        let envelope = decode_checksummed_envelope(bytes, CHUNK_MAGIC, CHUNK_FORMAT_VERSION)?;
        let decompressed = lz4_flex::decompress_size_prepended(envelope.payload)
            .map_err(|e| WorldIoError::Compression(e.to_string()))?;

        if decompressed.len() < 20 {
            return Err(WorldIoError::TruncatedData {
                context: "partition chunk header",
                expected: 20,
                actual: decompressed.len(),
            });
        }

        let x = i32::from_le_bytes([
            decompressed[0],
            decompressed[1],
            decompressed[2],
            decompressed[3],
        ]);
        let y = i32::from_le_bytes([
            decompressed[4],
            decompressed[5],
            decompressed[6],
            decompressed[7],
        ]);
        let z = i32::from_le_bytes([
            decompressed[8],
            decompressed[9],
            decompressed[10],
            decompressed[11],
        ]);
        let mut rev_buf = [0u8; 8];
        rev_buf.copy_from_slice(&decompressed[12..20]);
        let revision = u64::from_le_bytes(rev_buf);

        let entities = EntityRecord::decode_stream(&decompressed[20..], known_hashes)?;
        Ok(Self {
            coord: SpatialCellCoord::new(x, y, z),
            revision,
            entities,
        })
    }

    /// Encodes and LZ4-compresses many partition cells in parallel across `rayon` worker threads.
    ///
    /// # Errors
    /// Returns the first encoding error encountered across worker tasks.
    pub fn encode_chunks_parallel(
        cells: &[PartitionCell],
    ) -> WorldResult<Vec<(SpatialCellCoord, Vec<u8>)>> {
        cells
            .par_iter()
            .map(|cell| Ok((cell.coord, cell.encode_chunk()?)))
            .collect()
    }

    /// Decodes many `.aechunk` byte buffers in parallel across `rayon` worker threads.
    ///
    /// # Errors
    /// Returns the first validation or decompression error encountered.
    pub fn decode_chunks_parallel(
        raw_chunks: &[&[u8]],
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<Vec<PartitionCell>> {
        raw_chunks
            .par_iter()
            .map(|bytes| Self::decode_chunk(bytes, known_hashes))
            .collect()
    }
}

/// Output lists produced by [`CellStreamingPlanner::plan`] describing cells to stream in/out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamingPlan {
    /// Available cells within load radius not yet resident, ordered nearest-first.
    pub to_load: Vec<SpatialCellCoord>,
    /// Loaded cells beyond unload radius, ordered farthest-first.
    pub to_unload: Vec<SpatialCellCoord>,
}

/// Pure hysteresis planner determining which spatial chunks should load or unload around a focus point.
///
/// Keeping `unload_radius_cells >= load_radius_cells` prevents oscillation when the camera moves
/// repeatedly across a cell boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellStreamingPlanner {
    /// Side length of each cubic grid cell in world units.
    pub cell_size: f32,
    /// Chebyshev radius (in cells) within which absent cells are scheduled to load.
    pub load_radius_cells: i32,
    /// Chebyshev radius (in cells) beyond which loaded cells are scheduled to unload.
    pub unload_radius_cells: i32,
}

impl CellStreamingPlanner {
    /// Creates a planner enforcing positive `cell_size` and non-thrashing `unload >= load` radii.
    #[must_use]
    pub fn new(cell_size: f32, load_radius_cells: i32, unload_radius_cells: i32) -> Self {
        let clamped_load = load_radius_cells.max(0);
        let clamped_unload = unload_radius_cells.max(clamped_load);
        Self {
            cell_size: if cell_size.is_finite() && cell_size > 0.0 {
                cell_size
            } else {
                128.0
            },
            load_radius_cells: clamped_load,
            unload_radius_cells: clamped_unload,
        }
    }

    /// Computes cells to load and unload given the focus coordinate, existing world cells, and loaded set.
    #[must_use]
    pub fn plan(
        self,
        focus_pos: [f32; 3],
        available_cells: &HashSet<SpatialCellCoord>,
        loaded_cells: &HashSet<SpatialCellCoord>,
    ) -> StreamingPlan {
        let center = SpatialCellCoord::from_world_pos(focus_pos, self.cell_size);
        let mut to_load = Vec::new();
        let mut to_unload = Vec::new();

        for &coord in available_cells {
            if !loaded_cells.contains(&coord)
                && center.chebyshev_distance(coord) <= self.load_radius_cells
            {
                to_load.push(coord);
            }
        }

        for &coord in loaded_cells {
            if center.chebyshev_distance(coord) > self.unload_radius_cells {
                to_unload.push(coord);
            }
        }

        to_load.sort_by_key(|&c| (center.chebyshev_distance(c), c));
        to_unload.sort_by(|&a, &b| {
            center
                .chebyshev_distance(b)
                .cmp(&center.chebyshev_distance(a))
                .then_with(|| a.cmp(&b))
        });

        StreamingPlan { to_load, to_unload }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_record::ComponentRecord;
    use crate::guid::GuidGenerator;
    use ae_plugin_api::Position;

    #[test]
    fn test_spatial_cell_math_negative_and_boundary_cases() {
        assert_eq!(
            SpatialCellCoord::from_world_pos([-0.1, 0.0, 128.0], 128.0),
            SpatialCellCoord::new(-1, 0, 1)
        );
        assert_eq!(
            SpatialCellCoord::from_world_pos([f32::NAN, 10.0, f32::INFINITY], -5.0),
            SpatialCellCoord::new(0, 10, 0)
        );
        let (min_c, max_c) = SpatialCellCoord::new(-1, 2, 0).world_bounds(64.0);
        assert_eq!(min_c, [-64.0, 128.0, 0.0]);
        assert_eq!(max_c, [0.0, 192.0, 64.0]);
        assert_eq!(
            SpatialCellCoord::new(-1, 0, 2).relative_chunk_path(),
            "cells/-1_0_2.aechunk"
        );
    }

    #[test]
    fn test_parallel_chunk_encode_decode_roundtrip_and_corruption() {
        let mut ids = GuidGenerator::from_seed(9);
        let cells: Vec<PartitionCell> = (0..16)
            .map(|i| {
                let mut cell = PartitionCell::new(SpatialCellCoord::new(i, 0, -i));
                let mut rec = EntityRecord::new(ids.next_entity_guid(), None);
                rec.components.push(
                    ComponentRecord::from_value(
                        "Position",
                        1,
                        &Position::new(i as f32 * 10.0, 1.0, 2.0),
                    )
                    .expect("encode pos"),
                );
                cell.entities.push(rec);
                cell
            })
            .collect();

        let encoded = PartitionCell::encode_chunks_parallel(&cells).expect("parallel chunk encode");
        assert_eq!(encoded.len(), 16);

        let refs: Vec<&[u8]> = encoded.iter().map(|(_, b)| b.as_slice()).collect();
        let decoded =
            PartitionCell::decode_chunks_parallel(&refs, None).expect("parallel chunk decode");
        assert_eq!(decoded, cells);

        // Tamper one chunk payload and verify CRC failure
        let mut bad = encoded[0].1.clone();
        bad[30] ^= 0x55;
        assert!(matches!(
            PartitionCell::decode_chunk(&bad, None),
            Err(WorldIoError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_streaming_planner_hysteresis_and_ordering() {
        let available: HashSet<SpatialCellCoord> = [
            SpatialCellCoord::new(0, 0, 0),
            SpatialCellCoord::new(1, 0, 0),
            SpatialCellCoord::new(2, 0, 0),
            SpatialCellCoord::new(5, 0, 0),
        ]
        .into_iter()
        .collect();
        let loaded: HashSet<SpatialCellCoord> = [
            SpatialCellCoord::new(2, 0, 0),
            SpatialCellCoord::new(5, 0, 0),
        ]
        .into_iter()
        .collect();

        let planner = CellStreamingPlanner::new(100.0, 1, 2);
        let plan = planner.plan([10.0, 0.0, 0.0], &available, &loaded);
        assert_eq!(
            plan.to_load,
            vec![
                SpatialCellCoord::new(0, 0, 0),
                SpatialCellCoord::new(1, 0, 0),
            ]
        );
        // Cell (2,0,0) stays loaded inside hysteresis ring (distance 2 <= unload 2); (5,0,0) unloads.
        assert_eq!(plan.to_unload, vec![SpatialCellCoord::new(5, 0, 0)]);
    }

    #[test]
    fn test_multi_kilometer_floating_origin_cell_local_precision() {
        let cell_size = 128.0_f32;
        let far_world = [10_000_000.125_f64, -3.5_f64, 5_000_000.875_f64];
        let (cell, local) = SpatialCellCoord::split_world_pos_f64(far_world, cell_size);
        let roundtrip = cell.local_to_world_f64(local, cell_size);
        assert!((roundtrip[0] - far_world[0]).abs() < 1e-4);
        assert!((roundtrip[2] - far_world[2]).abs() < 1e-4);

        let origin = cell;
        let rel = cell.relative_pos_to_origin(local, origin, cell_size);
        assert!((rel[0] - local[0]).abs() < 1e-6);
    }
}