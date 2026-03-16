// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Map stream read/write for AFF4 containers.
//!
//! A map stream describes a logical-to-physical mapping from a contiguous
//! virtual address space (the image) to chunks stored in bevies. This is
//! used for both physical disk images (one contiguous map) and logical
//! containers with multiple file images.
//!
//! Each map has:
//! - A **map data file** (ZIP member): packed array of `MapEntry`,
//!   each 28 bytes, little-endian
//! - A **map idx file** (ZIP member): newline-separated list of target URNs,
//!   where the line number corresponds to the `target_id` in map entries

use crate::error::{Aff4Error, Aff4Result};
use crate::hashing::hash_hex;
use crate::types::{Aff4HashAlgorithm, MAP_ENTRY_SIZE};

// ─── Map Entry ───────────────────────────────────────────────────────────────

/// A single entry in a map data file.
///
/// Layout (28 bytes, little-endian):
/// ```text
/// mapped_offset:   u64  — virtual offset in the image stream
/// length:          u64  — length of this mapped region
/// target_offset:   u64  — byte offset within the target stream (bevy)
/// target_id:       u32  — index into the map idx file (target URN lookup)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapEntry {
    /// Virtual offset in the image/file stream.
    pub mapped_offset: u64,
    /// Length of this region in bytes.
    pub length: u64,
    /// Offset within the target bevy stream.
    pub target_offset: u64,
    /// Index into the map idx table (which bevy/stream this points to).
    pub target_id: u32,
}

impl MapEntry {
    /// Serialize to 28 bytes (little-endian).
    pub fn to_bytes(&self) -> [u8; MAP_ENTRY_SIZE] {
        let mut buf = [0u8; MAP_ENTRY_SIZE];
        buf[0..8].copy_from_slice(&self.mapped_offset.to_le_bytes());
        buf[8..16].copy_from_slice(&self.length.to_le_bytes());
        buf[16..24].copy_from_slice(&self.target_offset.to_le_bytes());
        buf[24..28].copy_from_slice(&self.target_id.to_le_bytes());
        buf
    }

    /// Deserialize from 28 bytes (little-endian).
    pub fn from_bytes(data: &[u8; MAP_ENTRY_SIZE]) -> Self {
        Self {
            mapped_offset: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            length: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            target_offset: u64::from_le_bytes(data[16..24].try_into().unwrap()),
            target_id: u32::from_le_bytes(data[24..28].try_into().unwrap()),
        }
    }
}

// ─── Map Writer ──────────────────────────────────────────────────────────────

/// Builds map data and idx buffers for an AFF4 image stream.
pub struct MapWriter {
    /// Map entries in order.
    entries: Vec<MapEntry>,
    /// Target URN list (idx file content).
    targets: Vec<String>,
}

impl MapWriter {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            targets: Vec::new(),
        }
    }

    /// Register a target URN and return its target_id.
    ///
    /// If the URN is already registered, returns the existing id.
    pub fn register_target(&mut self, urn: &str) -> u32 {
        if let Some(pos) = self.targets.iter().position(|t| t == urn) {
            return pos as u32;
        }
        let id = self.targets.len() as u32;
        self.targets.push(urn.to_string());
        id
    }

    /// Add a map entry that maps a virtual region to a target stream.
    pub fn add_entry(
        &mut self,
        mapped_offset: u64,
        length: u64,
        target_offset: u64,
        target_id: u32,
    ) {
        self.entries.push(MapEntry {
            mapped_offset,
            length,
            target_offset,
            target_id,
        });
    }

    /// Add a contiguous mapping for a bevy.
    ///
    /// Maps `bevy_chunks * chunk_size` bytes starting at `image_offset`
    /// to the bevy identified by `bevy_urn`.
    pub fn add_bevy_mapping(
        &mut self,
        image_offset: u64,
        bevy_urn: &str,
        bevy_size: u64,
    ) {
        let target_id = self.register_target(bevy_urn);
        self.add_entry(image_offset, bevy_size, 0, target_id);
    }

    /// Total number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Consume the writer and produce the map data, idx, and hashes.
    pub fn finish(self) -> MapWriteResult {
        // Serialize map entries (map data file)
        let mut map_data = Vec::with_capacity(self.entries.len() * MAP_ENTRY_SIZE);
        for entry in &self.entries {
            map_data.extend_from_slice(&entry.to_bytes());
        }

        // Serialize target list (map idx file)
        let map_idx = self.targets.join("\n");

        MapWriteResult {
            map_data,
            map_idx: map_idx.into_bytes(),
            entries: self.entries,
            targets: self.targets,
        }
    }

    /// Compute AFF4 map hashes.
    ///
    /// Returns (map_point_hash, map_idx_hash, map_path_hash) as hex strings.
    pub fn compute_hashes(&self, algorithm: Aff4HashAlgorithm) -> (String, String, String) {
        // map point hash = hash of the serialized map entries
        let mut map_data = Vec::with_capacity(self.entries.len() * MAP_ENTRY_SIZE);
        for entry in &self.entries {
            map_data.extend_from_slice(&entry.to_bytes());
        }
        let map_point_hash = hash_hex(&map_data, algorithm);

        // map idx hash = hash of the idx file content
        let map_idx = self.targets.join("\n");
        let map_idx_hash = hash_hex(map_idx.as_bytes(), algorithm);

        // map path hash = hash of the map stream URN (caller typically sets this)
        // For now, return empty — the writer fills this with the stream URN hash
        let map_path_hash = String::new();

        (map_point_hash, map_idx_hash, map_path_hash)
    }
}

