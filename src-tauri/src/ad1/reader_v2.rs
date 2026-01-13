// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Reader V2 - Improved from libad1 C implementation
//!
//! This module provides enhanced AD1 reading capabilities based on the
//! reference libad1 C implementation with improvements for:
//! - Better error handling
//! - Thread-safe operations
//! - Lazy loading support
//! - Efficient memory management

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, trace};

use super::types::*;

/// AD1 logical margin (header offset)
const AD1_LOGICAL_MARGIN: u64 = 512;

/// Item structure field offsets
pub const ITEM_NEXT_ADDR: u64 = 0x00;
pub const ITEM_FIRST_CHILD_ADDR: u64 = 0x08;
pub const ITEM_FIRST_METADATA_ADDR: u64 = 0x10;
pub const ITEM_ZLIB_METADATA_ADDR: u64 = 0x18;
pub const ITEM_DECOMPRESSED_SIZE: u64 = 0x20;
pub const ITEM_TYPE: u64 = 0x28;
pub const ITEM_NAME_LENGTH: u64 = 0x2C;
pub const ITEM_NAME: u64 = 0x30;

/// Metadata structure field offsets
pub const METADATA_NEXT_ADDR: u64 = 0x00;
pub const METADATA_CATEGORY: u64 = 0x08;
pub const METADATA_KEY: u64 = 0x0C;
pub const METADATA_DATA_LENGTH: u64 = 0x10;
pub const METADATA_DATA: u64 = 0x14;

/// Segment file handle with thread-safe access
#[derive(Debug)]
pub struct SegmentFile {
    pub segment_index: u32,
    pub filepath: PathBuf,
    pub size: u64,
    file: Arc<Mutex<File>>,
}

impl SegmentFile {
    pub fn open(filepath: PathBuf, segment_index: u32) -> Result<Self, Ad1Error> {
        let mut file = File::open(&filepath).map_err(|e| {
            Ad1Error::IoError(format!("Failed to open segment {}: {}", filepath.display(), e))
        })?;

        // Get file size
        let size = file
            .seek(SeekFrom::End(0))
            .map_err(|e| Ad1Error::IoError(format!("Failed to get file size: {}", e)))?;
        
        // Subtract logical margin to get data size
        let data_size = size.saturating_sub(AD1_LOGICAL_MARGIN);

        Ok(Self {
            segment_index,
            filepath,
            size: data_size,
            file: Arc::new(Mutex::new(file)),
        })
    }

    pub fn read_at(&self, offset: u64, length: u64) -> Result<Vec<u8>, Ad1Error> {
        let mut file = self.file.lock().map_err(|_| {
            Ad1Error::IoError("Failed to acquire file lock".to_string())
        })?;

        let mut buffer = vec![0u8; length as usize];
        
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| Ad1Error::IoError(format!("Seek failed: {}", e)))?;
        
        file.read_exact(&mut buffer)
            .map_err(|e| Ad1Error::IoError(format!("Read failed: {}", e)))?;

        Ok(buffer)
    }
}

/// Session V2 with improved multi-segment handling
#[derive(Debug)]
pub struct SessionV2 {
    pub segment_header: SegmentHeaderInfo,
    pub logical_header: LogicalHeaderInfo,
    pub segments: Vec<SegmentFile>,
    pub fragment_size: u64,
}

impl SessionV2 {
    /// Open AD1 file with all segments
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Ad1Error> {
        let path = path.as_ref();
        
        debug!("V2 opening file: {}", path.display());
        
        // Open first segment to read headers
        let mut first_file = File::open(path).map_err(|e| {
            Ad1Error::IoError(format!("Failed to open {}: {}", path.display(), e))
        })?;

        // Read segment header
        let segment_header = Self::read_segment_header(&mut first_file)?;
        
        // Read logical header
        let logical_header = Self::read_logical_header(&mut first_file)?;

        // Calculate fragment size
        let fragment_size = (segment_header.fragments_size as u64 * 65536)
            .saturating_sub(AD1_LOGICAL_MARGIN);

        debug!(
            "Opening AD1: {} segments, fragment_size={}",
            segment_header.segment_number, fragment_size
        );

        // Open all segments
        let mut segments = Vec::with_capacity(segment_header.segment_number as usize);
        
        for i in 1..=segment_header.segment_number {
            let segment_path = if i == 1 {
                path.to_path_buf()
            } else {
                Self::build_segment_path(path, i)?
            };

            let segment = SegmentFile::open(segment_path, i)?;
            segments.push(segment);
        }

        Ok(Self {
            segment_header,
            logical_header,
            segments,
            fragment_size,
        })
    }

