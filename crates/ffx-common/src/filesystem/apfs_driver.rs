// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Allow dead code for APFS binary structures - many fields are parsed but not yet used
#![allow(dead_code)]

//! # APFS Filesystem Driver
//!
//! Implements read-only APFS filesystem access for macOS/iOS disk images.
//! Based on Apple's APFS specification.
//!
//! ## References
//! - Apple File System Reference (2020)
//! - https://developer.apple.com/support/downloads/Apple-File-System-Reference.pdf

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::traits::{FilesystemDriver, FilesystemInfo, FilesystemType, SeekableBlockDevice};
use crate::vfs::{normalize_path, DirEntry, FileAttr, VfsError};

// =============================================================================
// APFS Constants
// =============================================================================

/// APFS container superblock magic 'NXSB'
const APFS_CONTAINER_MAGIC: u32 = 0x4E585342; // 'NXSB' in little-endian
/// APFS volume superblock magic 'APSB'
const APFS_VOLUME_MAGIC: u32 = 0x41505342; // 'APSB' in little-endian
/// Last fixed field read from the APFS container superblock before the volume OID table.
const APFS_NX_SUPERBLOCK_FIXED_FIELDS_LEN: usize = 168;
/// APFS object type mask
const OBJ_TYPE_MASK: u32 = 0x0000FFFF;
/// APFS object types
const OBJECT_TYPE_NX_SUPERBLOCK: u32 = 0x00000001;
const OBJECT_TYPE_FS: u32 = 0x0000000D;
const OBJECT_TYPE_BTREE: u32 = 0x00000002;
const OBJECT_TYPE_BTREE_NODE: u32 = 0x00000003;
#[allow(dead_code)]
const OBJECT_TYPE_FSTREE: u32 = 0x0000000E;
#[allow(dead_code)]
const OBJECT_TYPE_OMAP: u32 = 0x0000000B;

/// B-tree node flags
const BTNODE_ROOT: u16 = 0x0001;
const BTNODE_LEAF: u16 = 0x0002;
#[allow(dead_code)]
const BTNODE_FIXED_KV_SIZE: u16 = 0x0004;

/// Directory record types
const DREC_TYPE_MASK: u16 = 0x000F;
const DT_UNKNOWN: u8 = 0;
const DT_FIFO: u8 = 1;
const DT_CHR: u8 = 2;
const DT_DIR: u8 = 4;
const DT_BLK: u8 = 6;
const DT_REG: u8 = 8;
const DT_LNK: u8 = 10;
const DT_SOCK: u8 = 12;
#[allow(dead_code)]
const DT_WHT: u8 = 14;

/// Inode record types
const J_INODE_VAL_TYPE: u8 = 3;
const J_DIR_REC_TYPE: u8 = 9;
#[allow(dead_code)]
const J_FILE_EXTENT_TYPE: u8 = 8;
const J_DSTREAM_TYPE: u8 = 10;

/// Root inode ID
const ROOT_INODE_ID: u64 = 2;
/// Root directory inode ID
const ROOT_DIR_INODE_ID: u64 = 2;

fn clamp_read_end(offset: u64, size: usize, file_size: u64) -> u64 {
    let requested = u64::try_from(size).unwrap_or(u64::MAX);
    offset.saturating_add(requested).min(file_size)
}

fn bounded_apfs_read_len(start: u64, end: u64) -> Result<usize, VfsError> {
    let length = end
        .checked_sub(start)
        .ok_or_else(|| VfsError::Internal("APFS read range underflow".into()))?;

    usize::try_from(length).map_err(|_| VfsError::Internal("APFS read range too large".into()))
}

fn bounded_apfs_file_read(
    file_size: u64,
    offset: u64,
    requested_size: usize,
) -> Result<Option<(u64, usize)>, VfsError> {
    if offset > file_size {
        return Err(VfsError::OutOfBounds {
            offset,
            size: requested_size,
        });
    }

    if offset == file_size || requested_size == 0 {
        return Ok(None);
    }

    let read_end = clamp_read_end(offset, requested_size, file_size);
    let total_to_read = bounded_apfs_read_len(offset, read_end)?;
    Ok(Some((read_end, total_to_read)))
}

fn checked_apfs_extent_end(logical_offset: u64, extent_length: u64) -> Result<u64, VfsError> {
    logical_offset
        .checked_add(extent_length)
        .ok_or_else(|| VfsError::Internal("APFS extent end overflow".into()))
}

fn checked_apfs_physical_offset(
    base_offset: u64,
    extent_phys_block: u64,
    block_size: u32,
    read_start_in_extent: u64,
) -> Result<u64, VfsError> {
    base_offset
        .checked_add(
            extent_phys_block
                .checked_mul(block_size as u64)
                .ok_or_else(|| {
                    VfsError::Internal("APFS extent physical block offset overflow".into())
                })?,
        )
        .and_then(|offset| offset.checked_add(read_start_in_extent))
        .ok_or_else(|| VfsError::Internal("APFS physical read offset overflow".into()))
}

fn checked_apfs_block_offset(
    base_offset: u64,
    block_num: u64,
    block_size: u32,
) -> Result<u64, VfsError> {
    let block_offset = block_num
        .checked_mul(u64::from(block_size))
        .ok_or_else(|| VfsError::Internal("APFS block offset overflow".into()))?;
    base_offset
        .checked_add(block_offset)
        .ok_or_else(|| VfsError::Internal("APFS block offset overflow".into()))
}

fn checked_apfs_container_size(block_count: u64, block_size: u32) -> Result<u64, VfsError> {
    block_count
        .checked_mul(u64::from(block_size))
        .ok_or_else(|| VfsError::Internal("APFS container size overflow".into()))
}

fn has_buffer_range(buf: &[u8], start: usize, len: usize) -> bool {
    match start.checked_add(len) {
        Some(end) => end <= buf.len(),
        None => false,
    }
}

fn checked_slice(buf: &[u8], start: usize, len: usize) -> Option<&[u8]> {
    let end = start.checked_add(len)?;
    buf.get(start..end)
}

fn read_le_u16_at(buf: &[u8], start: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        checked_slice(buf, start, 2)?.try_into().ok()?,
    ))
}

fn read_le_u32_at(buf: &[u8], start: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        checked_slice(buf, start, 4)?.try_into().ok()?,
    ))
}

fn read_le_u64_at(buf: &[u8], start: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        checked_slice(buf, start, 8)?.try_into().ok()?,
    ))
}

fn checked_toc_entry_offset(toc_offset: usize, index: usize) -> Option<usize> {
    toc_offset.checked_add(index.checked_mul(8)?)
}

fn checked_value_offset(block_size: usize, val_offset: u16, val_len: u16) -> Option<usize> {
    block_size
        .checked_sub(val_offset as usize)?
        .checked_sub(val_len as usize)
}

fn checked_btree_offsets(node: &BtreeNodePhys) -> Option<(usize, usize)> {
    let toc_offset = 56usize.checked_add(node.table_space_offset as usize)?;
    let key_area_offset = toc_offset.checked_add(node.table_space_len as usize)?;
    Some((toc_offset, key_area_offset))
}

fn checked_kvloc_capacity(count: usize) -> Result<Vec<KvLoc>, VfsError> {
    let mut locs = Vec::new();
    locs.try_reserve_exact(count)
        .map_err(|_| VfsError::IoError("APFS TOC entry count too large".into()))?;
    Ok(locs)
}

fn read_apfs_exact(
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
            "{context} is truncated: read {bytes_read} of {} bytes at offset {offset}",
            buf.len()
        )));
    }
    Ok(())
}

