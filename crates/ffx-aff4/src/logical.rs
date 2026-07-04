// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AFF4-L logical file collection support (AFF4-L v1.1).
//!
//! Implements logical evidence containers where individual files and directories
//! are stored rather than raw disk images. Small files (≤1 MiB) are stored as
//! ZIP members with deflate compression. Large files are stored as full bevy
//! ImageStreams with chunked compression and hashing.
//!
//! ## AFF4-L Structure
//!
//! ```text
//! container.aff4
//! ├── container.description          (volume URN)
//! ├── version.txt                    (major=1, minor=1)
//! ├── <volume_path>/
//! │   ├── <file_urn_path>            (small files as ZIP segments)
//! │   ├── <file_urn_path>/
//! │   │   ├── 00000000               (bevy data for large files)
//! │   │   ├── 00000000.index         (bevy index)
//! │   │   └── 00000000.sha256        (block hashes)
//! │   └── ...
//! └── information.turtle             (RDF metadata for all files)
//! ```

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::bevy::BevyWriter;
use crate::error::{Aff4Error, Aff4Result};
use crate::hashing::StreamHasher;
use crate::helpers::{emit_progress, ensure_aff4_extension};
use crate::rdf::{self, RdfGraph};
use crate::types::*;
use crate::uri;

// ─── Logical Writer ──────────────────────────────────────────────────────────

/// Writer for AFF4-L (v1.1) logical file collection containers.
pub struct Aff4LogicalWriter;

impl Aff4LogicalWriter {
    /// Write a logical evidence container from a set of file/directory entries.
    ///
    /// # Arguments
    ///
    /// - `config` — Writer configuration (output_path, compression, hashes, case metadata)
    /// - `entries` — Logical file entries to include
    /// - `cancel_flag` — Optional cancellation flag
    /// - `progress_fn` — Optional progress callback
    ///
    /// # Returns
    ///
    /// Write result with output path, URNs, and statistics.
    pub fn write_logical(
        config: &Aff4WriterConfig,
        entries: &mut [Aff4LogicalEntry],
        cancel_flag: Option<&AtomicBool>,
        mut progress_fn: Option<Box<dyn FnMut(Aff4Progress) + Send>>,
    ) -> Aff4Result<Aff4WriteResult> {
        if entries.is_empty() {
            return Err(Aff4Error::NoSource);
        }

        let output_path = ensure_aff4_extension(&config.output_path);
        let volume_urn = uri::new_volume_urn();

        // Calculate totals for progress
        let total_bytes: u64 = entries.iter().map(|e| e.size).sum();
        let total_files = entries.len();
        let mut bytes_processed: u64 = 0;
        let mut files_processed: usize = 0;
        let mut file_count: usize = 0;

        emit_progress(
            &mut progress_fn,
            Aff4Phase::Preparing,
            0,
            total_bytes,
            "",
            0,
            total_files,
        );

        // Create output file and ZIP writer
        let out_file = std::fs::File::create(&output_path).map_err(Aff4Error::Io)?;
        let mut zip = ZipWriter::new(out_file);
        let options = SimpleFileOptions::default().large_file(true);

        // ── 1. container.description ─────────────────────────────────────────
        zip.start_file(CONTAINER_DESCRIPTION, options)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(volume_urn.as_bytes())
            .map_err(Aff4Error::Io)?;

        // ── 2. version.txt (v1.1 for logical) ───────────────────────────────
        zip.start_file(VERSION_TXT, options)
            .map_err(Aff4Error::Zip)?;
        let version_content = format!(
            "major=1\nminor=1\ntool={}\n",
            if config.tool_name.is_empty() {
                "CORE-FFX"
            } else {
                &config.tool_name
            }
        );
        zip.write_all(version_content.as_bytes())
            .map_err(Aff4Error::Io)?;

        // ── 3. Build RDF graph ───────────────────────────────────────────────
        let mut rdf = RdfGraph::new();

        // Volume metadata
        rdf.add(&volume_urn, rdf_predicates::RDF_TYPE, rdf_types::ZIP_VOLUME);
        if !config.case_number.is_empty() {
            rdf.add(
                &volume_urn,
                rdf_predicates::DC_CASE_NUMBER,
                &config.case_number,
            );
        }
        if !config.evidence_number.is_empty() {
            rdf.add(
                &volume_urn,
                rdf_predicates::DC_EVIDENCE_NUMBER,
                &config.evidence_number,
            );
        }
        if !config.examiner.is_empty() {
            rdf.add(&volume_urn, rdf_predicates::DC_EXAMINER, &config.examiner);
        }
        if !config.description.is_empty() {
            rdf.add(
                &volume_urn,
                rdf_predicates::DC_DESCRIPTION,
                &config.description,
            );
        }

        let now = chrono::Utc::now();
        rdf::add_datetime(
            &mut rdf,
            &volume_urn,
            rdf_predicates::LAST_WRITTEN,
            &now.to_rfc3339(),
        );

        // ── 4. Write file entries ────────────────────────────────────────────
        for entry in entries.iter_mut() {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    return Err(Aff4Error::Cancelled);
                }
            }

