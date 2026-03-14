// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! L01 write pipeline implementation.
//!
//! Contains the core `write()` method and its internal helpers for producing
//! single-segment and multi-segment L01 files. This module implements the
//! actual EWF v1 section layout, chunk table construction, and hash computation.

use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use md5::Digest as _;
// sha1::Digest is the same trait; importing md5::Digest covers both

use super::chunks;
use super::ltree;
use super::sections;
use super::segment;
use super::types::*;
use super::L01Writer;

/// Type alias for the progress callback wrapped in a [`RefCell`].
type ProgressCell = RefCell<Option<Box<dyn FnMut(L01WriteProgress) + Send>>>;

impl L01Writer {
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
        let total_bytes: u64 = file_entries.iter().map(|&i| self.entries[i].size).sum();

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
        let chunk_size =
            (self.config.block_size as usize) * (self.config.sectors_per_chunk as usize);

        let mut all_compressed_data = Vec::new();
        let mut all_chunk_tables = Vec::new();
        let mut image_md5 = md5::Md5::new();
        let mut image_sha1 = sha1::Sha1::new();
        let mut bytes_written: u64 = 0;
        let mut data_offset: u64 = 0;

        // Wrap progress_fn in RefCell so the per-chunk callback can emit
        // progress events without conflicting with the per-file emissions.
        let progress_cell: ProgressCell =
            RefCell::new(progress_fn);

