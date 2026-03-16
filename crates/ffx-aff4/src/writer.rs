// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AFF4 container writer for physical disk imaging (AFF4-Standard v1.0).
//!
//! Creates a ZIP64-based AFF4 container containing:
//! - `container.description` — volume URN (first ZIP member)
//! - `version.txt` — version and tool identity
//! - Bevy data/index/block-hash files — compressed image data
//! - Map data/idx files — virtual → physical address mapping
//! - `information.turtle` — RDF metadata

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::bevy::BevyWriter;
use crate::error::{Aff4Error, Aff4Result};
use crate::helpers::{emit_progress, ensure_aff4_extension};
use crate::hashing::{hash_chunk, hash_hex, StreamHasher};
use crate::map::MapWriter;
use crate::rdf::{self, RdfGraph};
use crate::types::*;
use crate::uri;

// ─── AFF4 Writer ─────────────────────────────────────────────────────────────

/// Writer for AFF4-Standard (v1.0) physical disk image containers.
///
/// ## Usage
///
/// ```rust,no_run
/// use std::fs::File;
/// use ffx_aff4::writer::Aff4Writer;
/// use ffx_aff4::types::Aff4WriterConfig;
///
/// let config = Aff4WriterConfig {
///     output_path: "image.aff4".into(),
///     ..Default::default()
/// };
/// let source = File::open("/dev/sda").unwrap();
/// let source_size = 1024 * 1024 * 1024; // 1 GiB
///
/// let result = Aff4Writer::write_physical(
///     &config,
///     source,
///     source_size,
///     None,
///     None,
/// ).unwrap();
/// ```
pub struct Aff4Writer;

impl Aff4Writer {
    /// Write a physical disk image to an AFF4 container.
    ///
    /// # Arguments
    ///
    /// - `config` — Writer configuration (compression, hashing, case metadata)
    /// - `source` — Readable + seekable source (disk device, file, etc.)
    /// - `source_size` — Total size of source data in bytes
    /// - `cancel_flag` — Optional cancellation flag
    /// - `progress_fn` — Optional callback for progress updates
    pub fn write_physical<R: Read + Seek>(
        config: &Aff4WriterConfig,
        mut source: R,
        source_size: u64,
        cancel_flag: Option<&AtomicBool>,
        mut progress_fn: Option<Box<dyn FnMut(Aff4Progress) + Send>>,
    ) -> Aff4Result<Aff4WriteResult> {
        // Validate config
        if config.output_path.as_os_str().is_empty() {
            return Err(Aff4Error::NoOutputPath);
        }
        if source_size == 0 {
            return Err(Aff4Error::NoSource);
        }

        // Emit: Preparing
        emit_progress(&mut progress_fn, Aff4Phase::Preparing, 0, source_size, "", 0, 0);

        // Generate URNs
        let volume_urn = uri::new_volume_urn();
        let image_urn = uri::new_image_urn();

        // Ensure output path has .aff4 extension
        let output_path = ensure_aff4_extension(&config.output_path);

        // Create the ZIP file
        let file = std::fs::File::create(&output_path)
            .map_err(Aff4Error::Io)?;
        let mut zip = ZipWriter::new(file);
        let stored_opts = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // ── 1. Write container.description (MUST be the first member) ────────
        zip.start_file(CONTAINER_DESCRIPTION, stored_opts)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(volume_urn.as_bytes())
            .map_err(Aff4Error::Io)?;

        // ── 2. Write version.txt ─────────────────────────────────────────────
        let version_content = format!(
            "major={}\nminor={}\ntool={}\n",
            config.version.major(),
            config.version.minor(),
            config.tool_name
        );
        zip.start_file(VERSION_TXT, stored_opts)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(version_content.as_bytes())
            .map_err(Aff4Error::Io)?;

        // ── 3. Write bevies ──────────────────────────────────────────────────
        let chunk_size = config.chunk_size;
        let chunks_per_segment = config.chunks_per_segment;

        let mut linear_hasher = StreamHasher::new(&config.linear_hashes);
        let mut map_writer = MapWriter::new();

        let mut bytes_written: u64 = 0;
        let mut bevy_index: u32 = 0;
        let mut all_block_hashes: HashMap<Aff4HashAlgorithm, Vec<Vec<u8>>> = HashMap::new();

        let mut read_buf = vec![0u8; chunk_size as usize];

        // Emit: WritingData
        emit_progress(
            &mut progress_fn,
            Aff4Phase::WritingData,
            0,
            source_size,
            "",
            0,
            0,
        );

        loop {
            // Check cancellation
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    // Clean up: drop zip writer to release file handle, delete output
                    drop(zip);
                    let _ = std::fs::remove_file(&output_path);
                    return Err(Aff4Error::Cancelled);
                }
            }

