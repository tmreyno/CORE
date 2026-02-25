// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Pure-Rust L01 (Logical Evidence File) writer.
//!
//! Creates EnCase-compatible L01 logical evidence containers from
//! source files on disk. Supports:
//!
//! - Hierarchical file/directory structures
//! - zlib chunk compression (none / fast / best)
//! - MD5 and SHA-1 image integrity hashes
//! - Per-file MD5/SHA-1 hashes
//! - Multi-segment output (.L01, .L02, ... .L99, .LAA, ...)
//! - Case metadata (case number, evidence number, examiner, etc.)
//! - Source tracking and permission groups
//! - Cancellation support via `AtomicBool`
//!
//! # Architecture
//!
//! ```text
//! L01Writer
//!   ├── types.rs     — Data types, constants, Adler-32
//!   ├── ltree.rs     — UTF-16LE ltree text builder (5 categories)
//!   ├── chunks.rs    — zlib chunk compression
//!   ├── sections.rs  — EWF v1 section writers
//!   └── segment.rs   — Multi-segment file support
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use ffx_check_lib::l01_writer::{L01Writer, L01WriterConfig, L01CaseInfo};
//! use std::path::PathBuf;
//!
//! let config = L01WriterConfig {
//!     output_path: PathBuf::from("/output/evidence.L01"),
//!     case_info: L01CaseInfo {
//!         case_number: "2024-001".into(),
//!         evidence_number: "E001".into(),
//!         description: "Logical acquisition".into(),
//!         examiner: "Forensic Examiner".into(),
//!         notes: String::new(),
//!     },
//!     ..Default::default()
//! };
//!
//! let mut writer = L01Writer::new(config);
//! writer.add_source_directory(&PathBuf::from("/evidence/files")).unwrap();
//! let result = writer.write(None, None).unwrap();
//! ```

pub mod chunks;
pub mod ltree;
pub mod sections;
pub mod segment;
pub mod types;

pub use types::*;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use md5::Digest as Md5Digest;

/// Pure-Rust L01 logical evidence file writer.
///
/// Build a file tree with `add_file()` / `add_directory()` / `add_source_directory()`,
/// then call `write()` to produce the L01 output.
pub struct L01Writer {
    config: L01WriterConfig,
    entries: Vec<LefFileEntry>,
    sources: Vec<LefSource>,
    permission_groups: Vec<LefPermissionGroup>,
    subjects: Vec<LefSubject>,
    next_identifier: u64,
}

