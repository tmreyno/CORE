// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AFF4 container reader for browsing, extracting, and verifying.
//!
//! Opens `.aff4` ZIP64 containers, parses RDF metadata, discovers image
//! streams, and provides random-access read + hash verification.

use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use zip::ZipArchive;

use crate::bevy::BevyIndexEntry;
use crate::error::{Aff4Error, Aff4Result};
use crate::hashing::StreamHasher;
use crate::helpers::emit_progress;
use crate::map::MapReader;
use crate::rdf::{self, RdfGraph};
use crate::types::*;
use crate::uri;

// ─── AFF4 Reader ─────────────────────────────────────────────────────────────

/// Reader for AFF4 containers (both Standard and Logical).
///
/// ## Usage
///
/// ```rust,no_run
/// use ffx_aff4::reader::Aff4Reader;
///
/// let mut reader = Aff4Reader::open("image.aff4").unwrap();
/// let info = reader.info();
/// println!("Volume: {}", info.volume_urn);
///
/// // Read data from the first stream
/// if let Some(stream) = info.streams.first() {
///     let mut buf = vec![0u8; 512];
///     let n = reader.read_at(&stream.urn, 0, &mut buf).unwrap();
/// }
/// ```
pub struct Aff4Reader<R: Read + Seek> {
    /// The ZIP archive.
    archive: ZipArchive<R>,

    /// Volume URN (from container.description).
    volume_urn: String,

    /// Parsed AFF4 version.
    version: Aff4Version,

    /// Tool that created the container.
    tool: String,

    /// RDF metadata graph.
    rdf: RdfGraph,

    /// Discovered image streams.
    streams: Vec<Aff4StreamInfo>,

    /// Case metadata.
    case_number: String,
    evidence_number: String,
    examiner: String,
    description: String,

    /// Parsed map readers, keyed by stream URN.
    maps: HashMap<String, MapReader>,

    /// Cached bevy index data, keyed by bevy ZIP path.
    bevy_indexes: HashMap<String, Vec<BevyIndexEntry>>,
}

impl Aff4Reader<std::fs::File> {
    /// Open an AFF4 container from a file path.
    pub fn open<P: AsRef<Path>>(path: P) -> Aff4Result<Self> {
        let file = std::fs::File::open(path.as_ref()).map_err(Aff4Error::Io)?;
        Self::from_reader(file)
    }
}

