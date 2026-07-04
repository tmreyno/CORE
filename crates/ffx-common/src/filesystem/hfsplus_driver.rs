// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Allow dead code for HFS+ binary structures - many fields are parsed but not yet used
#![allow(dead_code)]

//! # HFS+ Filesystem Driver
//!
//! Implements read-only HFS+ filesystem access for macOS/iOS disk images.
//! Based on Apple's HFS+ specification.
//!
//! ## References
//! - Apple Technical Note TN1150: HFS Plus Volume Format
//! - https://developer.apple.com/library/archive/technotes/tn/tn1150.html

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::traits::{FilesystemDriver, FilesystemInfo, FilesystemType, SeekableBlockDevice};
use crate::vfs::{normalize_path, DirEntry, FileAttr, VfsError};

// =============================================================================
// HFS+ Constants
// =============================================================================

/// HFS+ signature 'H+' (0x482B)
const HFSPLUS_SIGNATURE: u16 = 0x482B;
/// HFSX signature 'HX' (0x4858) - case-sensitive HFS+
const HFSX_SIGNATURE: u16 = 0x4858;
/// HFS+ volume header offset (always at sector 2, 1024 bytes)
const VOLUME_HEADER_OFFSET: u64 = 1024;
/// HFS+ volume header size
const VOLUME_HEADER_SIZE: usize = 512;

/// Catalog file ID (CNID)
const CATALOG_FILE_ID: u32 = 4;

fn checked_buffer_end(start: usize, len: usize) -> Option<usize> {
    start.checked_add(len)
}

fn has_buffer_range(buf: &[u8], start: usize, len: usize) -> bool {
    match checked_buffer_end(start, len) {
        Some(end) => end <= buf.len(),
        None => false,
    }
}

fn checked_record_offset(node_size: usize, index: usize) -> Option<usize> {
    let distance_from_end = index.checked_mul(2)?.checked_add(2)?;
    node_size.checked_sub(distance_from_end)
}

fn checked_u16_capacity(length: usize, context: &str) -> Result<Vec<u16>, VfsError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|_| VfsError::IoError(format!("{} allocation too large", context)))?;
    Ok(values)
}

fn checked_hfsplus_mul_u64(lhs: u64, rhs: u64, context: &str) -> Result<u64, VfsError> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| VfsError::IoError(format!("{} overflowed u64", context)))
}

fn checked_hfsplus_add_u64(lhs: u64, rhs: u64, context: &str) -> Result<u64, VfsError> {
    lhs.checked_add(rhs)
        .ok_or_else(|| VfsError::IoError(format!("{} overflowed u64", context)))
}

fn checked_hfsplus_block_offset(
    base_offset: u64,
    start_block: u64,
    block_size: u64,
    context: &str,
) -> Result<u64, VfsError> {
    let block_offset = checked_hfsplus_mul_u64(start_block, block_size, context)?;
    checked_hfsplus_add_u64(base_offset, block_offset, context)
}

fn checked_hfsplus_read_advance(current_offset: u64, bytes_read: usize) -> Result<u64, VfsError> {
    let bytes_read = u64::try_from(bytes_read)
        .map_err(|_| VfsError::IoError("HFS+ read byte count exceeded u64".into()))?;
    checked_hfsplus_add_u64(current_offset, bytes_read, "HFS+ logical read offset")
}

fn bounded_hfsplus_read_len(logical_size: u64, offset: u64, requested: usize) -> Option<usize> {
    let remaining = logical_size.checked_sub(offset)?;
    Some(requested.min(usize::try_from(remaining).unwrap_or(usize::MAX)))
}

fn read_hfsplus_exact(
    device: &dyn SeekableBlockDevice,
    offset: u64,
    buf: &mut [u8],
    context: &str,
) -> Result<(), VfsError> {
    let bytes_read = device
        .read_at(offset, buf)
        .map_err(|e| VfsError::IoError(e.to_string()))?;

    if bytes_read != buf.len() {
        return Err(VfsError::IoError(format!(
            "{} short read: expected {} bytes, got {}",
            context,
            buf.len(),
            bytes_read
        )));
    }

    Ok(())
}
/// Extents overflow file ID
#[allow(dead_code)]
const EXTENTS_FILE_ID: u32 = 3;
/// Allocation file ID
#[allow(dead_code)]
const ALLOCATION_FILE_ID: u32 = 5;
/// Attributes file ID
#[allow(dead_code)]
const ATTRIBUTES_FILE_ID: u32 = 8;

/// Root folder ID
const ROOT_FOLDER_ID: u32 = 2;

/// B-tree node types
const LEAF_NODE: i8 = -1;
const INDEX_NODE: i8 = 0;
const HEADER_NODE: i8 = 1;
#[allow(dead_code)]
const MAP_NODE: i8 = 2;

/// Catalog record types
const FOLDER_RECORD: u16 = 0x0001;
const FILE_RECORD: u16 = 0x0002;
const FOLDER_THREAD_RECORD: u16 = 0x0003;
const FILE_THREAD_RECORD: u16 = 0x0004;

// =============================================================================
// HFS+ Structures
// =============================================================================