            if entry.is_directory {
                // Directories are recorded in RDF only
                let dir_urn = file_urn(&volume_urn, &entry.original_path);
                rdf.add(&dir_urn, rdf_predicates::RDF_TYPE, rdf_types::FOLDER);
                rdf.add(&dir_urn, rdf_predicates::PATH_NAME, &entry.original_path);
                add_timestamps(&mut rdf, &dir_urn, entry);
                files_processed += 1;
                continue;
            }

            let file_urn = file_urn(&volume_urn, &entry.original_path);

            emit_progress(
                &mut progress_fn,
                Aff4Phase::WritingData,
                bytes_processed,
                total_bytes,
                &entry.original_path,
                files_processed,
                total_files,
            );

            let source_path = entry
                .source_path
                .as_ref()
                .ok_or(Aff4Error::NoSource)?
                .clone();

            if entry.size <= LOGICAL_SMALL_FILE_THRESHOLD {
                // Small file: store as ZIP segment with deflate
                write_small_file(
                    &mut zip,
                    config,
                    &mut rdf,
                    entry,
                    &file_urn,
                    &volume_urn,
                    &source_path,
                )?;
            } else {
                // Large file: store as bevy ImageStream
                write_large_file(
                    &mut zip,
                    config,
                    &mut rdf,
                    entry,
                    &file_urn,
                    &volume_urn,
                    &source_path,
                    cancel_flag,
                )?;
            }

            // Add common RDF metadata
            rdf.add(&file_urn, rdf_predicates::PATH_NAME, &entry.original_path);
            rdf::add_long(&mut rdf, &file_urn, rdf_predicates::SIZE, entry.size as i64);
            add_timestamps(&mut rdf, &file_urn, entry);

            // Add per-file hashes to RDF
            for (algo, digest) in &entry.hashes {
                rdf::add_hash(
                    &mut rdf,
                    &file_urn,
                    rdf_predicates::STORED_HASH,
                    algo.rdf_uri(),
                    digest,
                );
            }

