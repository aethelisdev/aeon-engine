// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Binary format signatures, CRC32 checksum envelopes, and bounds-checked stream readers.

use crate::error::{WorldIoError, WorldResult};

/// Magic signature identifying `.aeworld` binary world archives.
pub const WORLD_MAGIC: [u8; 8] = *b"AEWLD001";
/// Magic signature identifying `.aechunk` compressed spatial partition chunks.
pub const CHUNK_MAGIC: [u8; 8] = *b"AECHK001";
/// Magic signature identifying `.aepak` indexed asset containers.
pub const PAK_MAGIC: [u8; 8] = *b"AEPAK001";
/// Magic signature identifying `.aesave` runtime delta savegames.
pub const SAVE_MAGIC: [u8; 8] = *b"AESAV001";
/// Magic signature identifying cooked GPU mesh payloads.
pub const COOKED_MESH_MAGIC: [u8; 8] = *b"AECKM001";
/// Magic signature identifying cooked mipmapped GPU texture payloads.
pub const COOKED_TEXTURE_MAGIC: [u8; 8] = *b"AECKT001";
/// Magic signature identifying cooked decoded/PCM audio payloads.
pub const COOKED_AUDIO_MAGIC: [u8; 8] = *b"AECKA001";

/// Fixed byte size of the common [`BinaryEnvelopeHeader`] (`8 + 4 + 4 + 8 + 4`).
pub const ENVELOPE_HEADER_BYTES: usize = 28;

/// Common fixed-width header framing binary world, chunk, and save payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryEnvelopeHeader {
    /// 8-byte ASCII signature distinguishing the container format.
    pub magic: [u8; 8],
    /// Container format revision used for schema validation.
    pub format_version: u32,
    /// Format-specific bitflags (e.g., 2D vs 3D world, compression flags).
    pub flags: u32,
    /// Exact byte length of the following payload section.
    pub payload_len: u64,
    /// IEEE CRC32 checksum computed over the following `payload_len` bytes.
    pub payload_crc32: u32,
}

/// Decoded payload slice and validated metadata returned by [`decode_checksummed_envelope`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedEnvelope<'a> {
    /// Parsed header fields verified against `expected_magic` and `max_supported_version`.
    pub header: BinaryEnvelopeHeader,
    /// Validated payload slice whose CRC32 matched [`BinaryEnvelopeHeader::payload_crc32`].
    pub payload: &'a [u8],
}

/// Computes the IEEE CRC32 checksum of `data`.
#[must_use]
pub fn compute_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Verifies `data` against `expected_crc`, returning [`WorldIoError::ChecksumMismatch`] on failure.
pub fn verify_crc32(data: &[u8], expected_crc: u32) -> WorldResult<()> {
    let actual = compute_crc32(data);
    if actual != expected_crc {
        return Err(WorldIoError::ChecksumMismatch {
            expected: expected_crc,
            actual,
        });
    }
    Ok(())
}