    /// Build path for segment N (e.g., .ad1 -> .ad2, .ad3, etc.)
    fn build_segment_path(base_path: &Path, segment_num: u32) -> Result<PathBuf, Ad1Error> {
        let stem = base_path.file_stem().ok_or_else(|| {
            Ad1Error::InvalidFormat("Invalid file path".to_string())
        })?;
        
        let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
        
        Ok(parent.join(format!("{}.ad{}", stem.to_string_lossy(), segment_num)))
    }

    /// Read segment header from file
    fn read_segment_header(file: &mut File) -> Result<SegmentHeaderInfo, Ad1Error> {
        file.seek(SeekFrom::Start(0))
            .map_err(|e| Ad1Error::IoError(format!("Seek failed: {}", e)))?;

        let mut sig = [0u8; 16];
        file.read_exact(&mut sig)
            .map_err(|e| Ad1Error::IoError(format!("Read signature failed: {}", e)))?;

        // Check for encrypted AD1 (ADCRYPT signature)
        if &sig[..7] == b"ADCRYPT" {
            return Err(Ad1Error::EncryptedFile(
                "This AD1 file is encrypted. Encrypted AD1 files are not currently supported. \
                 Please decrypt the file using FTK Imager before opening.".to_string()
            ));
        }

        // Verify normal segment signature
        let expected = b"ADSEGMENTEDFILE\0";
        if &sig != expected {
            return Err(Ad1Error::InvalidFormat(format!(
                "Invalid segment signature: {:?}",
                sig
            )));
        }

        // Read fields at correct offsets (matching parser.rs utils.rs implementation)
        // These are NOT sequential! Must seek to each offset
        debug!("About to read segment header fields at offsets 0x18, 0x1C, 0x22, 0x28");
        
        file.seek(SeekFrom::Start(0x18))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x18 failed: {}", e)))?;
        let segment_index = Self::read_u32_le(file)?;    // At offset 0x18 = 24
        debug!("Read segment_index at 0x18: {}", segment_index);

        file.seek(SeekFrom::Start(0x1C))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x1C failed: {}", e)))?;
        let segment_number = Self::read_u32_le(file)?;   // At offset 0x1C = 28
        debug!("Read segment_number at 0x1C: {}", segment_number);

        file.seek(SeekFrom::Start(0x22))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x22 failed: {}", e)))?;
        let fragments_size = Self::read_u32_le(file)?;   // At offset 0x22 = 34
        debug!("Read fragments_size at 0x22: {}", fragments_size);

        file.seek(SeekFrom::Start(0x28))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x28 failed: {}", e)))?;
        let header_size = Self::read_u32_le(file)?;      // At offset 0x28 = 40
        debug!("Read header_size at 0x28: {}", header_size);

        debug!(
            "SegmentHeader: index={}, number={}, fragments_size={}, header_size={}",
            segment_index, segment_number, fragments_size, header_size
        );

        Ok(SegmentHeaderInfo {
            signature: String::from_utf8_lossy(&sig).to_string(),
            segment_index,
            segment_number,
            fragments_size,
            header_size,
        })
    }