impl<R: Read + Seek> Aff4Reader<R> {
    /// Open an AFF4 container from any reader.
    pub fn from_reader(reader: R) -> Aff4Result<Self> {
        let mut archive = ZipArchive::new(reader).map_err(Aff4Error::Zip)?;

        // ── 1. Read container.description ────────────────────────────────────
        let volume_urn = {
            let mut entry = archive
                .by_name(CONTAINER_DESCRIPTION)
                .map_err(|_| Aff4Error::MissingMember(CONTAINER_DESCRIPTION.to_string()))?;
            let mut content = String::new();
            entry.read_to_string(&mut content).map_err(Aff4Error::Io)?;
            content.trim().to_string()
        };

        if !volume_urn.starts_with("aff4://") {
            return Err(Aff4Error::InvalidContainer(
                "container.description does not contain a valid AFF4 URN".to_string(),
            ));
        }

        // ── 2. Read version.txt ──────────────────────────────────────────────
        let (version, tool) = {
            let mut entry = archive
                .by_name(VERSION_TXT)
                .map_err(|_| Aff4Error::MissingMember(VERSION_TXT.to_string()))?;
            let mut content = String::new();
            entry.read_to_string(&mut content).map_err(Aff4Error::Io)?;
            parse_version_txt(&content)
        };

        // ── 3. Read information.turtle ───────────────────────────────────────
        let rdf = {
            let mut entry = archive
                .by_name(INFORMATION_TURTLE)
                .map_err(|_| Aff4Error::MissingMember(INFORMATION_TURTLE.to_string()))?;
            let mut content = String::new();
            entry.read_to_string(&mut content).map_err(Aff4Error::Io)?;
            rdf::parse_turtle(&content)?
        };

        // ── 4. Discover image streams from RDF ──────────────────────────────
        let streams = discover_streams(&rdf);

        // ── 5. Extract case metadata ─────────────────────────────────────────
        let case_number = rdf
            .get_first(&volume_urn, rdf_predicates::DC_CASE_NUMBER)
            .unwrap_or("")
            .to_string();
        let evidence_number = rdf
            .get_first(&volume_urn, rdf_predicates::DC_EVIDENCE_NUMBER)
            .unwrap_or("")
            .to_string();
        let examiner = rdf
            .get_first(&volume_urn, rdf_predicates::DC_EXAMINER)
            .unwrap_or("")
            .to_string();
        let description = rdf
            .get_first(&volume_urn, rdf_predicates::DC_DESCRIPTION)
            .unwrap_or("")
            .to_string();

        // ── 6. Parse maps and bevy indexes ───────────────────────────────────
        let mut maps = HashMap::new();
        let mut bevy_indexes = HashMap::new();

        // Parse maps for each Image/Map stream
        for stream in &streams {
            // Try to load map data and idx
            let map_data_path = uri::map_data_path(&stream.urn, &volume_urn);
            let map_idx_path = uri::map_idx_path(&stream.urn, &volume_urn);

            let map_data = read_zip_member(&mut archive, &map_data_path);
            let map_idx = read_zip_member(&mut archive, &map_idx_path);

            if let (Some(md), Some(mi)) = (map_data, map_idx) {
                if let Ok(map_reader) = MapReader::from_data(&md, &mi) {
                    maps.insert(stream.urn.clone(), map_reader);
                }
            }

            // Pre-load bevy indexes for this stream
            for bevy_idx in 0u32..65536 {
                let index_path = uri::bevy_index_path(&stream.urn, &volume_urn, bevy_idx);
                if let Some(index_data) = read_zip_member(&mut archive, &index_path) {
                    let entries = parse_bevy_index(&index_data)?;
                    bevy_indexes.insert(index_path, entries);
                } else {
                    break; // No more bevies for this stream
                }
            }
        }

        Ok(Self {
            archive,
            volume_urn,
            version,
            tool,
            rdf,
            streams,
            case_number,
            evidence_number,
            examiner,
            description,
            maps,
            bevy_indexes,
        })
    }

    /// Get container metadata.
    pub fn info(&self) -> Aff4ContainerInfo {
        Aff4ContainerInfo {
            volume_urn: self.volume_urn.clone(),
            version: self.version,
            tool: self.tool.clone(),
            streams: self.streams.clone(),
            case_number: self.case_number.clone(),
            evidence_number: self.evidence_number.clone(),
            examiner: self.examiner.clone(),
            description: self.description.clone(),
        }
    }

    /// Get the RDF metadata graph.
    pub fn rdf(&self) -> &RdfGraph {
        &self.rdf
    }

    /// Get the volume URN.
    pub fn volume_urn(&self) -> &str {
        &self.volume_urn
    }

    /// Get the AFF4 version.
    pub fn version(&self) -> Aff4Version {
        self.version
    }

    /// Get discovered streams.
    pub fn streams(&self) -> &[Aff4StreamInfo] {
        &self.streams
    }

    /// Read data from a stream at a given offset.
    ///
    /// # Arguments
    ///
    /// - `stream_urn` — URN of the stream to read from
    /// - `offset` — byte offset within the stream
    /// - `buf` — buffer to fill
    ///
    /// # Returns
    ///
    /// Number of bytes read (may be less than `buf.len()` at end of stream).
    pub fn read_at(&mut self, stream_urn: &str, offset: u64, buf: &mut [u8]) -> Aff4Result<usize> {
        // Find the stream info
        let stream_info = self
            .streams
            .iter()
            .find(|s| s.urn == stream_urn)
            .ok_or_else(|| Aff4Error::StreamNotFound(stream_urn.to_string()))?
            .clone();

        if offset >= stream_info.size {
            return Ok(0);
        }

        let to_read = clamp_read_len(buf.len(), stream_info.size - offset);

        // Resolve through map if available
        if let Some(map) = self.maps.get(stream_urn).cloned() {
            return self.read_via_map(&map, &stream_info, offset, &mut buf[..to_read]);
        }

        // Direct stream read (no map — single bevy per chunk)
        self.read_direct(stream_urn, &stream_info, offset, &mut buf[..to_read])
    }

