// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EwfHandle - Main interface for EWF file access (E01/L01/Ex01/Lx01)
//! Similar to libewf_handle in libewf

use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use tracing::{debug, instrument, trace};

use crate::common::{
    binary::{read_u32_le, read_u64_le},
    segments::discover_e01_segments,
    FileIoPool,
};
use crate::containers::ContainerError;

use super::cache::ChunkCache;
use super::types::*;

/// Parsed EWF sections result: (segments, volume, chunk_locations, stored_hashes, header_info)
type ParsedSectionsResult = (
    Vec<SegmentFile>,
    Option<VolumeSection>,
    Vec<ChunkLocation>,
    Vec<StoredImageHash>,
    HeaderInfo,
);

// =============================================================================
// EWF Handle - Main Interface (like libewf_handle)
// =============================================================================

/// Handle for EWF format files (E01, L01, Ex01, Lx01)
pub struct EwfHandle {
    /// File I/O pool managing all segment files
    pub(crate) file_pool: FileIoPool,
    /// Parsed segment file metadata
    pub(crate) segments: Vec<SegmentFile>,
    /// Volume information from first segment
    pub(crate) volume: VolumeSection,
    /// Global chunk table mapping chunk_index -> (segment_index, section_index, chunk_in_table)
    pub(crate) chunk_table: Vec<ChunkLocation>,
    /// Chunk data cache
    chunk_cache: ChunkCache,
    /// Stored image hashes from hash/digest sections
    pub(crate) stored_hashes: Vec<StoredImageHash>,
    /// Header metadata (case info, examiner, dates, etc.)
    pub(crate) header_info: HeaderInfo,
    /// Pre-computed segment boundary cumulative offsets for fast lookup
    segment_cumulative_sizes: Vec<u64>,
}

impl EwfHandle {
    /// Open EWF file set (like libewf_handle_open)
    #[instrument(skip_all, fields(path))]
    pub fn open(path: &str) -> Result<Self, ContainerError> {
        debug!(path, "Opening EWF handle");

        // Step 1: Discover all segment files (like libewf_glob)
        let segment_paths = discover_e01_segments(path)?;
        debug!(
            segment_count = segment_paths.len(),
            "Discovered EWF segments"
        );

        // Step 2: Create file I/O pool
        let mut file_pool = FileIoPool::new(segment_paths, MAX_OPEN_FILES);

        // Step 3: Get segment file sizes for global offset conversion
        let mut segment_sizes = Vec::new();
        for i in 0..file_pool.get_file_count() {
            let file = file_pool.get_file(i)?;
            let size = file.metadata()?.len();
            segment_sizes.push(size);
        }

        // Step 3b: Pre-compute cumulative sizes for fast segment lookup
        let mut segment_cumulative_sizes = Vec::with_capacity(segment_sizes.len());
        let mut cumulative = 0u64;
        for &size in &segment_sizes {
            segment_cumulative_sizes.push(cumulative);
            cumulative += size;
        }

        // Step 4: Parse sections globally (not per-segment!)
        let (segments, volume_info, chunk_table, stored_hashes, header_info) =
            Self::parse_sections_globally(&mut file_pool, &segment_sizes)?;

        let volume = volume_info.ok_or_else(|| {
            ContainerError::ParseError(
                "No volume section found in EWF container. \
             The container may be corrupted or incomplete."
                    .to_string(),
            )
        })?;
        debug!(
            chunk_count = volume.chunk_count,
            sectors_per_chunk = volume.sectors_per_chunk,
            sector_count = volume.sector_count,
            "EWF volume info parsed"
        );

        // Step 5: Create chunk cache
        let chunk_cache = ChunkCache::new(256); // Cache last 256 chunks

        debug!(
            chunk_table_size = chunk_table.len(),
            stored_hashes = stored_hashes.len(),
            "E01 handle opened successfully"
        );

        Ok(Self {
            file_pool,
            segments,
            volume,
            chunk_table,
            chunk_cache,
            stored_hashes,
            header_info,
            segment_cumulative_sizes,
        })
    }