    /// Read logical header from file
    fn read_logical_header(file: &mut File) -> Result<LogicalHeaderInfo, Ad1Error> {
        file.seek(SeekFrom::Start(AD1_LOGICAL_MARGIN))
            .map_err(|e| Ad1Error::IoError(format!("Seek failed: {}", e)))?;

        let mut sig = [0u8; 16];
        file.read_exact(&mut sig)
            .map_err(|e| Ad1Error::IoError(format!("Read signature failed: {}", e)))?;

        // Verify signature
        let expected = b"ADLOGICALIMAGE\0\0";
        if &sig != expected {
            return Err(Ad1Error::InvalidFormat(format!(
                "Invalid logical signature: {:?}",
                sig
            )));
        }

        // Read fields at correct absolute offsets (matching libad1)
        // libad1: image_version = read_int_little_endian(ad1_file, 0x210);
        file.seek(SeekFrom::Start(0x210))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x210 failed: {}", e)))?;
        let image_version = Self::read_u32_le(file)?;

        // libad1: zlib_chunk_size = read_int_little_endian(ad1_file, 0x218);
        file.seek(SeekFrom::Start(0x218))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x218 failed: {}", e)))?;
        let zlib_chunk_size = Self::read_u32_le(file)?;

        // libad1: logical_metadata_addr = read_long_little_endian(ad1_file, 0x21c);
        file.seek(SeekFrom::Start(0x21c))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x21c failed: {}", e)))?;
        let logical_metadata_addr = Self::read_u64_le(file)?;

        // libad1: first_item_addr = read_long_little_endian(ad1_file, 0x224);
        file.seek(SeekFrom::Start(0x224))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x224 failed: {}", e)))?;
        let first_item_addr = Self::read_u64_le(file)?;

        // libad1: data_source_name_length = read_int_little_endian(ad1_file, 0x22c);
        file.seek(SeekFrom::Start(0x22c))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x22c failed: {}", e)))?;
        let data_source_name_length = Self::read_u32_le(file)?;

        // Read AD signature at 0x230
        file.seek(SeekFrom::Start(0x230))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x230 failed: {}", e)))?;
        let mut ad_sig = [0u8; 4];
        file.read_exact(&mut ad_sig)
            .map_err(|e| Ad1Error::IoError(format!("Read AD signature failed: {}", e)))?;

        // libad1: data_source_name_addr = read_long_little_endian(ad1_file, 0x234);
        file.seek(SeekFrom::Start(0x234))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x234 failed: {}", e)))?;
        let data_source_name_addr = Self::read_u64_le(file)?;

        // libad1: attrguid_footer_addr = read_long_little_endian(ad1_file, 0x23c);
        file.seek(SeekFrom::Start(0x23c))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x23c failed: {}", e)))?;
        let attrguid_footer_addr = Self::read_u64_le(file)?;

        // libad1: locsguid_footer_addr = read_long_little_endian(ad1_file, 0x24c);
        file.seek(SeekFrom::Start(0x24c))
            .map_err(|e| Ad1Error::IoError(format!("Seek to 0x24c failed: {}", e)))?;
        let locsguid_footer_addr = Self::read_u64_le(file)?;

        // DEBUG: Log critical values
        debug!(
            "LogicalHeader: image_version={}, zlib_chunk_size={}, first_item_addr=0x{:X} ({}), metadata_addr=0x{:X}",
            image_version, zlib_chunk_size, first_item_addr, first_item_addr, logical_metadata_addr
        );

        // libad1: Read data source name at 0x25c
        let data_source_name = if data_source_name_length > 0 {
            file.seek(SeekFrom::Start(0x25c))
                .map_err(|e| Ad1Error::IoError(format!("Seek to 0x25c failed: {}", e)))?;
            let mut name_buf = vec![0u8; data_source_name_length as usize];
            file.read_exact(&mut name_buf)
                .map_err(|e| Ad1Error::IoError(format!("Read data source name failed: {}", e)))?;
            String::from_utf8_lossy(&name_buf).to_string()
        } else {
            "".to_string()
        };

        Ok(LogicalHeaderInfo {
            signature: String::from_utf8_lossy(&sig).to_string(),
            image_version,
            zlib_chunk_size,
            logical_metadata_addr,
            first_item_addr,
            data_source_name_length,
            ad_signature: String::from_utf8_lossy(&ad_sig).to_string(),
            data_source_name_addr,
            attrguid_footer_addr,
            locsguid_footer_addr,
            data_source_name,
        })
    }