impl L01Writer {
    /// Create a new L01 writer with the given configuration.
    pub fn new(config: L01WriterConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            sources: Vec::new(),
            permission_groups: Vec::new(),
            subjects: Vec::new(),
            next_identifier: 1,
        }
    }

    /// Allocate and return the next unique entry identifier.
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_identifier;
        self.next_identifier += 1;
        id
    }

    // ── File/Directory Registration ─────────────────────────────────────

    /// Add a single file entry.
    ///
    /// Returns the assigned identifier.
    pub fn add_file(
        &mut self,
        name: String,
        size: u64,
        source_path: PathBuf,
        parent_id: u64,
    ) -> u64 {
        let id = self.alloc_id();
        let entry = LefFileEntry::new_file(id, name, size)
            .with_parent(parent_id)
            .with_source_path(source_path);
        self.entries.push(entry);
        id
    }

    /// Add a directory entry.
    ///
    /// Returns the assigned identifier.
    pub fn add_directory(&mut self, name: String, parent_id: u64) -> u64 {
        let id = self.alloc_id();
        let entry = LefFileEntry::new_directory(id, name).with_parent(parent_id);
        self.entries.push(entry);
        id
    }

    /// Add a pre-built file entry directly.
    pub fn add_entry(&mut self, mut entry: LefFileEntry) {
        if entry.identifier == 0 {
            entry.identifier = self.alloc_id();
        } else if entry.identifier >= self.next_identifier {
            self.next_identifier = entry.identifier + 1;
        }
        self.entries.push(entry);
    }

    /// Add an acquisition source.
    pub fn add_source(&mut self, source: LefSource) {
        self.sources.push(source);
    }

    /// Add a permission group.
    pub fn add_permission_group(&mut self, group: LefPermissionGroup) {
        self.permission_groups.push(group);
    }

    /// Add a subject.
    pub fn add_subject(&mut self, subject: LefSubject) {
        self.subjects.push(subject);
    }

    /// Recursively add all files and directories from a source directory.
    ///
    /// Preserves the directory hierarchy. Sets timestamps from filesystem
    /// metadata. Returns the number of entries added.
    pub fn add_source_directory(&mut self, dir_path: &Path) -> Result<usize, L01WriteError> {
        if !dir_path.exists() {
            return Err(L01WriteError::FileNotFound(
                dir_path.to_string_lossy().to_string(),
            ));
        }

        // Add a source for this directory
        let source_id = if self.sources.is_empty() {
            let sid = self.sources.len() as u64 + 1;
            self.sources.push(LefSource::from_path(sid, dir_path));
            sid
        } else {
            self.sources[0].identifier
        };

        let count = self.walk_directory(dir_path, 0, source_id)?;
        Ok(count)
    }

    /// Recursively walk a directory, adding entries.
    fn walk_directory(
        &mut self,
        dir_path: &Path,
        parent_id: u64,
        source_id: u64,
    ) -> Result<usize, L01WriteError> {
        let mut count = 0;

        let mut dir_entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir_path)
            .map_err(|e| L01WriteError::SourceReadError {
                path: dir_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?
            .filter_map(|e| e.ok())
            .collect();

        // Sort for deterministic output
        dir_entries.sort_by_key(|e| e.file_name());

        for dir_entry in dir_entries {
            let path = dir_entry.path();
            let file_name = dir_entry
                .file_name()
                .to_string_lossy()
                .to_string();

            // Skip hidden files/directories (starting with .)
            if file_name.starts_with('.') {
                continue;
            }

            let metadata = dir_entry.metadata().map_err(|e| {
                L01WriteError::SourceReadError {
                    path: path.to_string_lossy().to_string(),
                    reason: e.to_string(),
                }
            })?;

            if metadata.is_dir() {
                let dir_id = self.alloc_id();
                let mut entry =
                    LefFileEntry::new_directory(dir_id, file_name)
                        .with_parent(parent_id)
                        .with_source(source_id)
                        .with_source_path(path.clone());

                // Set timestamps
                set_timestamps_from_metadata(&mut entry, &metadata);

                self.entries.push(entry);
                count += 1;

                // Recurse
                count += self.walk_directory(&path, dir_id, source_id)?;
            } else if metadata.is_file() {
                let file_id = self.alloc_id();
                let size = metadata.len();
                let mut entry = LefFileEntry::new_file(file_id, file_name, size)
                    .with_parent(parent_id)
                    .with_source(source_id)
                    .with_source_path(path);

                set_timestamps_from_metadata(&mut entry, &metadata);

                self.entries.push(entry);
                count += 1;
            }
            // Skip symlinks, special files
        }

        Ok(count)
    }

    /// Get the total number of entries (files + directories).
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get the total size of all file entries.
    pub fn total_file_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.record_type == LefRecordType::File)
            .map(|e| e.size)
            .sum()
    }

    // ── Write Pipeline ──────────────────────────────────────────────────

    /// Write the L01 file.
    ///
    /// # Arguments
    /// - `cancel_flag` — Optional atomic flag to cancel the operation
    /// - `progress_fn` — Optional callback for progress reporting
    ///
    /// # Returns
    /// The write result containing output paths, statistics, and hashes.
    pub fn write(
        &mut self,
        cancel_flag: Option<&AtomicBool>,
        mut progress_fn: Option<Box<dyn FnMut(L01WriteProgress) + Send>>,
    ) -> Result<L01WriteResult, L01WriteError> {
        // Validation
        if self.config.output_path.as_os_str().is_empty() {
            return Err(L01WriteError::NoOutputPath);
        }
        if self.entries.is_empty() {
            return Err(L01WriteError::NoFiles);
        }
        if self.config.segment_size > 0 && self.config.segment_size < 1_048_576 {
            return Err(L01WriteError::SegmentTooSmall(self.config.segment_size));
        }

        let file_entries: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.record_type == LefRecordType::File)
            .map(|(i, _)| i)
            .collect();

        let total_files = file_entries.len();
        let total_dirs = self.entries.len() - total_files;
        let total_bytes: u64 = file_entries
            .iter()
            .map(|&i| self.entries[i].size)
            .sum();

        // ── Phase 1: Emit preparing progress ──
        emit_progress(
            &mut progress_fn,
            &self.config.output_path,
            "",
            0,
            total_files,
            0,
            total_bytes,
            L01WritePhase::Preparing,
        );

        // ── Phase 2: Compress file data and build chunk tables ──
        let chunk_size = (self.config.block_size as usize)
            * (self.config.sectors_per_chunk as usize);

        let mut all_compressed_data = Vec::new();
        let mut all_chunk_tables = Vec::new();
        let mut image_md5 = md5::Md5::new();
        let mut image_sha1 = sha1::Sha1::new();
        let mut bytes_written: u64 = 0;
        let mut data_offset: u64 = 0;

        for (file_idx, &entry_idx) in file_entries.iter().enumerate() {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    return Err(L01WriteError::Cancelled);
                }
            }

            let source_path = self.entries[entry_idx]
                .source_path
                .as_ref()
                .ok_or_else(|| {
                    L01WriteError::SourceReadError {
                        path: self.entries[entry_idx].name.clone(),
                        reason: "No source path set".to_string(),
                    }
                })?
                .clone();

            if !source_path.exists() {
                return Err(L01WriteError::FileNotFound(
                    source_path.to_string_lossy().to_string(),
                ));
            }

            let file_size = self.entries[entry_idx].size;

            emit_progress(
                &mut progress_fn,
                &self.config.output_path,
                &self.entries[entry_idx].name,
                file_idx,
                total_files,
                bytes_written,
                total_bytes,
                L01WritePhase::WritingData,
            );

            // Read and compress file data
            let mut file = BufReader::new(File::open(&source_path).map_err(|e| {
                L01WriteError::SourceReadError {
                    path: source_path.to_string_lossy().to_string(),
                    reason: e.to_string(),
                }
            })?);

            let (compressed, table) = chunks::compress_from_reader(
                &mut file,
                file_size,
                chunk_size,
                self.config.compression_level,
                data_offset,
                None,
            )?;

            // Compute per-file hashes
            let mut file_for_hash =
                BufReader::new(File::open(&source_path).map_err(|e| {
                    L01WriteError::SourceReadError {
                        path: source_path.to_string_lossy().to_string(),
                        reason: e.to_string(),
                    }
                })?);

            let (file_md5, file_sha1) = compute_file_hashes(&mut file_for_hash)?;
            self.entries[entry_idx].md5_hash = Some(hex::encode(&file_md5));
            self.entries[entry_idx].sha1_hash = Some(hex::encode(&file_sha1));

            // Update image-level hashes with the raw file data
            let mut file_for_img =
                BufReader::new(File::open(&source_path).map_err(|e| {
                    L01WriteError::SourceReadError {
                        path: source_path.to_string_lossy().to_string(),
                        reason: e.to_string(),
                    }
                })?);
            hash_reader_into(&mut file_for_img, &mut image_md5, &mut image_sha1)?;

            // Set data offset and size on entry
            self.entries[entry_idx].data_offset = data_offset;
            self.entries[entry_idx].data_size = file_size;

            data_offset += compressed.len() as u64;
            bytes_written += file_size;

            all_compressed_data.push(compressed);
            all_chunk_tables.push(table);
        }

        // ── Phase 3: Merge chunk data and tables ──
        emit_progress(
            &mut progress_fn,
            &self.config.output_path,
            "",
            total_files,
            total_files,
            bytes_written,
            total_bytes,
            L01WritePhase::BuildingTables,
        );

        let mut merged_compressed = Vec::new();
        let mut merged_table = ChunkTable::new(0); // base_offset set later

        for (compressed, table) in all_compressed_data
            .into_iter()
            .zip(all_chunk_tables.into_iter())
        {
            let base = merged_compressed.len() as u64;
            for chunk in &table.chunks {
                merged_table.add_chunk(
                    base + chunk.offset,
                    chunk.compressed_size,
                    chunk.is_compressed,
                );
            }
            merged_compressed.extend(compressed);
        }

        let total_compressed_bytes = merged_compressed.len() as u64;

        // ── Phase 4: Build ltree ──
        emit_progress(
            &mut progress_fn,
            &self.config.output_path,
            "",
            total_files,
            total_files,
            bytes_written,
            total_bytes,
            L01WritePhase::WritingLtree,
        );

        let ltree_text = ltree::build_ltree_text(
            &self.entries,
            &self.sources,
            &self.permission_groups,
            &self.subjects,
            total_bytes,
        );
        let ltree_utf16 = ltree::utf8_to_utf16le(&ltree_text);

        // ── Phase 5: Finalize hashes ──
        emit_progress(
            &mut progress_fn,
            &self.config.output_path,
            "",
            total_files,
            total_files,
            bytes_written,
            total_bytes,
            L01WritePhase::ComputingHash,
        );

        let image_md5_result = image_md5.finalize();
        let image_sha1_result = image_sha1.finalize();

        let mut md5_bytes = [0u8; 16];
        md5_bytes.copy_from_slice(&image_md5_result);

        let mut sha1_bytes = [0u8; 20];
        sha1_bytes.copy_from_slice(&image_sha1_result);

        // ── Phase 6: Write the L01 file(s) ──
        emit_progress(
            &mut progress_fn,
            &self.config.output_path,
            "",
            total_files,
            total_files,
            bytes_written,
            total_bytes,
            L01WritePhase::Finalizing,
        );

        let output_path = self.config.output_path.clone();

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Pre-compute metadata sections that are needed for size calculations
        let header_compressed = compress_text(
            build_header_text_static(&self.config.case_info).as_bytes(),
        )?;
        let header_section_size =
            SECTION_HEADER_SIZE as u64 + header_compressed.len() as u64;

        let header2_utf16 =
            ltree::utf8_to_utf16le(&build_header_text_static(&self.config.case_info));
        let header2_compressed = compress_text(&header2_utf16)?;
        let header2_section_size =
            SECTION_HEADER_SIZE as u64 + header2_compressed.len() as u64;

        let volume_section_size =
            (SECTION_HEADER_SIZE + VOLUME_DATA_SIZE) as u64;

        let ltypes_section_size = SECTION_HEADER_SIZE as u64 + 4;

        let ltree_compressed = compress_text(&ltree_utf16)?;
        let ltree_section_size =
            SECTION_HEADER_SIZE as u64 + 48 + ltree_compressed.len() as u64;

        let data_section_size =
            (SECTION_HEADER_SIZE + DATA_SECTION_DATA_SIZE) as u64;

        let hash_section_size =
            (SECTION_HEADER_SIZE + HASH_SECTION_DATA_SIZE) as u64;

        let digest_section_size =
            (SECTION_HEADER_SIZE + DIGEST_SECTION_DATA_SIZE) as u64;

        let done_section_size = SECTION_HEADER_SIZE as u64;
        let next_section_size = SECTION_HEADER_SIZE as u64;

        // Determine if multi-segment is needed
        let segment_size = self.config.segment_size;
        let use_segments = segment_size > 0;

        if use_segments {
            // ── Multi-segment write path ──
            self.write_multi_segment(
                &output_path,
                &merged_compressed,
                &merged_table,
                &header_compressed,
                header_section_size,
                &header2_compressed,
                header2_section_size,
                volume_section_size,
                &ltree_utf16,
                &ltree_compressed,
                ltypes_section_size,
                ltree_section_size,
                data_section_size,
                hash_section_size,
                digest_section_size,
                done_section_size,
                next_section_size,
                &md5_bytes,
                &sha1_bytes,
                total_bytes,
                total_compressed_bytes,
                total_files,
                total_dirs,
            )
        } else {
            // ── Single-segment write path (original fast path) ──
            self.write_single_segment(
                &output_path,
                &merged_compressed,
                &mut merged_table,
                &header_compressed,
                header_section_size,
                &header2_compressed,
                header2_section_size,
                volume_section_size,
                &ltree_utf16,
                &ltree_compressed,
                ltypes_section_size,
                ltree_section_size,
                data_section_size,
                hash_section_size,
                digest_section_size,
                done_section_size,
                &md5_bytes,
                &sha1_bytes,
                total_bytes,
                total_compressed_bytes,
                total_files,
                total_dirs,
            )
        }
    }

    /// Write all data to a single L01 file (no segment splitting).
    #[allow(clippy::too_many_arguments)]
    fn write_single_segment(
        &self,
        output_path: &std::path::Path,
        merged_compressed: &[u8],
        merged_table: &mut ChunkTable,
        header_compressed: &[u8],
        header_section_size: u64,
        header2_compressed: &[u8],
        header2_section_size: u64,
        volume_section_size: u64,
        ltree_utf16: &[u8],
        ltree_compressed: &[u8],
        ltypes_section_size: u64,
        ltree_section_size: u64,
        data_section_size: u64,
        hash_section_size: u64,
        digest_section_size: u64,
        done_section_size: u64,
        md5_bytes: &[u8; 16],
        sha1_bytes: &[u8; 20],
        total_bytes: u64,
        total_compressed_bytes: u64,
        total_files: usize,
        total_dirs: usize,
    ) -> Result<L01WriteResult, L01WriteError> {
        let sectors_section_size =
            SECTION_HEADER_SIZE as u64 + merged_compressed.len() as u64;

        let table_entries_size =
            merged_table.chunk_count() as u64 * 4 + 4;
        let table_section_size =
            SECTION_HEADER_SIZE as u64 + 24 + table_entries_size;
        let table2_section_size = table_section_size;

        // Build next_offset chain
        let mut pos: u64 = FILE_HEADER_SIZE as u64;
        let mut section_offsets = Vec::new();

        // header
        section_offsets.push(pos);
        pos += header_section_size;
        // header2
        section_offsets.push(pos);
        pos += header2_section_size;
        // volume
        section_offsets.push(pos);
        pos += volume_section_size;
        // sectors
        section_offsets.push(pos);
        pos += sectors_section_size;

        let sectors_data_start =
            section_offsets[3] + SECTION_HEADER_SIZE as u64;

        // table
        section_offsets.push(pos);
        pos += table_section_size;
        // table2
        section_offsets.push(pos);
        pos += table2_section_size;
        // ltypes
        section_offsets.push(pos);
        pos += ltypes_section_size;
        // ltree
        section_offsets.push(pos);
        pos += ltree_section_size;
        // data
        section_offsets.push(pos);
        pos += data_section_size;
        // hash
        section_offsets.push(pos);
        pos += hash_section_size;
        // digest
        section_offsets.push(pos);
        pos += digest_section_size;
        // done
        section_offsets.push(pos);

        merged_table.base_offset = sectors_data_start;

        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        // 1. File header
        sections::write_file_header(&mut writer, 1)?;

        // 2. Header section
        sections::write_section_header(
            &mut writer,
            SECTION_TYPE_HEADER,
            section_offsets[1],
            header_section_size,
        )?;
        writer.write_all(header_compressed)?;

        // 3. Header2 section
        sections::write_section_header(
            &mut writer,
            SECTION_TYPE_HEADER2,
            section_offsets[2],
            header2_section_size,
        )?;
        writer.write_all(header2_compressed)?;

        // 4. Volume section
        self.write_volume_section(
            &mut writer,
            section_offsets[3],
            volume_section_size,
            merged_table.chunk_count(),
        )?;

        // 5. Sectors section
        sections::write_section_header(
            &mut writer,
            SECTION_TYPE_SECTORS,
            section_offsets[4],
            sectors_section_size,
        )?;
        writer.write_all(merged_compressed)?;

        // 6. Table section
        sections::write_section_header(
            &mut writer,
            SECTION_TYPE_TABLE,
            section_offsets[5],
            table_section_size,
        )?;
        write_table_data(&mut writer, merged_table)?;

        // 7. Table2 section
        sections::write_section_header(
            &mut writer,
            SECTION_TYPE_TABLE2,
            section_offsets[6],
            table2_section_size,
        )?;
        write_table_data(&mut writer, merged_table)?;

        // 8–13. Metadata + done
        self.write_trailing_sections(
            &mut writer,
            &section_offsets[6..],
            ltree_utf16,
            ltree_compressed,
            ltypes_section_size,
            ltree_section_size,
            data_section_size,
            hash_section_size,
            digest_section_size,
            done_section_size,
            merged_table.chunk_count(),
            md5_bytes,
            sha1_bytes,
        )?;

        writer.flush()?;
        drop(writer);

        let compression_ratio = if total_bytes > 0 {
            total_compressed_bytes as f64 / total_bytes as f64
        } else {
            0.0
        };

        Ok(L01WriteResult {
            output_paths: vec![output_path.to_string_lossy().to_string()],
            total_files,
            total_directories: total_dirs,
            total_data_bytes: total_bytes,
            total_compressed_bytes,
            compression_ratio,
            md5_hash: Some(hex::encode(md5_bytes)),
            sha1_hash: Some(hex::encode(sha1_bytes)),
            segment_count: 1,
            chunk_count: merged_table.chunk_count(),
        })
    }

    /// Write data across multiple segment files when it exceeds segment_size.
    ///
    /// Multi-segment L01 layout:
    /// - **Segment 1** (.L01): file_header → header → header2 → volume → sectors(partial) → table → table2 → next
    /// - **Segment N** (.LNN): file_header → sectors(partial) → table → table2 → next
    /// - **Last Segment**: file_header → sectors(remaining) → table → table2 → ltypes → ltree → data → hash → digest → done
    #[allow(clippy::too_many_arguments)]
    fn write_multi_segment(
        &self,
        base_path: &std::path::Path,
        merged_compressed: &[u8],
        merged_table: &ChunkTable,
        header_compressed: &[u8],
        header_section_size: u64,
        header2_compressed: &[u8],
        header2_section_size: u64,
        volume_section_size: u64,
        ltree_utf16: &[u8],
        ltree_compressed: &[u8],
        ltypes_section_size: u64,
        ltree_section_size: u64,
        data_section_size: u64,
        hash_section_size: u64,
        digest_section_size: u64,
        done_section_size: u64,
        next_section_size: u64,
        md5_bytes: &[u8; 16],
        sha1_bytes: &[u8; 20],
        total_bytes: u64,
        total_compressed_bytes: u64,
        total_files: usize,
        total_dirs: usize,
    ) -> Result<L01WriteResult, L01WriteError> {
        let segment_size = self.config.segment_size;

        // Calculate overhead for the first segment (metadata sections)
        let first_seg_overhead = FILE_HEADER_SIZE as u64
            + header_section_size
            + header2_section_size
            + volume_section_size;

        // Overhead for trailing metadata on the last segment
        let _trailing_overhead = ltypes_section_size
            + ltree_section_size
            + data_section_size
            + hash_section_size
            + digest_section_size
            + done_section_size;

        // Each segment needs: sectors header + data + table + table2 + next/done
        // We split the chunk data across segments, grouping by chunks.
        // Each segment's sectors section contains some number of complete chunks.

        // Build chunk byte ranges: for each chunk, track its start..end in merged_compressed
        let mut chunk_ranges: Vec<(u64, u64)> = Vec::with_capacity(merged_table.chunks.len());
        for chunk in &merged_table.chunks {
            chunk_ranges.push((chunk.offset, chunk.offset + chunk.compressed_size as u64));
        }

        // Partition chunks into segments
        let mut segments: Vec<std::ops::Range<usize>> = Vec::new(); // chunk index ranges per segment
        let mut chunk_idx = 0;
        let total_chunks = chunk_ranges.len();

        while chunk_idx < total_chunks {
            let is_first = segments.is_empty();
            let seg_start = chunk_idx;

            // Available space for sectors data in this segment
            let overhead = if is_first { first_seg_overhead } else { FILE_HEADER_SIZE as u64 };
            // We need room for: sectors_header + data + table + table2 + next/done
            let sectors_header = SECTION_HEADER_SIZE as u64;
            // Minimum: overhead + sectors_header + at least one chunk + table + table2 + next
            let fixed_overhead = overhead + sectors_header;

            let mut sectors_data_size: u64 = 0;
            let mut chunks_in_seg = 0;

            while chunk_idx < total_chunks {
                let chunk_data_size = chunk_ranges[chunk_idx].1 - chunk_ranges[chunk_idx].0;
                let new_sectors_data_size = sectors_data_size + chunk_data_size;

                // Calculate table sizes for this many chunks
                let new_chunk_count = chunks_in_seg + 1;
                let table_entries_size = (new_chunk_count as u64) * 4 + 4;
                let table_section_size = SECTION_HEADER_SIZE as u64 + 24 + table_entries_size;
                let table2_section_size = table_section_size;

                // Total segment size if we include this chunk
                let total_seg_size = fixed_overhead
                    + new_sectors_data_size
                    + table_section_size
                    + table2_section_size
                    + next_section_size; // conservative: use 'next' size even for last

                if chunks_in_seg > 0 && total_seg_size > segment_size {
                    break; // This chunk would exceed segment size
                }

                sectors_data_size = new_sectors_data_size;
                chunks_in_seg += 1;
                chunk_idx += 1;
            }

            if chunks_in_seg == 0 {
                // Safety: this shouldn't happen because we always include at least 1 chunk
                return Err(L01WriteError::Internal(
                    "Failed to fit any chunks in segment".to_string(),
                ));
            }

            segments.push(seg_start..seg_start + chunks_in_seg);
        }

        let segment_count = segments.len() as u16;
        if segment_count > segment::MAX_SEGMENTS {
            return Err(L01WriteError::TooManySegments(
                segment_count,
                segment::MAX_SEGMENTS,
            ));
        }

        let mut output_paths = Vec::with_capacity(segments.len());

        for (seg_idx, chunk_range) in segments.iter().enumerate() {
            let seg_num = (seg_idx + 1) as u16;
            let is_first = seg_idx == 0;
            let is_last = seg_idx == segments.len() - 1;

            let seg_path = segment::segment_path(base_path, seg_num);

            let file = File::create(&seg_path)?;
            let mut writer = BufWriter::new(file);
            let mut pos: u64 = 0;

            // 1. File header
            sections::write_file_header(&mut writer, seg_num)?;
            pos += FILE_HEADER_SIZE as u64;

            // 2–4. First segment gets header, header2, volume
            if is_first {
                // header → header2 → volume
                let header_next = pos + header_section_size;
                sections::write_section_header(
                    &mut writer,
                    SECTION_TYPE_HEADER,
                    header_next,
                    header_section_size,
                )?;
                writer.write_all(header_compressed)?;
                pos += header_section_size;

                let header2_next = pos + header2_section_size;
                sections::write_section_header(
                    &mut writer,
                    SECTION_TYPE_HEADER2,
                    header2_next,
                    header2_section_size,
                )?;
                writer.write_all(header2_compressed)?;
                pos += header2_section_size;

                let volume_next = pos + volume_section_size;
                self.write_volume_section(
                    &mut writer,
                    volume_next,
                    volume_section_size,
                    merged_table.chunk_count(),
                )?;
                pos += volume_section_size;
            }

            // Build this segment's chunk table and sectors data
            let seg_chunks = &merged_table.chunks[chunk_range.clone()];
            let sectors_start = chunk_ranges[chunk_range.start].0 as usize;
            let sectors_end = chunk_ranges[chunk_range.end - 1].1 as usize;
            let seg_sectors_data = &merged_compressed[sectors_start..sectors_end];

            let seg_sectors_section_size =
                SECTION_HEADER_SIZE as u64 + seg_sectors_data.len() as u64;

            // Build segment-local chunk table with offsets relative to
            // this segment's sectors data start
            let seg_table_entries = seg_chunks.len() as u64 * 4 + 4;
            let seg_table_section_size =
                SECTION_HEADER_SIZE as u64 + 24 + seg_table_entries;
            let seg_table2_section_size = seg_table_section_size;

            let sectors_data_start_in_seg = pos + SECTION_HEADER_SIZE as u64;
            let base_offset_in_merged = chunk_ranges[chunk_range.start].0;

            let mut seg_chunk_table = ChunkTable::new(sectors_data_start_in_seg);
            for chunk in seg_chunks {
                seg_chunk_table.add_chunk(
                    chunk.offset - base_offset_in_merged,
                    chunk.compressed_size,
                    chunk.is_compressed,
                );
            }

            // Calculate remaining section offsets
            let sectors_next = pos + seg_sectors_section_size;
            let table_pos = sectors_next;
            let table_next = table_pos + seg_table_section_size;
            let table2_pos = table_next;
            let table2_next = table2_pos + seg_table2_section_size;

            // Write sectors
            sections::write_section_header(
                &mut writer,
                SECTION_TYPE_SECTORS,
                sectors_next,
                seg_sectors_section_size,
            )?;
            writer.write_all(seg_sectors_data)?;

            // Write table
            let after_table2 = table2_next;
            sections::write_section_header(
                &mut writer,
                SECTION_TYPE_TABLE,
                table_next,
                seg_table_section_size,
            )?;
            write_table_data(&mut writer, &seg_chunk_table)?;

            // Write table2
            sections::write_section_header(
                &mut writer,
                SECTION_TYPE_TABLE2,
                after_table2,
                seg_table2_section_size,
            )?;
            write_table_data(&mut writer, &seg_chunk_table)?;
            pos = after_table2;

            if is_last {
                // Last segment: write ltypes → ltree → data → hash → digest → done
                let ltypes_next = pos + ltypes_section_size;
                let ltree_pos = ltypes_next;
                let ltree_next = ltree_pos + ltree_section_size;
                let data_pos = ltree_next;
                let data_next = data_pos + data_section_size;
                let hash_pos = data_next;
                let hash_next = hash_pos + hash_section_size;
                let digest_pos = hash_next;
                let digest_next = digest_pos + digest_section_size;
                let done_pos = digest_next;

                // Build offset slice for write_trailing_sections
                // Indices: [0]=table2_next(pos before ltypes), already at pos
                let trailing_offsets = [
                    pos,           // before ltypes
                    ltypes_next,   // before ltree
                    ltree_next,    // before data
                    data_next,     // before hash
                    hash_next,     // before digest
                    digest_next,   // before done
                    done_pos,      // done position
                ];

                self.write_trailing_sections(
                    &mut writer,
                    &trailing_offsets,
                    ltree_utf16,
                    ltree_compressed,
                    ltypes_section_size,
                    ltree_section_size,
                    data_section_size,
                    hash_section_size,
                    digest_section_size,
                    done_section_size,
                    merged_table.chunk_count(),
                    md5_bytes,
                    sha1_bytes,
                )?;
            } else {
                // Non-last segment: write "next" section (no data, just header)
                sections::write_section_header(
                    &mut writer,
                    SECTION_TYPE_NEXT,
                    0, // next_offset = 0 for "next" section
                    next_section_size,
                )?;
            }

            writer.flush()?;
            drop(writer);
            output_paths.push(seg_path.to_string_lossy().to_string());
        }

        let compression_ratio = if total_bytes > 0 {
            total_compressed_bytes as f64 / total_bytes as f64
        } else {
            0.0
        };

        Ok(L01WriteResult {
            output_paths,
            total_files,
            total_directories: total_dirs,
            total_data_bytes: total_bytes,
            total_compressed_bytes,
            compression_ratio,
            md5_hash: Some(hex::encode(md5_bytes)),
            sha1_hash: Some(hex::encode(sha1_bytes)),
            segment_count: segment_count as u32,
            chunk_count: merged_table.chunk_count(),
        })
    }

    /// Write the volume section.
    fn write_volume_section<W: Write>(
        &self,
        writer: &mut W,
        next_offset: u64,
        volume_section_size: u64,
        chunk_count: u32,
    ) -> Result<(), L01WriteError> {
        sections::write_section_header(
            writer,
            SECTION_TYPE_VOLUME,
            next_offset,
            volume_section_size,
        )?;

        let mut vol_data = [0u8; VOLUME_DATA_SIZE];
        vol_data[0..4].copy_from_slice(&1u32.to_le_bytes());
        vol_data[4..8].copy_from_slice(&chunk_count.to_le_bytes());
        vol_data[8..12].copy_from_slice(
            &self.config.sectors_per_chunk.to_le_bytes(),
        );
        vol_data[12..16]
            .copy_from_slice(&self.config.block_size.to_le_bytes());
        let sector_count = chunk_count as u64
            * self.config.sectors_per_chunk as u64;
        vol_data[16..24].copy_from_slice(&sector_count.to_le_bytes());
        vol_data[36] = MEDIA_TYPE_LOGICAL;
        vol_data[56] = self.config.compression_level as u8;
        let guid = uuid::Uuid::new_v4();
        vol_data[60..76].copy_from_slice(guid.as_bytes());
        let checksum = adler32(&vol_data[0..90]);
        vol_data[90..94].copy_from_slice(&checksum.to_le_bytes());
        writer.write_all(&vol_data)?;

        Ok(())
    }

    /// Write trailing metadata sections: ltypes → ltree → data → hash → digest → done.
    ///
    /// `offsets` slice: [current_pos, ltypes_next, ltree_next, data_next, hash_next, digest_next, done_pos]
    #[allow(clippy::too_many_arguments)]
    fn write_trailing_sections<W: Write>(
        &self,
        writer: &mut W,
        offsets: &[u64],
        ltree_utf16: &[u8],
        ltree_compressed: &[u8],
        ltypes_section_size: u64,
        ltree_section_size: u64,
        data_section_size: u64,
        hash_section_size: u64,
        digest_section_size: u64,
        done_section_size: u64,
        chunk_count: u32,
        md5_bytes: &[u8; 16],
        sha1_bytes: &[u8; 20],
    ) -> Result<(), L01WriteError> {
        // offsets[0] = current pos (before ltypes)
        // offsets[1] = after ltypes (ltree start)
        // offsets[2] = after ltree (data start)
        // offsets[3] = after data (hash start)
        // offsets[4] = after hash (digest start)
        // offsets[5] = after digest (done start)

        // Ltypes
        sections::write_section_header(
            writer,
            SECTION_TYPE_LTYPES,
            offsets[1],
            ltypes_section_size,
        )?;
        writer.write_all(&[0u8; 4])?;

        // Ltree
        sections::write_section_header(
            writer,
            SECTION_TYPE_LTREE,
            offsets[2],
            ltree_section_size,
        )?;
        {
            let mut ltree_header = [0u8; 48];
            let ltree_md5 = md5::Md5::digest(ltree_utf16);
            ltree_header[0..16].copy_from_slice(&ltree_md5);
            let ltree_data_size = ltree_utf16.len() as u64;
            ltree_header[16..24]
                .copy_from_slice(&ltree_data_size.to_le_bytes());
            let hdr_checksum = adler32(&ltree_header[0..24]);
            ltree_header[24..28]
                .copy_from_slice(&hdr_checksum.to_le_bytes());
            writer.write_all(&ltree_header)?;
            writer.write_all(ltree_compressed)?;
        }

        // Data section
        sections::write_section_header(
            writer,
            SECTION_TYPE_DATA,
            offsets[3],
            data_section_size,
        )?;
        {
            let mut data = [0u8; DATA_SECTION_DATA_SIZE];
            data[0..4].copy_from_slice(
                &(MEDIA_TYPE_LOGICAL as u32).to_le_bytes(),
            );
            data[8..16].copy_from_slice(
                &(chunk_count as u64).to_le_bytes(),
            );
            data[16..20].copy_from_slice(
                &self.config.sectors_per_chunk.to_le_bytes(),
            );
            data[20..24]
                .copy_from_slice(&self.config.block_size.to_le_bytes());
            let sc = chunk_count as u64
                * self.config.sectors_per_chunk as u64;
            data[24..32].copy_from_slice(&sc.to_le_bytes());
            let guid2 = uuid::Uuid::new_v4();
            data[52..68].copy_from_slice(guid2.as_bytes());
            let ck = adler32(&data[0..68]);
            data[68..72].copy_from_slice(&ck.to_le_bytes());
            writer.write_all(&data)?;
        }

        // Hash section (MD5)
        sections::write_section_header(
            writer,
            SECTION_TYPE_HASH,
            offsets[4],
            hash_section_size,
        )?;
        {
            let mut hdata = [0u8; HASH_SECTION_DATA_SIZE];
            hdata[0..16].copy_from_slice(md5_bytes);
            let ck = adler32(&hdata[0..16]);
            hdata[16..20].copy_from_slice(&ck.to_le_bytes());
            writer.write_all(&hdata)?;
        }

        // Digest section (SHA-1)
        sections::write_section_header(
            writer,
            SECTION_TYPE_DIGEST,
            offsets[5],
            digest_section_size,
        )?;
        {
            let mut ddata = [0u8; DIGEST_SECTION_DATA_SIZE];
            ddata[0..20].copy_from_slice(sha1_bytes);
            let ck = adler32(&ddata[0..20]);
            ddata[20..24].copy_from_slice(&ck.to_le_bytes());
            writer.write_all(&ddata)?;
        }

        // Done section
        sections::write_section_header(
            writer,
            SECTION_TYPE_DONE,
            0,
            done_section_size,
        )?;

        Ok(())
    }
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Build header text (same as sections::build_header_text but callable without writer)
fn build_header_text_static(case_info: &L01CaseInfo) -> String {
    let now = chrono::Local::now();
    let acq_date = now.format("%Y %m %d %H %M %S").to_string();
    let system_date = acq_date.clone();

    let mut text = String::new();
    text.push_str("1\r\n");
    text.push_str("main\r\n");
    text.push_str("c\tn\ta\te\tt\tav\tov\tm\tu\tp\r\n");
    text.push_str(&case_info.case_number);
    text.push('\t');
    text.push_str(&case_info.evidence_number);
    text.push('\t');
    text.push_str(&case_info.description);
    text.push('\t');
    text.push_str(&case_info.examiner);
    text.push('\t');
    text.push_str(&case_info.notes);
    text.push('\t');
    text.push_str("CORE-FFX 1.0");
    text.push('\t');
    text.push_str(std::env::consts::OS);
    text.push('\t');
    text.push_str(&acq_date);
    text.push('\t');
    text.push_str(&system_date);
    text.push('\t');
    text.push('\t');
    text.push_str("\r\n");
    text.push_str("\r\n");

    text
}

