// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware Layer Cache & Invalidation Boundary Buffer Slicing
//!
//! Manages GPU instanced quad buffer allocations, persistent boundary slice tracking,
//! and partial GPU re-uploads (`queue.write_buffer`) for isolated sub-trees.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use crate::command::BoundarySlice;
use crate::quad::QuadInstance;
use iris_core::id::WidgetId;
use std::collections::{HashMap, HashSet};

/// Hardware layer cache managing boundary-level GPU instance buffer slices and partial uploads.
#[derive(Debug, Default)]
pub struct BoundaryLayerCache {
    /// Cached slices mapped by boundary WidgetId.
    pub slices: HashMap<WidgetId, Vec<BoundarySlice>>,
    /// Total number of quad instances allocated in the GPU buffer.
    pub total_instances: u32,
    /// Number of partial uploads executed since creation.
    pub partial_upload_count: u64,
    /// Number of full uploads executed since creation.
    pub full_upload_count: u64,
}

/// Target GPU buffer and allocation parameters for instance quad caching.
///
/// Encapsulates the hardware device, command queue, optional buffer handle,
/// and existing capacity to eliminate excessive parameter lists while
/// adhering strictly to Rule 11.2.
pub struct InstanceBufferTarget<'a> {
    /// Active WGPU logical device reference used to allocate buffers upon capacity expansion.
    pub device: &'a wgpu::Device,
    /// Active WGPU execution queue used to submit direct memory writes (`queue.write_buffer`).
    pub queue: &'a wgpu::Queue,
    /// Mutable reference to the optional hardware instance buffer.
    pub buffer: &'a mut Option<wgpu::Buffer>,
    /// Mutable reference to the recorded instance capacity in element count.
    pub capacity: &'a mut usize,
}

impl BoundaryLayerCache {
    /// Creates a new, empty [`BoundaryLayerCache`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all tracked boundary slices and resets instance counts.
    pub fn clear(&mut self) {
        self.slices.clear();
        self.total_instances = 0;
    }

    /// Synchronizes the GPU instanced quad buffer with CPU quad data, performing partial
    /// buffer updates when valid dirty boundaries are provided and slice layouts are stable.
    ///
    /// Returns `true` if a partial update was performed, or `false` if a full buffer upload occurred.
    pub fn update_instance_buffer(
        &mut self,
        target: InstanceBufferTarget<'_>,
        quads: &[QuadInstance],
        boundary_slices: &[(WidgetId, BoundarySlice)],
        dirty_boundaries: Option<&HashSet<WidgetId>>,
    ) -> bool {
        if quads.is_empty() {
            return false;
        }

        let needed_capacity = quads.len();
        let buffer_needs_realloc = target.buffer.is_none() || *target.capacity < needed_capacity;

        // Check if slice layout or total quad count changed compared to cache
        let slice_layout_matches =
            self.total_instances == quads.len() as u32 && self.matches_slices(boundary_slices);

        // If buffer was reallocated or slice layout shifted, we must perform a full upload
        if buffer_needs_realloc || !slice_layout_matches || dirty_boundaries.is_none() {
            if buffer_needs_realloc {
                let new_capacity = (needed_capacity * 2).max(128);
                let new_buf = target.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Iris UI Instanced Quad Buffer"),
                    size: (new_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                *target.buffer = Some(new_buf);
                *target.capacity = new_capacity;
            }

            if let Some(buf) = target.buffer.as_ref() {
                target
                    .queue
                    .write_buffer(buf, 0, bytemuck::cast_slice(quads));
            }

            self.rebuild_slices(boundary_slices, quads.len() as u32);
            self.full_upload_count += 1;
            return false;
        }

        // We have a stable buffer and valid dirty_boundaries set: perform partial slice upload
        let dirty_set = match dirty_boundaries {
            Some(set) => set,
            None => return false,
        };

        if dirty_set.is_empty() {
            // Zero dirty boundaries: zero bytes transferred!
            return true;
        }

        if let Some(buf) = target.buffer.as_ref() {
            let instance_size = std::mem::size_of::<QuadInstance>();
            for &dirty_id in dirty_set {
                if let Some(slices) = self.slices.get(&dirty_id) {
                    for slice in slices {
                        if slice.count > 0 {
                            let start = slice.start as usize;
                            let end = (slice.start + slice.count) as usize;
                            if end <= quads.len() {
                                let byte_offset = (start * instance_size) as u64;
                                let slice_quads = &quads[start..end];
                                target.queue.write_buffer(
                                    buf,
                                    byte_offset,
                                    bytemuck::cast_slice(slice_quads),
                                );
                            }
                        }
                    }
                }
            }
        }

        self.partial_upload_count += 1;
        true
    }

    /// Tests whether the incoming boundary slice list matches the currently cached slice structure.
    fn matches_slices(&self, boundary_slices: &[(WidgetId, BoundarySlice)]) -> bool {
        let mut count_by_id: HashMap<WidgetId, Vec<BoundarySlice>> = HashMap::new();
        for &(id, slice) in boundary_slices {
            count_by_id.entry(id).or_default().push(slice);
        }
        if count_by_id.len() != self.slices.len() {
            return false;
        }
        for (id, slices) in &count_by_id {
            if let Some(cached_slices) = self.slices.get(id) {
                if cached_slices != slices {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Rebuilds cached slice index from the compiler's output boundary slices.
    fn rebuild_slices(&mut self, boundary_slices: &[(WidgetId, BoundarySlice)], total: u32) {
        self.slices.clear();
        for &(id, slice) in boundary_slices {
            self.slices.entry(id).or_default().push(slice);
        }
        self.total_instances = total;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_cache_empty_quads() {
        let cache = BoundaryLayerCache::new();
        assert_eq!(cache.total_instances, 0);
        assert_eq!(cache.partial_upload_count, 0);
        assert_eq!(cache.full_upload_count, 0);
    }

    #[test]
    fn test_layer_cache_slice_rebuild_and_match() {
        let mut cache = BoundaryLayerCache::new();
        let mut tree = iris_core::tree::UiTree::new();
        let b1 = tree.create_node();
        let b2 = tree.create_node();

        let slices = vec![
            (
                b1,
                BoundarySlice {
                    start: 0,
                    count: 10,
                },
            ),
            (
                b2,
                BoundarySlice {
                    start: 10,
                    count: 20,
                },
            ),
        ];

        cache.rebuild_slices(&slices, 30);
        assert_eq!(cache.total_instances, 30);
        assert!(cache.matches_slices(&slices));

        // Different total count should not match
        let shifted_slices = vec![
            (
                b1,
                BoundarySlice {
                    start: 0,
                    count: 10,
                },
            ),
            (
                b2,
                BoundarySlice {
                    start: 10,
                    count: 25,
                },
            ),
        ];
        assert!(!cache.matches_slices(&shifted_slices));
    }
}