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
use crate::hashing::{compute_block_map_hash, hash_chunk, hash_hex, StreamHasher};
use crate::helpers::emit_progress;
use crate::map::MapReader;
use crate::rdf::{self, RdfGraph};
use crate::types::*;
use crate::uri;

const MAX_METADATA_MEMBER_BYTES: u64 = 16 * 1024 * 1024;

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
            read_zip_member_text(
                &mut archive,
                CONTAINER_DESCRIPTION,
                MAX_METADATA_MEMBER_BYTES,
            )?
            .trim()
            .to_string()
        };

        if !volume_urn.starts_with("aff4://") {
            return Err(Aff4Error::InvalidContainer(
                "container.description does not contain a valid AFF4 URN".to_string(),
            ));
        }

        // ── 2. Read version.txt ──────────────────────────────────────────────
        let (version, tool) = {
            let content =
                read_zip_member_text(&mut archive, VERSION_TXT, MAX_METADATA_MEMBER_BYTES)?;
            parse_version_txt(&content)
        };

        // ── 3. Read information.turtle ───────────────────────────────────────
        let rdf = {
            let content =
                read_zip_member_text(&mut archive, INFORMATION_TURTLE, MAX_METADATA_MEMBER_BYTES)?;
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

        let to_read = bounded_aff4_stream_read_len(offset, buf.len(), stream_info.size)?;
        if to_read == 0 {
            return Ok(0);
        }

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
        let chunk_in_bevy =
            usize::try_from(global_chunk_index % chunks_per_segment).map_err(|_| {
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

        // Read only the compressed chunk bytes from the bevy ZIP member.
        let data_path = uri::bevy_data_path(stream_urn, &self.volume_urn, bevy_index);
        let entry = required_bevy_index_entry(&entries, chunk_in_bevy, stream_urn, bevy_index)?;
        let compressed =
            read_zip_member_range(&mut self.archive, &data_path, entry.offset, entry.length)?
                .ok_or(Aff4Error::MissingMember(data_path))?;

        let is_stored = entry.length as u64 == chunk_size
            || (entry.length as u64 >= chunk_size
                && stream_info.compression == Aff4Compression::Stored);

        let decompressed = crate::compression::decompress_chunk(
            &compressed,
            stream_info.compression,
            is_stored || stream_info.compression == Aff4Compression::Stored,
        )?;

        // Copy the requested portion. A short decompressed chunk is corruption,
        // not a normal short read; callers should only see short reads at EOF.
        let available_in_stream = stream_info.size.checked_sub(offset).ok_or_else(|| {
            Aff4Error::InvalidContainer(format!(
                "Stream {stream_urn} read offset exceeds stream size"
            ))
        })?;
        let available_in_chunk = chunk_size - (offset % chunk_size);
        let expected_to_copy =
            clamp_read_len(buf.len(), available_in_stream.min(available_in_chunk));
        let chunk_slice = required_decompressed_chunk_slice(
            &decompressed,
            offset_in_chunk,
            expected_to_copy,
            stream_urn,
        )?;
        buf[..expected_to_copy].copy_from_slice(chunk_slice);

        Ok(expected_to_copy)
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
                return Err(Aff4Error::InvalidContainer(format!(
                    "Short read while verifying stream {stream_urn}: expected {size} bytes, verified {bytes_verified} bytes"
                )));
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

        let block_map_checks = self.verify_map_hashes(&stream_info)?;
        let block_map_errors = block_map_checks
            .iter()
            .filter(|check| !check.passed)
            .count() as u64;
        let chunk_errors = chunk_errors + block_map_errors;
        let passed = linear_checks.iter().all(|c| c.passed)
            && block_map_checks.iter().all(|c| c.passed)
            && chunk_errors == 0;

        Ok(Aff4VerifyResult {
            passed,
            linear_checks,
            block_map_checks,
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

    fn verify_map_hashes(
        &mut self,
        stream_info: &Aff4StreamInfo,
    ) -> Aff4Result<Vec<Aff4HashCheck>> {
        let Some(map_reader) = self.maps.get(&stream_info.urn).cloned() else {
            return Ok(Vec::new());
        };

        let needs_map_data = !stream_info.map_point_hashes.is_empty()
            || !stream_info.map_idx_hashes.is_empty()
            || !stream_info.block_map_hashes.is_empty();
        if !needs_map_data {
            return Ok(Vec::new());
        }

        let map_data_path = uri::map_data_path(&stream_info.urn, &self.volume_urn);
        let map_idx_path = uri::map_idx_path(&stream_info.urn, &self.volume_urn);
        let map_data = read_zip_member(&mut self.archive, &map_data_path)
            .ok_or_else(|| Aff4Error::MissingMember(map_data_path.clone()))?;
        let map_idx = read_zip_member(&mut self.archive, &map_idx_path)
            .ok_or_else(|| Aff4Error::MissingMember(map_idx_path.clone()))?;

        let mut checks = Vec::new();

        for (algorithm, expected) in &stream_info.map_point_hashes {
            let actual = hash_hex(&map_data, *algorithm);
            checks.push(Aff4HashCheck {
                algorithm: *algorithm,
                expected: expected.clone(),
                actual: actual.clone(),
                passed: *expected == actual,
            });
        }

        for (algorithm, expected) in &stream_info.map_idx_hashes {
            let actual = hash_hex(&map_idx, *algorithm);
            checks.push(Aff4HashCheck {
                algorithm: *algorithm,
                expected: expected.clone(),
                actual: actual.clone(),
                passed: *expected == actual,
            });
        }

        for (algorithm, expected) in &stream_info.block_map_hashes {
            let actual = self.compute_block_map_hash_for_reader(
                &stream_info.urn,
                &map_reader,
                *algorithm,
                &map_data,
                &map_idx,
            );
            checks.push(Aff4HashCheck {
                algorithm: *algorithm,
                expected: expected.clone(),
                actual: actual.clone(),
                passed: *expected == actual,
            });
        }

        Ok(checks)
    }

    fn compute_block_map_hash_for_reader(
        &mut self,
        stream_urn: &str,
        map_reader: &MapReader,
        algorithm: Aff4HashAlgorithm,
        map_data: &[u8],
        map_idx: &[u8],
    ) -> String {
        let mut concatenated_block_hashes = Vec::new();
        for bevy_index in bevy_indexes_from_map(map_reader) {
            let hash_path = uri::bevy_block_hash_path(
                stream_urn,
                &self.volume_urn,
                bevy_index,
                algorithm.block_hash_extension(),
            );
            let Some(hash_data) = read_zip_member(&mut self.archive, &hash_path) else {
                return String::new();
            };
            concatenated_block_hashes.extend_from_slice(&hash_data);
        }

        let map_point_hash = hash_chunk(map_data, algorithm);
        let map_idx_hash = hash_chunk(map_idx, algorithm);
        hex::encode(compute_block_map_hash(
            algorithm,
            &concatenated_block_hashes,
            &map_point_hash,
            &map_idx_hash,
        ))
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

            let hashes = collect_rdf_hashes(rdf, &subject, rdf_predicates::STORED_HASH);
            let map_point_hashes =
                collect_rdf_hashes(rdf, &subject, rdf_predicates::MAP_POINT_HASH);
            let map_idx_hashes = collect_rdf_hashes(rdf, &subject, rdf_predicates::MAP_IDX_HASH);
            let block_map_hashes =
                collect_rdf_hashes(rdf, &subject, rdf_predicates::BLOCK_MAP_HASH);

            streams.push(Aff4StreamInfo {
                urn: subject,
                stream_type: type_uri.to_string(),
                size,
                compression,
                chunk_size,
                chunks_per_segment,
                hashes,
                map_point_hashes,
                map_idx_hashes,
                block_map_hashes,
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
    // Formats accepted:
    // - "http://aff4.org/Schema#SHA256:abcdef1234..."
    // - "^^hash:http://aff4.org/Schema#SHA256:abcdef1234..."
    // - "http://aff4.org/Schema#SHA256|abcdef1234..." from typed Turtle literals
    if let Some((uri_part, digest)) = value.split_once('|') {
        let algo = Aff4HashAlgorithm::from_rdf_uri(uri_part)?;
        return Some((algo, digest.to_string()));
    }

    let value = value.strip_prefix("^^hash:").unwrap_or(value);
    let (uri_part, digest) = value.rsplit_once(':')?;
    let algo = Aff4HashAlgorithm::from_rdf_uri(uri_part)?;
    Some((algo, digest.to_string()))
}

fn collect_rdf_hashes(
    rdf: &RdfGraph,
    subject: &str,
    predicate: &str,
) -> HashMap<Aff4HashAlgorithm, String> {
    let mut hashes = HashMap::new();
    if let Some(hash_values) = rdf.get(subject, predicate) {
        for hash_str in hash_values {
            if let Some((algo, digest)) = parse_rdf_hash(hash_str) {
                hashes.insert(algo, digest);
            }
        }
    }
    hashes
}

fn bevy_indexes_from_map(map_reader: &MapReader) -> Vec<u32> {
    let mut indexes = Vec::new();
    for entry in map_reader.entries() {
        let Some(target_urn) = map_reader.target_urn(entry.target_id) else {
            continue;
        };
        let Some(last_segment) = target_urn.rsplit('/').next() else {
            continue;
        };
        let Ok(bevy_index) = u32::from_str_radix(last_segment, 16) else {
            continue;
        };
        if !indexes.contains(&bevy_index) {
            indexes.push(bevy_index);
        }
    }
    indexes
}

/// Read a ZIP member by name, returning None if it doesn't exist.
fn read_zip_member<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Option<Vec<u8>> {
    let mut entry = archive.by_name(name).ok()?;
    let capacity = usize::try_from(entry.size()).ok()?;
    let mut data = Vec::new();
    data.try_reserve_exact(capacity).ok()?;
    entry.read_to_end(&mut data).ok()?;
    Some(data)
}

fn read_zip_member_text<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
    max_bytes: u64,
) -> Aff4Result<String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| Aff4Error::MissingMember(name.to_string()))?;
    if entry.size() > max_bytes {
        return Err(Aff4Error::InvalidContainer(format!(
            "AFF4 metadata member {name} is too large: {} bytes",
            entry.size()
        )));
    }

    let capacity = usize::try_from(entry.size()).map_err(|_| {
        Aff4Error::InvalidContainer(format!("AFF4 metadata member {name} is too large"))
    })?;
    let mut content = String::new();
    content.try_reserve_exact(capacity).map_err(|_| {
        Aff4Error::InvalidContainer(format!(
            "AFF4 metadata member {name} allocation is too large"
        ))
    })?;
    entry.read_to_string(&mut content).map_err(Aff4Error::Io)?;
    Ok(content)
}

fn read_zip_member_range<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
    offset: u64,
    length: u32,
) -> Aff4Result<Option<Vec<u8>>> {
    let mut entry = match archive.by_name(name) {
        Ok(entry) => entry,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(err) => return Err(Aff4Error::Zip(err)),
    };
    let length = usize::try_from(length)
        .map_err(|_| Aff4Error::InvalidContainer(format!("ZIP member {name} range too large")))?;
    let end = offset
        .checked_add(length as u64)
        .ok_or_else(|| Aff4Error::InvalidContainer(format!("ZIP member {name} range overflow")))?;
    if end > entry.size() {
        return Err(Aff4Error::InvalidContainer(format!(
            "ZIP member {name} range {}..{} exceeds member size {}",
            offset,
            end,
            entry.size()
        )));
    }

    let skipped = std::io::copy(&mut entry.by_ref().take(offset), &mut std::io::sink())
        .map_err(Aff4Error::Io)?;
    if skipped != offset {
        return Err(Aff4Error::InvalidContainer(format!(
            "Short ZIP member skip for {name}: expected {offset} bytes, skipped {skipped}"
        )));
    }
    let mut data = Vec::new();
    data.try_reserve_exact(length).map_err(|_| {
        Aff4Error::InvalidContainer(format!("ZIP member {name} range allocation is too large"))
    })?;
    entry
        .take(length as u64)
        .read_to_end(&mut data)
        .map_err(Aff4Error::Io)?;
    if data.len() != length {
        return Err(Aff4Error::InvalidContainer(format!(
            "Short ZIP member read for {name}: expected {length} bytes, read {}",
            data.len()
        )));
    }
    Ok(Some(data))
}

fn clamp_read_len(requested: usize, available: u64) -> usize {
    requested.min(usize::try_from(available).unwrap_or(usize::MAX))
}

fn validate_stream_layout(
    stream_urn: &str,
    stream_info: &Aff4StreamInfo,
) -> Aff4Result<(u64, u64)> {
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

fn bounded_aff4_stream_read_len(
    offset: u64,
    requested_len: usize,
    stream_size: u64,
) -> Aff4Result<usize> {
    if offset > stream_size {
        return Err(Aff4Error::SeekOutOfRange {
            offset,
            size: stream_size,
        });
    }

    let remaining = stream_size - offset;
    let remaining = usize::try_from(remaining).unwrap_or(usize::MAX);
    Ok(requested_len.min(remaining))
}

fn required_bevy_index_entry<'a>(
    entries: &'a [BevyIndexEntry],
    chunk_in_bevy: usize,
    stream_urn: &str,
    bevy_index: u32,
) -> Aff4Result<&'a BevyIndexEntry> {
    entries.get(chunk_in_bevy).ok_or_else(|| {
        Aff4Error::InvalidContainer(format!(
            "Stream {stream_urn} missing chunk {chunk_in_bevy} in bevy {bevy_index}: bevy index has {} entries",
            entries.len()
        ))
    })
}

fn required_decompressed_chunk_slice<'a>(
    decompressed: &'a [u8],
    offset_in_chunk: usize,
    requested: usize,
    stream_urn: &str,
) -> Aff4Result<&'a [u8]> {
    let end = offset_in_chunk.checked_add(requested).ok_or_else(|| {
        Aff4Error::InvalidContainer(format!(
            "Stream {stream_urn} decompressed chunk slice overflow"
        ))
    })?;
    decompressed
        .get(offset_in_chunk..end)
        .ok_or_else(|| {
            Aff4Error::InvalidContainer(format!(
                "Stream {stream_urn} decompressed chunk short read: expected bytes {}..{}, chunk has {} bytes",
                offset_in_chunk,
                end,
                decompressed.len()
            ))
        })
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
        let chunk: [u8; BEVY_INDEX_ENTRY_SIZE] =
            chunk.try_into().map_err(|_| Aff4Error::InvalidBevyIndex {
                offset: start,
                reason: format!("index entry at byte {} has invalid length", start),
            })?;
        entries.push(BevyIndexEntry::from_bytes(&chunk));
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::Aff4Writer;
    use std::io::{Cursor, Write};

    fn test_stream_info() -> Aff4StreamInfo {
        Aff4StreamInfo {
            urn: "aff4://test-stream".to_string(),
            stream_type: "http://aff4.org/Schema#Image".to_string(),
            size: 1024,
            compression: Aff4Compression::Stored,
            chunk_size: 512,
            chunks_per_segment: 4,
            hashes: std::collections::HashMap::new(),
            map_point_hashes: std::collections::HashMap::new(),
            map_idx_hashes: std::collections::HashMap::new(),
            block_map_hashes: std::collections::HashMap::new(),
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
    fn test_parse_rdf_hash_accepts_internal_and_typed_forms() {
        let internal = "^^hash:http://aff4.org/Schema#SHA256:abcdef1234567890";
        let (algo, digest) = parse_rdf_hash(internal).unwrap();
        assert_eq!(algo, Aff4HashAlgorithm::Sha256);
        assert_eq!(digest, "abcdef1234567890");

        let typed = "http://aff4.org/Schema#SHA256|abcdef1234567890";
        let (algo, digest) = parse_rdf_hash(typed).unwrap();
        assert_eq!(algo, Aff4HashAlgorithm::Sha256);
        assert_eq!(digest, "abcdef1234567890");
    }

    #[test]
    fn test_validate_stream_layout_rejects_zero_chunk_size() {
        let mut stream = test_stream_info();
        stream.chunk_size = 0;

        let err = validate_stream_layout(&stream.urn, &stream).unwrap_err();
        assert!(
            matches!(err, Aff4Error::InvalidContainer(message) if message.contains("chunkSize 0"))
        );
    }

    #[test]
    fn test_bounded_aff4_stream_read_len_allows_exact_eof() {
        assert_eq!(bounded_aff4_stream_read_len(1024, 512, 1024).unwrap(), 0);
    }

    #[test]
    fn test_bounded_aff4_stream_read_len_rejects_offset_past_eof() {
        let err = bounded_aff4_stream_read_len(1025, 512, 1024).unwrap_err();
        assert!(matches!(
            err,
            Aff4Error::SeekOutOfRange {
                offset: 1025,
                size: 1024
            }
        ));
    }

    #[test]
    fn test_bounded_aff4_stream_read_len_clamps_to_remaining() {
        assert_eq!(bounded_aff4_stream_read_len(1000, 512, 1024).unwrap(), 24);
        assert_eq!(bounded_aff4_stream_read_len(0, 8, u64::MAX).unwrap(), 8);
    }

    #[test]
    fn test_required_bevy_index_entry_rejects_missing_chunk() {
        let err = required_bevy_index_entry(&[], 2, "aff4://stream", 0).unwrap_err();

        assert!(
            matches!(err, Aff4Error::InvalidContainer(message) if message.contains("missing chunk 2"))
        );
    }

    #[test]
    fn test_required_decompressed_chunk_slice_returns_requested_range() {
        let slice = required_decompressed_chunk_slice(b"abcdef", 2, 3, "aff4://stream")
            .expect("valid decompressed chunk slice should be returned");

        assert_eq!(slice, b"cde");
    }

    #[test]
    fn test_required_decompressed_chunk_slice_rejects_short_chunk() {
        let err = required_decompressed_chunk_slice(b"abcd", 2, 4, "aff4://stream")
            .expect_err("short decompressed chunk should fail");

        assert!(
            matches!(err, Aff4Error::InvalidContainer(message) if message.contains("short read"))
        );
    }

    #[test]
    fn test_parse_bevy_index_rejects_trailing_bytes() {
        let err = parse_bevy_index(&[0u8; BEVY_INDEX_ENTRY_SIZE + 1]).unwrap_err();
        assert!(matches!(err, Aff4Error::InvalidBevyIndex { .. }));
    }

    #[test]
    fn test_read_zip_member_range_reads_requested_slice() {
        let mut zip_bytes = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut zip_bytes);
            writer
                .start_file("member.bin", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"0123456789abcdef").unwrap();
            writer.finish().unwrap();
        }

        zip_bytes.set_position(0);
        let mut archive = zip::ZipArchive::new(zip_bytes).unwrap();
        let data = read_zip_member_range(&mut archive, "member.bin", 4, 6)
            .unwrap()
            .unwrap();
        assert_eq!(data, b"456789");
    }

    #[test]
    fn test_read_zip_member_range_rejects_out_of_bounds() {
        let mut zip_bytes = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut zip_bytes);
            writer
                .start_file("member.bin", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"0123456789abcdef").unwrap();
            writer.finish().unwrap();
        }

        zip_bytes.set_position(0);
        let mut archive = zip::ZipArchive::new(zip_bytes).unwrap();
        let err = read_zip_member_range(&mut archive, "member.bin", 8, 16).unwrap_err();
        assert!(matches!(err, Aff4Error::InvalidContainer(_)));
    }

    #[test]
    fn test_read_zip_member_text_rejects_oversized_metadata() {
        let mut zip_bytes = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut zip_bytes);
            writer
                .start_file(
                    "information.turtle",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            writer.write_all(b"too-large").unwrap();
            writer.finish().unwrap();
        }

        zip_bytes.set_position(0);
        let mut archive = zip::ZipArchive::new(zip_bytes).unwrap();
        let err = read_zip_member_text(&mut archive, "information.turtle", 4).unwrap_err();
        assert!(matches!(err, Aff4Error::InvalidContainer(_)));
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
        assert!(!info.streams[0].hashes.is_empty());
        assert!(!info.streams[0].map_point_hashes.is_empty());
        assert!(!info.streams[0].map_idx_hashes.is_empty());
        assert!(!info.streams[0].block_map_hashes.is_empty());

        let mut read_buf = vec![0u8; source_data.len()];
        let n = reader.read_at(&stream_urn, 0, &mut read_buf).unwrap();
        assert_eq!(n, source_data.len());
        assert_eq!(read_buf, source_data);

        let verify_result = reader.verify(&stream_urn, None, None).unwrap();
        assert!(verify_result.passed);
        assert_eq!(verify_result.linear_checks.len(), 1);
        assert_eq!(verify_result.block_map_checks.len(), 3);
        assert!(verify_result
            .block_map_checks
            .iter()
            .all(|check| check.passed));
    }

    #[test]
    fn test_verify_rejects_stream_size_larger_than_data() {
        let source_data = vec![0x42u8; 65536];
        let source_size = source_data.len() as u64;

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("short_verify_test");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            compression: Aff4Compression::Stored,
            linear_hashes: vec![Aff4HashAlgorithm::Sha256],
            block_hashes: vec![Aff4HashAlgorithm::Sha256],
            ..Default::default()
        };

        let write_result =
            Aff4Writer::write_physical(&config, Cursor::new(source_data), source_size, None, None)
                .unwrap();

        let corrupt_path = dir.path().join("short_verify_corrupt.aff4");
        rewrite_information_turtle_size(
            &write_result.output_path,
            &corrupt_path,
            source_size,
            source_size + 1024,
        );

        let mut reader = Aff4Reader::open(&corrupt_path).unwrap();
        let stream_urn = reader.info().streams[0].urn.clone();
        let err = reader.verify(&stream_urn, None, None).unwrap_err();

        assert!(
            err.to_string()
                .contains("Short read while verifying stream"),
            "unexpected error: {}",
            err
        );
    }

    fn rewrite_information_turtle_size(
        source_path: &std::path::Path,
        dest_path: &std::path::Path,
        old_size: u64,
        new_size: u64,
    ) {
        use std::io::{Read, Write};
        use zip::write::SimpleFileOptions;

        let source = std::fs::File::open(source_path).unwrap();
        let mut archive = zip::ZipArchive::new(source).unwrap();
        let dest = std::fs::File::create(dest_path).unwrap();
        let mut writer = zip::ZipWriter::new(dest);
        let options = SimpleFileOptions::default();

        for index in 0..archive.len() {
            let mut member = archive.by_index(index).unwrap();
            let name = member.name().to_string();
            let mut data = Vec::new();
            member.read_to_end(&mut data).unwrap();
            if name == INFORMATION_TURTLE {
                let mut turtle = String::from_utf8(data).unwrap();
                let old_literal = format!("aff4:size \"{}\"", old_size);
                let new_literal = format!("aff4:size \"{}\"", new_size);
                let replaced = turtle.replacen(&old_literal, &new_literal, 1);
                assert_ne!(replaced, turtle, "expected RDF size literal to be present");
                turtle = replaced;
                data = turtle.into_bytes();
            }

            writer.start_file(name, options).unwrap();
            writer.write_all(&data).unwrap();
        }

        writer.finish().unwrap();
    }

    #[test]
    fn test_bevy_indexes_from_map_uses_unique_target_ids() {
        let map_data = [
            crate::map::MapEntry {
                mapped_offset: 0,
                length: 512,
                target_offset: 0,
                target_id: 0,
            }
            .to_bytes(),
            crate::map::MapEntry {
                mapped_offset: 512,
                length: 512,
                target_offset: 0,
                target_id: 1,
            }
            .to_bytes(),
            crate::map::MapEntry {
                mapped_offset: 1024,
                length: 512,
                target_offset: 512,
                target_id: 1,
            }
            .to_bytes(),
        ]
        .concat();
        let map_idx = b"aff4://vol/image/00000000\naff4://vol/image/00000001";
        let map_reader = MapReader::from_data(&map_data, map_idx).unwrap();

        assert_eq!(bevy_indexes_from_map(&map_reader), vec![0, 1]);
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