/// Compress text data with zlib
fn compress_text(data: &[u8]) -> Result<Vec<u8>, L01WriteError> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))
}

/// Write table data (header + entries + checksum) without section header
fn write_table_data<W: Write>(
    writer: &mut W,
    table: &ChunkTable,
) -> Result<(), L01WriteError> {
    // Table header (24 bytes)
    let mut header = [0u8; 24];
    header[0..4].copy_from_slice(&table.chunk_count().to_le_bytes());
    header[8..16].copy_from_slice(&table.base_offset.to_le_bytes());
    let header_checksum = adler32(&header[0..20]);
    header[20..24].copy_from_slice(&header_checksum.to_le_bytes());
    writer.write_all(&header)?;

    // Chunk offset entries
    let mut entries_data = Vec::with_capacity(table.chunk_count() as usize * 4);
    for chunk in &table.chunks {
        let mut offset_value = chunk.offset as u32;
        if chunk.is_compressed {
            offset_value |= 0x8000_0000;
        }
        entries_data.extend_from_slice(&offset_value.to_le_bytes());
    }
    writer.write_all(&entries_data)?;

    // Entries checksum
    let entries_checksum = adler32(&entries_data);
    writer.write_all(&entries_checksum.to_le_bytes())?;

    Ok(())
}