            bytes_processed += entry.size;
            files_processed += 1;
            file_count += 1;
        }

        // ── 5. information.turtle ────────────────────────────────────────────
        emit_progress(
            &mut progress_fn,
            Aff4Phase::WritingMetadata,
            bytes_processed,
            total_bytes,
            "",
            files_processed,
            total_files,
        );

        let turtle = rdf::serialize_turtle(&rdf);
        zip.start_file(INFORMATION_TURTLE, options)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(turtle.as_bytes()).map_err(Aff4Error::Io)?;

        // ── 6. Finalize ──────────────────────────────────────────────────────
        emit_progress(
            &mut progress_fn,
            Aff4Phase::Finalizing,
            total_bytes,
            total_bytes,
            "",
            files_processed,
            total_files,
        );

        zip.set_comment(volume_urn.as_str());
        zip.finish().map_err(Aff4Error::Zip)?;

        // Collect all per-file hashes into result
        let all_hashes: HashMap<Aff4HashAlgorithm, String> = HashMap::new();
        // Linear hashes are per-file for logical containers;
        // we report the count of hashed files.

        Ok(Aff4WriteResult {
            output_path: output_path.clone(),
            volume_urn,
            image_urn: String::new(), // No single image URN for logical
            total_bytes: bytes_processed,
            container_bytes: std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0),
            compression_ratio: if bytes_processed > 0 {
                let container_bytes = std::fs::metadata(&output_path)
                    .map(|m| m.len())
                    .unwrap_or(bytes_processed);
                container_bytes as f64 / bytes_processed as f64
            } else {
                1.0
            },
            bevy_count: 0,
            file_count,
            linear_hashes: all_hashes,
            block_map_hashes: HashMap::new(),
        })
    }
}

// ─── Small File Writer (ZIP Segment) ─────────────────────────────────────────

/// Write a small file as a ZIP segment with deflate compression and per-file hashing.
fn write_small_file<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    config: &Aff4WriterConfig,
    rdf: &mut RdfGraph,
    entry: &mut Aff4LogicalEntry,
    file_urn: &str,
    volume_urn: &str,
    source_path: &Path,
) -> Aff4Result<()> {
    // Read the snapshotted file length. Do not chase bytes appended after
    // enumeration, and do not accept sources that became shorter.
    let mut source_file = std::fs::File::open(source_path).map_err(Aff4Error::Io)?;
    let mut data = Vec::with_capacity(entry.size as usize);
    Read::by_ref(&mut source_file)
        .take(entry.size)
        .read_to_end(&mut data)
        .map_err(Aff4Error::Io)?;
    validate_logical_entry_size(source_path, entry.size, data.len() as u64)?;

    // Compute per-file hashes
    let hash_algos: Vec<Aff4HashAlgorithm> = if config.linear_hashes.is_empty() {
        vec![Aff4HashAlgorithm::Sha256, Aff4HashAlgorithm::Md5]
    } else {
        config.linear_hashes.clone()
    };

    let mut hasher = StreamHasher::new(&hash_algos);
    hasher.update(&data);
    let hashes: HashMap<Aff4HashAlgorithm, String> = hasher.finalize().into_iter().collect();
    entry.hashes = hashes;

    // Store as ZIP segment at <volume_path>/<file_urn_path>
    let volume_path = uri::urn_to_zip_path(volume_urn, volume_urn);
    let zip_path = uri::urn_to_zip_path(file_urn, volume_urn);
    let zip_member = if zip_path.starts_with(&format!("{}/", volume_path)) {
        zip_path.clone()
    } else {
        format!("{}/{}", volume_path, zip_path)
    };

    let zip_compression = if matches!(config.compression, Aff4Compression::Stored) {
        zip::CompressionMethod::Stored
    } else {
        zip::CompressionMethod::Deflated
    };

    let options = SimpleFileOptions::default()
        .compression_method(zip_compression)
        .large_file(true);

    zip.start_file(&zip_member, options)
        .map_err(Aff4Error::Zip)?;
    zip.write_all(&data).map_err(Aff4Error::Io)?;

    // RDF: mark as FileImage stored as ZIP segment
    rdf.add(file_urn, rdf_predicates::RDF_TYPE, rdf_types::FILE_IMAGE);
    rdf.add(file_urn, rdf_predicates::RDF_TYPE, rdf_types::ZIP_SEGMENT);

    Ok(())
}