fn ensure_apfs_extent_read_available(
    saw_data_extent: bool,
    total_to_read: usize,
    inode_id: u64,
    extent_count: usize,
) -> Result<(), VfsError> {
    if !saw_data_extent && total_to_read > 0 {
        return Err(VfsError::Internal(format!(
            "No file extents found for inode {inode_id} (found {extent_count} extents total)"
        )));
    }

    Ok(())
}

// =============================================================================
// APFS Structures
// =============================================================================

/// APFS Object Header (common to all objects)
#[derive(Debug, Clone)]
struct ObjPhysHeader {
    /// Checksum (Fletcher 64)
    #[allow(dead_code)]
    cksum: u64,
    /// Object ID
    oid: u64,
    /// Transaction ID
    #[allow(dead_code)]
    xid: u64,
    /// Object type and flags
    obj_type: u32,
    /// Object subtype
    #[allow(dead_code)]
    obj_subtype: u32,
}

/// APFS Container Superblock
#[derive(Debug, Clone)]
struct NxSuperblock {
    /// Object header
    #[allow(dead_code)]
    header: ObjPhysHeader,
    /// Magic number ('NXSB')
    magic: u32,
    /// Block size
    block_size: u32,
    /// Block count
    block_count: u64,
    /// Maximum number of volumes
    #[allow(dead_code)]
    max_file_systems: u32,
    /// Object map OID
    omap_oid: u64,
    /// Array of volume OIDs
    fs_oid: Vec<u64>,
}

/// APFS Volume Superblock
#[derive(Debug, Clone)]
struct ApfsSuperblock {
    /// Object header
    #[allow(dead_code)]
    header: ObjPhysHeader,
    /// Magic number ('APSB')
    magic: u32,
    /// Volume index
    #[allow(dead_code)]
    vol_index: u32,
    /// Object map OID
    #[allow(dead_code)]
    omap_oid: u64,
    /// Root tree OID
    root_tree_oid: u64,
    /// Root tree type
    #[allow(dead_code)]
    root_tree_type: u32,
    /// Volume name
    vol_name: String,
}

/// Object map physical record
#[derive(Debug, Clone)]
struct OmapPhys {
    /// Object header
    #[allow(dead_code)]
    header: ObjPhysHeader,
    /// Tree OID
    tree_oid: u64,
}

/// B-tree node info
#[derive(Debug, Clone)]
struct BtreeNodePhys {
    /// Object header
    header: ObjPhysHeader,
    /// Flags (root, leaf, fixed_kv)
    flags: u16,
    /// Level (0 = leaf)
    level: u16,
    /// Number of keys
    nkeys: u32,
    /// Table of contents offset
    table_space_offset: u16,
    /// Table of contents length
    table_space_len: u16,
    /// Free space offset
    #[allow(dead_code)]
    free_space_offset: u16,
    /// Free space length
    #[allow(dead_code)]
    free_space_len: u16,
    /// Key free list offset
    #[allow(dead_code)]
    key_free_list_offset: u16,
    /// Key free list length
    #[allow(dead_code)]
    key_free_list_len: u16,
    /// Value free list offset
    #[allow(dead_code)]
    val_free_list_offset: u16,
    /// Value free list length
    #[allow(dead_code)]
    val_free_list_len: u16,
}

/// Key-value location in B-tree node
#[derive(Debug, Clone, Copy)]
struct KvLoc {
    key_offset: u16,
    key_len: u16,
    val_offset: u16,
    val_len: u16,
}

/// Inode record
#[derive(Debug, Clone)]
struct InodeRecord {
    /// Parent inode ID
    #[allow(dead_code)]
    parent_id: u64,
    /// Private ID (for file data)
    #[allow(dead_code)]
    private_id: u64,
    /// Creation time
    #[allow(dead_code)]
    create_time: u64,
    /// Modification time
    #[allow(dead_code)]
    mod_time: u64,
    /// Change time
    #[allow(dead_code)]
    change_time: u64,
    /// Access time
    #[allow(dead_code)]
    access_time: u64,
    /// Flags
    #[allow(dead_code)]
    flags: u64,
    /// Number of children (for directories)
    nchildren: u32,
    /// BSD flags
    #[allow(dead_code)]
    bsd_flags: u32,
    /// Owner UID
    #[allow(dead_code)]
    uid: u32,
    /// Group GID
    #[allow(dead_code)]
    gid: u32,
    /// Mode (permissions + file type)
    mode: u16,
    /// Name (if available)
    name: Option<String>,
}

/// Directory record
#[derive(Debug, Clone)]
struct DrecRecord {
    /// Inode ID this entry points to
    file_id: u64,
    /// Date added
    #[allow(dead_code)]
    date_added: u64,
    /// Type (from mode)
    d_type: u8,
    /// Name
    name: String,
}

/// Data stream record
#[derive(Debug, Clone)]
struct DstreamRecord {
    /// Size
    size: u64,
    /// Allocated size
    #[allow(dead_code)]
    alloced_size: u64,
    /// Default crypto ID
    #[allow(dead_code)]
    default_crypto_id: u64,
}

/// Catalog entry
#[derive(Debug, Clone)]
enum CatalogEntry {
    Inode(InodeRecord),
    Drec(DrecRecord),
    Dstream(DstreamRecord),
}

// =============================================================================
// APFS Driver
// =============================================================================

/// Type alias for directory cache: inode_id -> list of (name, type, child_id)
type DirCacheMap = HashMap<u64, Vec<(String, u8, u64)>>;

/// APFS filesystem driver
pub struct ApfsDriver {
    /// Filesystem info
    info: FilesystemInfo,
    /// Block device
    device: Arc<dyn SeekableBlockDevice>,
    /// Partition offset
    offset: u64,
    /// Container superblock
    container: NxSuperblock,
    /// Active volume superblock (first volume)
    volume: ApfsSuperblock,
    /// Block size
    block_size: u32,
    /// Directory cache: inode_id -> list of (name, type, child_id)
    dir_cache: RwLock<DirCacheMap>,
    /// Inode cache: inode_id -> InodeRecord
    inode_cache: RwLock<HashMap<u64, InodeRecord>>,
    /// File size cache: inode_id -> size
    size_cache: RwLock<HashMap<u64, u64>>,
}