    /// Read data through the map (resolves virtual → physical → bevy → chunk).
    fn read_via_map(
        &mut self,
        map: &MapReader,
        stream_info: &Aff4StreamInfo,
        offset: u64,
        buf: &mut [u8],
    ) -> Aff4Result<usize> {
        let mut total_read = 0;
        let mut current_offset = offset;

        while total_read < buf.len() {
            let Some((_target_urn, target_offset, available)) = map.resolve(current_offset) else {
                break;
            };

            let remaining_in_buf = buf.len() - total_read;
            let to_read_from_target = clamp_read_len(remaining_in_buf, available);

            // Read from the target stream (bevy) — use the image URN (stream_info.urn)
            // for bevy path lookup, not the map's target_urn (which is a bevy URN
            // like aff4://uuid/00000000 and would produce double-nested paths).
            let n = self.read_from_bevy(
                &stream_info.urn,
                stream_info,
                target_offset,
                &mut buf[total_read..total_read + to_read_from_target],
            )?;

            if n == 0 {
                break;
            }

            total_read += n;
            current_offset += n as u64;
        }

        Ok(total_read)
    }

    /// Read directly from a stream without a map (identity mapping).
    fn read_direct(
        &mut self,
        stream_urn: &str,
        stream_info: &Aff4StreamInfo,
        offset: u64,
        buf: &mut [u8],
    ) -> Aff4Result<usize> {
        self.read_from_bevy(stream_urn, stream_info, offset, buf)
    }

    /// Read decompressed data from a bevy at a given offset.
    fn read_from_bevy(
        &mut self,
        stream_urn: &str,
        stream_info: &Aff4StreamInfo,
        offset: u64,
        buf: &mut [u8],
    ) -> Aff4Result<usize> {
        let (chunk_size, chunks_per_segment) = validate_stream_layout(stream_urn, stream_info)?;

        // Determine which bevy and chunk
        let global_chunk_index = offset / chunk_size;
        let bevy_index = u32::try_from(global_chunk_index / chunks_per_segment).map_err(|_| {
            Aff4Error::InvalidContainer(format!(
                "Stream {} resolves to a bevy index that exceeds u32",
                stream_urn
            ))
        })?;
        let chunk_in_bevy = usize::try_from(global_chunk_index % chunks_per_segment).map_err(|_| {
            Aff4Error::InvalidContainer(format!(
                "Stream {} resolves to a chunk index that exceeds usize",
                stream_urn
            ))
        })?;
        let offset_in_chunk = (offset % chunk_size) as usize;

        // Load bevy index
        let index_path = uri::bevy_index_path(stream_urn, &self.volume_urn, bevy_index);
        let entries = if let Some(cached) = self.bevy_indexes.get(&index_path) {
            cached.clone()
        } else {
            let index_data = read_zip_member(&mut self.archive, &index_path)
                .ok_or_else(|| Aff4Error::MissingMember(index_path.clone()))?;
            let entries = parse_bevy_index(&index_data)?;
            self.bevy_indexes.insert(index_path, entries.clone());
            entries
        };

        if chunk_in_bevy >= entries.len() {
            return Ok(0); // Beyond the last chunk in this bevy
        }

        // Read bevy data
        let data_path = uri::bevy_data_path(stream_urn, &self.volume_urn, bevy_index);
        let bevy_data = read_zip_member(&mut self.archive, &data_path)
            .ok_or(Aff4Error::MissingMember(data_path))?;

        // Extract and decompress the chunk
        let entry = &entries[chunk_in_bevy];
        let compressed_range = checked_bevy_data_range(entry, bevy_data.len())?;
        let compressed = &bevy_data[compressed_range];

        let is_stored = entry.length as u64 == chunk_size
            || (entry.length as u64 >= chunk_size
                && stream_info.compression == Aff4Compression::Stored);

        let decompressed = crate::compression::decompress_chunk(
            compressed,
            stream_info.compression,
            is_stored || stream_info.compression == Aff4Compression::Stored,
        )?;

        // Copy requested portion
        let available_in_chunk = decompressed.len().saturating_sub(offset_in_chunk);
        let to_copy = std::cmp::min(buf.len(), available_in_chunk);
        buf[..to_copy].copy_from_slice(&decompressed[offset_in_chunk..offset_in_chunk + to_copy]);

        Ok(to_copy)
    }