/// HFS+ Volume Header
#[derive(Debug, Clone)]
struct HfsPlusVolumeHeader {
    /// Signature ('H+' or 'HX')
    signature: u16,
    /// Version (4 for HFS+, 5 for HFSX)
    version: u16,
    /// Volume attributes
    #[allow(dead_code)]
    attributes: u32,
    /// Block size in bytes
    block_size: u32,
    /// Total blocks
    total_blocks: u32,
    /// Free blocks
    #[allow(dead_code)]
    free_blocks: u32,
    /// Allocation file fork data
    #[allow(dead_code)]
    allocation_file: HfsPlusForkData,
    /// Extents overflow file fork data
    #[allow(dead_code)]
    extents_file: HfsPlusForkData,
    /// Catalog file fork data
    catalog_file: HfsPlusForkData,
    /// Attributes file fork data
    #[allow(dead_code)]
    attributes_file: HfsPlusForkData,
    /// Startup file fork data
    #[allow(dead_code)]
    startup_file: HfsPlusForkData,
}

/// HFS+ Fork Data (extent information)
#[derive(Debug, Clone, Default)]
struct HfsPlusForkData {
    /// Logical size in bytes
    logical_size: u64,
    /// Clump size
    #[allow(dead_code)]
    clump_size: u32,
    /// Total blocks
    #[allow(dead_code)]
    total_blocks: u32,
    /// First 8 extent records
    extents: [HfsPlusExtentDescriptor; 8],
}

/// HFS+ Extent Descriptor
#[derive(Debug, Clone, Copy, Default)]
struct HfsPlusExtentDescriptor {
    /// Start block
    start_block: u32,
    /// Block count
    block_count: u32,
}

/// B-tree Node Descriptor
#[derive(Debug, Clone)]
struct BTNodeDescriptor {
    /// Forward link
    #[allow(dead_code)]
    flink: u32,
    /// Backward link
    #[allow(dead_code)]
    blink: u32,
    /// Node type
    kind: i8,
    /// Node height
    #[allow(dead_code)]
    height: u8,
    /// Number of records
    num_records: u16,
}

/// B-tree Header Record
#[derive(Debug, Clone)]
struct BTHeaderRecord {
    /// Tree depth
    #[allow(dead_code)]
    tree_depth: u16,
    /// Root node number
    root_node: u32,
    /// Total leaf records
    #[allow(dead_code)]
    leaf_records: u32,
    /// First leaf node
    #[allow(dead_code)]
    first_leaf_node: u32,
    /// Last leaf node
    #[allow(dead_code)]
    last_leaf_node: u32,
    /// Node size
    node_size: u16,
    /// Max key length
    #[allow(dead_code)]
    max_key_length: u16,
    /// Total nodes
    #[allow(dead_code)]
    total_nodes: u32,
    /// Free nodes
    #[allow(dead_code)]
    free_nodes: u32,
}

/// Catalog key
#[derive(Debug, Clone)]
struct HfsPlusCatalogKey {
    /// Parent folder ID (CNID)
    parent_id: u32,
    /// Node name (UTF-16BE)
    node_name: String,
}

/// Catalog folder record
#[derive(Debug, Clone)]
struct HfsPlusCatalogFolder {
    /// Record type (should be FOLDER_RECORD)
    #[allow(dead_code)]
    record_type: u16,
    /// Flags
    #[allow(dead_code)]
    flags: u16,
    /// Folder ID (CNID)
    folder_id: u32,
    /// Create date
    #[allow(dead_code)]
    create_date: u32,
    /// Content modification date
    #[allow(dead_code)]
    content_mod_date: u32,
    /// Attribute modification date
    #[allow(dead_code)]
    attribute_mod_date: u32,
    /// Access date
    #[allow(dead_code)]
    access_date: u32,
    /// Backup date
    #[allow(dead_code)]
    backup_date: u32,
    /// BSD info
    #[allow(dead_code)]
    bsd_info: HfsPlusBsdInfo,
    /// Valence (number of items in folder)
    valence: u32,
}

/// Catalog file record
#[derive(Debug, Clone)]
struct HfsPlusCatalogFile {
    /// Record type (should be FILE_RECORD)
    #[allow(dead_code)]
    record_type: u16,
    /// Flags
    #[allow(dead_code)]
    flags: u16,
    /// File ID (CNID)
    file_id: u32,
    /// Create date
    #[allow(dead_code)]
    create_date: u32,
    /// Content modification date
    #[allow(dead_code)]
    content_mod_date: u32,
    /// Attribute modification date
    #[allow(dead_code)]
    attribute_mod_date: u32,
    /// Access date
    #[allow(dead_code)]
    access_date: u32,
    /// Backup date
    #[allow(dead_code)]
    backup_date: u32,
    /// BSD info
    #[allow(dead_code)]
    bsd_info: HfsPlusBsdInfo,
    /// Data fork
    data_fork: HfsPlusForkData,
    /// Resource fork
    #[allow(dead_code)]
    resource_fork: HfsPlusForkData,
}