impl ApfsDriver {
    /// Create a new APFS driver
    pub fn new(
        device: Box<dyn SeekableBlockDevice>,
        offset: u64,
        size: u64,
    ) -> Result<Self, VfsError> {
        let device: Arc<dyn SeekableBlockDevice> = Arc::from(device);

        // Read container superblock (at block 0)
        let container = Self::read_container_superblock(&device, offset)?;

        // Validate magic
        if container.magic != APFS_CONTAINER_MAGIC {
            return Err(VfsError::Internal(format!(
                "Invalid APFS container magic: 0x{:08X}",
                container.magic
            )));
        }

        let block_size = container.block_size;

        // Find first volume
        let volume = Self::find_first_volume(&device, offset, &container)?;

        let total_size = checked_apfs_container_size(container.block_count, block_size)?;

        let info = FilesystemInfo {
            fs_type: FilesystemType::Apfs,
            label: if volume.vol_name.is_empty() {
                None
            } else {
                Some(volume.vol_name.clone())
            },
            total_size: total_size.min(size),
            free_space: None, // Would need to scan space manager
            cluster_size: block_size,
        };

        Ok(Self {
            info,
            device,
            offset,
            container,
            volume,
            block_size,
            dir_cache: RwLock::new(HashMap::new()),
            inode_cache: RwLock::new(HashMap::new()),
            size_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Read a block from the device
    fn read_block(&self, block_num: u64) -> Result<Vec<u8>, VfsError> {
        let mut buf = vec![0u8; self.block_size as usize];
        let block_offset = checked_apfs_block_offset(self.offset, block_num, self.block_size)?;
        read_apfs_exact(self.device.as_ref(), block_offset, &mut buf, "APFS block")?;
        Ok(buf)
    }

    /// Parse object header
    fn parse_obj_header(buf: &[u8]) -> Result<ObjPhysHeader, VfsError> {
        if buf.len() < 32 {
            return Err(VfsError::IoError(
                "Buffer too small for object header".into(),
            ));
        }

        Ok(ObjPhysHeader {
            cksum: read_le_u64_at(buf, 0)
                .ok_or_else(|| VfsError::IoError("APFS object checksum out of range".into()))?,
            oid: read_le_u64_at(buf, 8)
                .ok_or_else(|| VfsError::IoError("APFS object oid out of range".into()))?,
            xid: read_le_u64_at(buf, 16)
                .ok_or_else(|| VfsError::IoError("APFS object xid out of range".into()))?,
            obj_type: read_le_u32_at(buf, 24)
                .ok_or_else(|| VfsError::IoError("APFS object type out of range".into()))?,
            obj_subtype: read_le_u32_at(buf, 28)
                .ok_or_else(|| VfsError::IoError("APFS object subtype out of range".into()))?,
        })
    }

    /// Read container superblock
    fn read_container_superblock(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
    ) -> Result<NxSuperblock, VfsError> {
        // First read to get block size (at offset 36)
        let mut header_buf = vec![0u8; 64];
        let bytes_read = device
            .read_at(offset, &mut header_buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        if bytes_read < header_buf.len() {
            return Err(VfsError::IoError(format!(
                "APFS container header is truncated: read {} bytes",
                bytes_read
            )));
        }

        let block_size = read_le_u32_at(&header_buf, 36)
            .ok_or_else(|| VfsError::IoError("APFS block size out of range".into()))?;
        if block_size == 0 || block_size > 65536 {
            return Err(VfsError::IoError(format!(
                "Invalid APFS block size: {}",
                block_size
            )));
        }
        if (block_size as usize) < APFS_NX_SUPERBLOCK_FIXED_FIELDS_LEN {
            return Err(VfsError::IoError(format!(
                "APFS container block size {} is too small for container superblock fields",
                block_size
            )));
        }

        // Now read full block
        let mut buf = vec![0u8; block_size as usize];
        let bytes_read = device
            .read_at(offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        if bytes_read < buf.len() {
            return Err(VfsError::IoError(format!(
                "APFS container block is truncated: read {} of {} bytes",
                bytes_read,
                buf.len()
            )));
        }

        let header = Self::parse_obj_header(&buf)?;

        // Verify it's a container superblock
        if (header.obj_type & OBJ_TYPE_MASK) != OBJECT_TYPE_NX_SUPERBLOCK {
            return Err(VfsError::IoError(format!(
                "Expected container superblock, got type 0x{:08X}",
                header.obj_type
            )));
        }

        let magic = read_le_u32_at(&buf, 32)
            .ok_or_else(|| VfsError::IoError("APFS container magic out of range".into()))?;
        let block_count = read_le_u64_at(&buf, 40)
            .ok_or_else(|| VfsError::IoError("APFS block count out of range".into()))?;
        let max_file_systems = read_le_u32_at(&buf, 100)
            .ok_or_else(|| VfsError::IoError("APFS max filesystem count out of range".into()))?;
        let omap_oid = read_le_u64_at(&buf, 160)
            .ok_or_else(|| VfsError::IoError("APFS object map oid out of range".into()))?;

        // Read volume OIDs (up to 100 volumes, starting at offset 168)
        let mut fs_oid = Vec::new();
        for i in 0..std::cmp::min(max_file_systems as usize, 100) {
            let oid_offset = 168 + i * 8;
            if oid_offset + 8 <= buf.len() {
                if let Some(oid) = read_le_u64_at(&buf, oid_offset) {
                    if oid != 0 {
                        fs_oid.push(oid);
                    }
                }
            }
        }

        Ok(NxSuperblock {
            header,
            magic,
            block_size,
            block_count,
            max_file_systems,
            omap_oid,
            fs_oid,
        })
    }

    /// Find first volume in container
    fn find_first_volume(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
        container: &NxSuperblock,
    ) -> Result<ApfsSuperblock, VfsError> {
        // Read object map to resolve volume OIDs
        let omap_block =
            Self::read_block_static(device, offset, container.block_size, container.omap_oid)?;

        let omap = Self::parse_omap(&omap_block)?;

        // Get first volume OID
        let vol_oid = container
            .fs_oid
            .first()
            .ok_or_else(|| VfsError::IoError("No volumes in container".into()))?;

        // Resolve volume OID through omap
        let vol_paddr = Self::resolve_oid(
            device,
            offset,
            container.block_size,
            omap.tree_oid,
            *vol_oid,
        )?;

        // Read volume superblock
        let vol_block = Self::read_block_static(device, offset, container.block_size, vol_paddr)?;

        Self::parse_volume_superblock(&vol_block)
    }

    /// Read a block (static version)
    fn read_block_static(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
        block_size: u32,
        block_num: u64,
    ) -> Result<Vec<u8>, VfsError> {
        let mut buf = vec![0u8; block_size as usize];
        let block_offset = checked_apfs_block_offset(offset, block_num, block_size)?;
        read_apfs_exact(device.as_ref(), block_offset, &mut buf, "APFS block")?;
        Ok(buf)
    }

    /// Parse object map
    fn parse_omap(buf: &[u8]) -> Result<OmapPhys, VfsError> {
        if buf.len() < 88 {
            return Err(VfsError::IoError("Buffer too small for omap".into()));
        }

        let header = Self::parse_obj_header(buf)?;
        let tree_oid = read_le_u64_at(buf, 48)
            .ok_or_else(|| VfsError::IoError("APFS object map tree oid out of range".into()))?;

        Ok(OmapPhys { header, tree_oid })
    }

    /// Resolve OID to physical address through B-tree
    fn resolve_oid(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
        block_size: u32,
        tree_oid: u64,
        target_oid: u64,
    ) -> Result<u64, VfsError> {
        // Read the B-tree root
        let node_buf = Self::read_block_static(device, offset, block_size, tree_oid)?;
        Self::search_btree_for_oid(device, offset, block_size, &node_buf, target_oid)
    }

    /// Search B-tree for OID mapping
    fn search_btree_for_oid(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
        block_size: u32,
        node_buf: &[u8],
        target_oid: u64,
    ) -> Result<u64, VfsError> {
        let _header = Self::parse_obj_header(node_buf)?;
        let node = Self::parse_btree_node(node_buf)?;

        let is_leaf = (node.flags & BTNODE_LEAF) != 0;
        let (toc_offset, key_area_offset) = checked_btree_offsets(&node)
            .ok_or_else(|| VfsError::IoError("APFS B-tree offset overflow".into()))?;

        // Get key-value locations from TOC
        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            // Read key (OID + XID)
            let Some(key_offset) = key_area_offset.checked_add(kvloc.key_offset as usize) else {
                continue;
            };
            if !has_buffer_range(node_buf, key_offset, 16) {
                continue;
            }

            let Some(key_oid) = read_le_u64_at(node_buf, key_offset) else {
                continue;
            };

            if is_leaf {
                if key_oid == target_oid {
                    // Value is at end of block, working backwards
                    let Some(val_offset) =
                        checked_value_offset(block_size as usize, kvloc.val_offset, kvloc.val_len)
                    else {
                        continue;
                    };
                    if has_buffer_range(node_buf, val_offset, 16) {
                        // Skip flags (4 bytes) and size (4 bytes), get paddr
                        let Some(paddr_offset) = val_offset.checked_add(8) else {
                            continue;
                        };
                        let Some(paddr) = read_le_u64_at(node_buf, paddr_offset) else {
                            continue;
                        };
                        return Ok(paddr);
                    }
                }
            } else {
                // Index node - check if we should descend
                if key_oid >= target_oid {
                    let Some(val_offset) =
                        checked_value_offset(block_size as usize, kvloc.val_offset, kvloc.val_len)
                    else {
                        continue;
                    };
                    if has_buffer_range(node_buf, val_offset, 8) {
                        let Some(child_oid) = read_le_u64_at(node_buf, val_offset) else {
                            continue;
                        };
                        let child_buf =
                            Self::read_block_static(device, offset, block_size, child_oid)?;
                        return Self::search_btree_for_oid(
                            device, offset, block_size, &child_buf, target_oid,
                        );
                    }
                }
            }
        }

        // If not found in leaf, try last child in index nodes
        if !is_leaf && !kvlocs.is_empty() {
            let Some(last) = kvlocs.last() else {
                return Err(VfsError::NotFound(format!("OID {} not found", target_oid)));
            };
            let Some(val_offset) =
                checked_value_offset(block_size as usize, last.val_offset, last.val_len)
            else {
                return Err(VfsError::NotFound(format!("OID {} not found", target_oid)));
            };
            if has_buffer_range(node_buf, val_offset, 8) {
                let Some(child_oid) = read_le_u64_at(node_buf, val_offset) else {
                    return Err(VfsError::NotFound(format!("OID {} not found", target_oid)));
                };
                let child_buf = Self::read_block_static(device, offset, block_size, child_oid)?;
                return Self::search_btree_for_oid(
                    device, offset, block_size, &child_buf, target_oid,
                );
            }
        }

        Err(VfsError::NotFound(format!("OID {} not found", target_oid)))
    }

    /// Parse B-tree node
    fn parse_btree_node(buf: &[u8]) -> Result<BtreeNodePhys, VfsError> {
        if buf.len() < 56 {
            return Err(VfsError::IoError("Buffer too small for B-tree node".into()));
        }

        let header = Self::parse_obj_header(buf)?;

        Ok(BtreeNodePhys {
            header,
            flags: read_le_u16_at(buf, 32)
                .ok_or_else(|| VfsError::IoError("APFS B-tree flags out of range".into()))?,
            level: read_le_u16_at(buf, 34)
                .ok_or_else(|| VfsError::IoError("APFS B-tree level out of range".into()))?,
            nkeys: read_le_u32_at(buf, 36)
                .ok_or_else(|| VfsError::IoError("APFS B-tree key count out of range".into()))?,
            table_space_offset: read_le_u16_at(buf, 40).ok_or_else(|| {
                VfsError::IoError("APFS B-tree table-space offset out of range".into())
            })?,
            table_space_len: read_le_u16_at(buf, 42).ok_or_else(|| {
                VfsError::IoError("APFS B-tree table-space length out of range".into())
            })?,
            free_space_offset: read_le_u16_at(buf, 44).ok_or_else(|| {
                VfsError::IoError("APFS B-tree free-space offset out of range".into())
            })?,
            free_space_len: read_le_u16_at(buf, 46).ok_or_else(|| {
                VfsError::IoError("APFS B-tree free-space length out of range".into())
            })?,
            key_free_list_offset: read_le_u16_at(buf, 48).ok_or_else(|| {
                VfsError::IoError("APFS B-tree key free-list offset out of range".into())
            })?,
            key_free_list_len: read_le_u16_at(buf, 50).ok_or_else(|| {
                VfsError::IoError("APFS B-tree key free-list length out of range".into())
            })?,
            val_free_list_offset: read_le_u16_at(buf, 52).ok_or_else(|| {
                VfsError::IoError("APFS B-tree value free-list offset out of range".into())
            })?,
            val_free_list_len: read_le_u16_at(buf, 54).ok_or_else(|| {
                VfsError::IoError("APFS B-tree value free-list length out of range".into())
            })?,
        })
    }

    /// Parse table of contents
    fn parse_toc(buf: &[u8], toc_offset: usize, nkeys: usize) -> Result<Vec<KvLoc>, VfsError> {
        let mut locs = checked_kvloc_capacity(nkeys)?;

        for i in 0..nkeys {
            let Some(entry_offset) = checked_toc_entry_offset(toc_offset, i) else {
                break;
            };
            if !has_buffer_range(buf, entry_offset, 8) {
                break;
            }

            locs.push(KvLoc {
                key_offset: read_le_u16_at(buf, entry_offset)
                    .ok_or_else(|| VfsError::IoError("APFS TOC key offset out of range".into()))?,
                key_len: read_le_u16_at(
                    buf,
                    entry_offset.checked_add(2).ok_or_else(|| {
                        VfsError::IoError("APFS TOC key length offset overflow".into())
                    })?,
                )
                .ok_or_else(|| VfsError::IoError("APFS TOC key length out of range".into()))?,
                val_offset: read_le_u16_at(
                    buf,
                    entry_offset.checked_add(4).ok_or_else(|| {
                        VfsError::IoError("APFS TOC value offset overflow".into())
                    })?,
                )
                .ok_or_else(|| VfsError::IoError("APFS TOC value offset out of range".into()))?,
                val_len: read_le_u16_at(
                    buf,
                    entry_offset.checked_add(6).ok_or_else(|| {
                        VfsError::IoError("APFS TOC value length offset overflow".into())
                    })?,
                )
                .ok_or_else(|| VfsError::IoError("APFS TOC value length out of range".into()))?,
            });
        }

        Ok(locs)
    }

    /// Parse volume superblock
    fn parse_volume_superblock(buf: &[u8]) -> Result<ApfsSuperblock, VfsError> {
        if buf.len() < 1000 {
            return Err(VfsError::IoError(
                "Buffer too small for volume superblock".into(),
            ));
        }

        let header = Self::parse_obj_header(buf)?;

        // Verify it's a volume superblock
        if (header.obj_type & OBJ_TYPE_MASK) != OBJECT_TYPE_FS {
            return Err(VfsError::IoError(format!(
                "Expected volume superblock, got type 0x{:08X}",
                header.obj_type
            )));
        }

        let magic = read_le_u32_at(buf, 32)
            .ok_or_else(|| VfsError::IoError("APFS volume magic out of range".into()))?;
        let vol_index = read_le_u32_at(buf, 36)
            .ok_or_else(|| VfsError::IoError("APFS volume index out of range".into()))?;
        let omap_oid = read_le_u64_at(buf, 80)
            .ok_or_else(|| VfsError::IoError("APFS volume object map oid out of range".into()))?;
        let root_tree_oid = read_le_u64_at(buf, 88)
            .ok_or_else(|| VfsError::IoError("APFS root tree oid out of range".into()))?;
        let root_tree_type = read_le_u32_at(buf, 96)
            .ok_or_else(|| VfsError::IoError("APFS root tree type out of range".into()))?;

        // Volume name is at offset 754 (256 bytes max)
        let name_start = 754;
        let name_end = std::cmp::min(name_start + 256, buf.len());
        let name_bytes = &buf[name_start..name_end];
        let vol_name = name_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .cloned()
            .collect::<Vec<u8>>();
        let vol_name = String::from_utf8_lossy(&vol_name).to_string();

        Ok(ApfsSuperblock {
            header,
            magic,
            vol_index,
            omap_oid,
            root_tree_oid,
            root_tree_type,
            vol_name,
        })
    }

    /// Traverse catalog tree to find directory entries
    fn find_directory_entries(&self, parent_id: u64) -> Result<Vec<(String, u8, u64)>, VfsError> {
        // Check cache
        if let Some(entries) = self.dir_cache.read().get(&parent_id) {
            return Ok(entries.clone());
        }

        let mut entries = Vec::new();

        // Read the root tree
        let root_block = self.read_block(self.volume.root_tree_oid)?;
        self.traverse_catalog_tree(&root_block, parent_id, &mut entries)?;

        // Cache results
        self.dir_cache.write().insert(parent_id, entries.clone());

        Ok(entries)
    }

    /// Traverse catalog B-tree looking for directory entries
    fn traverse_catalog_tree(
        &self,
        node_buf: &[u8],
        parent_id: u64,
        entries: &mut Vec<(String, u8, u64)>,
    ) -> Result<(), VfsError> {
        let node = Self::parse_btree_node(node_buf)?;
        let is_leaf = (node.flags & BTNODE_LEAF) != 0;
        let (toc_offset, key_area_offset) = checked_btree_offsets(&node)
            .ok_or_else(|| VfsError::IoError("APFS B-tree offset overflow".into()))?;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let Some(key_offset) = key_area_offset.checked_add(kvloc.key_offset as usize) else {
                continue;
            };
            if !has_buffer_range(node_buf, key_offset, 10) {
                continue;
            }

            // Catalog key: obj_id (8) + type (1)
            let Some(obj_id) = read_le_u64_at(node_buf, key_offset) else {
                continue;
            };
            let rec_type = node_buf[key_offset + 8];

            // Clear high bits that indicate type
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                let Some(val_offset) =
                    checked_value_offset(self.block_size as usize, kvloc.val_offset, kvloc.val_len)
                else {
                    continue;
                };

                if rec_type == J_DIR_REC_TYPE && inode_id == parent_id {
                    // This is a directory record for our parent
                    if let Ok(drec) = self.parse_drec_value(&node_buf[val_offset..]) {
                        entries.push((drec.name, drec.d_type, drec.file_id));
                    }
                }
            } else {
                // Index node - traverse children that might contain our parent
                let Some(val_offset) =
                    checked_value_offset(self.block_size as usize, kvloc.val_offset, kvloc.val_len)
                else {
                    continue;
                };
                if has_buffer_range(node_buf, val_offset, 8) {
                    let Some(child_addr) = read_le_u64_at(node_buf, val_offset) else {
                        continue;
                    };
                    if let Ok(child_buf) = self.read_block(child_addr) {
                        self.traverse_catalog_tree(&child_buf, parent_id, entries)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse directory record value
    fn parse_drec_value(&self, buf: &[u8]) -> Result<DrecRecord, VfsError> {
        if buf.len() < 18 {
            return Err(VfsError::IoError("Buffer too small for drec".into()));
        }

        let file_id = read_le_u64_at(buf, 0).ok_or_else(|| {
            VfsError::IoError("APFS directory record file id out of range".into())
        })?;
        let date_added = read_le_u64_at(buf, 8).ok_or_else(|| {
            VfsError::IoError("APFS directory record date added out of range".into())
        })?;
        let flags = read_le_u16_at(buf, 16)
            .ok_or_else(|| VfsError::IoError("APFS directory record flags out of range".into()))?;
        let d_type = (flags & DREC_TYPE_MASK) as u8;

        // Name follows at offset 18 (null-terminated UTF-8)
        let name_bytes: Vec<u8> = buf[18..].iter().take_while(|&&b| b != 0).cloned().collect();
        let name = String::from_utf8_lossy(&name_bytes).to_string();

        Ok(DrecRecord {
            file_id,
            date_added,
            d_type,
            name,
        })
    }

    /// Get inode record
    fn get_inode(&self, inode_id: u64) -> Result<InodeRecord, VfsError> {
        // Check cache
        if let Some(inode) = self.inode_cache.read().get(&inode_id) {
            return Ok(inode.clone());
        }

        // Search catalog tree for inode
        let root_block = self.read_block(self.volume.root_tree_oid)?;
        let inode = self.find_inode_in_tree(&root_block, inode_id)?;

        // Cache result
        self.inode_cache.write().insert(inode_id, inode.clone());

        Ok(inode)
    }

    /// Find inode in catalog tree
    fn find_inode_in_tree(&self, node_buf: &[u8], target_id: u64) -> Result<InodeRecord, VfsError> {
        let node = Self::parse_btree_node(node_buf)?;
        let is_leaf = (node.flags & BTNODE_LEAF) != 0;
        let (toc_offset, key_area_offset) = checked_btree_offsets(&node)
            .ok_or_else(|| VfsError::IoError("APFS B-tree offset overflow".into()))?;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let Some(key_offset) = key_area_offset.checked_add(kvloc.key_offset as usize) else {
                continue;
            };
            if !has_buffer_range(node_buf, key_offset, 10) {
                continue;
            }

            let Some(obj_id) = read_le_u64_at(node_buf, key_offset) else {
                continue;
            };
            let rec_type = node_buf[key_offset + 8];
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                if rec_type == J_INODE_VAL_TYPE && inode_id == target_id {
                    let Some(val_offset) = checked_value_offset(
                        self.block_size as usize,
                        kvloc.val_offset,
                        kvloc.val_len,
                    ) else {
                        continue;
                    };
                    return self.parse_inode_value(&node_buf[val_offset..]);
                }
            } else {
                let Some(val_offset) =
                    checked_value_offset(self.block_size as usize, kvloc.val_offset, kvloc.val_len)
                else {
                    continue;
                };
                if has_buffer_range(node_buf, val_offset, 8) {
                    let Some(child_addr) = read_le_u64_at(node_buf, val_offset) else {
                        continue;
                    };
                    if let Ok(child_buf) = self.read_block(child_addr) {
                        if let Ok(inode) = self.find_inode_in_tree(&child_buf, target_id) {
                            return Ok(inode);
                        }
                    }
                }
            }
        }

        Err(VfsError::NotFound(format!("Inode {} not found", target_id)))
    }

    /// Parse inode value
    fn parse_inode_value(&self, buf: &[u8]) -> Result<InodeRecord, VfsError> {
        if buf.len() < 92 {
            return Err(VfsError::IoError("Buffer too small for inode".into()));
        }

        Ok(InodeRecord {
            parent_id: read_le_u64_at(buf, 0)
                .ok_or_else(|| VfsError::IoError("APFS inode parent id out of range".into()))?,
            private_id: read_le_u64_at(buf, 8)
                .ok_or_else(|| VfsError::IoError("APFS inode private id out of range".into()))?,
            create_time: read_le_u64_at(buf, 16)
                .ok_or_else(|| VfsError::IoError("APFS inode create time out of range".into()))?,
            mod_time: read_le_u64_at(buf, 24)
                .ok_or_else(|| VfsError::IoError("APFS inode modify time out of range".into()))?,
            change_time: read_le_u64_at(buf, 32)
                .ok_or_else(|| VfsError::IoError("APFS inode change time out of range".into()))?,
            access_time: read_le_u64_at(buf, 40)
                .ok_or_else(|| VfsError::IoError("APFS inode access time out of range".into()))?,
            flags: read_le_u64_at(buf, 48)
                .ok_or_else(|| VfsError::IoError("APFS inode flags out of range".into()))?,
            nchildren: read_le_u32_at(buf, 56)
                .ok_or_else(|| VfsError::IoError("APFS inode child count out of range".into()))?,
            bsd_flags: read_le_u32_at(buf, 68)
                .ok_or_else(|| VfsError::IoError("APFS inode BSD flags out of range".into()))?,
            uid: read_le_u32_at(buf, 72)
                .ok_or_else(|| VfsError::IoError("APFS inode uid out of range".into()))?,
            gid: read_le_u32_at(buf, 76)
                .ok_or_else(|| VfsError::IoError("APFS inode gid out of range".into()))?,
            mode: read_le_u16_at(buf, 80)
                .ok_or_else(|| VfsError::IoError("APFS inode mode out of range".into()))?,
            name: None,
        })
    }

    /// Get file size from dstream
    fn get_file_size(&self, inode_id: u64) -> Result<u64, VfsError> {
        // Check cache
        if let Some(&size) = self.size_cache.read().get(&inode_id) {
            return Ok(size);
        }

        // Search catalog tree for dstream
        let root_block = self.read_block(self.volume.root_tree_oid)?;
        let size = self.find_dstream_size(&root_block, inode_id).unwrap_or(0);

        // Cache result
        self.size_cache.write().insert(inode_id, size);

        Ok(size)
    }

    /// Find dstream (data stream) size in catalog tree
    fn find_dstream_size(&self, node_buf: &[u8], target_id: u64) -> Result<u64, VfsError> {
        let node = Self::parse_btree_node(node_buf)?;
        let is_leaf = (node.flags & BTNODE_LEAF) != 0;
        let (toc_offset, key_area_offset) = checked_btree_offsets(&node)
            .ok_or_else(|| VfsError::IoError("APFS B-tree offset overflow".into()))?;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let Some(key_offset) = key_area_offset.checked_add(kvloc.key_offset as usize) else {
                continue;
            };
            if !has_buffer_range(node_buf, key_offset, 10) {
                continue;
            }

            let Some(obj_id) = read_le_u64_at(node_buf, key_offset) else {
                continue;
            };
            let rec_type = node_buf[key_offset + 8];
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                if rec_type == J_DSTREAM_TYPE && inode_id == target_id {
                    let Some(val_offset) = checked_value_offset(
                        self.block_size as usize,
                        kvloc.val_offset,
                        kvloc.val_len,
                    ) else {
                        continue;
                    };
                    if has_buffer_range(node_buf, val_offset, 24) {
                        let Some(size) = read_le_u64_at(node_buf, val_offset) else {
                            continue;
                        };
                        return Ok(size);
                    }
                }
            } else {
                let Some(val_offset) =
                    checked_value_offset(self.block_size as usize, kvloc.val_offset, kvloc.val_len)
                else {
                    continue;
                };
                if has_buffer_range(node_buf, val_offset, 8) {
                    let Some(child_addr) = read_le_u64_at(node_buf, val_offset) else {
                        continue;
                    };
                    if let Ok(child_buf) = self.read_block(child_addr) {
                        if let Ok(size) = self.find_dstream_size(&child_buf, target_id) {
                            return Ok(size);
                        }
                    }
                }
            }
        }

        Err(VfsError::NotFound(format!(
            "Dstream for inode {} not found",
            target_id
        )))
    }

    /// Resolve path to inode ID
    fn resolve_path(&self, path: &str) -> Result<u64, VfsError> {
        let normalized = normalize_path(path);
        if normalized == "/" {
            return Ok(ROOT_DIR_INODE_ID);
        }

        let parts: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current_id = ROOT_DIR_INODE_ID;

        for part in parts {
            let entries = self.find_directory_entries(current_id)?;
            let found = entries
                .iter()
                .find(|(name, _, _)| name.eq_ignore_ascii_case(part));

            match found {
                Some((_, d_type, child_id)) => {
                    if *d_type != DT_DIR {
                        // Allow non-directory for last component
                    }
                    current_id = *child_id;
                }
                None => {
                    return Err(VfsError::NotFound(normalized));
                }
            }
        }

        Ok(current_id)
    }

    /// Convert APFS d_type to VFS file_type
    fn dtype_to_filetype(d_type: u8) -> u8 {
        match d_type {
            DT_DIR => 4,
            DT_REG => 8,
            DT_LNK => 10,
            DT_BLK => 6,
            DT_CHR => 2,
            DT_FIFO => 1,
            DT_SOCK => 12,
            _ => 0,
        }
    }
}

impl FilesystemDriver for ApfsDriver {
    fn info(&self) -> &FilesystemInfo {
        &self.info
    }

    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);

        if normalized == "/" {
            return Ok(FileAttr {
                size: 0,
                is_directory: true,
                permissions: 0o755,
                nlink: 2,
                inode: ROOT_DIR_INODE_ID,
                ..Default::default()
            });
        }

        let inode_id = self.resolve_path(path)?;
        let inode = self.get_inode(inode_id)?;

        let is_dir = (inode.mode & 0xF000) == 0x4000; // S_IFDIR
        let size = if is_dir {
            0
        } else {
            self.get_file_size(inode_id).unwrap_or(0)
        };

        Ok(FileAttr {
            size,
            is_directory: is_dir,
            permissions: inode.mode & 0o777,
            nlink: if is_dir { 2 + inode.nchildren } else { 1 },
            inode: inode_id,
            ..Default::default()
        })
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let inode_id = self.resolve_path(path)?;
        let entries = self.find_directory_entries(inode_id)?;

        Ok(entries
            .iter()
            .map(|(name, d_type, file_id)| DirEntry {
                name: name.clone(),
                is_directory: *d_type == DT_DIR,
                inode: *file_id,
                file_type: Self::dtype_to_filetype(*d_type),
            })
            .collect())
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let inode_id = self.resolve_path(path)?;
        let inode = self.get_inode(inode_id)?;

        let is_dir = (inode.mode & 0xF000) == 0x4000;
        if is_dir {
            return Err(VfsError::NotAFile(path.to_string()));
        }

        let file_size = self.get_file_size(inode_id).unwrap_or(0);
        let Some((read_end, total_to_read)) = bounded_apfs_file_read(file_size, offset, size)?
        else {
            return Ok(Vec::new());
        };

        // Find file extents from catalog tree
        let root_block = self.read_block(self.volume.root_tree_oid)?;
        let mut extents = Vec::new();
        self.find_file_extents(&root_block, inode_id, &mut extents)?;

        // Sort extents by logical offset
        extents.sort_by_key(|e| e.0);

        // Read data from extents, mapping logical offset to physical blocks
        let mut result = vec![0u8; total_to_read];
        let mut saw_data_extent = false;

        for &(extent_logical_offset, extent_phys_block, extent_length) in &extents {
            let extent_end = checked_apfs_extent_end(extent_logical_offset, extent_length)?;

            // Skip extents before our read range
            if extent_end <= offset {
                continue;
            }
            // Stop if we've read past our range
            if extent_logical_offset >= read_end {
                break;
            }

            // Calculate overlap between read range and this extent
            let read_start_in_extent = offset.saturating_sub(extent_logical_offset);
            let read_end_in_extent = std::cmp::min(
                extent_length,
                read_end
                    .checked_sub(extent_logical_offset)
                    .ok_or_else(|| VfsError::Internal("APFS extent overlap underflow".into()))?,
            );

            if read_end_in_extent <= read_start_in_extent {
                continue;
            }

            let bytes_from_extent =
                bounded_apfs_read_len(read_start_in_extent, read_end_in_extent)?;
            if bytes_from_extent == 0 {
                continue;
            }

            // Calculate which physical blocks to read
            let phys_byte_offset = checked_apfs_physical_offset(
                self.offset,
                extent_phys_block,
                self.block_size,
                read_start_in_extent,
            )?;

            let mut extent_buf = vec![0u8; bytes_from_extent];
            read_apfs_exact(
                self.device.as_ref(),
                phys_byte_offset,
                &mut extent_buf,
                "APFS file extent",
            )?;

            // Copy into result at the right position
            let dest_offset = if extent_logical_offset > offset {
                usize::try_from(
                    extent_logical_offset
                        .checked_sub(offset)
                        .ok_or_else(|| VfsError::Internal("APFS destination underflow".into()))?,
                )
                .map_err(|_| VfsError::Internal("APFS destination offset too large".into()))?
            } else {
                0
            };

            let available_dest = total_to_read
                .checked_sub(dest_offset)
                .ok_or_else(|| VfsError::Internal("APFS destination range underflow".into()))?;
            let copy_len = std::cmp::min(bytes_from_extent, available_dest);
            if copy_len == 0 {
                continue;
            }

            let dest_end = dest_offset
                .checked_add(copy_len)
                .ok_or_else(|| VfsError::Internal("APFS destination range overflow".into()))?;
            result[dest_offset..dest_end].copy_from_slice(&extent_buf[..copy_len]);
            saw_data_extent = true;
        }

        // If no extents matched, the file may be inline or sparse
        ensure_apfs_extent_read_available(saw_data_extent, total_to_read, inode_id, extents.len())?;

        Ok(result)
    }
}

impl ApfsDriver {
    /// Find file extent records in the catalog B-tree for a given inode.
    /// Each extent is (logical_offset, physical_block_num, length_in_bytes).
    fn find_file_extents(
        &self,
        node_buf: &[u8],
        target_id: u64,
        extents: &mut Vec<(u64, u64, u64)>,
    ) -> Result<(), VfsError> {
        let node = Self::parse_btree_node(node_buf)?;
        let is_leaf = (node.flags & BTNODE_LEAF) != 0;
        let (toc_offset, key_area_offset) = checked_btree_offsets(&node)
            .ok_or_else(|| VfsError::IoError("APFS B-tree offset overflow".into()))?;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let Some(key_offset) = key_area_offset.checked_add(kvloc.key_offset as usize) else {
                continue;
            };
            if !has_buffer_range(node_buf, key_offset, 16) {
                continue;
            }

            let Some(obj_id) = read_le_u64_at(node_buf, key_offset) else {
                continue;
            };
            let rec_type = node_buf[key_offset + 8];
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                // File extent key: [obj_id(8)] [type(1)] [pad(3)] [logical_offset(8)]
                if rec_type == J_FILE_EXTENT_TYPE && inode_id == target_id {
                    // Parse logical offset from key (offset 12, 8 bytes)
                    let logical_offset = if has_buffer_range(node_buf, key_offset + 12, 8) {
                        read_le_u64_at(node_buf, key_offset + 12).unwrap_or(0)
                    } else {
                        0
                    };

                    // Parse extent value: [flags(8)] [phys_block_num(8)] [length(8)]
                    let Some(val_offset) = checked_value_offset(
                        self.block_size as usize,
                        kvloc.val_offset,
                        kvloc.val_len,
                    ) else {
                        continue;
                    };
                    if has_buffer_range(node_buf, val_offset, 24) {
                        // Skip 8-byte flags field
                        let Some(phys_block) = read_le_u64_at(node_buf, val_offset + 8) else {
                            continue;
                        };
                        let Some(length) = read_le_u64_at(node_buf, val_offset + 16) else {
                            continue;
                        };
                        extents.push((logical_offset, phys_block, length));
                    }
                }
            } else {
                // Internal node - recurse into child
                let Some(val_offset) =
                    checked_value_offset(self.block_size as usize, kvloc.val_offset, kvloc.val_len)
                else {
                    continue;
                };
                if has_buffer_range(node_buf, val_offset, 8) {
                    let Some(child_addr) = read_le_u64_at(node_buf, val_offset) else {
                        continue;
                    };
                    if let Ok(child_buf) = self.read_block(child_addr) {
                        self.find_file_extents(&child_buf, target_id, extents)?;
                    }
                }
            }
        }

        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ffx_errors::ContainerError;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    struct MockBlockDevice {
        data: Vec<u8>,
    }