            // Create bevy writer for this segment
            let mut bevy_writer = BevyWriter::new(
                config.compression,
                chunk_size,
                &config.block_hashes,
            );

            let mut bevy_data_written: u64 = 0;
            let mut chunks_in_bevy: u32 = 0;

            // Read chunks into this bevy
            while chunks_in_bevy < chunks_per_segment {
                let remaining = source_size - bytes_written;
                if remaining == 0 {
                    break;
                }

                let to_read = std::cmp::min(remaining, chunk_size as u64) as usize;
                let buf = &mut read_buf[..to_read];
                source.read_exact(buf).map_err(Aff4Error::Io)?;

                // Update linear hash
                linear_hasher.update(buf);

                // Add chunk to bevy
                bevy_writer.add_chunk(buf)?;

                bevy_data_written += to_read as u64;
                bytes_written += to_read as u64;
                chunks_in_bevy += 1;

                // Emit progress (throttled to every 64 chunks)
                if chunks_in_bevy.is_multiple_of(64) || bytes_written == source_size {
                    emit_progress(
                        &mut progress_fn,
                        Aff4Phase::WritingData,
                        bytes_written,
                        source_size,
                        "",
                        0,
                        0,
                    );
                }
            }

            if chunks_in_bevy == 0 {
                break;
            }

            // Finish the bevy
            let bevy_result = bevy_writer.finish();

            // Register bevy in map
            let bevy_urn = format!("{}/{:08x}", image_urn, bevy_index);
            map_writer.add_bevy_mapping(
                bytes_written - bevy_data_written,
                &bevy_urn,
                bevy_data_written,
            );

            // Write bevy data as ZIP member
            let data_path = uri::bevy_data_path(&image_urn, &volume_urn, bevy_index);
            zip.start_file(&data_path, stored_opts)
                .map_err(Aff4Error::Zip)?;
            zip.write_all(&bevy_result.data)
                .map_err(Aff4Error::Io)?;

            // Write bevy index as ZIP member
            let index_path = uri::bevy_index_path(&image_urn, &volume_urn, bevy_index);
            zip.start_file(&index_path, stored_opts)
                .map_err(Aff4Error::Zip)?;
            zip.write_all(&bevy_result.index)
                .map_err(Aff4Error::Io)?;

            // Write block hash files as ZIP members
            for (algo, hash_data) in &bevy_result.block_hashes {
                let hash_path = uri::bevy_block_hash_path(
                    &image_urn,
                    &volume_urn,
                    bevy_index,
                    algo.block_hash_extension(),
                );
                zip.start_file(&hash_path, stored_opts)
                    .map_err(Aff4Error::Zip)?;
                zip.write_all(hash_data)
                    .map_err(Aff4Error::Io)?;

                // Accumulate for block map hash computation
                all_block_hashes
                    .entry(*algo)
                    .or_default()
                    .push(hash_data.clone());
            }

            bevy_index += 1;

