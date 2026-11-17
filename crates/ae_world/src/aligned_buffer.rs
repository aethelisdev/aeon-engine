// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 16-byte aligned memory buffer enabling zero-copy, panic-free `bytemuck` casts
//! for cooked GPU vertex, index, and mipmap payloads under `#![forbid(unsafe_code)]`.

use bytemuck::{Pod, PodCastError, Zeroable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Minimum guaranteed byte alignment of [`AlignedBuffer`] backing storage.
pub const BUFFER_ALIGNMENT_BYTES: usize = 16;

/// Single 16-byte aligned storage word guaranteed to contain zero internal padding.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
struct Align16([u8; 16]);

/// Byte buffer backed by 16-byte aligned storage words for direct GPU buffer uploads.
///
/// In `100%` safe Rust (`unsafe_code = "forbid"`), decompressing raw mesh or texture bytes
/// into an `AlignedBuffer` guarantees the backing byte slice address is a multiple of `16`,
/// allowing [`AlignedBuffer::try_cast_slice`] to reinterpret vertex/index arrays without extra copies.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AlignedBuffer {
    words: Vec<Align16>,
    byte_len: usize,
}

impl AlignedBuffer {
    /// Creates an empty 16-byte aligned buffer of length `0`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            words: Vec::new(),
            byte_len: 0,
        }
    }

    /// Allocates a zero-initialized 16-byte aligned buffer covering `byte_len` valid bytes.
    #[must_use]
    pub fn with_zeroed_len(byte_len: usize) -> Self {
        let word_count = byte_len.div_ceil(BUFFER_ALIGNMENT_BYTES);
        Self {
            words: vec![Align16::zeroed(); word_count],
            byte_len,
        }
    }

    /// Copies `bytes` into a freshly allocated 16-byte aligned buffer.
    #[must_use]
    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut out = Self::with_zeroed_len(bytes.len());
        out.as_mut_bytes().copy_from_slice(bytes);
        out
    }

    /// Returns the number of logical bytes stored in this buffer (excluding alignment tail padding).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.byte_len
    }

    /// Returns `true` when the logical byte length is `0`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.byte_len == 0
    }

    /// Returns an immutable view over the valid logical bytes (`0..self.len()`).
    ///
    /// When non-empty, the returned slice pointer is aligned to at least [`BUFFER_ALIGNMENT_BYTES`].
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        let raw: &[u8] = bytemuck::cast_slice(&self.words);
        &raw[..self.byte_len]
    }

    /// Returns a mutable view over the valid logical bytes (`0..self.len()`) for direct in-place
    /// decompression or streaming reads.
    #[must_use]
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        let raw: &mut [u8] = bytemuck::cast_slice_mut(&mut self.words);
        &mut raw[..self.byte_len]
    }

    /// Converts the underlying bytes into an owned `Vec<u8>`.
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    /// Safely reinterprets the valid bytes as a slice of `T` without allocation or copying.
    ///
    /// Works reliably for all GPU vertex/index/uniform types with alignment `<= 16`.
    ///
    /// # Errors
    /// Returns [`PodCastError::SizeMismatch`] if `self.len()` is not an exact multiple of
    /// `core::mem::size_of::<T>()`, or [`PodCastError::TargetAlignmentGreaterAndInputNotAligned`]
    /// if `T` requires alignment greater than `16` and the allocation happens not to satisfy it.
    pub fn try_cast_slice<T: Pod>(&self) -> Result<&[T], PodCastError> {
        bytemuck::try_cast_slice(self.as_bytes())
    }
}

impl AsRef<[u8]> for AlignedBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Serialize for AlignedBuffer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> Deserialize<'de> for AlignedBuffer {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        Ok(Self::from_slice(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
    struct TestGpuVertex {
        pos: [f32; 3],
        pad: f32,
        uv: [f32; 2],
        color: [f32; 2],
    }

    #[test]
    fn test_aligned_buffer_satisfies_16_byte_alignment_and_zero_copy_pod_cast() {
        let vertices = [
            TestGpuVertex {
                pos: [1.0, 2.0, 3.0],
                pad: 0.0,
                uv: [0.25, 0.75],
                color: [1.0, 0.5],
            },
            TestGpuVertex {
                pos: [-4.5, 0.0, 9.25],
                pad: 1.0,
                uv: [0.5, 0.5],
                color: [0.2, 0.8],
            },
        ];
        let raw_bytes: &[u8] = bytemuck::cast_slice(&vertices);
        let buf = AlignedBuffer::from_slice(raw_bytes);

        assert_eq!(buf.len(), raw_bytes.len());
        assert!(!buf.is_empty());
        assert_eq!(
            (buf.as_bytes().as_ptr() as usize) % BUFFER_ALIGNMENT_BYTES,
            0
        );

        let cast: &[TestGpuVertex] = buf
            .try_cast_slice()
            .expect("aligned vertex payload casts cleanly");
        assert_eq!(cast, &vertices);
    }

    #[test]
    fn test_aligned_buffer_empty_and_size_mismatch_return_cleanly() {
        let empty = AlignedBuffer::new();
        assert!(empty.is_empty());
        let empty_f32: &[f32] = empty.try_cast_slice().expect("empty casts cleanly");
        assert!(empty_f32.is_empty());

        let unaligned_len = AlignedBuffer::from_slice(&[1u8, 2, 3]);
        assert_eq!(
            (unaligned_len.as_bytes().as_ptr() as usize) % BUFFER_ALIGNMENT_BYTES,
            0
        );
        assert!(matches!(
            unaligned_len.try_cast_slice::<u32>(),
            Err(PodCastError::OutputSliceWouldHaveSlop)
        ));
    }

    #[test]
    fn test_aligned_buffer_postcard_roundtrip_retains_16_byte_alignment() {
        let src = AlignedBuffer::from_slice(&[0xAAu8; 37]);
        let encoded = postcard::to_stdvec(&src).expect("encode aligned buffer");
        let decoded: AlignedBuffer = postcard::from_bytes(&encoded).expect("decode aligned buffer");
        assert_eq!(decoded, src);
        assert_eq!(
            (decoded.as_bytes().as_ptr() as usize) % BUFFER_ALIGNMENT_BYTES,
            0
        );
    }
}