fn validate_logical_entry_size(source_path: &Path, expected: u64, actual: u64) -> Aff4Result<()> {
    if actual == expected {
        return Ok(());
    }

    Err(Aff4Error::WriteError(format!(
        "AFF4 logical export incomplete for {}: expected {} bytes from snapshot, processed {} bytes",
        source_path.display(),
        expected,
        actual
    )))
}

// ─── Large File Writer (Bevy ImageStream) ────────────────────────────────────

/// Write a large file as a full bevy ImageStream with chunked compression.
#[allow(clippy::too_many_arguments)]
fn write_large_file<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    config: &Aff4WriterConfig,
    rdf: &mut RdfGraph,
    entry: &mut Aff4LogicalEntry,
    file_urn: &str,
    volume_urn: &str,
    source_path: &Path,
    cancel_flag: Option<&AtomicBool>,
) -> Aff4Result<()> {
    let chunk_size = config.chunk_size as usize;
    let chunks_per_segment = config.chunks_per_segment;

    let block_hashes: Vec<Aff4HashAlgorithm> = if config.block_hashes.is_empty() {
        vec![Aff4HashAlgorithm::Sha256]
    } else {
        config.block_hashes.clone()
    };

    let linear_algos: Vec<Aff4HashAlgorithm> = if config.linear_hashes.is_empty() {
        vec![Aff4HashAlgorithm::Sha256, Aff4HashAlgorithm::Md5]
    } else {
        config.linear_hashes.clone()
    };

    let mut linear_hasher = StreamHasher::new(&linear_algos);
    let mut source_file = std::fs::File::open(source_path).map_err(Aff4Error::Io)?;

    let options = SimpleFileOptions::default().large_file(true);
    let mut bevy_index: u32 = 0;
    let mut bevy_writer = BevyWriter::new(config.compression, chunk_size as u32, &block_hashes);
    let mut chunk_buf = vec![0u8; chunk_size];
    let mut chunks_in_current = 0u32;
    let mut total_bytes_read: u64 = 0;
    let mut bevy_data_written: u64 = 0;
    let mut map_writer = crate::map::MapWriter::new();

    while total_bytes_read < entry.size {
        if let Some(flag) = cancel_flag {
            if flag.load(Ordering::Relaxed) {
                return Err(Aff4Error::Cancelled);
            }
        }

        let remaining = entry.size - total_bytes_read;
        let to_read = chunk_size.min(remaining as usize);
        source_file
            .read_exact(&mut chunk_buf[..to_read])
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Aff4Error::WriteError(format!(
                        "AFF4 logical export incomplete for {}: expected {} bytes from snapshot, processed {} bytes",
                        source_path.display(),
                        entry.size,
                        total_bytes_read
                    ))
                } else {
                    Aff4Error::Io(e)
                }
            })?;

        let chunk = &chunk_buf[..to_read];
        linear_hasher.update(chunk);
        bevy_writer.add_chunk(chunk)?;
        chunks_in_current += 1;
        bevy_data_written += to_read as u64;
        total_bytes_read += to_read as u64;

        // Flush bevy when segment is full
        if chunks_in_current >= chunks_per_segment {
            flush_bevy(zip, bevy_writer, file_urn, volume_urn, bevy_index, &options)?;

            // Map: register bevy mapping (image_offset, bevy_urn, bevy_size)
            let bevy_urn = format!("{}/{:08x}", file_urn, bevy_index);
            map_writer.add_bevy_mapping(
                total_bytes_read - bevy_data_written,
                &bevy_urn,
                bevy_data_written,
            );

            bevy_index += 1;
            chunks_in_current = 0;
            bevy_data_written = 0;
            bevy_writer = BevyWriter::new(config.compression, chunk_size as u32, &block_hashes);
        }
    }

    validate_logical_entry_size(source_path, entry.size, total_bytes_read)?;

    // Flush remaining bevy data
    if chunks_in_current > 0 {
        flush_bevy(zip, bevy_writer, file_urn, volume_urn, bevy_index, &options)?;

        let bevy_urn = format!("{}/{:08x}", file_urn, bevy_index);
        map_writer.add_bevy_mapping(
            total_bytes_read - bevy_data_written,
            &bevy_urn,
            bevy_data_written,
        );
    }

    // Write map if we have multiple bevies
    if bevy_index > 0 || total_bytes_read > 0 {
        let map_result = map_writer.finish();
        if !map_result.map_data.is_empty() {
            let map_path = uri::map_data_path(file_urn, volume_urn);
            zip.start_file(&map_path, options).map_err(Aff4Error::Zip)?;
            zip.write_all(&map_result.map_data).map_err(Aff4Error::Io)?;

            let idx_path = uri::map_idx_path(file_urn, volume_urn);
            zip.start_file(&idx_path, options).map_err(Aff4Error::Zip)?;
            zip.write_all(&map_result.map_idx).map_err(Aff4Error::Io)?;
        }
    }

    // Finalize linear hashes
    let file_hashes: HashMap<Aff4HashAlgorithm, String> =
        linear_hasher.finalize().into_iter().collect();
    entry.hashes = file_hashes;

    // RDF: mark as FileImage + ImageStream
    rdf.add(file_urn, rdf_predicates::RDF_TYPE, rdf_types::FILE_IMAGE);
    rdf.add(file_urn, rdf_predicates::RDF_TYPE, rdf_types::IMAGE_STREAM);
    rdf::add_integer(rdf, file_urn, rdf_predicates::CHUNK_SIZE, chunk_size as u64);
    rdf::add_integer(
        rdf,
        file_urn,
        rdf_predicates::CHUNKS_PER_SEGMENT,
        chunks_per_segment as u64,
    );
    if let Some(comp_uri) = config.compression.rdf_uri() {
        rdf::add_uri(rdf, file_urn, rdf_predicates::COMPRESSION_METHOD, comp_uri);
    }

    Ok(())
}