/// Set timestamps on a LefFileEntry from filesystem metadata.
fn set_timestamps_from_metadata(
    entry: &mut LefFileEntry,
    metadata: &std::fs::Metadata,
) {
    use std::time::UNIX_EPOCH;

    if let Ok(created) = metadata.created() {
        if let Ok(duration) = created.duration_since(UNIX_EPOCH) {
            entry.creation_time = duration.as_secs() as i64;
        }
    }
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            entry.modification_time = duration.as_secs() as i64;
            entry.entry_modification_time = duration.as_secs() as i64;
        }
    }
    if let Ok(accessed) = metadata.accessed() {
        if let Ok(duration) = accessed.duration_since(UNIX_EPOCH) {
            entry.access_time = duration.as_secs() as i64;
        }
    }
}

/// Compute MD5 and SHA-1 hashes for a file reader.
fn compute_file_hashes<R: std::io::Read>(
    reader: &mut R,
) -> Result<([u8; 16], [u8; 20]), L01WriteError> {
    let mut md5_hasher = md5::Md5::new();
    let mut sha1_hasher = sha1::Sha1::new();
    let mut buf = [0u8; 65536];

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                md5_hasher.update(&buf[..n]);
                sha1_hasher.update(&buf[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(L01WriteError::Io(e)),
        }
    }

    let md5_result = md5_hasher.finalize();
    let sha1_result = sha1_hasher.finalize();

    let mut md5_bytes = [0u8; 16];
    md5_bytes.copy_from_slice(&md5_result);

    let mut sha1_bytes = [0u8; 20];
    sha1_bytes.copy_from_slice(&sha1_result);

    Ok((md5_bytes, sha1_bytes))
}