/// BSD-style permissions info
#[derive(Debug, Clone, Default)]
struct HfsPlusBsdInfo {
    /// Owner ID
    #[allow(dead_code)]
    owner_id: u32,
    /// Group ID
    #[allow(dead_code)]
    group_id: u32,
    /// Admin flags
    #[allow(dead_code)]
    admin_flags: u8,
    /// Owner flags
    #[allow(dead_code)]
    owner_flags: u8,
    /// File mode
    #[allow(dead_code)]
    file_mode: u16,
    /// Special (for symlinks/devices)
    #[allow(dead_code)]
    special: u32,
}

/// Catalog entry (either folder or file)
#[derive(Debug, Clone)]
enum CatalogEntry {
    Folder(HfsPlusCatalogFolder),
    File(HfsPlusCatalogFile),
}

// =============================================================================
// HFS+ Driver
// =============================================================================

/// HFS+ filesystem driver
pub struct HfsPlusDriver {
    /// Filesystem info
    info: FilesystemInfo,
    /// Block device
    device: Arc<dyn SeekableBlockDevice>,
    /// Partition offset
    offset: u64,
    /// Volume header
    header: HfsPlusVolumeHeader,
    /// Catalog B-tree header
    catalog_header: BTHeaderRecord,
    /// Directory cache: path -> list of entries
    dir_cache: RwLock<HashMap<String, Vec<(String, CatalogEntry)>>>,
}

impl HfsPlusDriver {
    /// Create a new HFS+ driver
    pub fn new(
        device: Box<dyn SeekableBlockDevice>,
        offset: u64,
        size: u64,
    ) -> Result<Self, VfsError> {
        let device: Arc<dyn SeekableBlockDevice> = Arc::from(device);

        // Read volume header
        let mut buf = vec![0u8; VOLUME_HEADER_SIZE];
        let volume_header_offset =
            checked_hfsplus_add_u64(offset, VOLUME_HEADER_OFFSET, "HFS+ volume header offset")?;
        device
            .read_at(volume_header_offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        let header = Self::parse_volume_header(&buf)?;

        // Validate signature
        if header.signature != HFSPLUS_SIGNATURE && header.signature != HFSX_SIGNATURE {
            return Err(VfsError::Internal(format!(
                "Invalid HFS+ signature: 0x{:04X}",
                header.signature
            )));
        }

        // Read catalog B-tree header
        let catalog_header = Self::read_btree_header(&device, offset, &header)?;

        let total_size = checked_hfsplus_mul_u64(
            header.block_size as u64,
            header.total_blocks as u64,
            "HFS+ total size",
        )?;
        let free_space = checked_hfsplus_mul_u64(
            header.free_blocks as u64,
            header.block_size as u64,
            "HFS+ free space",
        )?;

        let info = FilesystemInfo {
            // HFSX is still HFS+ (case-sensitive variant)
            fs_type: FilesystemType::HfsPlus,
            label: None, // Could extract from catalog
            total_size: total_size.min(size),
            free_space: Some(free_space),
            cluster_size: header.block_size,
        };

        Ok(Self {
            info,
            device,
            offset,
            header,
            catalog_header,
            dir_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Parse volume header from buffer
    fn parse_volume_header(buf: &[u8]) -> Result<HfsPlusVolumeHeader, VfsError> {
        if buf.len() < VOLUME_HEADER_SIZE {
            return Err(VfsError::IoError(
                "Buffer too small for volume header".into(),
            ));
        }

        let signature = u16::from_be_bytes([buf[0], buf[1]]);
        let version = u16::from_be_bytes([buf[2], buf[3]]);
        let attributes = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let block_size = u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]);
        let total_blocks = u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]);
        let free_blocks = u32::from_be_bytes([buf[48], buf[49], buf[50], buf[51]]);

        // Parse fork data for special files (HFS+ TN1150 actual offsets)
        // The volume header structure places fork data at these offsets:
        // allocationFile starts at offset 112
        // extentsFile starts at offset 192
        // catalogFile starts at offset 272
        // attributesFile starts at offset 352
        // startupFile starts at offset 432
        let allocation_file = Self::parse_fork_data(&buf[112..192]);
        let extents_file = Self::parse_fork_data(&buf[192..272]);
        let catalog_file = Self::parse_fork_data(&buf[272..352]);
        let attributes_file = Self::parse_fork_data(&buf[352..432]);
        let startup_file = if buf.len() >= 512 {
            Self::parse_fork_data(&buf[432..512])
        } else {
            HfsPlusForkData::default()
        };

        Ok(HfsPlusVolumeHeader {
            signature,
            version,
            attributes,
            block_size,
            total_blocks,
            free_blocks,
            allocation_file,
            extents_file,
            catalog_file,
            attributes_file,
            startup_file,
        })
    }