    struct MockBlockReader {
        cursor: Cursor<Vec<u8>>,
    }

    impl Read for MockBlockReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.cursor.read(buf)
        }
    }

    impl Seek for MockBlockReader {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.cursor.seek(pos)
        }
    }

    impl super::super::traits::BlockReader for MockBlockReader {}

    impl super::super::traits::BlockDevice for MockBlockDevice {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
            let start = usize::try_from(offset).unwrap_or(self.data.len());
            if start >= self.data.len() {
                return Ok(0);
            }

            let available = self.data.len() - start;
            let to_copy = available.min(buf.len());
            buf[..to_copy].copy_from_slice(&self.data[start..start + to_copy]);
            Ok(to_copy)
        }

        fn size(&self) -> u64 {
            self.data.len() as u64
        }
    }

    impl super::super::traits::SeekableBlockDevice for MockBlockDevice {
        fn reader_at(&self, offset: u64) -> Box<dyn super::super::traits::BlockReader> {
            let start = usize::try_from(offset)
                .unwrap_or(self.data.len())
                .min(self.data.len());
            Box::new(MockBlockReader {
                cursor: Cursor::new(self.data[start..].to_vec()),
            })
        }
    }

    #[test]
    fn test_apfs_magic_constants() {
        assert_eq!(APFS_CONTAINER_MAGIC, 0x4E585342);
        assert_eq!(APFS_VOLUME_MAGIC, 0x41505342);
    }

    #[test]
    fn test_apfs_filesystem_type() {
        assert_eq!(FilesystemType::Apfs.to_string(), "APFS");
    }

    #[test]
    fn test_dtype_conversions() {
        assert_eq!(ApfsDriver::dtype_to_filetype(DT_DIR), 4);
        assert_eq!(ApfsDriver::dtype_to_filetype(DT_REG), 8);
        assert_eq!(ApfsDriver::dtype_to_filetype(DT_LNK), 10);
    }

    #[test]
    fn test_root_inode_id() {
        assert_eq!(ROOT_INODE_ID, 2);
        assert_eq!(ROOT_DIR_INODE_ID, 2);
    }

    #[test]
    fn test_clamp_read_end_respects_file_boundary() {
        assert_eq!(clamp_read_end(10, 8, 15), 15);
    }

    #[test]
    fn test_clamp_read_end_handles_overflow() {
        assert_eq!(clamp_read_end(u64::MAX - 2, 8, u64::MAX), u64::MAX);
    }

    #[test]
    fn test_bounded_apfs_read_len_rejects_underflow() {
        let err =
            bounded_apfs_read_len(10, 9).expect_err("APFS read range underflow should be rejected");

        assert!(matches!(err, VfsError::Internal(_)));
    }

    #[test]
    fn test_apfs_little_endian_readers_reject_short_ranges() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7];

        assert_eq!(read_le_u16_at(&bytes, 6), None);
        assert_eq!(read_le_u32_at(&bytes, 4), None);
        assert_eq!(read_le_u64_at(&bytes, 0), None);
    }

    #[test]
    fn test_bounded_apfs_file_read_allows_exact_eof_and_zero_size() {
        assert_eq!(bounded_apfs_file_read(128, 128, 16).unwrap(), None);
        assert_eq!(bounded_apfs_file_read(128, 0, 0).unwrap(), None);
    }

    #[test]
    fn test_bounded_apfs_file_read_rejects_offset_past_eof() {
        let err = bounded_apfs_file_read(128, 129, 16).unwrap_err();
        assert!(matches!(
            err,
            VfsError::OutOfBounds {
                offset: 129,
                size: 16
            }
        ));
    }

    #[test]
    fn test_bounded_apfs_file_read_clamps_to_remaining() {
        assert_eq!(
            bounded_apfs_file_read(128, 120, 16).unwrap(),
            Some((128, 8))
        );
    }

    #[test]
    fn test_ensure_apfs_extent_read_available_allows_sparse_tail() {
        assert!(ensure_apfs_extent_read_available(true, 128, 42, 1).is_ok());
    }

    #[test]
    fn test_ensure_apfs_extent_read_available_rejects_missing_extents() {
        let err = ensure_apfs_extent_read_available(false, 128, 42, 0)
            .expect_err("APFS read without any matching data extent should fail");

        assert!(matches!(err, VfsError::Internal(_)));
        assert!(err.to_string().contains("inode 42"));
    }

    #[test]
    fn test_checked_apfs_extent_end_rejects_overflow() {
        let err = checked_apfs_extent_end(u64::MAX - 1, 8)
            .expect_err("APFS extent end overflow should be rejected");

        assert!(matches!(err, VfsError::Internal(_)));
    }

    #[test]
    fn test_checked_apfs_physical_offset_rejects_overflow() {
        let err = checked_apfs_physical_offset(u64::MAX - 1, 1, 2, 0)
            .expect_err("APFS physical offset overflow should be rejected");

        assert!(matches!(err, VfsError::Internal(_)));
    }

    #[test]
    fn test_checked_apfs_block_offset_rejects_overflow() {
        let err = checked_apfs_block_offset(u64::MAX - 1, 1, 2)
            .expect_err("APFS block offset overflow should be rejected");

        assert!(matches!(err, VfsError::Internal(_)));
    }

    #[test]
    fn test_checked_apfs_block_offset_adds_base_and_block_offset() {
        assert_eq!(checked_apfs_block_offset(10, 2, 4).unwrap(), 18);
    }

    #[test]
    fn test_checked_apfs_container_size_rejects_overflow() {
        let err = checked_apfs_container_size(u64::MAX, 2)
            .expect_err("APFS container size overflow should be rejected");

        assert!(matches!(err, VfsError::Internal(_)));
    }

    #[test]
    fn test_parse_btree_node_rejects_short_buffer() {
        let err = ApfsDriver::parse_btree_node(&[0u8; 55])
            .expect_err("short APFS B-tree node should be rejected");

        assert!(matches!(err, VfsError::IoError(message) if message.contains("B-tree node")));
    }

    #[test]
    fn test_parse_drec_value_rejects_short_buffer() {
        let device: Arc<dyn super::super::traits::SeekableBlockDevice> =
            Arc::new(MockBlockDevice {
                data: vec![0; 4096],
            });
        let driver = ApfsDriver {
            info: FilesystemInfo {
                fs_type: FilesystemType::Apfs,
                label: Some("test".to_string()),
                total_size: 4096,
                free_space: None,
                cluster_size: 4096,
            },
            device,
            offset: 0,
            container: NxSuperblock {
                header: ObjPhysHeader {
                    cksum: 0,
                    oid: 0,
                    xid: 0,
                    obj_type: OBJECT_TYPE_NX_SUPERBLOCK,
                    obj_subtype: 0,
                },
                magic: APFS_CONTAINER_MAGIC,
                block_size: 4096,
                block_count: 1,
                max_file_systems: 1,
                omap_oid: 0,
                fs_oid: vec![1],
            },
            volume: ApfsSuperblock {
                header: ObjPhysHeader {
                    cksum: 0,
                    oid: 0,
                    xid: 0,
                    obj_type: OBJECT_TYPE_FS,
                    obj_subtype: 0,
                },
                magic: APFS_VOLUME_MAGIC,
                vol_index: 0,
                omap_oid: 0,
                root_tree_oid: 0,
                root_tree_type: 0,
                vol_name: "test".to_string(),
            },
            block_size: 4096,
            dir_cache: RwLock::new(HashMap::new()),
            inode_cache: RwLock::new(HashMap::new()),
            size_cache: RwLock::new(HashMap::new()),
        };

        let err = driver
            .parse_drec_value(&[0u8; 17])
            .expect_err("short APFS directory record should be rejected");

        assert!(matches!(err, VfsError::IoError(message) if message.contains("drec")));
    }

    #[test]
    fn test_read_container_superblock_rejects_short_header() {
        let device: Arc<dyn super::super::traits::SeekableBlockDevice> =
            Arc::new(MockBlockDevice { data: vec![0; 32] });

        let err = ApfsDriver::read_container_superblock(&device, 0)
            .expect_err("short APFS header should be rejected");

        assert!(
            matches!(err, VfsError::IoError(message) if message.contains("header is truncated"))
        );
    }

    #[test]
    fn test_read_container_superblock_rejects_short_full_block() {
        let mut data = vec![0u8; 128];
        data[36..40].copy_from_slice(&4096u32.to_le_bytes());
        let device: Arc<dyn super::super::traits::SeekableBlockDevice> =
            Arc::new(MockBlockDevice { data });

        let err = ApfsDriver::read_container_superblock(&device, 0)
            .expect_err("short APFS container block should be rejected");

        assert!(
            matches!(err, VfsError::IoError(message) if message.contains("block is truncated"))
        );
    }

    #[test]
    fn test_read_container_superblock_rejects_tiny_advertised_block_size() {
        let mut data = vec![0u8; 64];
        data[36..40].copy_from_slice(&64u32.to_le_bytes());
        let device: Arc<dyn super::super::traits::SeekableBlockDevice> =
            Arc::new(MockBlockDevice { data });

        let err = ApfsDriver::read_container_superblock(&device, 0)
            .expect_err("tiny APFS container block size should be rejected");

        assert!(
            matches!(err, VfsError::IoError(message) if message.contains("too small for container superblock fields"))
        );
    }

    #[test]
    fn test_checked_toc_entry_offset_handles_overflow() {
        assert_eq!(checked_toc_entry_offset(usize::MAX - 4, 1), None);
    }

    #[test]
    fn test_apfs_checked_slice_rejects_overflowing_range() {
        assert!(checked_slice(&[0u8; 8], usize::MAX, 1).is_none());
    }

    #[test]
    fn test_apfs_little_endian_read_helpers_reject_overflowing_offsets() {
        let bytes = [0u8; 8];

        assert!(read_le_u16_at(&bytes, usize::MAX).is_none());
        assert!(read_le_u64_at(&bytes, usize::MAX).is_none());
    }

    #[test]
    fn test_parse_toc_reads_valid_entry_with_checked_helpers() {
        let mut buf = vec![0u8; 64];
        buf[16..18].copy_from_slice(&2u16.to_le_bytes());
        buf[18..20].copy_from_slice(&4u16.to_le_bytes());
        buf[20..22].copy_from_slice(&6u16.to_le_bytes());
        buf[22..24].copy_from_slice(&8u16.to_le_bytes());

        let entries = ApfsDriver::parse_toc(&buf, 16, 1).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key_offset, 2);
        assert_eq!(entries[0].key_len, 4);
        assert_eq!(entries[0].val_offset, 6);
        assert_eq!(entries[0].val_len, 8);
    }

    #[test]
    fn test_checked_value_offset_rejects_underflow() {
        assert_eq!(checked_value_offset(64, 40, 32), None);
    }

    #[test]
    fn test_checked_kvloc_capacity_rejects_huge_allocation() {
        let err = checked_kvloc_capacity(usize::MAX)
            .expect_err("huge APFS TOC entry count should be rejected");

        assert!(matches!(err, VfsError::IoError(_)));
    }

    #[test]
    fn test_read_apfs_exact_accepts_full_read() {
        let device = MockBlockDevice {
            data: vec![1, 2, 3, 4],
        };
        let mut buf = [0u8; 2];

        read_apfs_exact(&device, 1, &mut buf, "APFS test").unwrap();

        assert_eq!(buf, [2, 3]);
    }

    #[test]
    fn test_read_apfs_exact_rejects_short_read() {
        let device = MockBlockDevice {
            data: vec![1, 2, 3],
        };
        let mut buf = [0u8; 4];

        let err = read_apfs_exact(&device, 0, &mut buf, "APFS test")
            .expect_err("short APFS exact read should fail");

        assert!(
            matches!(err, VfsError::IoError(message) if message.contains("APFS test is truncated"))
        );
    }

    #[test]
    fn test_read_block_static_rejects_short_block() {
        let device: Arc<dyn super::super::traits::SeekableBlockDevice> =
            Arc::new(MockBlockDevice { data: vec![0; 32] });

        let err = ApfsDriver::read_block_static(&device, 0, 64, 0)
            .expect_err("short APFS block should fail");

        assert!(
            matches!(err, VfsError::IoError(message) if message.contains("APFS block is truncated"))
        );
    }
}