    /// Parse sections globally across all segments (next_offset is global!)
    fn parse_sections_globally(
        file_pool: &mut FileIoPool,
        segment_sizes: &[u64],
    ) -> Result<ParsedSectionsResult, ContainerError> {
        // Initialize segment file structures
        let mut segments: Vec<SegmentFile> = (0..file_pool.get_file_count())
            .map(|i| SegmentFile {
                file_index: i,
                segment_number: (i + 1) as u16,
                file_size: segment_sizes[i],
                sections: Vec::new(),
            })
            .collect();

        let mut volume_info: Option<VolumeSection> = None;
        let mut chunk_locations = Vec::new();
        let mut stored_hashes: Vec<StoredImageHash> = Vec::new();
        let mut header_info = HeaderInfo::default();

        // Track sectors section for delta chunk scanning
        let mut sectors_data_offset: Option<u64> = None;
        let mut sectors_data_size: Option<u64> = None;

        // Start at offset 13 in first segment (after file header)
        let mut current_global_offset = 13u64;
        let mut last_sectors_offset: Option<u64> = None;
        let mut section_count = 0;
        const MAX_SECTIONS: u32 = 10000;

        trace!("Starting global section walk...");

        loop {
            if section_count >= MAX_SECTIONS {
                trace!("Reached max sections limit");
                break;
            }
            section_count += 1;

            // Convert global offset to (segment_index, offset_in_segment)
            let (mut seg_idx, offset_in_seg) =
                Self::global_to_segment_offset(current_global_offset, segment_sizes)?;

            // Check if we have enough space for section descriptor
            if offset_in_seg + 32 > segment_sizes[seg_idx] {
                trace!(
                    "Not enough space for section descriptor at global offset {}",
                    current_global_offset
                );
                break;
            }

            // Read section descriptor
            let file = file_pool.get_file(seg_idx)?;
            let section_desc = match Self::read_section_descriptor(file, offset_in_seg) {
                Ok(desc) => desc,
                Err(e) => {
                    trace!(
                        "Failed to read section at global offset {}: {}",
                        current_global_offset,
                        e
                    );
                    break;
                }
            };

            let section_type = String::from_utf8_lossy(&section_desc.section_type)
                .trim_matches('\0')
                .to_string();

            trace!(
                "Section '{}' at global offset {} (seg {}, offset {})",
                section_type,
                current_global_offset,
                seg_idx,
                offset_in_seg
            );

            // Create section entry
            let mut seg_section = SegmentSection {
                section_type: section_type.clone(),
                offset_in_segment: offset_in_seg,
                size: section_desc.size,
                data_offset: None,
                table_data: None,
            };

            // Handle different section types
            match section_type.as_str() {
                "header" => {
                    // Header section contains zlib-compressed case metadata
                    let data_global_offset = current_global_offset + 76;
                    let (data_seg_idx, data_offset_in_seg) =
                        Self::global_to_segment_offset(data_global_offset, segment_sizes)?;
                    seg_section.data_offset = Some(data_global_offset);

                    // Only parse first header section
                    if header_info.acquiry_date.is_none() {
                        if let Ok(parsed_header) = Self::read_header_section(
                            file_pool,
                            data_seg_idx,
                            data_offset_in_seg,
                            section_desc.size.saturating_sub(76),
                        ) {
                            trace!(
                                "  Parsed header: case={:?} examiner={:?} acquiry_date={:?}",
                                parsed_header.case_number,
                                parsed_header.examiner_name,
                                parsed_header.acquiry_date
                            );
                            header_info = parsed_header;
                        }
                    }
                }
                "volume" | "disk" => {
                    let data_global_offset = current_global_offset + 76;
                    let (data_seg_idx, data_offset_in_seg) =
                        Self::global_to_segment_offset(data_global_offset, segment_sizes)?;
                    seg_section.data_offset = Some(data_global_offset);

                    if volume_info.is_none() {
                        volume_info = Some(Self::read_volume_section(
                            file_pool,
                            data_seg_idx,
                            data_offset_in_seg,
                        )?);
                    }
                }
                "sectors" => {
                    let data_global_offset = current_global_offset + 76;
                    seg_section.data_offset = Some(data_global_offset);
                    last_sectors_offset = Some(data_global_offset);

                    sectors_data_offset = Some(data_global_offset);
                    sectors_data_size = Some(section_desc.size.saturating_sub(76));
                }
                "table" => {
                    let data_global_offset = current_global_offset + 76;
                    let (data_seg_idx, data_offset_in_seg) =
                        Self::global_to_segment_offset(data_global_offset, segment_sizes)?;
                    seg_section.data_offset = Some(data_global_offset);

                    if let Some(sectors_base) = last_sectors_offset {
                        trace!(
                            "  Reading {} at seg {} offset {}, sectors_base={}",
                            section_type,
                            data_seg_idx,
                            data_offset_in_seg,
                            sectors_base
                        );
                        let file = file_pool.get_file(data_seg_idx)?;
                        if let Ok(table) = Self::read_table_section(
                            file,
                            data_offset_in_seg,
                            section_desc.size,
                            sectors_base,
                        ) {
                            trace!(
                                "  Table has {} chunk offsets, base_offset={}",
                                table.offsets.len(),
                                table.base_offset
                            );
                            for (chunk_in_table, &offset) in table.offsets.iter().enumerate() {
                                chunk_locations.push(ChunkLocation {
                                    segment_index: seg_idx,
                                    section_index: segments[seg_idx].sections.len(),
                                    chunk_in_table,
                                    offset,
                                    base_offset: table.base_offset,
                                    sectors_base,
                                    is_delta_chunk: false,
                                });
                            }
                            seg_section.table_data = Some(table);
                        } else {
                            trace!("  Failed to read table section");
                        }
                    } else {
                        trace!("  Skipping {} - no sectors_base set", section_type);
                    }
                }
                "table2" => {
                    trace!("  Skipping table2 section (contains checksums)");
                }
                "hash" => {
                    let data_global_offset = current_global_offset + 76;
                    let (data_seg_idx, data_offset_in_seg) =
                        Self::global_to_segment_offset(data_global_offset, segment_sizes)?;

                    if let Ok(hashes) =
                        Self::read_hash_section(file_pool, data_seg_idx, data_offset_in_seg)
                    {
                        trace!("  Found {} hashes in hash section", hashes.len());
                        stored_hashes.extend(hashes);
                    }
                }
                "digest" => {
                    let data_global_offset = current_global_offset + 76;
                    let (data_seg_idx, data_offset_in_seg) =
                        Self::global_to_segment_offset(data_global_offset, segment_sizes)?;

                    if let Ok(hashes) = Self::read_digest_section(
                        file_pool,
                        data_seg_idx,
                        data_offset_in_seg,
                        section_desc.size,
                    ) {
                        trace!("  Found {} hashes in digest section", hashes.len());
                        stored_hashes.extend(hashes);
                    }
                }
                "done" => {
                    segments[seg_idx].sections.push(seg_section);
                    trace!("Reached 'done' section, stopping");
                    break;
                }
                "next" => {
                    if section_desc.next_offset == current_global_offset {
                        if seg_idx + 1 < segments.len() {
                            seg_idx += 1;
                            let next_segment_start: u64 = segment_sizes.iter().take(seg_idx).sum();
                            current_global_offset = next_segment_start + 13;
                            trace!(
                                "Moving to segment {} at global offset {}",
                                seg_idx,
                                current_global_offset
                            );
                            continue;
                        } else {
                            trace!("No more segments, stopping");
                            break;
                        }
                    }
                }
                _ => {}
            }

            segments[seg_idx].sections.push(seg_section);

            if section_desc.next_offset == 0 || section_desc.next_offset == current_global_offset {
                trace!("Section chain ended");
                break;
            }

            let segment_start: u64 = segment_sizes.iter().take(seg_idx).sum();
            current_global_offset = segment_start + section_desc.next_offset;
        }

        trace!(
            "Parsed {} sections, {} chunk locations",
            section_count,
            chunk_locations.len()
        );

        // If no chunk locations were found, try delta chunk scanning
        if chunk_locations.is_empty() {
            if let (Some(vol), Some(sectors_offset), Some(sectors_size)) =
                (&volume_info, sectors_data_offset, sectors_data_size)
            {
                trace!(
                    "No table found - attempting delta chunk scan at offset {} size {}",
                    sectors_offset,
                    sectors_size
                );

                if let Ok(delta_locations) = Self::scan_delta_chunks(
                    file_pool,
                    segment_sizes,
                    sectors_offset,
                    sectors_size,
                    vol.chunk_count as usize,
                    vol.sectors_per_chunk,
                    vol.bytes_per_sector,
                ) {
                    trace!("Found {} delta chunks", delta_locations.len());
                    chunk_locations = delta_locations;
                }
            }
        }

        Ok((
            segments,
            volume_info,
            chunk_locations,
            stored_hashes,
            header_info,
        ))
    }