    /// Verify the integrity of a stream by checking all hashes.
    pub fn verify(
        &mut self,
        stream_urn: &str,
        cancel_flag: Option<&AtomicBool>,
        mut progress_fn: Option<Box<dyn FnMut(Aff4Progress) + Send>>,
    ) -> Aff4Result<Aff4VerifyResult> {
        let stream_info = self
            .streams
            .iter()
            .find(|s| s.urn == stream_urn)
            .ok_or_else(|| Aff4Error::StreamNotFound(stream_urn.to_string()))?
            .clone();

        let size = stream_info.size;
        let chunk_size = stream_info.chunk_size as usize;

        // Determine which hash algorithms to verify
        let hash_algos: Vec<Aff4HashAlgorithm> = stream_info.hashes.keys().copied().collect();

        let mut linear_hasher = StreamHasher::new(&hash_algos);
        let mut bytes_verified: u64 = 0;
        let mut chunks_verified: u64 = 0;
        let chunk_errors: u64 = 0;

        let mut buf = vec![0u8; chunk_size];

        // Emit: Verifying
        emit_progress(&mut progress_fn, Aff4Phase::Verifying, 0, size, "", 0, 0);

        while bytes_verified < size {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    return Err(Aff4Error::Cancelled);
                }
            }

            let remaining = (size - bytes_verified) as usize;
            let to_read = std::cmp::min(remaining, chunk_size);
            let read_buf = &mut buf[..to_read];

            let n = self.read_at(stream_urn, bytes_verified, read_buf)?;
            if n == 0 {
                break;
            }

            linear_hasher.update(&read_buf[..n]);
            bytes_verified += n as u64;
            chunks_verified += 1;

