// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent binary identity types (`EntityGuid`, `AssetGuid`) and deterministic component
//! type hashing shared across the engine, ECS registry, world archive, and plugin boundary.

pub use ae_ecs::component_type_hash;
use core::fmt;
use serde::{Deserialize, Serialize};

/// Stable 128-bit persistent identifier assigned to serialized ECS entities.
///
/// Replaces fragile name-based references (`parent_name`) across world archives,
/// spatial partition chunks, prefabs, and runtime delta save slots.
/// A value of `0` ([`EntityGuid::NIL`]) denotes an unassigned or null reference.
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct EntityGuid(pub u128);

impl EntityGuid {
    /// Sentinel zero GUID representing an unassigned or missing entity reference.
    pub const NIL: Self = Self(0);

    /// Wraps a raw 128-bit unsigned integer into an `EntityGuid`.
    #[must_use]
    pub const fn from_raw(raw: u128) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw 128-bit integer value.
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Returns `true` when this identifier equals [`EntityGuid::NIL`].
    #[must_use]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }

    /// Encodes the identifier into a little-endian 16-byte array for portable binary I/O.
    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }

    /// Decodes an `EntityGuid` from a little-endian 16-byte array.
    #[must_use]
    pub const fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self(u128::from_le_bytes(bytes))
    }

    /// Parses either a 32-character hexadecimal string or a canonical `8-4-4-4-12` UUID string.
    ///
    /// # Errors
    /// Returns an error string when the input length, hyphen placement, or hex digits are invalid.
    pub fn parse_hex(input: &str) -> Result<Self, &'static str> {
        parse_u128_uuid_hex(input).map(Self)
    }
}

impl fmt::Display for EntityGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_u128_uuid(self.0, f)
    }
}

/// Stable 128-bit persistent identifier assigned to engine assets and resource entries.
///
/// Decouples world archives, prefabs, and packed bundles from mutable host filesystem paths
/// across desktop and mobile targets. A value of `0` ([`AssetGuid::NIL`]) denotes no asset.
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct AssetGuid(pub u128);

impl AssetGuid {
    /// Sentinel zero GUID representing a missing or unset asset reference.
    pub const NIL: Self = Self(0);

    /// Wraps a raw 128-bit unsigned integer into an `AssetGuid`.
    #[must_use]
    pub const fn from_raw(raw: u128) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw 128-bit integer value.
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Returns `true` when this identifier equals [`AssetGuid::NIL`].
    #[must_use]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }

    /// Encodes the asset identifier into a little-endian 16-byte array for portable binary I/O.
    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }

    /// Decodes an `AssetGuid` from a little-endian 16-byte array.
    #[must_use]
    pub const fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self(u128::from_le_bytes(bytes))
    }

    /// Parses either a 32-character hexadecimal string or a canonical `8-4-4-4-12` UUID string.
    ///
    /// # Errors
    /// Returns an error string when the input length, hyphen placement, or hex digits are invalid.
    pub fn parse_hex(input: &str) -> Result<Self, &'static str> {
        parse_u128_uuid_hex(input).map(Self)
    }
}

impl fmt::Display for AssetGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_u128_uuid(self.0, f)
    }
}

fn format_u128_uuid(value: u128, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let b = value.to_be_bytes();
    write!(
        f,
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7],
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

fn parse_u128_uuid_hex(input: &str) -> Result<u128, &'static str> {
    let trimmed = input.trim();
    let bytes = trimmed.as_bytes();
    let expect_hyphens = match bytes.len() {
        32 => false,
        36 => true,
        _ => return Err("GUID hex string must be 32 or 36 (8-4-4-4-12) characters"),
    };

    if expect_hyphens
        && (bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-')
    {
        return Err("Hyphenated GUID must use canonical 8-4-4-4-12 layout");
    }

    let mut out: u128 = 0;
    let mut digits = 0usize;
    for (idx, &ch) in bytes.iter().enumerate() {
        if expect_hyphens && matches!(idx, 8 | 13 | 18 | 23) {
            continue;
        }
        let nibble = match ch {
            b'0'..=b'9' => u128::from(ch - b'0'),
            b'a'..=b'f' => u128::from(ch - b'a' + 10),
            b'A'..=b'F' => u128::from(ch - b'A' + 10),
            _ => return Err("GUID contains non-hexadecimal character"),
        };
        out = (out << 4) | nibble;
        digits += 1;
    }

    if digits != 32 {
        return Err("GUID must contain exactly 32 hexadecimal digits");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_guid_nil_and_byte_roundtrip() {
        assert!(EntityGuid::NIL.is_nil());
        assert_eq!(EntityGuid::default(), EntityGuid::NIL);
        assert_eq!(EntityGuid::NIL.as_u128(), 0);

        let sample = EntityGuid::from_raw(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
        assert!(!sample.is_nil());
        assert_eq!(EntityGuid::from_le_bytes(sample.to_le_bytes()), sample);
        assert_eq!(
            EntityGuid::from_le_bytes(EntityGuid::from_raw(u128::MAX).to_le_bytes()).as_u128(),
            u128::MAX
        );
    }

    #[test]
    fn test_guid_hex_display_and_parse_invariants() {
        let entity = EntityGuid::from_raw(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);
        let formatted = entity.to_string();
        assert_eq!(formatted, "00112233-4455-6677-8899-aabbccddeeff");
        assert_eq!(EntityGuid::parse_hex(&formatted), Ok(entity));
        assert_eq!(
            EntityGuid::parse_hex("00112233445566778899AABBCCDDEEFF"),
            Ok(entity)
        );

        let asset = AssetGuid::from_raw(0xfedc_ba98_7654_3210_0123_4567_89ab_cdef);
        assert_eq!(AssetGuid::parse_hex(&asset.to_string()), Ok(asset));
        assert!(AssetGuid::NIL.is_nil());
    }

    #[test]
    fn test_guid_parse_rejects_malformed_inputs() {
        assert!(EntityGuid::parse_hex("").is_err());
        assert!(EntityGuid::parse_hex("00112233").is_err());
        assert!(EntityGuid::parse_hex("00112233-4455-6677-8899-aabbccddeefg").is_err());
        assert!(EntityGuid::parse_hex("001122334-455-6677-8899-aabbccddeeff").is_err());
        assert!(AssetGuid::parse_hex("00112233_4455_6677_8899_aabbccddeeff").is_err());
    }

    #[test]
    fn test_component_type_hash_determinism_and_sensitivity() {
        const HASH_POS: u64 = component_type_hash("Position");
        assert_eq!(HASH_POS, component_type_hash("Position"));
        assert_ne!(HASH_POS, component_type_hash("position"));
        assert_ne!(HASH_POS, component_type_hash("Rotation"));
        assert_ne!(component_type_hash(""), 0);
        assert_ne!(
            component_type_hash("EntityGuid"),
            component_type_hash("AssetGuid")
        );
    }
}