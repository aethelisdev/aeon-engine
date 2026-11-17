// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Error types shared across binary world archives, chunks, paks, VFS, and savegames.

use thiserror::Error;

/// Canonical result type returned by all `ae_world` serialization, archive, and VFS APIs.
pub type WorldResult<T> = Result<T, WorldIoError>;

/// Exhaustive error enum describing validation, I/O, compression, and lookup failures
/// when reading or writing binary worlds, partition chunks, asset paks, or save slots.
#[derive(Debug, Error)]
pub enum WorldIoError {
    /// Underlying operating-system or stream I/O error.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),

    /// Envelope header did not start with the expected 8-byte format signature.
    #[error("Invalid binary magic: expected {expected:?}, found {found:?}")]
    InvalidMagic {
        /// Expected 8-byte ASCII magic signature.
        expected: [u8; 8],
        /// Actual 8-byte signature found in the input stream.
        found: [u8; 8],
    },

    /// Binary format version is newer than what this reader supports.
    #[error("Unsupported binary format version {found} (max supported {max_supported})")]
    UnsupportedVersion {
        /// Format version stored in the file header.
        found: u32,
        /// Highest format version known to this build.
        max_supported: u32,
    },

    /// CRC32 integrity verification failed due to file corruption or truncation.
    #[error("CRC32 checksum mismatch: expected 0x{expected:08x}, computed 0x{actual:08x}")]
    ChecksumMismatch {
        /// Checksum recorded in the archive header or TOC.
        expected: u32,
        /// Checksum computed over the loaded bytes.
        actual: u32,
    },

    /// Input buffer ended prematurely before decoding the expected structure.
    #[error("Truncated binary data in {context}: needed {expected} bytes, got {actual}")]
    TruncatedData {
        /// Human-readable field or section being decoded.
        context: &'static str,
        /// Minimum number of bytes required.
        expected: usize,
        /// Actual remaining byte count.
        actual: usize,
    },

    /// Payload failed `postcard` serialization or deserialization.
    #[error("Binary codec failure: {0}")]
    Serialization(String),

    /// LZ4 compression or decompression encountered malformed or oversized data.
    #[error("Block compression failure: {0}")]
    Compression(String),

    /// Structural constraint violation (e.g. non-positive cell size, path traversal, invalid index).
    #[error("Invalid binary layout: {0}")]
    InvalidLayout(String),

    /// Requested asset, chunk, or save slot was not found.
    #[error("Entry not found: {0}")]
    EntryNotFound(String),
}