            // Emit progress every 64 chunks
            if chunks_verified.is_multiple_of(64) || bytes_verified >= size {
                emit_progress(
                    &mut progress_fn,
                    Aff4Phase::Verifying,
                    bytes_verified,
                    size,
                    "",
                    0,
                    0,
                );
            }
        }

        // Verify linear hashes
        let computed_hashes: HashMap<Aff4HashAlgorithm, String> =
            linear_hasher.finalize().into_iter().collect();

        let mut linear_checks = Vec::new();
        for (algo, expected) in &stream_info.hashes {
            let actual = computed_hashes.get(algo).cloned().unwrap_or_default();
            linear_checks.push(Aff4HashCheck {
                algorithm: *algo,
                expected: expected.clone(),
                actual: actual.clone(),
                passed: *expected == actual,
            });
        }

        let passed = linear_checks.iter().all(|c| c.passed) && chunk_errors == 0;

        Ok(Aff4VerifyResult {
            passed,
            linear_checks,
            block_map_checks: Vec::new(), // TODO: block-level verification
            chunks_verified,
            chunk_errors,
        })
    }

    /// List all ZIP members in the container.
    pub fn zip_members(&self) -> Vec<String> {
        (0..self.archive.len())
            .filter_map(|i| self.archive.name_for_index(i).map(|n| n.to_string()))
            .collect()
    }

    /// Read raw bytes from a specific ZIP member.
    pub fn read_member(&mut self, name: &str) -> Aff4Result<Vec<u8>> {
        read_zip_member(&mut self.archive, name)
            .ok_or_else(|| Aff4Error::MissingMember(name.to_string()))
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Parse version.txt into (version, tool).
fn parse_version_txt(content: &str) -> (Aff4Version, String) {
    let mut minor: u32 = 0;
    let mut tool = String::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("minor=") {
            minor = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("tool=") {
            tool = val.trim().to_string();
        }
    }

    let version = if minor >= 1 {
        Aff4Version::Logical
    } else {
        Aff4Version::Standard
    };

    (version, tool)
}

/// Discover image streams from the RDF graph.
fn discover_streams(rdf: &RdfGraph) -> Vec<Aff4StreamInfo> {
    let mut streams = Vec::new();

    // Find subjects that are Image, Map, ImageStream, or FileImage
    let image_types = [
        rdf_types::IMAGE,
        rdf_types::MAP,
        rdf_types::IMAGE_STREAM,
        rdf_types::DISK_IMAGE,
        rdf_types::CONTIGUOUS_IMAGE,
        rdf_types::FILE_IMAGE,
    ];

    for type_uri in &image_types {
        for subject in rdf.subjects_with_type(type_uri) {
            // Skip if already discovered
            if streams.iter().any(|s: &Aff4StreamInfo| s.urn == subject) {
                continue;
            }

            let size = rdf
                .get_first(&subject, rdf_predicates::SIZE)
                .and_then(parse_rdf_integer)
                .unwrap_or(0) as u64;

            let chunk_size = rdf
                .get_first(&subject, rdf_predicates::CHUNK_SIZE)
                .and_then(parse_rdf_integer)
                .unwrap_or(DEFAULT_CHUNK_SIZE as i64) as u32;

            let chunks_per_segment =
                rdf.get_first(&subject, rdf_predicates::CHUNKS_PER_SEGMENT)
                    .and_then(parse_rdf_integer)
                    .unwrap_or(DEFAULT_CHUNKS_PER_SEGMENT as i64) as u32;

            let compression = rdf
                .get_first(&subject, rdf_predicates::COMPRESSION_METHOD)
                .and_then(Aff4Compression::from_rdf_uri)
                .unwrap_or(Aff4Compression::Stored);

            // Collect stored hashes
            let mut hashes = HashMap::new();
            if let Some(hash_values) = rdf.get(&subject, rdf_predicates::STORED_HASH) {
                for hash_str in hash_values {
                    // Hash format: "<hash_uri>:<hex_digest>" or just hex
                    if let Some((algo, digest)) = parse_rdf_hash(hash_str) {
                        hashes.insert(algo, digest);
                    }
                }
            }

            streams.push(Aff4StreamInfo {
                urn: subject,
                stream_type: type_uri.to_string(),
                size,
                compression,
                chunk_size,
                chunks_per_segment,
                hashes,
            });
        }
    }

    streams
}

/// Parse an RDF integer literal.
///
/// Handles formats like `"32768"`, `"32768"^^<xsd:integer>`, `^^int:32768`
fn parse_rdf_integer(value: &str) -> Option<i64> {
    // Handle ^^int: prefix
    if let Some(stripped) = value.strip_prefix("^^int:") {
        return stripped.parse().ok();
    }
    // Handle ^^long: prefix
    if let Some(stripped) = value.strip_prefix("^^long:") {
        return stripped.parse().ok();
    }
    // Handle "value"^^<type> format
    let clean = value.trim_matches('"').split("^^").next().unwrap_or(value);
    clean.trim_matches('"').parse().ok()
}

/// Parse a stored hash value from RDF.
///
/// Expected format: `<hash_algorithm_uri>:<hex_digest>`
fn parse_rdf_hash(value: &str) -> Option<(Aff4HashAlgorithm, String)> {
    // Format: "http://aff4.org/Schema#SHA256:abcdef1234..."
    let (uri_part, digest) = value.rsplit_once(':')?;
    let algo = Aff4HashAlgorithm::from_rdf_uri(uri_part)?;
    Some((algo, digest.to_string()))
}

/// Read a ZIP member by name, returning None if it doesn't exist.
fn read_zip_member<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Option<Vec<u8>> {
    let mut entry = archive.by_name(name).ok()?;
    let mut data = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut data).ok()?;
    Some(data)
}