    /// Arbitrary read across segments (handles multi-segment spanning)
    pub fn arbitrary_read(&self, offset: u64, length: u64) -> Result<Vec<u8>, Ad1Error> {
        let mut result = Vec::with_capacity(length as usize);
        let mut remaining = length;
        let mut current_offset = offset;

        while remaining > 0 {
            // Calculate which segment contains this offset
            let segment_idx = (current_offset / self.fragment_size) as usize;
            
            if segment_idx >= self.segments.len() {
                return Err(Ad1Error::OutOfRange {
                    offset: current_offset,
                    max: (self.segments.len() as u64) * self.fragment_size,
                });
            }

            let segment = &self.segments[segment_idx];
            let offset_in_segment = current_offset % self.fragment_size;
            
            // How much can we read from this segment?
            let available = segment.size.saturating_sub(offset_in_segment);
            let to_read = remaining.min(available);

            // Read from this segment (add logical margin to offset)
            let chunk = segment.read_at(offset_in_segment + AD1_LOGICAL_MARGIN, to_read)?;
            result.extend_from_slice(&chunk);

            remaining -= to_read;
            current_offset += to_read;
        }

        Ok(result)
    }

    /// Read u32 little-endian
    fn read_u32_le(file: &mut File) -> Result<u32, Ad1Error> {
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)
            .map_err(|e| Ad1Error::IoError(format!("Read u32 failed: {}", e)))?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Read u64 little-endian
    fn read_u64_le(file: &mut File) -> Result<u64, Ad1Error> {
        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)
            .map_err(|e| Ad1Error::IoError(format!("Read u64 failed: {}", e)))?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Read u32 at arbitrary offset
    pub fn read_u32_at(&self, offset: u64) -> Result<u32, Ad1Error> {
        let bytes = self.arbitrary_read(offset, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read u64 at arbitrary offset
    pub fn read_u64_at(&self, offset: u64) -> Result<u64, Ad1Error> {
        let bytes = self.arbitrary_read(offset, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read item header at offset (WITHOUT recursion or children)
    pub fn read_item_at(&self, offset: u64) -> Result<ItemHeader, Ad1Error> {
        if offset == 0 {
            return Err(Ad1Error::InvalidFormat("Item offset is 0".to_string()));
        }

        debug!(
            "Attempting to read item at offset=0x{:X} ({} decimal), fragment_size={}, segments={}",
            offset, offset, self.fragment_size, self.segments.len()
        );

        trace!("Reading item at offset 0x{:X}", offset);

        let next_item_addr = self.read_u64_at(offset + ITEM_NEXT_ADDR)?;
        debug!(
            "Read item @0x{:X}: next_item=0x{:X} ({} decimal)",
            offset, next_item_addr, next_item_addr
        );
        
        let first_child_addr = self.read_u64_at(offset + ITEM_FIRST_CHILD_ADDR)?;
        let first_metadata_addr = self.read_u64_at(offset + ITEM_FIRST_METADATA_ADDR)?;
        let zlib_metadata_addr = self.read_u64_at(offset + ITEM_ZLIB_METADATA_ADDR)?;
        let decompressed_size = self.read_u64_at(offset + ITEM_DECOMPRESSED_SIZE)?;
        let item_type = self.read_u32_at(offset + ITEM_TYPE)?;
        let name_length = self.read_u32_at(offset + ITEM_NAME_LENGTH)?;

        // Read item name bytes
        let name_bytes = self.arbitrary_read(offset + ITEM_NAME, name_length as u64)?;
        
        // DEBUG: Log first 20 bytes
        debug!(
            "Item name bytes (first 20): {:?}",
            &name_bytes[..std::cmp::min(20, name_bytes.len())]
        );
        
        // Convert to string, replacing slashes with underscores
        let name = Self::decode_item_name(&name_bytes);

        // Read parent folder address
        let parent_folder = self.read_u64_at(offset + ITEM_NAME + name_length as u64)?;

        Ok(ItemHeader {
            offset,
            next_item_addr,
            first_child_addr,
            first_metadata_addr,
            zlib_metadata_addr,
            decompressed_size,
            item_type,
            name_length,
            name,
            parent_folder,
        })
    }

    /// Decode item name from bytes (handles UTF-16LE and slash replacement)
    fn decode_item_name(bytes: &[u8]) -> String {
        // First try UTF-8 (like parser.rs does)
        let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
        let utf8_result = String::from_utf8_lossy(&bytes[..end]);
        
        // If UTF-8 decoding produced reasonable ASCII/UTF-8 text, use it
        if utf8_result.chars().all(|c| c.is_ascii() || !c.is_ascii_control()) {
            debug!("Item name (UTF-8): '{}'", utf8_result);
            return utf8_result
                .chars()
                .map(|c| if c == '/' { '_' } else { c })
                .collect();
        }
        
        // Otherwise try UTF-16LE
        let mut u16_chars = Vec::new();
        for chunk in bytes.chunks_exact(2) {
            u16_chars.push(u16::from_le_bytes([chunk[0], chunk[1]]));
        }
        let utf16_result = String::from_utf16_lossy(&u16_chars);
        debug!("Item name (UTF-16LE): '{}'", utf16_result);
        utf16_result
            .chars()
            .map(|c| if c == '/' { '_' } else { c })
            .collect()
    }

    /// Read metadata at offset
    pub fn read_metadata_at(&self, offset: u64) -> Result<MetadataEntry, Ad1Error> {
        if offset == 0 {
            return Err(Ad1Error::InvalidFormat("Metadata offset is 0".to_string()));
        }

        let next_metadata_addr = self.read_u64_at(offset + METADATA_NEXT_ADDR)?;
        let category = self.read_u32_at(offset + METADATA_CATEGORY)?;
        let key = self.read_u32_at(offset + METADATA_KEY)?;
        let data_length = self.read_u32_at(offset + METADATA_DATA_LENGTH)?;

        let data = self.arbitrary_read(offset + METADATA_DATA, data_length as u64)?;

        Ok(MetadataEntry {
            offset,
            next_metadata_addr,
            category,
            key,
            data_length,
            data,
        })
    }

    /// Read all metadata in chain starting at offset
    pub fn read_metadata_chain(&self, start_addr: u64) -> Result<Vec<MetadataEntry>, Ad1Error> {
        let mut metadata_list = Vec::new();
        let mut current_addr = start_addr;

        while current_addr != 0 {
            let metadata = self.read_metadata_at(current_addr)?;
            current_addr = metadata.next_metadata_addr;
            metadata_list.push(metadata);
        }

        Ok(metadata_list)
    }

    /// Read children items at address (non-recursive, returns immediate children only)
    pub fn read_children_at(&self, parent_addr: u64) -> Result<Vec<ItemHeader>, Ad1Error> {
        let mut children = Vec::new();
        let mut current_addr = parent_addr;

        while current_addr != 0 {
            let item = self.read_item_at(current_addr)?;
            current_addr = item.next_item_addr;
            children.push(item);
        }

        Ok(children)
    }

    /// Build full path for an item by traversing parent chain
    /// Based on libad1's build_item_path() function
    pub fn build_item_path(&self, item: &ItemHeader) -> String {
        let mut path_parts = vec![item.name.clone()];
        let mut current_parent = item.parent_folder;
        
        // Traverse parent chain (limit to prevent infinite loops)
        let mut depth = 0;
        const MAX_DEPTH: usize = 100;
        
        while current_parent != 0 && depth < MAX_DEPTH {
            match self.read_item_at(current_parent) {
                Ok(parent_item) => {
                    path_parts.push(parent_item.name.clone());
                    current_parent = parent_item.parent_folder;
                    depth += 1;
                }
                Err(e) => {
                    debug!("Failed to read parent at 0x{:X}: {}", current_parent, e);
                    break;
                }
            }
        }
        
        // Reverse to get root-first order
        path_parts.reverse();
        path_parts.join("/")
    }

    /// Get the path length for an item (for buffer allocation)
    pub fn get_path_length(&self, item: &ItemHeader) -> usize {
        self.build_item_path(item).len()
    }
}

/// Item header structure (matches libad1 ad1_item_header)
#[derive(Debug, Clone)]
pub struct ItemHeader {
    pub offset: u64,
    pub next_item_addr: u64,
    pub first_child_addr: u64,
    pub first_metadata_addr: u64,
    pub zlib_metadata_addr: u64,
    pub decompressed_size: u64,
    pub item_type: u32,
    pub name_length: u32,
    pub name: String,
    pub parent_folder: u64,
}

/// Metadata entry structure (matches libad1 ad1_metadata)
#[derive(Debug, Clone)]
pub struct MetadataEntry {
    pub offset: u64,
    pub next_metadata_addr: u64,
    pub category: u32,
    pub key: u32,
    pub data_length: u32,
    pub data: Vec<u8>,
}