/// Frames `payload` with a 28-byte little-endian [`BinaryEnvelopeHeader`] and CRC32 integrity tag.
#[must_use]
pub fn encode_checksummed_envelope(
    magic: [u8; 8],
    format_version: u32,
    flags: u32,
    payload: &[u8],
) -> Vec<u8> {
    let payload_crc32 = compute_crc32(payload);
    let mut out = Vec::with_capacity(ENVELOPE_HEADER_BYTES + payload.len());
    out.extend_from_slice(&magic);
    out.extend_from_slice(&format_version.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(&payload_crc32.to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Validates envelope header magic, version ceiling, exact payload length, and CRC32 checksum.
///
/// # Errors
/// Returns [`WorldIoError::TruncatedData`], [`WorldIoError::InvalidMagic`],
/// [`WorldIoError::UnsupportedVersion`], or [`WorldIoError::ChecksumMismatch`] when checks fail.
pub fn decode_checksummed_envelope<'a>(
    bytes: &'a [u8],
    expected_magic: [u8; 8],
    max_supported_version: u32,
) -> WorldResult<DecodedEnvelope<'a>> {
    let mut reader = ByteReader::new(bytes);
    let magic = reader.read_fixed_8("envelope magic")?;
    if magic != expected_magic {
        return Err(WorldIoError::InvalidMagic {
            expected: expected_magic,
            found: magic,
        });
    }

    let format_version = reader.read_u32("envelope format_version")?;
    if format_version == 0 || format_version > max_supported_version {
        return Err(WorldIoError::UnsupportedVersion {
            found: format_version,
            max_supported: max_supported_version,
        });
    }

    let flags = reader.read_u32("envelope flags")?;
    let payload_len = reader.read_u64("envelope payload_len")?;
    let payload_crc32 = reader.read_u32("envelope payload_crc32")?;
    let payload_usize = usize::try_from(payload_len).map_err(|_| {
        WorldIoError::InvalidLayout(format!(
            "Payload length {payload_len} exceeds addressable RAM"
        ))
    })?;
    let payload = reader.read_slice(payload_usize, "envelope payload")?;
    verify_crc32(payload, payload_crc32)?;

    Ok(DecodedEnvelope {
        header: BinaryEnvelopeHeader {
            magic,
            format_version,
            flags,
            payload_len,
            payload_crc32,
        },
        payload,
    })
}

/// Bounds-checked sequential little-endian slice reader that never panics on truncated buffers.
#[derive(Debug, Clone)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> ByteReader<'a> {
    /// Creates a new reader starting at offset `0` over `data`.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    /// Returns the number of unread bytes remaining in the input slice.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    /// Reads an exact `len`-byte subslice, advancing the cursor.
    ///
    /// # Errors
    /// Returns [`WorldIoError::TruncatedData`] if fewer than `len` bytes remain.
    pub fn read_slice(&mut self, len: usize, context: &'static str) -> WorldResult<&'a [u8]> {
        let rem = self.remaining();
        if rem < len {
            return Err(WorldIoError::TruncatedData {
                context,
                expected: len,
                actual: rem,
            });
        }
        let start = self.cursor;
        let end = start + len;
        self.cursor = end;
        Ok(&self.data[start..end])
    }

    /// Advances the cursor forward by `len` bytes without allocating or decoding the skipped payload.
    ///
    /// # Errors
    /// Returns [`WorldIoError::TruncatedData`] if fewer than `len` bytes remain.
    pub fn skip(&mut self, len: usize, context: &'static str) -> WorldResult<()> {
        let _ = self.read_slice(len, context)?;
        Ok(())
    }

    /// Reads an 8-byte fixed array (used for binary magic tags).
    pub fn read_fixed_8(&mut self, context: &'static str) -> WorldResult<[u8; 8]> {
        let slice = self.read_slice(8, context)?;
        let mut out = [0u8; 8];
        out.copy_from_slice(slice);
        Ok(out)
    }

    /// Reads a little-endian `u16`.
    pub fn read_u16(&mut self, context: &'static str) -> WorldResult<u16> {
        let slice = self.read_slice(2, context)?;
        Ok(u16::from_le_bytes([slice[0], slice[1]]))
    }

    /// Reads a little-endian `u32`.
    pub fn read_u32(&mut self, context: &'static str) -> WorldResult<u32> {
        let slice = self.read_slice(4, context)?;
        Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    /// Reads a little-endian `u64`.
    pub fn read_u64(&mut self, context: &'static str) -> WorldResult<u64> {
        let slice = self.read_slice(8, context)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(slice);
        Ok(u64::from_le_bytes(buf))
    }

    /// Reads a little-endian `u128`.
    pub fn read_u128(&mut self, context: &'static str) -> WorldResult<u128> {
        let slice = self.read_slice(16, context)?;
        let mut buf = [0u8; 16];
        buf.copy_from_slice(slice);
        Ok(u128::from_le_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_roundtrip_and_corruption_detection() {
        let payload = b"binary-world-payload";
        let framed = encode_checksummed_envelope(WORLD_MAGIC, 1, 0b101, payload);
        let decoded = decode_checksummed_envelope(&framed, WORLD_MAGIC, 1)
            .expect("valid envelope must decode cleanly");
        assert_eq!(decoded.header.format_version, 1);
        assert_eq!(decoded.header.flags, 0b101);
        assert_eq!(decoded.payload, payload);

        // Wrong magic must fail
        assert!(matches!(
            decode_checksummed_envelope(&framed, CHUNK_MAGIC, 1),
            Err(WorldIoError::InvalidMagic { .. })
        ));

        // Unsupported version must fail
        let future_ver = encode_checksummed_envelope(WORLD_MAGIC, 99, 0, payload);
        assert!(matches!(
            decode_checksummed_envelope(&future_ver, WORLD_MAGIC, 1),
            Err(WorldIoError::UnsupportedVersion { found: 99, .. })
        ));

        // Bit flip must trigger ChecksumMismatch
        let mut corrupted = framed.clone();
        if let Some(last) = corrupted.last_mut() {
            *last ^= 0xff;
        }
        assert!(matches!(
            decode_checksummed_envelope(&corrupted, WORLD_MAGIC, 1),
            Err(WorldIoError::ChecksumMismatch { .. })
        ));

        // Truncated buffer must return TruncatedData instead of panicking
        let truncated = &framed[..framed.len() - 5];
        assert!(matches!(
            decode_checksummed_envelope(truncated, WORLD_MAGIC, 1),
            Err(WorldIoError::TruncatedData { .. })
        ));
    }
}