fn clamp_read_len(requested: usize, available: u64) -> usize {
    requested.min(usize::try_from(available).unwrap_or(usize::MAX))
}

fn validate_stream_layout(stream_urn: &str, stream_info: &Aff4StreamInfo) -> Aff4Result<(u64, u64)> {
    let chunk_size = u64::from(stream_info.chunk_size);
    if chunk_size == 0 {
        return Err(Aff4Error::InvalidContainer(format!(
            "Stream {} has chunkSize 0",
            stream_urn
        )));
    }

    let chunks_per_segment = u64::from(stream_info.chunks_per_segment);
    if chunks_per_segment == 0 {
        return Err(Aff4Error::InvalidContainer(format!(
            "Stream {} has chunksPerSegment 0",
            stream_urn
        )));
    }

    Ok((chunk_size, chunks_per_segment))
}

fn checked_bevy_data_range(
    entry: &BevyIndexEntry,
    data_len: usize,
) -> Aff4Result<std::ops::Range<usize>> {
    let start = usize::try_from(entry.offset).map_err(|_| Aff4Error::InvalidBevyIndex {
        offset: entry.offset,
        reason: "chunk offset exceeds addressable range".to_string(),
    })?;
    let end = start
        .checked_add(entry.length as usize)
        .ok_or_else(|| Aff4Error::InvalidBevyIndex {
            offset: entry.offset,
            reason: format!(
                "chunk range overflows usize for offset {} and length {}",
                entry.offset, entry.length
            ),
        })?;

    if end > data_len {
        return Err(Aff4Error::InvalidBevyIndex {
            offset: entry.offset,
            reason: format!(
                "chunk data range {}..{} exceeds bevy data size {}",
                start, end, data_len
            ),
        });
    }

    Ok(start..end)
}