            // If we've read everything, break
            if bytes_written >= source_size {
                break;
            }
        }

        // ── 4. Write map ─────────────────────────────────────────────────────
        emit_progress(
            &mut progress_fn,
            Aff4Phase::WritingMetadata,
            bytes_written,
            source_size,
            "",
            0,
            0,
        );

        let map_result = map_writer.finish();

        let map_data_zip_path = uri::map_data_path(&image_urn, &volume_urn);
        zip.start_file(&map_data_zip_path, stored_opts)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(&map_result.map_data)
            .map_err(Aff4Error::Io)?;

        let map_idx_zip_path = uri::map_idx_path(&image_urn, &volume_urn);
        zip.start_file(&map_idx_zip_path, stored_opts)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(&map_result.map_idx)
            .map_err(Aff4Error::Io)?;

        // ── 5. Compute hashes ────────────────────────────────────────────────
        emit_progress(
            &mut progress_fn,
            Aff4Phase::ComputingHashes,
            bytes_written,
            source_size,
            "",
            0,
            0,
        );

        // Linear hashes
        let linear_hashes: HashMap<Aff4HashAlgorithm, String> =
            linear_hasher.finalize().into_iter().collect();

        // Block map hashes (hash of all concatenated block hashes per algorithm)
        let mut block_map_hashes: HashMap<Aff4HashAlgorithm, String> = HashMap::new();
        for (algo, bevy_hashes) in &all_block_hashes {
            let mut concatenated = Vec::new();
            for bh in bevy_hashes {
                concatenated.extend_from_slice(bh);
            }
            let map_point_hash = hash_chunk(&map_result.map_data, *algo);
            let map_idx_hash = hash_chunk(&map_result.map_idx, *algo);
            let bmh = crate::hashing::compute_block_map_hash(
                *algo,
                &concatenated,
                &map_point_hash,
                &map_idx_hash,
            );
            block_map_hashes.insert(*algo, hex::encode(&bmh));
        }

        // ── 6. Write information.turtle (RDF metadata) ───────────────────────
        let rdf_content = build_information_turtle(
            config,
            &volume_urn,
            &image_urn,
            source_size,
            &linear_hashes,
            &block_map_hashes,
            &map_result.map_data,
            &map_result.map_idx,
        );

        zip.start_file(INFORMATION_TURTLE, stored_opts)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(rdf_content.as_bytes())
            .map_err(Aff4Error::Io)?;

        // ── 7. Set ZIP comment and finalize ──────────────────────────────────
        emit_progress(
            &mut progress_fn,
            Aff4Phase::Finalizing,
            bytes_written,
            source_size,
            "",
            0,
            0,
        );

        zip.set_comment(volume_urn.as_str());
        let finished = zip.finish().map_err(Aff4Error::Zip)?;
        let container_bytes = finished.metadata().map(|m| m.len()).unwrap_or(0);

        let compression_ratio = if source_size > 0 {
            container_bytes as f64 / source_size as f64
        } else {
            1.0
        };

        Ok(Aff4WriteResult {
            output_path,
            volume_urn,
            image_urn,
            total_bytes: source_size,
            container_bytes,
            compression_ratio,
            bevy_count: bevy_index,
            file_count: 0, // physical mode has no logical files
            linear_hashes,
            block_map_hashes,
        })
    }
}

// ─── RDF Metadata Builder ────────────────────────────────────────────────────