/// Flush a completed bevy to the ZIP archive.
fn flush_bevy<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    bevy_writer: BevyWriter,
    stream_urn: &str,
    volume_urn: &str,
    bevy_index: u32,
    options: &SimpleFileOptions,
) -> Aff4Result<()> {
    let result = bevy_writer.finish();

    // Write bevy data
    let data_path = uri::bevy_data_path(stream_urn, volume_urn, bevy_index);
    zip.start_file(&data_path, *options)
        .map_err(Aff4Error::Zip)?;
    zip.write_all(&result.data).map_err(Aff4Error::Io)?;

    // Write bevy index
    let index_path = uri::bevy_index_path(stream_urn, volume_urn, bevy_index);
    zip.start_file(&index_path, *options)
        .map_err(Aff4Error::Zip)?;
    zip.write_all(&result.index).map_err(Aff4Error::Io)?;

    // Write block hash files
    for (algo, hash_data) in &result.block_hashes {
        let ext = algo.block_hash_extension();
        let hash_path = uri::bevy_block_hash_path(stream_urn, volume_urn, bevy_index, ext);
        zip.start_file(&hash_path, *options)
            .map_err(Aff4Error::Zip)?;
        zip.write_all(hash_data).map_err(Aff4Error::Io)?;
    }

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Generate a URN for a logical file within a volume.
fn file_urn(volume_urn: &str, relative_path: &str) -> String {
    // AFF4-L files are named as: <volume_urn>/<path>
    // But since volume URN is aff4://<uuid>, the file URN is aff4://<uuid>/<path>
    let clean_path = relative_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    format!("{}/{}", volume_urn, clean_path)
}

/// Add timestamp metadata to RDF for an entry.
fn add_timestamps(rdf: &mut RdfGraph, urn: &str, entry: &Aff4LogicalEntry) {
    if let Some(ts) = entry.last_written {
        let dt = chrono::DateTime::from_timestamp_nanos(ts);
        rdf::add_datetime(rdf, urn, rdf_predicates::LAST_WRITTEN, &dt.to_rfc3339());
    }
    if let Some(ts) = entry.last_accessed {
        let dt = chrono::DateTime::from_timestamp_nanos(ts);
        rdf::add_datetime(rdf, urn, rdf_predicates::LAST_ACCESSED, &dt.to_rfc3339());
    }
    if let Some(ts) = entry.record_changed {
        let dt = chrono::DateTime::from_timestamp_nanos(ts);
        rdf::add_datetime(rdf, urn, rdf_predicates::RECORD_CHANGED, &dt.to_rfc3339());
    }
    if let Some(ts) = entry.birth_time {
        let dt = chrono::DateTime::from_timestamp_nanos(ts);
        rdf::add_datetime(rdf, urn, rdf_predicates::BIRTH_TIME, &dt.to_rfc3339());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::Aff4Reader;

    #[test]
    fn test_write_logical_small_files() {
        let dir = tempfile::tempdir().unwrap();

        // Create test source files
        let file1_path = dir.path().join("test1.txt");
        std::fs::write(&file1_path, "Hello, AFF4-L!").unwrap();

        let file2_path = dir.path().join("test2.txt");
        std::fs::write(&file2_path, "Second file content").unwrap();

        let output = dir.path().join("logical_test");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            compression: Aff4Compression::Deflate,
            version: Aff4Version::Logical,
            case_number: "LOG-001".to_string(),
            examiner: "Logical Examiner".to_string(),
            ..Default::default()
        };

        let mut entries = vec![
            Aff4LogicalEntry::from_source(file1_path, "documents/test1.txt".to_string()),
            Aff4LogicalEntry::from_source(file2_path, "documents/test2.txt".to_string()),
        ];

        let result = Aff4LogicalWriter::write_logical(&config, &mut entries, None, None).unwrap();

        assert!(result.output_path.exists());
        assert_eq!(result.file_count, 2);

        // Verify per-file hashes were computed
        assert!(!entries[0].hashes.is_empty(), "Entry 0 should have hashes");
        assert!(!entries[1].hashes.is_empty(), "Entry 1 should have hashes");

        // Open and verify
        let reader = Aff4Reader::open(&result.output_path).unwrap();
        let info = reader.info();
        assert_eq!(info.version, Aff4Version::Logical);
        assert_eq!(info.case_number, "LOG-001");
    }

    #[test]
    fn test_write_logical_with_directory() {
        let dir = tempfile::tempdir().unwrap();

        let file_path = dir.path().join("readme.md");
        std::fs::write(&file_path, "# Hello").unwrap();

        let output = dir.path().join("dir_test");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            version: Aff4Version::Logical,
            ..Default::default()
        };

        let mut entries = vec![
            Aff4LogicalEntry {
                original_path: "documents".to_string(),
                size: 0,
                is_directory: true,
                source_path: None,
                last_written: None,
                last_accessed: None,
                record_changed: None,
                birth_time: None,
                hashes: HashMap::new(),
            },
            Aff4LogicalEntry::from_source(file_path, "documents/readme.md".to_string()),
        ];

        let result = Aff4LogicalWriter::write_logical(&config, &mut entries, None, None).unwrap();

        assert!(result.output_path.exists());
        assert_eq!(result.file_count, 1); // Only files, not directories

        // Verify RDF has folder entry
        let reader = Aff4Reader::open(&result.output_path).unwrap();
        let rdf = reader.rdf();
        // The directory URN should have a FOLDER type
        let folder_subjects = rdf.subjects_with_type(rdf_types::FOLDER);
        assert!(!folder_subjects.is_empty(), "Should have a folder in RDF");
    }

    #[test]
    fn test_file_urn_generation() {
        let volume = "aff4://12345678-1234-1234-1234-123456789abc";
        let urn = file_urn(volume, "documents/report.pdf");
        assert_eq!(
            urn,
            "aff4://12345678-1234-1234-1234-123456789abc/documents/report.pdf"
        );
    }

    #[test]
    fn test_file_urn_backslash_normalization() {
        let volume = "aff4://uuid-here";
        let urn = file_urn(volume, "path\\to\\file.txt");
        assert_eq!(urn, "aff4://uuid-here/path/to/file.txt");
    }

    #[test]
    fn test_logical_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("data.bin");
        std::fs::write(&file_path, vec![0u8; 1024]).unwrap();

        let output = dir.path().join("cancel_test");

        let config = Aff4WriterConfig {
            output_path: output.clone(),
            ..Default::default()
        };

        let mut entries = vec![Aff4LogicalEntry::from_source(
            file_path,
            "data.bin".to_string(),
        )];

        let cancel = AtomicBool::new(true);
        let result = Aff4LogicalWriter::write_logical(&config, &mut entries, Some(&cancel), None);

        assert!(result.is_err());
        match result.unwrap_err() {
            Aff4Error::Cancelled => {}
            other => panic!("Expected Cancelled, got: {:?}", other),
        }
    }

    #[test]
    fn test_logical_small_file_rejects_short_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("small.txt");
        std::fs::write(&file_path, b"snapshot").unwrap();
        let mut entries = vec![Aff4LogicalEntry::from_source(
            file_path.clone(),
            "small.txt".to_string(),
        )];
        std::fs::write(&file_path, b"short").unwrap();

        let config = Aff4WriterConfig {
            output_path: dir.path().join("small-short"),
            ..Default::default()
        };

        let result = Aff4LogicalWriter::write_logical(&config, &mut entries, None, None);

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("AFF4 logical export incomplete"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_logical_small_file_caps_grown_snapshot() {
        use std::io::Read as _;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("small.txt");
        std::fs::write(&file_path, b"orig").unwrap();
        let mut entries = vec![Aff4LogicalEntry::from_source(
            file_path.clone(),
            "small.txt".to_string(),
        )];
        std::fs::write(&file_path, b"orig-extra").unwrap();

        let config = Aff4WriterConfig {
            output_path: dir.path().join("small-grown"),
            compression: Aff4Compression::Stored,
            ..Default::default()
        };

        let result = Aff4LogicalWriter::write_logical(&config, &mut entries, None, None).unwrap();
        assert_eq!(result.total_bytes, 4);

        let mut archive =
            zip::ZipArchive::new(std::fs::File::open(&result.output_path).unwrap()).unwrap();
        let file_urn = file_urn(&result.volume_urn, "small.txt");
        let volume_path = uri::urn_to_zip_path(&result.volume_urn, &result.volume_urn);
        let zip_path = uri::urn_to_zip_path(&file_urn, &result.volume_urn);
        let zip_member = if zip_path.starts_with(&format!("{}/", volume_path)) {
            zip_path
        } else {
            format!("{}/{}", volume_path, zip_path)
        };
        let mut member = archive.by_name(&zip_member).unwrap();
        let mut data = Vec::new();
        member.read_to_end(&mut data).unwrap();
        assert_eq!(data, b"orig");
    }

    #[test]
    fn test_logical_large_file_rejects_short_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large.bin");
        std::fs::write(
            &file_path,
            vec![0x42; (LOGICAL_SMALL_FILE_THRESHOLD + 1024) as usize],
        )
        .unwrap();
        let mut entries = vec![Aff4LogicalEntry::from_source(
            file_path.clone(),
            "large.bin".to_string(),
        )];
        std::fs::write(&file_path, vec![0x42; 1024]).unwrap();

        let config = Aff4WriterConfig {
            output_path: dir.path().join("large-short"),
            chunk_size: 4096,
            chunks_per_segment: 2,
            ..Default::default()
        };

        let result = Aff4LogicalWriter::write_logical(&config, &mut entries, None, None);

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("AFF4 logical export incomplete"),
            "unexpected error: {}",
            err
        );
    }
}