/// Parse bevy index data into entries.
fn parse_bevy_index(data: &[u8]) -> Aff4Result<Vec<BevyIndexEntry>> {
    let mut chunks = data.chunks_exact(BEVY_INDEX_ENTRY_SIZE);
    if !chunks.remainder().is_empty() {
        return Err(Aff4Error::InvalidBevyIndex {
            offset: data.len() as u64,
            reason: format!(
                "index size {} is not a multiple of {}",
                data.len(),
                BEVY_INDEX_ENTRY_SIZE
            ),
        });
    }

    let mut entries = Vec::with_capacity(data.len() / BEVY_INDEX_ENTRY_SIZE);
    for (index, chunk) in chunks.by_ref().enumerate() {
        let start = (index * BEVY_INDEX_ENTRY_SIZE) as u64;
        let chunk: [u8; BEVY_INDEX_ENTRY_SIZE] = chunk.try_into().map_err(|_| {
            Aff4Error::InvalidBevyIndex {
                offset: start,
                reason: format!("index entry at byte {} has invalid length", start),
            }
        })?;
        entries.push(BevyIndexEntry::from_bytes(&chunk));
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::Aff4Writer;
    use std::io::Cursor;

    fn test_stream_info() -> Aff4StreamInfo {
        Aff4StreamInfo {
            urn: "aff4://test-stream".to_string(),
            stream_type: "http://aff4.org/Schema#Image".to_string(),
            size: 1024,
            compression: Aff4Compression::Stored,
            chunk_size: 512,
            chunks_per_segment: 4,
            hashes: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_parse_version_txt() {
        let content = "major=1\nminor=0\ntool=CORE-FFX\n";
        let (version, tool) = parse_version_txt(content);
        assert_eq!(version, Aff4Version::Standard);
        assert_eq!(tool, "CORE-FFX");
    }

    #[test]
    fn test_parse_version_txt_logical() {
        let content = "major=1\nminor=1\ntool=MyTool\n";
        let (version, tool) = parse_version_txt(content);
        assert_eq!(version, Aff4Version::Logical);
        assert_eq!(tool, "MyTool");
    }

    #[test]
    fn test_parse_rdf_integer() {
        assert_eq!(parse_rdf_integer("32768"), Some(32768));
        assert_eq!(parse_rdf_integer("^^int:32768"), Some(32768));
        assert_eq!(parse_rdf_integer("^^long:1073741824"), Some(1073741824));
        assert_eq!(parse_rdf_integer("\"32768\"^^<xsd:integer>"), Some(32768));
    }

    #[test]
    fn test_parse_rdf_hash() {
        let value = "http://aff4.org/Schema#SHA256:abcdef1234567890";
        let (algo, digest) = parse_rdf_hash(value).unwrap();
        assert_eq!(algo, Aff4HashAlgorithm::Sha256);
        assert_eq!(digest, "abcdef1234567890");
    }

    #[test]
    fn test_validate_stream_layout_rejects_zero_chunk_size() {
        let mut stream = test_stream_info();
        stream.chunk_size = 0;

        let err = validate_stream_layout(&stream.urn, &stream).unwrap_err();
        assert!(matches!(err, Aff4Error::InvalidContainer(message) if message.contains("chunkSize 0")));
    }

    #[test]
    fn test_parse_bevy_index_rejects_trailing_bytes() {
        let err = parse_bevy_index(&[0u8; BEVY_INDEX_ENTRY_SIZE + 1]).unwrap_err();
        assert!(matches!(err, Aff4Error::InvalidBevyIndex { .. }));
    }

    #[test]
    fn test_checked_bevy_data_range_rejects_out_of_bounds() {
        let entry = BevyIndexEntry {
            offset: 8,
            length: 16,
        };

        let err = checked_bevy_data_range(&entry, 12).unwrap_err();
        assert!(matches!(err, Aff4Error::InvalidBevyIndex { .. }));
    }

    #[test]
    fn test_write_then_read_roundtrip() {
        // Write a small container
        let source_data = vec![0x42u8; 65536]; // 64 KiB
        let source_size = source_data.len() as u64;

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("roundtrip_test");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            compression: Aff4Compression::Stored,
            linear_hashes: vec![Aff4HashAlgorithm::Sha256],
            block_hashes: vec![Aff4HashAlgorithm::Sha256],
            case_number: "TEST-001".to_string(),
            examiner: "Test Examiner".to_string(),
            ..Default::default()
        };

        let write_result = Aff4Writer::write_physical(
            &config,
            Cursor::new(source_data.clone()),
            source_size,
            None,
            None,
        )
        .unwrap();

        // Read it back
        let mut reader = Aff4Reader::open(&write_result.output_path).unwrap();
        let info = reader.info();

        assert_eq!(info.volume_urn, write_result.volume_urn);
        assert_eq!(info.version, Aff4Version::Standard);
        assert_eq!(info.case_number, "TEST-001");
        assert_eq!(info.examiner, "Test Examiner");
        assert!(!info.streams.is_empty());

        // Read data back and compare
        let stream_urn = info.streams[0].urn.clone();
        let mut read_buf = vec![0u8; source_data.len()];
        let n = reader.read_at(&stream_urn, 0, &mut read_buf).unwrap();
        assert_eq!(n, source_data.len());
        assert_eq!(read_buf, source_data);
    }

    #[test]
    fn test_zip_members_listed() {
        let source_data = vec![0xFFu8; 32768];
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("members_test");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            compression: Aff4Compression::Stored,
            ..Default::default()
        };

        let result =
            Aff4Writer::write_physical(&config, Cursor::new(source_data), 32768, None, None)
                .unwrap();

        let reader = Aff4Reader::open(&result.output_path).unwrap();
        let members = reader.zip_members();

        assert!(members.contains(&CONTAINER_DESCRIPTION.to_string()));
        assert!(members.contains(&VERSION_TXT.to_string()));
        assert!(members.contains(&INFORMATION_TURTLE.to_string()));
    }
}