    /// Scan for delta/inline chunks in sectors section
    fn scan_delta_chunks(
        file_pool: &mut FileIoPool,
        segment_sizes: &[u64],
        sectors_offset: u64,
        sectors_size: u64,
        expected_chunks: usize,
        sectors_per_chunk: u32,
        bytes_per_sector: u32,
    ) -> Result<Vec<ChunkLocation>, ContainerError> {
        let chunk_size = sectors_per_chunk as usize * bytes_per_sector as usize;
        let mut locations = Vec::with_capacity(expected_chunks);
        let mut current_offset = sectors_offset;
        let end_offset = sectors_offset + sectors_size;

        let (seg_idx, offset_in_seg) =
            Self::global_to_segment_offset(current_offset, segment_sizes)?;
        let file = file_pool.get_file(seg_idx)?;

        trace!("Scanning delta chunks: sectors_offset={}, sectors_size={}, expected_chunks={}, chunk_size={}", 
                 sectors_offset, sectors_size, expected_chunks, chunk_size);

        for chunk_idx in 0..expected_chunks {
            if current_offset >= end_offset {
                trace!("Delta scan reached end of sectors at chunk {}", chunk_idx);
                break;
            }

            let local_offset = current_offset - sectors_offset + offset_in_seg;

            file.seek(SeekFrom::Start(local_offset))?;

            let mut header = [0u8; 4];
            file.read_exact(&mut header)?;

            let raw_size = u32::from_le_bytes(header);
            let is_compressed = (raw_size & 0x80000000) != 0;
            let data_size = (raw_size & 0x7FFFFFFF) as u64;

            if chunk_idx < 5 {
                trace!(
                    "  Delta chunk[{}]: offset={}, raw_size={:#x}, compressed={}, data_size={}",
                    chunk_idx,
                    current_offset,
                    raw_size,
                    is_compressed,
                    data_size
                );
            }

            locations.push(ChunkLocation {
                segment_index: seg_idx,
                section_index: 0,
                chunk_in_table: chunk_idx,
                offset: raw_size as u64,
                base_offset: 0,
                sectors_base: current_offset,
                is_delta_chunk: true,
            });

            current_offset += 4 + data_size;

            if data_size < chunk_size as u64 && !is_compressed {
                trace!(
                    "  Warning: uncompressed chunk smaller than expected: {} < {}",
                    data_size,
                    chunk_size
                );
            }
        }

        Ok(locations)
    }