        for (file_idx, &entry_idx) in file_entries.iter().enumerate() {
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    return Err(L01WriteError::Cancelled);
                }
            }

            let source_path = self.entries[entry_idx]
                .source_path
                .as_ref()
                .ok_or_else(|| L01WriteError::SourceReadError {
                    path: self.entries[entry_idx].name.clone(),
                    reason: "No source path set".to_string(),
                })?
                .clone();

            if !source_path.exists() {
                return Err(L01WriteError::FileNotFound(
                    source_path.to_string_lossy().to_string(),
                ));
            }

            let file_size = self.entries[entry_idx].size;

            emit_progress_cell(
                &progress_cell,
                &self.config.output_path,
                &self.entries[entry_idx].name,
                file_idx,
                total_files,
                bytes_written,
                total_bytes,
                L01WritePhase::WritingData,
            );

            // Read and compress file data — with per-chunk progress for large files
            // Use 1MB BufReader for fewer syscalls on large forensic files
            let mut file = BufReader::with_capacity(1024 * 1024, File::open(&source_path).map_err(|e| {
                L01WriteError::SourceReadError {
                    path: source_path.to_string_lossy().to_string(),
                    reason: e.to_string(),
                }
            })?);

            let file_bytes_base = bytes_written;
            let entry_name = self.entries[entry_idx].name.clone();
            let output_path_ref = self.config.output_path.clone();
            let mut last_chunk_emit = std::time::Instant::now();

            let mut chunk_progress = |chunk_bytes_so_far: u64| {
                let now = std::time::Instant::now();
                // Throttle to at most every 250ms to avoid flooding the event bus
                if now.duration_since(last_chunk_emit).as_millis() >= 250 {
                    emit_progress_cell(
                        &progress_cell,
                        &output_path_ref,
                        &entry_name,
                        file_idx,
                        total_files,
                        file_bytes_base + chunk_bytes_so_far,
                        total_bytes,
                        L01WritePhase::WritingData,
                    );
                    last_chunk_emit = now;
                }
            };

            // Per-file hashers for inline computation during compression
            let mut file_md5_hasher = md5::Md5::new();
            let mut file_sha1_hasher = sha1::Sha1::new();

            let (compressed, table, _hash_result) = chunks::compress_and_hash_from_reader(
                &mut file,
                file_size,
                chunk_size,
                self.config.compression_level,
                data_offset,
                Some(&mut chunk_progress),
                Some(chunks::InlineHashers {
                    file_md5: &mut file_md5_hasher,
                    file_sha1: &mut file_sha1_hasher,
                    image_md5: &mut image_md5,
                    image_sha1: &mut image_sha1,
                }),
            )?;

            // Extract per-file hashes from the inline hashers
            let file_md5_result = file_md5_hasher.finalize();
            let file_sha1_result = file_sha1_hasher.finalize();
            self.entries[entry_idx].md5_hash = Some(hex::encode(file_md5_result));
            self.entries[entry_idx].sha1_hash = Some(hex::encode(file_sha1_result));

            // Set data offset and size on entry
            self.entries[entry_idx].data_offset = data_offset;
            self.entries[entry_idx].data_size = file_size;

            data_offset += compressed.len() as u64;
            bytes_written += file_size;

            // Post-file progress — shows updated bytes after file completes
            emit_progress_cell(
                &progress_cell,
                &self.config.output_path,
                &self.entries[entry_idx].name,
                file_idx + 1,
                total_files,
                bytes_written,
                total_bytes,
                L01WritePhase::WritingData,
            );

            all_compressed_data.push(compressed);
            all_chunk_tables.push(table);
        }

        // Unwrap progress_fn from RefCell for remaining phases
        let mut progress_fn = progress_cell.into_inner();

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
        let header_compressed =
            compress_text(build_header_text_static(&self.config.case_info).as_bytes())?;
        let header_section_size = SECTION_HEADER_SIZE as u64 + header_compressed.len() as u64;

        let header2_utf16 =
            ltree::utf8_to_utf16le(&build_header_text_static(&self.config.case_info));
        let header2_compressed = compress_text(&header2_utf16)?;
        let header2_section_size = SECTION_HEADER_SIZE as u64 + header2_compressed.len() as u64;

        let volume_section_size = (SECTION_HEADER_SIZE + VOLUME_DATA_SIZE) as u64;

        let ltypes_section_size = SECTION_HEADER_SIZE as u64 + 4;

        let ltree_compressed = compress_text(&ltree_utf16)?;
        let ltree_section_size = SECTION_HEADER_SIZE as u64 + 48 + ltree_compressed.len() as u64;

        let data_section_size = (SECTION_HEADER_SIZE + DATA_SECTION_DATA_SIZE) as u64;

        let hash_section_size = (SECTION_HEADER_SIZE + HASH_SECTION_DATA_SIZE) as u64;

        let digest_section_size = (SECTION_HEADER_SIZE + DIGEST_SECTION_DATA_SIZE) as u64;

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
        let sectors_section_size = SECTION_HEADER_SIZE as u64 + merged_compressed.len() as u64;

        let table_entries_size = merged_table.chunk_count() as u64 * 4 + 4;
        let table_section_size = SECTION_HEADER_SIZE as u64 + 24 + table_entries_size;
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

        let sectors_data_start = section_offsets[3] + SECTION_HEADER_SIZE as u64;

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

        // Ensure the output file has the .L01 extension (consistent with multi-segment path)
        let actual_path = segment::segment_path(output_path, 1);
        let file = File::create(&actual_path)?;
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
            output_paths: vec![actual_path.to_string_lossy().to_string()],
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
            let overhead = if is_first {
                first_seg_overhead
            } else {
                FILE_HEADER_SIZE as u64
            };
            // We need room for: sectors_header + data + table + table2 + next
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
            let seg_table_section_size = SECTION_HEADER_SIZE as u64 + 24 + seg_table_entries;
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
                let trailing_offsets = [
                    pos,         // before ltypes
                    ltypes_next, // before ltree
                    ltree_next,  // before data
                    data_next,   // before hash
                    hash_next,   // before digest
                    digest_next, // before done
                    done_pos,    // done position
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
        vol_data[8..12].copy_from_slice(&self.config.sectors_per_chunk.to_le_bytes());
        vol_data[12..16].copy_from_slice(&self.config.block_size.to_le_bytes());
        let sector_count = chunk_count as u64 * self.config.sectors_per_chunk as u64;
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
        // Ltypes
        sections::write_section_header(
            writer,
            SECTION_TYPE_LTYPES,
            offsets[1],
            ltypes_section_size,
        )?;
        writer.write_all(&[0u8; 4])?;

        // Ltree
        sections::write_section_header(writer, SECTION_TYPE_LTREE, offsets[2], ltree_section_size)?;
        {
            let mut ltree_header = [0u8; 48];
            let ltree_md5 = md5::Md5::digest(ltree_utf16);
            ltree_header[0..16].copy_from_slice(&ltree_md5);
            let ltree_data_size = ltree_utf16.len() as u64;
            ltree_header[16..24].copy_from_slice(&ltree_data_size.to_le_bytes());
            let hdr_checksum = adler32(&ltree_header[0..24]);
            ltree_header[24..28].copy_from_slice(&hdr_checksum.to_le_bytes());
            writer.write_all(&ltree_header)?;
            writer.write_all(ltree_compressed)?;
        }

        // Data section
        sections::write_section_header(writer, SECTION_TYPE_DATA, offsets[3], data_section_size)?;
        {
            let mut data = [0u8; DATA_SECTION_DATA_SIZE];
            data[0..4].copy_from_slice(&(MEDIA_TYPE_LOGICAL as u32).to_le_bytes());
            data[8..16].copy_from_slice(&(chunk_count as u64).to_le_bytes());
            data[16..20].copy_from_slice(&self.config.sectors_per_chunk.to_le_bytes());
            data[20..24].copy_from_slice(&self.config.block_size.to_le_bytes());
            let sc = chunk_count as u64 * self.config.sectors_per_chunk as u64;
            data[24..32].copy_from_slice(&sc.to_le_bytes());
            let guid2 = uuid::Uuid::new_v4();
            data[52..68].copy_from_slice(guid2.as_bytes());
            let ck = adler32(&data[0..68]);
            data[68..72].copy_from_slice(&ck.to_le_bytes());
            writer.write_all(&data)?;
        }

        // Hash section (MD5)
        sections::write_section_header(writer, SECTION_TYPE_HASH, offsets[4], hash_section_size)?;
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
        sections::write_section_header(writer, SECTION_TYPE_DONE, 0, done_section_size)?;

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
fn write_table_data<W: Write>(writer: &mut W, table: &ChunkTable) -> Result<(), L01WriteError> {
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
pub(crate) fn set_timestamps_from_metadata(entry: &mut LefFileEntry, metadata: &std::fs::Metadata) {
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

/// Emit a progress event.
#[allow(clippy::too_many_arguments)]
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

/// Emit a progress event via a [`RefCell`]-wrapped callback.
///
/// Used during Phase 2 (file compression loop) where both the per-file
/// and per-chunk code paths need to emit progress events.
#[allow(clippy::too_many_arguments)]
fn emit_progress_cell(
    progress_cell: &ProgressCell,
    output_path: &Path,
    current_file: &str,
    files_processed: usize,
    total_files: usize,
    bytes_written: u64,
    total_bytes: u64,
    phase: L01WritePhase,
) {
    if let Ok(mut guard) = progress_cell.try_borrow_mut() {
        if let Some(ref mut f) = *guard {
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
}

// ── Constants re-exported from sections for use in size calculations ──
const HASH_SECTION_DATA_SIZE: usize = 20;
const DIGEST_SECTION_DATA_SIZE: usize = 24;
const DATA_SECTION_DATA_SIZE: usize = 72;