/// Hash reader data into image-level MD5 and SHA-1 hashers.
fn hash_reader_into<R: std::io::Read>(
    reader: &mut R,
    md5: &mut md5::Md5,
    sha1: &mut sha1::Sha1,
) -> Result<(), L01WriteError> {
    let mut buf = [0u8; 65536];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                md5.update(&buf[..n]);
                sha1.update(&buf[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(L01WriteError::Io(e)),
        }
    }
    Ok(())
}

/// Emit a progress event.
fn emit_progress(
    progress_fn: &mut Option<Box<dyn FnMut(L01WriteProgress) + Send>>,
    output_path: &Path,
    current_file: &str,
    files_processed: usize,
    total_files: usize,
    bytes_written: u64,
    total_bytes: u64,
    phase: L01WritePhase,
) {
    if let Some(ref mut f) = progress_fn {
        let percent = if total_bytes > 0 {
            (bytes_written as f64 / total_bytes as f64) * 100.0
        } else if total_files > 0 {
            (files_processed as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };

        f(L01WriteProgress {
            path: output_path.to_string_lossy().to_string(),
            current_file: current_file.to_string(),
            files_processed,
            total_files,
            bytes_written,
            total_bytes,
            percent,
            phase,
        });
    }
}

// ── Constants re-exported from sections for use in size calculations ──
const HASH_SECTION_DATA_SIZE: usize = 20;
const DIGEST_SECTION_DATA_SIZE: usize = 24;
const DATA_SECTION_DATA_SIZE: usize = 72;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_files(dir: &Path) {
        std::fs::create_dir_all(dir.join("subdir")).unwrap();
        std::fs::write(dir.join("file1.txt"), b"Hello World").unwrap();
        std::fs::write(dir.join("file2.bin"), vec![0xABu8; 1024]).unwrap();
        std::fs::write(dir.join("subdir/nested.txt"), b"Nested content").unwrap();
    }

    #[test]
    fn test_writer_new() {
        let config = L01WriterConfig::default();
        let writer = L01Writer::new(config);
        assert_eq!(writer.entry_count(), 0);
        assert_eq!(writer.total_file_size(), 0);
    }

    #[test]
    fn test_writer_add_file() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        let id = writer.add_file(
            "test.txt".into(),
            100,
            PathBuf::from("/tmp/test.txt"),
            0,
        );
        assert_eq!(id, 1);
        assert_eq!(writer.entry_count(), 1);
        assert_eq!(writer.total_file_size(), 100);
    }

    #[test]
    fn test_writer_add_directory() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        let dir_id = writer.add_directory("docs".into(), 0);
        let file_id = writer.add_file(
            "readme.txt".into(),
            50,
            PathBuf::from("/tmp/readme.txt"),
            dir_id,
        );

        assert_eq!(dir_id, 1);
        assert_eq!(file_id, 2);
        assert_eq!(writer.entry_count(), 2);
        assert_eq!(writer.total_file_size(), 50);
    }

    #[test]
    fn test_writer_add_source_directory() {
        let tmp = TempDir::new().unwrap();
        create_test_files(tmp.path());

        let mut writer = L01Writer::new(L01WriterConfig::default());
        let count = writer.add_source_directory(tmp.path()).unwrap();

        // Should find: subdir, file1.txt, file2.bin, subdir/nested.txt
        assert_eq!(count, 4);
        assert_eq!(writer.entry_count(), 4);
    }

    #[test]
    fn test_writer_add_source_directory_not_found() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        let result =
            writer.add_source_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_validation_no_output() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        writer.add_file("test.txt".into(), 100, PathBuf::from("/tmp/test.txt"), 0);

        let result = writer.write(None, None);
        assert!(matches!(result, Err(L01WriteError::NoOutputPath)));
    }

    #[test]
    fn test_writer_validation_no_files() {
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: PathBuf::from("/tmp/test.L01"),
            ..Default::default()
        });

        let result = writer.write(None, None);
        assert!(matches!(result, Err(L01WriteError::NoFiles)));
    }

    #[test]
    fn test_writer_validation_segment_too_small() {
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: PathBuf::from("/tmp/test.L01"),
            segment_size: 100, // Too small
            ..Default::default()
        });
        writer.add_file("test.txt".into(), 100, PathBuf::from("/tmp/test.txt"), 0);

        let result = writer.write(None, None);
        assert!(matches!(result, Err(L01WriteError::SegmentTooSmall(_))));
    }

    #[test]
    fn test_write_complete_l01() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            case_info: L01CaseInfo {
                case_number: "TEST-001".into(),
                evidence_number: "E001".into(),
                description: "Test L01 output".into(),
                examiner: "Test Examiner".into(),
                notes: "Integration test".into(),
            },
            ..Default::default()
        });

        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        // Verify result
        assert_eq!(result.output_paths.len(), 1);
        assert_eq!(result.total_files, 3); // file1.txt, file2.bin, nested.txt
        assert_eq!(result.total_directories, 1); // subdir
        assert!(result.total_data_bytes > 0);
        assert!(result.md5_hash.is_some());
        assert!(result.sha1_hash.is_some());
        assert_eq!(result.segment_count, 1);
        assert!(result.chunk_count > 0);

        // Verify the file exists and has LVF signature
        assert!(output_path.exists());
        let written = std::fs::read(&output_path).unwrap();
        assert_eq!(&written[0..3], b"LVF");
        assert_eq!(&written[0..8], LVF_SIGNATURE);

        // File should be non-trivially sized
        assert!(written.len() > FILE_HEADER_SIZE + SECTION_HEADER_SIZE);
    }

    #[test]
    fn test_write_with_cancellation() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let cancel = AtomicBool::new(true); // Cancel immediately
        let result = writer.write(Some(&cancel), None);
        assert!(matches!(result, Err(L01WriteError::Cancelled)));
    }

    #[test]
    fn test_write_with_progress() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            case_info: L01CaseInfo {
                case_number: "PROG-001".into(),
                ..Default::default()
            },
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let progress_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pc = progress_count.clone();

        let result = writer.write(
            None,
            Some(Box::new(move |_progress| {
                pc.fetch_add(1, Ordering::Relaxed);
            })),
        );

        assert!(result.is_ok());
        assert!(progress_count.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_write_no_compression() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("data.bin"), vec![0x42u8; 512]).unwrap();

        let output_path = tmp.path().join("nocomp.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            compression_level: CompressionLevel::None,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();
        assert!(result.total_compressed_bytes > 0);
        assert!(output_path.exists());
    }

    #[test]
    fn test_segment_extension() {
        assert_eq!(segment::segment_extension(1), "L01");
        assert_eq!(segment::segment_extension(50), "L50");
        assert_eq!(segment::segment_extension(100), "LAA");
    }

    // ── Multi-segment write tests ──

    /// Helper to create source files with pseudorandom data that resists
    /// zlib compression. Uses a simple PRNG to produce near-uniform bytes.
    fn create_large_test_files(dir: &Path, count: usize, each_size: usize) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        std::fs::create_dir_all(dir).unwrap();
        for i in 0..count {
            let mut data = vec![0u8; each_size];
            // Simple LCG PRNG seeded by file index
            let mut state: u64 = {
                let mut h = DefaultHasher::new();
                i.hash(&mut h);
                h.finish()
            };
            for byte in data.iter_mut() {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                *byte = (state >> 33) as u8;
            }
            std::fs::write(dir.join(format!("file{}.bin", i)), &data).unwrap();
        }
    }

    /// Minimum segment size constant for tests (mirrors production value)
    const MIN_SEGMENT: u64 = 1_048_576;

    #[test]
    fn test_write_multi_segment_creates_multiple_files() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB of incompressible data → at min segment (1 MB) → ≥2 segments
        create_large_test_files(&source_dir, 4, 768 * 1024);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            case_info: L01CaseInfo {
                case_number: "SPLIT-001".into(),
                ..Default::default()
            },
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        // Should produce multiple segments
        assert!(
            result.segment_count > 1,
            "Expected multiple segments, got {}",
            result.segment_count
        );
        assert_eq!(result.output_paths.len(), result.segment_count as usize);

        // All segment files should exist
        for path in &result.output_paths {
            assert!(
                std::path::Path::new(path).exists(),
                "Segment file {} doesn't exist",
                path
            );
        }

        // First segment should have L01 extension
        assert!(
            result.output_paths[0].ends_with(".L01"),
            "First segment should have .L01 extension: {}",
            result.output_paths[0]
        );

        // All segments should have LVF signature
        for path in &result.output_paths {
            let data = std::fs::read(path).unwrap();
            assert_eq!(
                &data[0..8],
                LVF_SIGNATURE,
                "Segment {} missing LVF signature",
                path
            );
        }
    }

    #[test]
    fn test_write_multi_segment_sequential_extensions() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 8 files × 768 KB = 6 MB → at min segment → ≥3 segments
        create_large_test_files(&source_dir, 8, 768 * 1024);

        let output_path = tmp.path().join("multi.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        assert!(
            result.segment_count >= 3,
            "Expected at least 3 segments, got {}",
            result.segment_count
        );

        // Verify sequential extension naming
        for (idx, path) in result.output_paths.iter().enumerate() {
            let expected_ext = segment::segment_extension((idx + 1) as u16);
            assert!(
                path.ends_with(&format!(".{}", expected_ext)),
                "Segment {} should end with .{}, got {}",
                idx,
                expected_ext,
                path
            );
        }
    }

    #[test]
    fn test_write_multi_segment_preserves_hashes() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB
        create_large_test_files(&source_dir, 4, 768 * 1024);

        // Single-segment write
        let single_output = tmp.path().join("single.L01");
        let mut single_writer = L01Writer::new(L01WriterConfig {
            output_path: single_output.clone(),
            ..Default::default()
        });
        single_writer.add_source_directory(&source_dir).unwrap();
        let single_result = single_writer.write(None, None).unwrap();

        // Multi-segment write of the same data
        let multi_output = tmp.path().join("multi.L01");
        let mut multi_writer = L01Writer::new(L01WriterConfig {
            output_path: multi_output.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        multi_writer.add_source_directory(&source_dir).unwrap();
        let multi_result = multi_writer.write(None, None).unwrap();

        // Both should have the same data hashes (content is identical)
        assert_eq!(single_result.md5_hash, multi_result.md5_hash);
        assert_eq!(single_result.sha1_hash, multi_result.sha1_hash);
        assert_eq!(single_result.total_files, multi_result.total_files);
        assert_eq!(single_result.total_data_bytes, multi_result.total_data_bytes);
    }

    #[test]
    fn test_write_segment_size_zero_means_single() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("single.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: 0, // Zero means no splitting
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        assert_eq!(result.segment_count, 1);
        assert_eq!(result.output_paths.len(), 1);
    }

    #[test]
    fn test_write_multi_segment_segment_number_in_header() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB → ≥2 segments at 1 MB
        create_large_test_files(&source_dir, 4, 768 * 1024);

        let output_path = tmp.path().join("segtest.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        // Each segment file header should contain the correct segment number
        // LVF file header: bytes 0-7 = LVF signature, byte 8 = 0x01,
        // bytes 9-10 = segment number (u16 LE), bytes 11-12 = fields end
        for (idx, path) in result.output_paths.iter().enumerate() {
            let data = std::fs::read(path).unwrap();
            assert!(data.len() > 13, "Segment {} too small", idx);

            // Check segment number at offset 9 (2 bytes LE)
            let seg_num = u16::from_le_bytes([data[9], data[10]]);
            assert_eq!(
                seg_num,
                (idx + 1) as u16,
                "Segment {} has wrong segment number in header: expected {}, got {}",
                idx,
                idx + 1,
                seg_num
            );
        }
    }

    #[test]
    fn test_write_multi_segment_last_has_done_section() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB → ≥2 segments at 1 MB
        create_large_test_files(&source_dir, 4, 768 * 1024);

        let output_path = tmp.path().join("done.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();
        assert!(result.segment_count > 1);

        // Last segment should contain "done" section type somewhere
        let last_path = result.output_paths.last().unwrap();
        let data = std::fs::read(last_path).unwrap();

        // Search for "done" section type in section headers (16 bytes type field, null-padded)
        let done_marker = b"done\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let found_done = data
            .windows(16)
            .any(|w| w == done_marker);
        assert!(found_done, "Last segment should contain 'done' section");

        // Non-last segments should contain "next" section
        if result.segment_count > 1 {
            let first_path = &result.output_paths[0];
            let first_data = std::fs::read(first_path).unwrap();
            let next_marker = b"next\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let found_next = first_data
                .windows(16)
                .any(|w| w == next_marker);
            assert!(found_next, "Non-last segment should contain 'next' section");
        }
    }

    #[test]
    fn test_write_multi_segment_total_data_correct() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        let file_sizes: Vec<usize> = vec![200_000, 400_000, 300_000, 150_000, 250_000];
        std::fs::create_dir_all(&source_dir).unwrap();
        for (i, &size) in file_sizes.iter().enumerate() {
            // Pseudorandom data for each file
            let data: Vec<u8> = (0..size)
                .map(|j| ((j as u64 * 31 + i as u64 * 97) % 256) as u8)
                .collect();
            std::fs::write(source_dir.join(format!("f{}.bin", i)), &data).unwrap();
        }

        let output_path = tmp.path().join("totalcheck.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        let expected_total: u64 = file_sizes.iter().map(|&s| s as u64).sum();
        assert_eq!(
            result.total_data_bytes, expected_total,
            "Total data bytes mismatch"
        );
        assert_eq!(result.total_files, 5);
        assert!(result.chunk_count > 0);
    }
}