    /// Convert global byte offset to (segment_index, offset_in_segment)
    /// Static version for use during parsing
    fn global_to_segment_offset(
        global_offset: u64,
        segment_sizes: &[u64],
    ) -> Result<(usize, u64), ContainerError> {
        let mut cumulative = 0u64;
        for (idx, &size) in segment_sizes.iter().enumerate() {
            if global_offset < cumulative + size {
                return Ok((idx, global_offset - cumulative));
            }
            cumulative += size;
        }
        Err(ContainerError::ParseError(format!(
            "Global offset {} beyond all segments",
            global_offset
        )))
    }

    /// Fast segment lookup using pre-computed cumulative sizes with binary search
    #[inline]
    fn global_to_segment_fast(&self, global_offset: u64) -> Result<(usize, u64), ContainerError> {
        // Binary search for the segment containing this offset
        let segment_count = self.segment_cumulative_sizes.len();
        if segment_count == 0 {
            return Err(ContainerError::ParseError("No segments".into()));
        }

        // Binary search: find largest cumulative that's <= global_offset
        let mut lo = 0;
        let mut hi = segment_count;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if mid + 1 < segment_count && self.segment_cumulative_sizes[mid + 1] <= global_offset {
                lo = mid + 1;
            } else if self.segment_cumulative_sizes[mid] > global_offset {
                hi = mid;
            } else {
                // Found it - cumulative[mid] <= global_offset < cumulative[mid+1]
                let offset_in_segment = global_offset - self.segment_cumulative_sizes[mid];
                // Bounds check
                if offset_in_segment >= self.segments[mid].file_size {
                    return Err(ContainerError::ParseError(format!(
                        "Offset {} beyond segment {} size {}",
                        global_offset, mid, self.segments[mid].file_size
                    )));
                }
                return Ok((mid, offset_in_segment));
            }
        }

        // Check if we're in the last segment
        if lo < segment_count {
            let offset_in_segment = global_offset - self.segment_cumulative_sizes[lo];
            if offset_in_segment < self.segments[lo].file_size {
                return Ok((lo, offset_in_segment));
            }
        }