/// Build the `information.turtle` content with all AFF4 metadata.
#[allow(clippy::too_many_arguments)]
fn build_information_turtle(
    config: &Aff4WriterConfig,
    volume_urn: &str,
    image_urn: &str,
    source_size: u64,
    linear_hashes: &HashMap<Aff4HashAlgorithm, String>,
    block_map_hashes: &HashMap<Aff4HashAlgorithm, String>,
    map_data: &[u8],
    map_idx: &[u8],
) -> String {
    let mut graph = RdfGraph::new();

    // Volume metadata
    graph.add(volume_urn, rdf_predicates::RDF_TYPE, rdf_types::ZIP_VOLUME);
    graph.add(
        volume_urn,
        rdf_predicates::INTERFACE,
        rdf_types::ZIP_VOLUME,
    );

    // Image (map) metadata — the image URN represents the Map object
    graph.add(image_urn, rdf_predicates::RDF_TYPE, rdf_types::MAP);
    graph.add(image_urn, rdf_predicates::RDF_TYPE, rdf_types::IMAGE);
    rdf::add_long(&mut graph, image_urn, rdf_predicates::SIZE, source_size as i64);
    rdf::add_integer(
        &mut graph,
        image_urn,
        rdf_predicates::CHUNK_SIZE,
        config.chunk_size as u64,
    );
    rdf::add_integer(
        &mut graph,
        image_urn,
        rdf_predicates::CHUNKS_PER_SEGMENT,
        config.chunks_per_segment as u64,
    );

    // Compression method
    if let Some(comp_uri) = config.compression.rdf_uri() {
        rdf::add_uri(&mut graph, image_urn, rdf_predicates::COMPRESSION_METHOD, comp_uri);
    }

    // Target stream (the image stream data is stored at the same URN path)
    rdf::add_uri(
        &mut graph,
        image_urn,
        rdf_predicates::DEPENDENT_STREAM,
        image_urn,
    );

    // Linear (whole-stream) hashes
    for (algo, hex_digest) in linear_hashes {
        rdf::add_hash(&mut graph, image_urn, rdf_predicates::STORED_HASH, algo.rdf_uri(), hex_digest);
    }

    // Map hashes
    let map_point_hash_hex = hash_hex(map_data, Aff4HashAlgorithm::Sha256);
    let map_idx_hash_hex = hash_hex(map_idx, Aff4HashAlgorithm::Sha256);
    rdf::add_hash(
        &mut graph,
        image_urn,
        rdf_predicates::MAP_POINT_HASH,
        Aff4HashAlgorithm::Sha256.rdf_uri(),
        &map_point_hash_hex,
    );
    rdf::add_hash(
        &mut graph,
        image_urn,
        rdf_predicates::MAP_IDX_HASH,
        Aff4HashAlgorithm::Sha256.rdf_uri(),
        &map_idx_hash_hex,
    );

    // Block map hashes
    for (algo, hex_digest) in block_map_hashes {
        rdf::add_hash(
            &mut graph,
            image_urn,
            rdf_predicates::BLOCK_MAP_HASH,
            algo.rdf_uri(),
            hex_digest,
        );
    }

    // Case metadata (Dublin Core)
    if !config.case_number.is_empty() {
        graph.add(
            volume_urn,
            rdf_predicates::DC_CASE_NUMBER,
            &config.case_number,
        );
    }
    if !config.evidence_number.is_empty() {
        graph.add(
            volume_urn,
            rdf_predicates::DC_EVIDENCE_NUMBER,
            &config.evidence_number,
        );
    }
    if !config.examiner.is_empty() {
        graph.add(
            volume_urn,
            rdf_predicates::DC_EXAMINER,
            &config.examiner,
        );
    }
    if !config.description.is_empty() {
        graph.add(
            volume_urn,
            rdf_predicates::DC_DESCRIPTION,
            &config.description,
        );
    }

    // Creation timestamp
    rdf::add_datetime(
        &mut graph,
        volume_urn,
        rdf_predicates::LAST_WRITTEN,
        &chrono::Utc::now().to_rfc3339(),
    );

    rdf::serialize_turtle(&graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_physical_small() {
        // Create a small source buffer (64 KiB = 2 chunks at 32 KiB)
        let source_data = vec![0xABu8; 65536];
        let source_size = source_data.len() as u64;
        let source = Cursor::new(source_data.clone());

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("test_image");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            compression: Aff4Compression::Deflate,
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunks_per_segment: DEFAULT_CHUNKS_PER_SEGMENT,
            linear_hashes: vec![Aff4HashAlgorithm::Sha256],
            block_hashes: vec![Aff4HashAlgorithm::Sha256],
            ..Default::default()
        };

        let result = Aff4Writer::write_physical(&config, source, source_size, None, None).unwrap();

        assert!(result.output_path.exists());
        assert_eq!(result.total_bytes, source_size);
        assert!(result.bevy_count >= 1);
        assert!(!result.linear_hashes.is_empty());
        assert!(!result.volume_urn.is_empty());
        assert!(!result.image_urn.is_empty());

        // Verify the output is a valid ZIP
        let file = std::fs::File::open(&result.output_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Must have container.description as a member
        {
            let mut desc = archive.by_name(CONTAINER_DESCRIPTION).unwrap();
            let mut desc_content = String::new();
            std::io::Read::read_to_string(&mut desc, &mut desc_content).unwrap();
            assert_eq!(desc_content, result.volume_urn);
        }

        // Must have version.txt
        {
            let mut ver = archive.by_name(VERSION_TXT).unwrap();
            let mut ver_content = String::new();
            std::io::Read::read_to_string(&mut ver, &mut ver_content).unwrap();
            assert!(ver_content.contains("major=1"));
            assert!(ver_content.contains("minor=0"));
        }

        // Must have information.turtle
        assert!(archive.by_name(INFORMATION_TURTLE).is_ok());
    }

    #[test]
    fn test_write_physical_cancel() {
        let source_data = vec![0u8; 1024 * 1024]; // 1 MiB
        let source = Cursor::new(source_data);

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("cancel_test");

        let cancel = AtomicBool::new(true); // Pre-cancelled

        let config = Aff4WriterConfig {
            output_path: output,
            ..Default::default()
        };

        let result = Aff4Writer::write_physical(&config, source, 1024 * 1024, Some(&cancel), None);
        assert!(matches!(result, Err(Aff4Error::Cancelled)));
    }
}