impl Default for MapWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of finishing a map write.
pub struct MapWriteResult {
    /// Serialized map entries (28 bytes each).
    pub map_data: Vec<u8>,
    /// Serialized target URN list (newline-separated, UTF-8).
    pub map_idx: Vec<u8>,
    /// Parsed entries (for further processing).
    pub entries: Vec<MapEntry>,
    /// Target URN list.
    pub targets: Vec<String>,
}

// ─── Map Reader ──────────────────────────────────────────────────────────────

/// Parsed map for reading and resolving virtual offsets.
#[derive(Clone)]
pub struct MapReader {
    /// Sorted map entries.
    entries: Vec<MapEntry>,
    /// Target URN list.
    targets: Vec<String>,
}

impl MapReader {
    /// Parse a map from raw data and idx bytes.
    pub fn from_data(map_data: &[u8], map_idx: &[u8]) -> Aff4Result<Self> {
        // Parse entries
        if map_data.len() % MAP_ENTRY_SIZE != 0 {
            return Err(Aff4Error::InvalidMapEntry(
                "map data length not a multiple of entry size".to_string(),
            ));
        }

        let count = map_data.len() / MAP_ENTRY_SIZE;
        let mut entries = Vec::with_capacity(count);

        for i in 0..count {
            let start = i * MAP_ENTRY_SIZE;
            let chunk: [u8; MAP_ENTRY_SIZE] = map_data[start..start + MAP_ENTRY_SIZE]
                .try_into()
                .unwrap();
            entries.push(MapEntry::from_bytes(&chunk));
        }

        // Sort by mapped_offset for binary search
        entries.sort_by_key(|e| e.mapped_offset);

        // Parse target URN list
        let idx_str = std::str::from_utf8(map_idx)
            .map_err(|e| Aff4Error::InvalidMapEntry(format!("invalid UTF-8 in map idx: {}", e)))?;
        let targets: Vec<String> = idx_str
            .lines()
            .map(|l| l.to_string())
            .collect();

        Ok(Self { entries, targets })
    }

    /// Number of map entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Resolve a virtual offset to a target URN and target offset.
    ///
    /// Returns `(target_urn, target_offset, available_length)` or None
    /// if the offset is not mapped.
    pub fn resolve(&self, virtual_offset: u64) -> Option<(&str, u64, u64)> {
        // Binary search for the entry containing this offset
        let idx = match self
            .entries
            .binary_search_by_key(&virtual_offset, |e| e.mapped_offset)
        {
            Ok(i) => i,
            Err(0) => return None,
            Err(i) => i - 1,
        };

        let entry = &self.entries[idx];
        let end = entry.mapped_offset + entry.length;

        if virtual_offset >= end {
            return None;
        }

        let delta = virtual_offset - entry.mapped_offset;
        let target_urn = self.targets.get(entry.target_id as usize)?;
        let target_offset = entry.target_offset + delta;
        let available = entry.length - delta;

        Some((target_urn, target_offset, available))
    }

    /// Get the target URN for a given target_id.
    pub fn target_urn(&self, target_id: u32) -> Option<&str> {
        self.targets.get(target_id as usize).map(|s| s.as_str())
    }

    /// Get all entries.
    pub fn entries(&self) -> &[MapEntry] {
        &self.entries
    }