        Err(ContainerError::ParseError(format!(
            "Global offset {} beyond all segments",
            global_offset
        )))
    }

    /// Read multiple consecutive chunks efficiently - minimizes I/O by batching
    /// Returns decompressed chunks in order
    pub fn read_chunks_batch(
        &mut self,
        start_chunk: usize,
        count: usize,
    ) -> Result<Vec<Vec<u8>>, ContainerError> {
        let chunk_size =
            (self.volume.sectors_per_chunk as usize) * (self.volume.bytes_per_sector as usize);
        let total_chunks = self.get_chunk_count();
        let end_chunk = (start_chunk + count).min(total_chunks);
        let actual_count = end_chunk - start_chunk;

        if actual_count == 0 {
            return Ok(Vec::new());
        }

        // Collect chunk metadata first to plan I/O
        let segment_sizes: Vec<u64> = self.segments.iter().map(|s| s.file_size).collect();

        // Group chunks by segment for efficient sequential reading
        let mut segment_groups: Vec<(usize, Vec<(usize, ChunkLocation)>)> = Vec::new();
        let mut current_seg = usize::MAX;

        for chunk_idx in start_chunk..end_chunk {
            let location = match self.chunk_table.get(chunk_idx) {
                Some(loc) => loc.clone(),
                None => {
                    // Zero-filled chunk
                    if current_seg != usize::MAX {
                        segment_groups
                            .last_mut()
                            .expect("segment_groups non-empty when current_seg is set")
                            .1
                            .push((
                                chunk_idx,
                                ChunkLocation {
                                    segment_index: 0,
                                    section_index: 0,
                                    chunk_in_table: 0,
                                    offset: 0,
                                    base_offset: 0,
                                    sectors_base: 0,
                                    is_delta_chunk: false,
                                },
                            ));
                    } else {
                        segment_groups.push((
                            0,
                            vec![(
                                chunk_idx,
                                ChunkLocation {
                                    segment_index: 0,
                                    section_index: 0,
                                    chunk_in_table: 0,
                                    offset: 0,
                                    base_offset: 0,
                                    sectors_base: 0,
                                    is_delta_chunk: false,
                                },
                            )],
                        ));
                        current_seg = 0;
                    }
                    continue;
                }
            };

            if location.segment_index != current_seg {
                segment_groups.push((location.segment_index, vec![(chunk_idx, location)]));
                current_seg = segment_groups
                    .last()
                    .expect("just pushed to segment_groups")
                    .0;
            } else {
                segment_groups
                    .last_mut()
                    .expect("segment_groups non-empty when current_seg is set")
                    .1
                    .push((chunk_idx, location));
            }
        }

        // Read and decompress chunks
        let mut results: Vec<(usize, Vec<u8>)> = Vec::with_capacity(actual_count);

        for (seg_idx, chunks_in_seg) in segment_groups {
            let file = self.file_pool.get_file(self.segments[seg_idx].file_index)?;

            for (chunk_idx, location) in chunks_in_seg {
                // Zero-filled chunk
                if location.offset == 0 && location.sectors_base == 0 {
                    let final_size = if chunk_idx == total_chunks - 1 {
                        let remaining =
                            self.volume.sector_count % self.volume.sectors_per_chunk as u64;
                        if remaining > 0 {
                            (remaining * self.volume.bytes_per_sector as u64) as usize
                        } else {
                            chunk_size
                        }
                    } else {
                        chunk_size
                    };
                    results.push((chunk_idx, vec![0u8; final_size]));
                    continue;
                }

                // Calculate actual offset
                let (offset_in_segment, is_compressed) = if location.is_delta_chunk {
                    let is_compressed = (location.offset & 0x80000000) != 0;
                    let data_offset = location.sectors_base + 4;
                    let (_, offset_in_seg) =
                        Self::global_to_segment_offset(data_offset, &segment_sizes)?;
                    (offset_in_seg, is_compressed)
                } else {
                    let is_compressed = (location.offset & 0x80000000) != 0;
                    let offset_value = location.offset & 0x7FFFFFFF;
                    let segment_local_offset = if location.base_offset > 0 {
                        location.base_offset + offset_value
                    } else {
                        offset_value
                    };
                    let segment_start: u64 =
                        segment_sizes.iter().take(location.segment_index).sum();
                    let absolute_offset = segment_start + segment_local_offset;
                    let (_, offset_in_seg) =
                        Self::global_to_segment_offset(absolute_offset, &segment_sizes)?;
                    (offset_in_seg, is_compressed)
                };

                file.seek(SeekFrom::Start(offset_in_segment))?;

                let mut chunk_data = if is_compressed {
                    let buffered =
                        std::io::BufReader::with_capacity(65536, file.take(chunk_size as u64 * 2));
                    let mut decoder = ZlibDecoder::new(buffered);
                    let mut decompressed = Vec::with_capacity(chunk_size);
                    decoder.read_to_end(&mut decompressed)?;
                    decompressed
                } else {
                    let mut uncompressed = vec![0u8; chunk_size];
                    file.read_exact(&mut uncompressed)?;
                    uncompressed
                };

                // Truncate last chunk if needed
                if chunk_idx == total_chunks - 1 {
                    let remaining = self.volume.sector_count % self.volume.sectors_per_chunk as u64;
                    if remaining > 0 {
                        let final_size = (remaining * self.volume.bytes_per_sector as u64) as usize;
                        if chunk_data.len() > final_size {
                            chunk_data.truncate(final_size);
                        }
                    }
                }

                results.push((chunk_idx, chunk_data));
            }
        }

        // Sort by chunk index and extract data
        results.sort_by_key(|(idx, _)| *idx);
        Ok(results.into_iter().map(|(_, data)| data).collect())
    }

    // =========================================================================
    // Random Access Methods (like libewf_handle_read_buffer_at_offset)
    // =========================================================================

    /// Get the total size of the decompressed image in bytes
    pub fn get_media_size(&self) -> u64 {
        self.volume.sector_count * self.volume.bytes_per_sector as u64
    }

    /// Get chunk size in bytes
    pub fn get_chunk_size(&self) -> u32 {
        self.volume.sectors_per_chunk * self.volume.bytes_per_sector
    }

    /// Read bytes at arbitrary offset from the decompressed image
    /// This is the primary method for filesystem parsing
    pub fn read_at(&mut self, offset: u64, length: usize) -> Result<Vec<u8>, ContainerError> {
        let media_size = self.get_media_size();
        let chunk_size = self.get_chunk_size() as u64;

        // Bounds check
        if offset >= media_size {
            return Err(ContainerError::ParseError(format!(
                "Offset {} beyond media size {}",
                offset, media_size
            )));
        }

        // Adjust length if it would read past end
        let actual_length = std::cmp::min(length as u64, media_size - offset) as usize;
        if actual_length == 0 {
            return Ok(Vec::new());
        }

        // Calculate which chunks we need
        let start_chunk = (offset / chunk_size) as usize;
        let end_offset = offset + actual_length as u64;
        let end_chunk = end_offset.div_ceil(chunk_size) as usize;
        let chunk_count = end_chunk - start_chunk;

        // Read all needed chunks
        let chunks = if chunk_count > 1 {
            self.read_chunks_batch(start_chunk, chunk_count)?
        } else {
            vec![self.read_chunk(start_chunk)?]
        };

        // Concatenate chunk data
        let mut all_data: Vec<u8> = Vec::with_capacity(chunk_count * chunk_size as usize);
        for chunk in chunks {
            all_data.extend_from_slice(&chunk);
        }

        // Extract the requested byte range
        let offset_in_first_chunk = (offset % chunk_size) as usize;
        let end_pos = offset_in_first_chunk + actual_length;

        if end_pos > all_data.len() {
            // Handle case where last chunk is smaller (end of media)
            let available = all_data.len().saturating_sub(offset_in_first_chunk);
            Ok(all_data[offset_in_first_chunk..offset_in_first_chunk + available].to_vec())
        } else {
            Ok(all_data[offset_in_first_chunk..end_pos].to_vec())
        }
    }

    /// Read a single sector at the given sector index
    pub fn read_sector(&mut self, sector_index: u64) -> Result<Vec<u8>, ContainerError> {
        let offset = sector_index * self.volume.bytes_per_sector as u64;
        self.read_at(offset, self.volume.bytes_per_sector as usize)
    }

    /// Read multiple consecutive sectors
    pub fn read_sectors(
        &mut self,
        start_sector: u64,
        count: u64,
    ) -> Result<Vec<u8>, ContainerError> {
        let offset = start_sector * self.volume.bytes_per_sector as u64;
        let length = count as usize * self.volume.bytes_per_sector as usize;
        self.read_at(offset, length)
    }

    /// Read a chunk by global index (like libewf_handle_read_buffer)
    pub fn read_chunk(&mut self, chunk_index: usize) -> Result<Vec<u8>, ContainerError> {
        self.read_chunk_internal(chunk_index, true)
    }

    /// Read chunk without caching - optimized for sequential access patterns
    pub fn read_chunk_no_cache(&mut self, chunk_index: usize) -> Result<Vec<u8>, ContainerError> {
        self.read_chunk_internal(chunk_index, false)
    }

    fn read_chunk_internal(
        &mut self,
        chunk_index: usize,
        use_cache: bool,
    ) -> Result<Vec<u8>, ContainerError> {
        // Check cache first
        if use_cache {
            if let Some(cached_data) = self.chunk_cache.get(chunk_index) {
                return Ok(Arc::try_unwrap(cached_data).unwrap_or_else(|arc| (*arc).clone()));
            }
        }

        let chunk_size =
            (self.volume.sectors_per_chunk as usize) * (self.volume.bytes_per_sector as usize);

        let location = match self.chunk_table.get(chunk_index) {
            Some(loc) => loc.clone(),
            None => {
                let expected_chunks = self.volume.chunk_count as usize;
                if chunk_index >= expected_chunks {
                    return Err(ContainerError::ParseError(format!(
                        "Chunk {} beyond expected count {}",
                        chunk_index, expected_chunks
                    )));
                }

                let final_chunk_size = if chunk_index == expected_chunks - 1 {
                    let remaining_sectors =
                        self.volume.sector_count % self.volume.sectors_per_chunk as u64;
                    if remaining_sectors > 0 {
                        (remaining_sectors * self.volume.bytes_per_sector as u64) as usize
                    } else {
                        chunk_size
                    }
                } else {
                    chunk_size
                };

                return Ok(vec![0u8; final_chunk_size]);
            }
        };

        if location.offset == 0 && location.sectors_base == 0 {
            return Ok(vec![0u8; chunk_size]);
        }

        // Use fast lookup with pre-computed cumulative sizes
        let (seg_idx, offset_in_segment, is_compressed) = if location.is_delta_chunk {
            let is_compressed = (location.offset & 0x80000000) != 0;
            let data_offset = location.sectors_base + 4;

            let (seg_idx, offset_in_seg) = self.global_to_segment_fast(data_offset)?;

            if chunk_index < 3 {
                trace!(
                    "Delta chunk {}: sectors_base={} data_offset={} compressed={} seg={} local={}",
                    chunk_index,
                    location.sectors_base,
                    data_offset,
                    is_compressed,
                    seg_idx,
                    offset_in_seg
                );
            }

            (seg_idx, offset_in_seg, is_compressed)
        } else {
            let is_compressed = (location.offset & 0x80000000) != 0;
            let offset_value = location.offset & 0x7FFFFFFF;

            let segment_local_offset = if location.base_offset > 0 {
                location.base_offset + offset_value
            } else {
                offset_value
            };

            // Use fast lookup
            let segment_start = self.segment_cumulative_sizes[location.segment_index];
            let absolute_offset = segment_start + segment_local_offset;

            let (seg_idx, offset_in_segment) = self.global_to_segment_fast(absolute_offset)?;

            if chunk_index < 3 {
                trace!("Chunk {}: offset={:#x} compressed={} offset_value={} base_offset={} sectors_base={} absolute={} seg={} local={}", 
                         chunk_index, location.offset, is_compressed, offset_value, location.base_offset, location.sectors_base, absolute_offset, seg_idx, offset_in_segment);
            }

            (seg_idx, offset_in_segment, is_compressed)
        };

        let file = self.file_pool.get_file(self.segments[seg_idx].file_index)?;

        file.seek(SeekFrom::Start(offset_in_segment))?;

        let mut chunk_data = if is_compressed {
            let buffered =
                std::io::BufReader::with_capacity(65536, file.take(chunk_size as u64 * 2));
            let mut decoder = ZlibDecoder::new(buffered);
            let mut decompressed = Vec::with_capacity(chunk_size);
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        } else {
            let mut uncompressed = vec![0u8; chunk_size];
            file.read_exact(&mut uncompressed)?;
            uncompressed
        };

        // Truncate last chunk if needed
        let expected_chunks = self
            .volume
            .sector_count
            .div_ceil(self.volume.sectors_per_chunk as u64);
        if chunk_index == (expected_chunks as usize - 1) {
            let remaining_sectors = self.volume.sector_count % self.volume.sectors_per_chunk as u64;
            if remaining_sectors > 0 {
                let final_size = (remaining_sectors * self.volume.bytes_per_sector as u64) as usize;
                trace!(
                    "Last chunk {}: original size={}, remaining_sectors={}, truncating to {}",
                    chunk_index,
                    chunk_data.len(),
                    remaining_sectors,
                    final_size
                );
                if chunk_data.len() > final_size {
                    chunk_data.truncate(final_size);
                }
            }
        }

        if use_cache {
            self.chunk_cache.insert(chunk_index, chunk_data.clone());
        }

        Ok(chunk_data)
    }

    pub fn get_volume_info(&self) -> &VolumeSection {
        &self.volume
    }

    pub fn get_chunk_count(&self) -> usize {
        self.volume.chunk_count as usize
    }
    /// Get the number of chunks actually stored in the table
    pub fn get_stored_chunk_count(&self) -> usize {
        self.chunk_table.len()
    }

    // =========================================================================
    // Section Reading Helper Methods
    // =========================================================================

    fn read_section_descriptor(
        file: &mut File,
        offset: u64,
    ) -> Result<SectionDescriptor, ContainerError> {
        file.seek(SeekFrom::Start(offset))?;

        let mut section_type = [0u8; 16];
        file.read_exact(&mut section_type)?;

        let next_offset = read_u64_le(file)?;
        let size = read_u64_le(file)?;

        Ok(SectionDescriptor {
            section_type,
            next_offset,
            size,
        })
    }

    fn read_volume_section(
        file_pool: &mut FileIoPool,
        file_index: usize,
        offset: u64,
    ) -> Result<VolumeSection, ContainerError> {
        trace!(
            "read_volume_section: file_index={}, offset={}",
            file_index,
            offset
        );

        let file = file_pool.get_file(file_index)?;

        file.seek(SeekFrom::Start(offset))?;

        let mut raw_bytes = [0u8; 20];
        file.read_exact(&mut raw_bytes)?;
        trace!("Raw volume bytes: {:02x?}", &raw_bytes);

        file.seek(SeekFrom::Start(offset))?;

        let _media_and_padding = read_u32_le(file)?;
        let chunk_count = read_u32_le(file)?;
        let sectors_per_chunk = read_u32_le(file)?;
        let bytes_per_sector = read_u32_le(file)?;
        let sector_count = read_u64_le(file)?;

        trace!(
            "Volume: chunk_count={}, sectors_per_chunk={}, bytes_per_sector={}, sector_count={}",
            chunk_count,
            sectors_per_chunk,
            bytes_per_sector,
            sector_count
        );

        Ok(VolumeSection {
            chunk_count,
            sectors_per_chunk,
            bytes_per_sector,
            sector_count,
            compression_level: 1,
        })
    }

    fn read_table_section(
        file: &mut File,
        offset: u64,
        size: u64,
        _sectors_base: u64,
    ) -> Result<TableSection, ContainerError> {
        file.seek(SeekFrom::Start(offset))?;

        let mut header_bytes = [0u8; 24];
        file.read_exact(&mut header_bytes)?;

        trace!("    Raw table header bytes: {:02x?}", &header_bytes);

        let entry_count = u32::from_le_bytes([
            header_bytes[0],
            header_bytes[1],
            header_bytes[2],
            header_bytes[3],
        ]);
        let base_offset = u64::from_le_bytes([
            header_bytes[8],
            header_bytes[9],
            header_bytes[10],
            header_bytes[11],
            header_bytes[12],
            header_bytes[13],
            header_bytes[14],
            header_bytes[15],
        ]);

        let chunk_count = if entry_count > 0 {
            entry_count
        } else {
            ((size.saturating_sub(24 + 4)) / 4) as u32
        };

        trace!(
            "    Table: entry_count={}, base_offset={}, using_count={}",
            entry_count,
            base_offset,
            chunk_count
        );

        let mut offsets = Vec::with_capacity(chunk_count as usize);
        for i in 0..chunk_count {
            let raw_offset = read_u32_le(file)? as u64;
            offsets.push(raw_offset);

            if i < 5 {
                let is_compressed = (raw_offset & 0x80000000) != 0;
                let offset_value = raw_offset & 0x7FFFFFFF;
                let current_pos = file.stream_position().unwrap_or(0);
                trace!(
                    "      Offset[{}] at file_pos={}: raw={:#x} compressed={} value={}",
                    i,
                    current_pos - 4,
                    raw_offset,
                    is_compressed,
                    offset_value
                );
            }
        }

        Ok(TableSection {
            chunk_count,
            base_offset,
            offsets,
        })
    }

    /// Read hash section from EWF file (EWF1 format)
    fn read_hash_section(
        file_pool: &mut FileIoPool,
        file_index: usize,
        offset: u64,
    ) -> Result<Vec<StoredImageHash>, ContainerError> {
        let file = file_pool.get_file(file_index)?;
        file.seek(SeekFrom::Start(offset))?;

        let mut hashes = Vec::new();

        let mut md5_bytes = [0u8; 16];
        if file.read_exact(&mut md5_bytes).is_ok() && md5_bytes.iter().any(|&b| b != 0) {
            let md5_hash = md5_bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            hashes.push(StoredImageHash {
                algorithm: "MD5".to_string(),
                hash: md5_hash,
                verified: None,
                timestamp: None,
                source: Some("container".to_string()),
                offset: Some(offset),
                size: Some(16),
            });
        }

        Ok(hashes)
    }

    /// Read digest section from EWF2 format
    fn read_digest_section(
        file_pool: &mut FileIoPool,
        file_index: usize,
        offset: u64,
        size: u64,
    ) -> Result<Vec<StoredImageHash>, ContainerError> {
        let file = file_pool.get_file(file_index)?;
        file.seek(SeekFrom::Start(offset))?;

        let mut hashes = Vec::new();
        let mut current_offset = offset;

        let mut md5_bytes = [0u8; 16];
        if file.read_exact(&mut md5_bytes).is_ok() && md5_bytes.iter().any(|&b| b != 0) {
            let md5_hash = md5_bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            hashes.push(StoredImageHash {
                algorithm: "MD5".to_string(),
                hash: md5_hash,
                verified: None,
                timestamp: None,
                source: Some("container".to_string()),
                offset: Some(current_offset),
                size: Some(16),
            });
            current_offset += 16;
        }

        if size >= 36 {
            let mut sha1_bytes = [0u8; 20];
            if file.read_exact(&mut sha1_bytes).is_ok() && sha1_bytes.iter().any(|&b| b != 0) {
                let sha1_hash = sha1_bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                hashes.push(StoredImageHash {
                    algorithm: "SHA1".to_string(),
                    hash: sha1_hash,
                    verified: None,
                    timestamp: None,
                    source: Some("container".to_string()),
                    offset: Some(current_offset),
                    size: Some(20),
                });
            }
        }

        Ok(hashes)
    }

    /// Read header section containing case metadata (zlib-compressed)
    fn read_header_section(
        file_pool: &mut FileIoPool,
        file_index: usize,
        offset: u64,
        size: u64,
    ) -> Result<HeaderInfo, ContainerError> {
        let file = file_pool.get_file(file_index)?;
        file.seek(SeekFrom::Start(offset))?;

        // Read compressed data
        let mut compressed = vec![0u8; size as usize];
        file.read_exact(&mut compressed)?;

        // Find zlib magic (78 9c) and decompress
        let zlib_start = compressed
            .windows(2)
            .position(|w| w == [0x78, 0x9c])
            .unwrap_or(0);

        let mut decoder = ZlibDecoder::new(&compressed[zlib_start..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed)?;

        // Parse tab-separated header format:
        // Line 0: version (e.g., "1")
        // Line 1: "main" or category
        // Line 2: column headers (tab-separated): c, n, a, e, t, av, ov, m, u, p, r
        // Line 3: values (tab-separated)
        let lines: Vec<&str> = decompressed.lines().collect();

        let mut header = HeaderInfo::default();

        if lines.len() >= 4 {
            let headers: Vec<&str> = lines[2].split('\t').collect();
            let values: Vec<&str> = lines[3].split('\t').collect();

            for (i, &col) in headers.iter().enumerate() {
                if i < values.len() {
                    let val = values[i].trim();
                    if !val.is_empty() {
                        match col {
                            "c" => header.case_number = Some(val.to_string()),
                            "n" => header.evidence_number = Some(val.to_string()),
                            "a" => header.description = Some(val.to_string()),
                            "e" => header.examiner_name = Some(val.to_string()),
                            "t" => header.notes = Some(val.to_string()),
                            "av" => header.acquiry_sw_version = Some(val.to_string()),
                            "ov" => header.acquiry_os = Some(val.to_string()),
                            "m" => {
                                // Parse date format "2017 12 14 11 52 41" -> "2017-12-14 11:52:41"
                                if let Some(formatted) = Self::parse_ewf_date(val) {
                                    header.acquiry_date = Some(formatted);
                                }
                            }
                            "u" => {
                                if let Some(formatted) = Self::parse_ewf_date(val) {
                                    header.system_date = Some(formatted);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(header)
    }

    /// Parse EWF date format "YYYY MM DD HH MM SS" to "YYYY-MM-DD HH:MM:SS"
    fn parse_ewf_date(s: &str) -> Option<String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() >= 6 {
            Some(format!(
                "{}-{}-{} {}:{}:{}",
                parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
            ))
        } else {
            None
        }
    }
}