    /// Parse fork data from buffer
    fn parse_fork_data(buf: &[u8]) -> HfsPlusForkData {
        if buf.len() < 80 {
            return HfsPlusForkData::default();
        }

        let logical_size = u64::from_be_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ]);
        let clump_size = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let total_blocks = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);

        let mut extents = [HfsPlusExtentDescriptor::default(); 8];
        for (i, extent) in extents.iter_mut().enumerate() {
            let ext_offset = 16 + i * 8;
            if ext_offset + 8 <= buf.len() {
                *extent = HfsPlusExtentDescriptor {
                    start_block: u32::from_be_bytes([
                        buf[ext_offset],
                        buf[ext_offset + 1],
                        buf[ext_offset + 2],
                        buf[ext_offset + 3],
                    ]),
                    block_count: u32::from_be_bytes([
                        buf[ext_offset + 4],
                        buf[ext_offset + 5],
                        buf[ext_offset + 6],
                        buf[ext_offset + 7],
                    ]),
                };
            }
        }

        HfsPlusForkData {
            logical_size,
            clump_size,
            total_blocks,
            extents,
        }
    }

    /// Read B-tree header from catalog file
    fn read_btree_header(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
        header: &HfsPlusVolumeHeader,
    ) -> Result<BTHeaderRecord, VfsError> {
        // Read first node of catalog B-tree (header node)
        let first_extent = &header.catalog_file.extents[0];

        tracing::debug!(
            "HFS+ catalog_file: logical_size={}, total_blocks={}, first_extent: start={}, count={}",
            header.catalog_file.logical_size,
            header.catalog_file.total_blocks,
            first_extent.start_block,
            first_extent.block_count
        );

        if first_extent.block_count == 0 {
            return Err(VfsError::IoError("Catalog file has no extents".into()));
        }

        let catalog_offset = checked_hfsplus_block_offset(
            offset,
            first_extent.start_block as u64,
            header.block_size as u64,
            "HFS+ catalog header offset",
        )?;

        // Read header node (at least 512 bytes for header record)
        let mut node_buf = vec![0u8; 512];
        device
            .read_at(catalog_offset, &mut node_buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        // Parse node descriptor
        let node_desc = Self::parse_node_descriptor(&node_buf);
        if node_desc.kind != HEADER_NODE {
            return Err(VfsError::IoError(format!(
                "Expected header node, got kind {}",
                node_desc.kind
            )));
        }

        // Header record starts after node descriptor (14 bytes)
        let header_buf = &node_buf[14..];
        Self::parse_btree_header(header_buf)
    }

    /// Parse B-tree node descriptor
    fn parse_node_descriptor(buf: &[u8]) -> BTNodeDescriptor {
        BTNodeDescriptor {
            flink: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            blink: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            kind: buf[8] as i8,
            height: buf[9],
            num_records: u16::from_be_bytes([buf[10], buf[11]]),
        }
    }

    /// Parse B-tree header record
    fn parse_btree_header(buf: &[u8]) -> Result<BTHeaderRecord, VfsError> {
        if buf.len() < 106 {
            return Err(VfsError::IoError(
                "Buffer too small for B-tree header".into(),
            ));
        }

        Ok(BTHeaderRecord {
            tree_depth: u16::from_be_bytes([buf[0], buf[1]]),
            root_node: u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]),
            leaf_records: u32::from_be_bytes([buf[6], buf[7], buf[8], buf[9]]),
            first_leaf_node: u32::from_be_bytes([buf[10], buf[11], buf[12], buf[13]]),
            last_leaf_node: u32::from_be_bytes([buf[14], buf[15], buf[16], buf[17]]),
            node_size: u16::from_be_bytes([buf[18], buf[19]]),
            max_key_length: u16::from_be_bytes([buf[20], buf[21]]),
            total_nodes: u32::from_be_bytes([buf[22], buf[23], buf[24], buf[25]]),
            free_nodes: u32::from_be_bytes([buf[26], buf[27], buf[28], buf[29]]),
        })
    }

    /// Read a B-tree node by node number
    fn read_node(&self, node_num: u32) -> Result<Vec<u8>, VfsError> {
        let node_size = self.catalog_header.node_size as usize;
        let mut node_buf = vec![0u8; node_size];

        // Calculate physical offset
        let node_byte_offset = checked_hfsplus_mul_u64(
            node_num as u64,
            node_size as u64,
            "HFS+ catalog node byte offset",
        )?;
        let mut remaining = node_byte_offset;
        let mut physical_offset = None;

        // Find extent containing this offset
        for extent in &self.header.catalog_file.extents {
            if extent.block_count == 0 {
                break;
            }
            let extent_bytes = checked_hfsplus_mul_u64(
                extent.block_count as u64,
                self.header.block_size as u64,
                "HFS+ catalog extent size",
            )?;
            if remaining < extent_bytes {
                let extent_offset = checked_hfsplus_block_offset(
                    self.offset,
                    extent.start_block as u64,
                    self.header.block_size as u64,
                    "HFS+ catalog node physical extent offset",
                )?;
                physical_offset = Some(checked_hfsplus_add_u64(
                    extent_offset,
                    remaining,
                    "HFS+ catalog node physical offset",
                )?);
                break;
            }
            remaining -= extent_bytes;
        }

        let physical_offset = physical_offset.ok_or_else(|| {
            VfsError::IoError(format!("Node {} beyond catalog extents", node_num))
        })?;

        read_hfsplus_exact(
            self.device.as_ref(),
            physical_offset,
            &mut node_buf,
            "HFS+ catalog node",
        )?;

        Ok(node_buf)
    }

    /// Get record offsets from a node
    fn get_record_offsets(&self, node_buf: &[u8], num_records: u16) -> Result<Vec<u16>, VfsError> {
        let node_size = self.catalog_header.node_size as usize;
        let desired_capacity = usize::from(num_records)
            .saturating_add(1)
            .min(node_buf.len() / 2);
        let mut offsets = checked_u16_capacity(desired_capacity, "HFS+ record offsets")?;

        // Record offsets are stored at the end of the node, going backwards
        for i in 0..=num_records {
            let Some(offset_pos) = checked_record_offset(node_size, i as usize) else {
                break;
            };
            if has_buffer_range(node_buf, offset_pos, 2) {
                let offset = u16::from_be_bytes([node_buf[offset_pos], node_buf[offset_pos + 1]]);
                offsets.push(offset);
            }
        }

        Ok(offsets)
    }

    /// Parse catalog key from buffer, return (key, bytes consumed)
    fn parse_catalog_key(buf: &[u8]) -> Result<(HfsPlusCatalogKey, usize), VfsError> {
        if buf.len() < 8 {
            return Err(VfsError::IoError("Buffer too small for catalog key".into()));
        }

        let key_length = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let parent_id = u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]);
        let name_length = u16::from_be_bytes([buf[6], buf[7]]) as usize;

        // Parse UTF-16BE name
        let chars_to_read = name_length.min(buf.len().saturating_sub(8) / 2);
        let mut name_chars = checked_u16_capacity(chars_to_read, "HFS+ catalog key name")?;
        for i in 0..chars_to_read {
            let char_offset = 8 + i * 2;
            let code_unit = u16::from_be_bytes([buf[char_offset], buf[char_offset + 1]]);
            name_chars.push(code_unit);
        }
        let node_name = String::from_utf16_lossy(&name_chars);

        // Key length + 2 bytes for the key length field itself
        let total_consumed = key_length + 2;
        // Align to 2-byte boundary
        let aligned = (total_consumed + 1) & !1;

        Ok((
            HfsPlusCatalogKey {
                parent_id,
                node_name,
            },
            aligned,
        ))
    }

    /// Parse catalog folder record
    fn parse_folder_record(&self, buf: &[u8]) -> Result<HfsPlusCatalogFolder, VfsError> {
        if buf.len() < 88 {
            return Err(VfsError::IoError(
                "Buffer too small for folder record".into(),
            ));
        }

        Ok(HfsPlusCatalogFolder {
            record_type: u16::from_be_bytes([buf[0], buf[1]]),
            flags: u16::from_be_bytes([buf[2], buf[3]]),
            valence: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            folder_id: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            create_date: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            content_mod_date: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            attribute_mod_date: u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]),
            access_date: u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]),
            backup_date: u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]),
            bsd_info: HfsPlusBsdInfo {
                owner_id: u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]),
                group_id: u32::from_be_bytes([buf[36], buf[37], buf[38], buf[39]]),
                admin_flags: buf[40],
                owner_flags: buf[41],
                file_mode: u16::from_be_bytes([buf[42], buf[43]]),
                special: u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]),
            },
        })
    }

    /// Parse catalog file record
    fn parse_file_record(&self, buf: &[u8]) -> Result<HfsPlusCatalogFile, VfsError> {
        if buf.len() < 248 {
            return Err(VfsError::IoError("Buffer too small for file record".into()));
        }

        Ok(HfsPlusCatalogFile {
            record_type: u16::from_be_bytes([buf[0], buf[1]]),
            flags: u16::from_be_bytes([buf[2], buf[3]]),
            file_id: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            create_date: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            content_mod_date: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            attribute_mod_date: u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]),
            access_date: u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]),
            backup_date: u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]),
            bsd_info: HfsPlusBsdInfo {
                owner_id: u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]),
                group_id: u32::from_be_bytes([buf[36], buf[37], buf[38], buf[39]]),
                admin_flags: buf[40],
                owner_flags: buf[41],
                file_mode: u16::from_be_bytes([buf[42], buf[43]]),
                special: u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]),
            },
            // Data fork starts at offset 88
            data_fork: Self::parse_fork_data(&buf[88..168]),
            // Resource fork starts at offset 168
            resource_fork: Self::parse_fork_data(&buf[168..248]),
        })
    }

    /// Search for entries in a folder by parent ID
    fn find_folder_entries(&self, parent_id: u32) -> Result<Vec<(String, CatalogEntry)>, VfsError> {
        let mut entries = Vec::new();

        // Start from root node and traverse
        let root_node = self.catalog_header.root_node;
        if root_node == 0 {
            return Ok(entries);
        }

        self.traverse_btree(root_node, parent_id, &mut entries)?;

        Ok(entries)
    }

    /// Traverse B-tree to find entries for a parent ID
    fn traverse_btree(
        &self,
        node_num: u32,
        parent_id: u32,
        entries: &mut Vec<(String, CatalogEntry)>,
    ) -> Result<(), VfsError> {
        let node_buf = self.read_node(node_num)?;
        let node_desc = Self::parse_node_descriptor(&node_buf);

        if node_desc.kind == INDEX_NODE {
            // Index node - find child nodes to traverse
            let offsets = self.get_record_offsets(&node_buf, node_desc.num_records)?;

            for i in 0..node_desc.num_records as usize {
                if i + 1 >= offsets.len() {
                    break;
                }
                let record_start = offsets[i] as usize;
                if !has_buffer_range(&node_buf, record_start, 6) {
                    continue;
                }

                let (key, key_size) = Self::parse_catalog_key(&node_buf[record_start..])?;

                // Get child node pointer (after key)
                let Some(ptr_offset) = checked_buffer_end(record_start, key_size) else {
                    continue;
                };
                if has_buffer_range(&node_buf, ptr_offset, 4) {
                    let child_node = u32::from_be_bytes([
                        node_buf[ptr_offset],
                        node_buf[ptr_offset + 1],
                        node_buf[ptr_offset + 2],
                        node_buf[ptr_offset + 3],
                    ]);

                    // Only traverse if this subtree might contain our parent_id
                    if key.parent_id <= parent_id || i == 0 {
                        self.traverse_btree(child_node, parent_id, entries)?;
                    }
                }
            }
        } else if node_desc.kind == LEAF_NODE {
            // Leaf node - extract matching records
            let offsets = self.get_record_offsets(&node_buf, node_desc.num_records)?;

            for i in 0..node_desc.num_records as usize {
                if i + 1 >= offsets.len() {
                    break;
                }
                let record_start = offsets[i] as usize;
                let record_end = offsets[i + 1] as usize;
                if record_start >= record_end || !has_buffer_range(&node_buf, record_start, 6) {
                    continue;
                }

                let (key, key_size) = Self::parse_catalog_key(&node_buf[record_start..])?;

                if key.parent_id == parent_id && !key.node_name.is_empty() {
                    // Parse the record data
                    let Some(data_offset) = checked_buffer_end(record_start, key_size) else {
                        continue;
                    };
                    if has_buffer_range(&node_buf, data_offset, 2) {
                        let record_type =
                            u16::from_be_bytes([node_buf[data_offset], node_buf[data_offset + 1]]);

                        match record_type {
                            FOLDER_RECORD => {
                                if let Ok(folder) =
                                    self.parse_folder_record(&node_buf[data_offset..])
                                {
                                    entries.push((
                                        key.node_name.clone(),
                                        CatalogEntry::Folder(folder),
                                    ));
                                }
                            }
                            FILE_RECORD => {
                                if let Ok(file) = self.parse_file_record(&node_buf[data_offset..]) {
                                    entries.push((key.node_name.clone(), CatalogEntry::File(file)));
                                }
                            }
                            FOLDER_THREAD_RECORD | FILE_THREAD_RECORD => {
                                // Thread records point back to parent - skip
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Check if we need to follow forward link
            if node_desc.flink != 0 {
                // Check first key of next node to see if it might have more entries
                let next_buf = self.read_node(node_desc.flink)?;
                let next_desc = Self::parse_node_descriptor(&next_buf);
                if next_desc.num_records > 0 {
                    let next_offsets = self.get_record_offsets(&next_buf, next_desc.num_records)?;
                    if !next_offsets.is_empty() {
                        let first_record = next_offsets[0] as usize;
                        if has_buffer_range(&next_buf, first_record, 6) {
                            if let Ok((first_key, _)) =
                                Self::parse_catalog_key(&next_buf[first_record..])
                            {
                                if first_key.parent_id == parent_id {
                                    self.traverse_btree(node_desc.flink, parent_id, entries)?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Resolve path to folder ID
    fn resolve_path(&self, path: &str) -> Result<u32, VfsError> {
        let normalized = normalize_path(path);
        if normalized == "/" {
            return Ok(ROOT_FOLDER_ID);
        }

        let parts: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current_id = ROOT_FOLDER_ID;

        for part in parts {
            let entries = self.find_folder_entries(current_id)?;
            let found = entries
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(part));

            match found {
                Some((_, CatalogEntry::Folder(folder))) => {
                    current_id = folder.folder_id;
                }
                Some((_, CatalogEntry::File(_))) => {
                    // Last component can be a file
                    return Err(VfsError::NotADirectory(normalized));
                }
                None => {
                    return Err(VfsError::NotFound(normalized));
                }
            }
        }

        Ok(current_id)
    }

    /// Find a specific entry by path
    fn find_entry(&self, path: &str) -> Result<CatalogEntry, VfsError> {
        let normalized = normalize_path(path);
        if normalized == "/" {
            return Ok(CatalogEntry::Folder(HfsPlusCatalogFolder {
                record_type: FOLDER_RECORD,
                flags: 0,
                folder_id: ROOT_FOLDER_ID,
                create_date: 0,
                content_mod_date: 0,
                attribute_mod_date: 0,
                access_date: 0,
                backup_date: 0,
                bsd_info: HfsPlusBsdInfo::default(),
                valence: 0,
            }));
        }

        let parts: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current_id = ROOT_FOLDER_ID;

        for (i, part) in parts.iter().enumerate() {
            let entries = self.find_folder_entries(current_id)?;
            let found = entries
                .into_iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(part));

            match found {
                Some((_, entry)) => {
                    if i == parts.len() - 1 {
                        return Ok(entry);
                    }
                    match &entry {
                        CatalogEntry::Folder(folder) => {
                            current_id = folder.folder_id;
                        }
                        CatalogEntry::File(_) => {
                            return Err(VfsError::NotADirectory(normalized));
                        }
                    }
                }
                None => {
                    return Err(VfsError::NotFound(normalized));
                }
            }
        }

        Err(VfsError::NotFound(normalized))
    }

    /// Read file data from fork
    fn read_fork_data(
        &self,
        fork: &HfsPlusForkData,
        offset: u64,
        size: usize,
    ) -> Result<Vec<u8>, VfsError> {
        if offset >= fork.logical_size {
            return Ok(Vec::new());
        }

        let actual_size =
            bounded_hfsplus_read_len(fork.logical_size, offset, size).ok_or_else(|| {
                VfsError::IoError("HFS+ fork read offset exceeded logical size".into())
            })?;
        let mut result = vec![0u8; actual_size];
        let mut bytes_read = 0usize;
        let mut current_offset = offset;

        // Track position within extents
        let mut extent_logical_start = 0u64;

        for extent in &fork.extents {
            if extent.block_count == 0 {
                break;
            }

            let extent_size = checked_hfsplus_mul_u64(
                extent.block_count as u64,
                self.header.block_size as u64,
                "HFS+ extent size",
            )?;
            let extent_logical_end = checked_hfsplus_add_u64(
                extent_logical_start,
                extent_size,
                "HFS+ extent logical end",
            )?;

            // Check if this extent contains data we need
            if current_offset < extent_logical_end && bytes_read < actual_size {
                // Calculate where to start reading in this extent
                let extent_offset = current_offset.saturating_sub(extent_logical_start);

                // Calculate physical offset
                let physical_block_offset = checked_hfsplus_block_offset(
                    self.offset,
                    extent.start_block as u64,
                    self.header.block_size as u64,
                    "HFS+ physical block offset",
                )?;
                let physical_offset = checked_hfsplus_add_u64(
                    physical_block_offset,
                    extent_offset,
                    "HFS+ physical read offset",
                )?;

                // How much to read from this extent
                let available_in_extent =
                    extent_size.checked_sub(extent_offset).ok_or_else(|| {
                        VfsError::IoError("HFS+ extent offset exceeded extent size".into())
                    })?;
                let available_in_extent =
                    usize::try_from(available_in_extent).unwrap_or(usize::MAX);
                let to_read = (actual_size - bytes_read).min(available_in_extent);

                read_hfsplus_exact(
                    self.device.as_ref(),
                    physical_offset,
                    &mut result[bytes_read..bytes_read + to_read],
                    "HFS+ file extent",
                )?;

                bytes_read += to_read;
                current_offset = checked_hfsplus_read_advance(current_offset, to_read)?;
            }

            extent_logical_start = extent_logical_end;
        }

        result.truncate(bytes_read);
        Ok(result)
    }
}

impl FilesystemDriver for HfsPlusDriver {
    fn info(&self) -> &FilesystemInfo {
        &self.info
    }

    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let entry = self.find_entry(path)?;

        match entry {
            CatalogEntry::Folder(folder) => Ok(FileAttr {
                size: 0,
                is_directory: true,
                permissions: 0o755,
                nlink: 2 + folder.valence,
                inode: folder.folder_id as u64,
                ..Default::default()
            }),
            CatalogEntry::File(file) => Ok(FileAttr {
                size: file.data_fork.logical_size,
                is_directory: false,
                permissions: 0o644,
                nlink: 1,
                inode: file.file_id as u64,
                ..Default::default()
            }),
        }
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        // Check cache first
        let normalized = normalize_path(path);
        if let Some(cached) = self.dir_cache.read().get(&normalized) {
            return Ok(cached
                .iter()
                .map(|(name, entry)| match entry {
                    CatalogEntry::Folder(f) => DirEntry {
                        name: name.clone(),
                        is_directory: true,
                        inode: f.folder_id as u64,
                        file_type: 4,
                    },
                    CatalogEntry::File(f) => DirEntry {
                        name: name.clone(),
                        is_directory: false,
                        inode: f.file_id as u64,
                        file_type: 8,
                    },
                })
                .collect());
        }

        let folder_id = self.resolve_path(path)?;
        let entries = self.find_folder_entries(folder_id)?;

        // Cache the results
        self.dir_cache.write().insert(normalized, entries.clone());

        Ok(entries
            .iter()
            .map(|(name, entry)| match entry {
                CatalogEntry::Folder(f) => DirEntry {
                    name: name.clone(),
                    is_directory: true,
                    inode: f.folder_id as u64,
                    file_type: 4,
                },
                CatalogEntry::File(f) => DirEntry {
                    name: name.clone(),
                    is_directory: false,
                    inode: f.file_id as u64,
                    file_type: 8,
                },
            })
            .collect())
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let entry = self.find_entry(path)?;

        match entry {
            CatalogEntry::Folder(_) => Err(VfsError::NotAFile(path.to_string())),
            CatalogEntry::File(file) => self.read_fork_data(&file.data_fork, offset, size),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::traits::{BlockDevice, BlockReader, SeekableBlockDevice};
    use super::*;
    use ffx_errors::ContainerError;
    use std::io::Cursor;

    struct MockBlockDevice {
        data: Vec<u8>,
    }

    impl BlockDevice for MockBlockDevice {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
            let start = usize::try_from(offset).unwrap_or(usize::MAX);
            if start >= self.data.len() {
                return Ok(0);
            }

            let to_read = buf.len().min(self.data.len() - start);
            buf[..to_read].copy_from_slice(&self.data[start..start + to_read]);
            Ok(to_read)
        }

        fn size(&self) -> u64 {
            self.data.len() as u64
        }
    }

    impl SeekableBlockDevice for MockBlockDevice {
        fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
            let start = usize::try_from(offset).unwrap_or(self.data.len());
            Box::new(Cursor::new(
                self.data[start.min(self.data.len())..].to_vec(),
            ))
        }
    }

    #[test]
    fn test_hfsplus_signature_constants() {
        assert_eq!(HFSPLUS_SIGNATURE, 0x482B);
        assert_eq!(HFSX_SIGNATURE, 0x4858);
    }

    #[test]
    fn test_hfsplus_filesystem_type() {
        assert_eq!(FilesystemType::HfsPlus.to_string(), "HFS+");
    }

    #[test]
    fn test_catalog_record_types() {
        assert_eq!(FOLDER_RECORD, 0x0001);
        assert_eq!(FILE_RECORD, 0x0002);
        assert_eq!(FOLDER_THREAD_RECORD, 0x0003);
        assert_eq!(FILE_THREAD_RECORD, 0x0004);
    }

    #[test]
    fn test_node_types() {
        assert_eq!(LEAF_NODE, -1);
        assert_eq!(INDEX_NODE, 0);
        assert_eq!(HEADER_NODE, 1);
    }

    #[test]
    fn test_special_folder_ids() {
        assert_eq!(ROOT_FOLDER_ID, 2);
        assert_eq!(CATALOG_FILE_ID, 4);
    }

    #[test]
    fn test_checked_record_offset_rejects_underflow() {
        assert_eq!(checked_record_offset(8, 4), None);
    }

    #[test]
    fn test_has_buffer_range_rejects_overflow() {
        assert!(!has_buffer_range(b"abcd", usize::MAX - 1, 8));
    }

    #[test]
    fn test_checked_u16_capacity_rejects_huge_allocation() {
        assert!(checked_u16_capacity(usize::MAX, "test").is_err());
    }

    #[test]
    fn test_checked_hfsplus_mul_rejects_overflow() {
        let err = checked_hfsplus_mul_u64(u64::MAX, 2, "test").unwrap_err();

        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_checked_hfsplus_add_rejects_overflow() {
        let err = checked_hfsplus_add_u64(u64::MAX, 1, "test").unwrap_err();

        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_checked_hfsplus_block_offset_rejects_overflow() {
        assert!(checked_hfsplus_block_offset(u64::MAX, 1, 1, "test").is_err());
    }

    #[test]
    fn test_checked_hfsplus_read_advance_adds_bytes_read() {
        assert_eq!(checked_hfsplus_read_advance(40, 2).unwrap(), 42);
    }

    #[test]
    fn test_checked_hfsplus_read_advance_rejects_overflow() {
        let err = checked_hfsplus_read_advance(u64::MAX, 1).unwrap_err();

        assert!(err.to_string().contains("HFS+ logical read offset"));
    }

    #[test]
    fn test_bounded_hfsplus_read_len_rejects_underflow() {
        assert_eq!(bounded_hfsplus_read_len(8, 4, 16), Some(4));
        assert_eq!(bounded_hfsplus_read_len(8, 8, 16), Some(0));
        assert_eq!(bounded_hfsplus_read_len(8, 9, 16), None);
    }

    #[test]
    fn test_read_hfsplus_exact_reads_full_buffer() {
        let device = MockBlockDevice {
            data: b"abcdef".to_vec(),
        };
        let mut buf = [0u8; 3];

        read_hfsplus_exact(&device, 2, &mut buf, "test")
            .expect("full HFS+ exact read should succeed");

        assert_eq!(&buf, b"cde");
    }

    #[test]
    fn test_read_hfsplus_exact_rejects_short_read() {
        let device = MockBlockDevice {
            data: b"abcd".to_vec(),
        };
        let mut buf = [0u8; 4];

        let err = read_hfsplus_exact(&device, 2, &mut buf, "test")
            .expect_err("short HFS+ exact read should fail");

        assert!(matches!(err, VfsError::IoError(message) if message.contains("short read")));
    }

    #[test]
    fn test_parse_catalog_key_requires_name_length_header() {
        assert!(HfsPlusDriver::parse_catalog_key(&[0; 7]).is_err());
    }
}