    /// Get total mapped size (max mapped_offset + length across all entries).
    pub fn total_mapped_size(&self) -> u64 {
        self.entries
            .iter()
            .map(|e| e.mapped_offset + e.length)
            .max()
            .unwrap_or(0)
    }

    /// Verify map hashes against expected values.
    pub fn verify_hashes(
        &self,
        algorithm: Aff4HashAlgorithm,
        expected_point_hash: &str,
        expected_idx_hash: &str,
    ) -> (bool, bool) {
        // Reserialize for hashing
        let mut map_data = Vec::with_capacity(self.entries.len() * MAP_ENTRY_SIZE);
        for entry in &self.entries {
            map_data.extend_from_slice(&entry.to_bytes());
        }
        let actual_point = hash_hex(&map_data, algorithm);

        let idx_content = self.targets.join("\n");
        let actual_idx = hash_hex(idx_content.as_bytes(), algorithm);

        (
            actual_point == expected_point_hash,
            actual_idx == expected_idx_hash,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_entry_roundtrip() {
        let entry = MapEntry {
            mapped_offset: 0x0000_1000,
            length: 0x0000_8000,
            target_offset: 0,
            target_id: 0,
        };
        let bytes = entry.to_bytes();
        let parsed = MapEntry::from_bytes(&bytes);
        assert_eq!(entry, parsed);
    }

    #[test]
    fn test_map_write_read() {
        let mut writer = MapWriter::new();

        let bevy0 = writer.register_target("aff4://vol-1/image/00000000");
        let bevy1 = writer.register_target("aff4://vol-1/image/00000001");

        writer.add_entry(0, 32 * 1024 * 1024, 0, bevy0);
        writer.add_entry(32 * 1024 * 1024, 32 * 1024 * 1024, 0, bevy1);

        assert_eq!(writer.entry_count(), 2);

        let result = writer.finish();
        let reader = MapReader::from_data(&result.map_data, &result.map_idx).unwrap();

        assert_eq!(reader.entry_count(), 2);
        assert_eq!(reader.total_mapped_size(), 64 * 1024 * 1024);
    }

    #[test]
    fn test_map_resolve() {
        let mut writer = MapWriter::new();

        let bevy0 = writer.register_target("aff4://vol/img/00000000");
        let bevy1 = writer.register_target("aff4://vol/img/00000001");

        let chunk_size: u64 = 32768;
        let bevy_size: u64 = chunk_size * 1024; // 32 MiB

        writer.add_entry(0, bevy_size, 0, bevy0);
        writer.add_entry(bevy_size, bevy_size, 0, bevy1);

        let result = writer.finish();
        let reader = MapReader::from_data(&result.map_data, &result.map_idx).unwrap();

        // Offset 0 → bevy0
        let (urn, offset, avail) = reader.resolve(0).unwrap();
        assert_eq!(urn, "aff4://vol/img/00000000");
        assert_eq!(offset, 0);
        assert_eq!(avail, bevy_size);

        // Offset in middle of bevy0
        let (urn, offset, _) = reader.resolve(chunk_size * 500).unwrap();
        assert_eq!(urn, "aff4://vol/img/00000000");
        assert_eq!(offset, chunk_size * 500);

        // Offset at start of bevy1
        let (urn, offset, _) = reader.resolve(bevy_size).unwrap();
        assert_eq!(urn, "aff4://vol/img/00000001");
        assert_eq!(offset, 0);

        // Offset beyond all entries
        assert!(reader.resolve(bevy_size * 3).is_none());
    }

    #[test]
    fn test_map_bevy_mapping() {
        let mut writer = MapWriter::new();

        writer.add_bevy_mapping(0, "aff4://v/i/00000000", 33554432);
        writer.add_bevy_mapping(33554432, "aff4://v/i/00000001", 33554432);

        let result = writer.finish();
        let reader = MapReader::from_data(&result.map_data, &result.map_idx).unwrap();

        assert_eq!(reader.total_mapped_size(), 67108864);
    }

    #[test]
    fn test_map_hash_verification() {
        let mut writer = MapWriter::new();
        writer.add_bevy_mapping(0, "aff4://vol/img/00000000", 1024);

        let (point_hash, idx_hash, _) = writer.compute_hashes(Aff4HashAlgorithm::Sha256);

        let result = writer.finish();
        let reader = MapReader::from_data(&result.map_data, &result.map_idx).unwrap();

        let (point_ok, idx_ok) =
            reader.verify_hashes(Aff4HashAlgorithm::Sha256, &point_hash, &idx_hash);
        assert!(point_ok);
        assert!(idx_ok);
    }
}
