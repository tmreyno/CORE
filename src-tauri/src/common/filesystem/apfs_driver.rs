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
use crate::common::vfs::{normalize_path, DirEntry, FileAttr, VfsError};

// =============================================================================
// APFS Constants
// =============================================================================

/// APFS container superblock magic 'NXSB'
const APFS_CONTAINER_MAGIC: u32 = 0x4E585342; // 'NXSB' in little-endian
/// APFS volume superblock magic 'APSB'
const APFS_VOLUME_MAGIC: u32 = 0x41505342; // 'APSB' in little-endian
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

        let total_size = container.block_count * block_size as u64;

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
        let block_offset = self.offset + (block_num * self.block_size as u64);
        self.device
            .read_at(block_offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
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
            cksum: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            oid: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            xid: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            obj_type: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
            obj_subtype: u32::from_le_bytes(buf[28..32].try_into().unwrap()),
        })
    }

    /// Read container superblock
    fn read_container_superblock(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
    ) -> Result<NxSuperblock, VfsError> {
        // First read to get block size (at offset 36)
        let mut header_buf = vec![0u8; 64];
        device
            .read_at(offset, &mut header_buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        let block_size = u32::from_le_bytes(header_buf[36..40].try_into().unwrap());
        if block_size == 0 || block_size > 65536 {
            return Err(VfsError::IoError(format!(
                "Invalid APFS block size: {}",
                block_size
            )));
        }

        // Now read full block
        let mut buf = vec![0u8; block_size as usize];
        device
            .read_at(offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        let header = Self::parse_obj_header(&buf)?;

        // Verify it's a container superblock
        if (header.obj_type & OBJ_TYPE_MASK) != OBJECT_TYPE_NX_SUPERBLOCK {
            return Err(VfsError::IoError(format!(
                "Expected container superblock, got type 0x{:08X}",
                header.obj_type
            )));
        }

        let magic = u32::from_le_bytes(buf[32..36].try_into().unwrap());
        let block_count = u64::from_le_bytes(buf[40..48].try_into().unwrap());
        let max_file_systems = u32::from_le_bytes(buf[100..104].try_into().unwrap());
        let omap_oid = u64::from_le_bytes(buf[160..168].try_into().unwrap());

        // Read volume OIDs (up to 100 volumes, starting at offset 168)
        let mut fs_oid = Vec::new();
        for i in 0..std::cmp::min(max_file_systems as usize, 100) {
            let oid_offset = 168 + i * 8;
            if oid_offset + 8 <= buf.len() {
                let oid = u64::from_le_bytes(buf[oid_offset..oid_offset + 8].try_into().unwrap());
                if oid != 0 {
                    fs_oid.push(oid);
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
        let block_offset = offset + (block_num * block_size as u64);
        device
            .read_at(block_offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        Ok(buf)
    }

    /// Parse object map
    fn parse_omap(buf: &[u8]) -> Result<OmapPhys, VfsError> {
        if buf.len() < 88 {
            return Err(VfsError::IoError("Buffer too small for omap".into()));
        }

        let header = Self::parse_obj_header(buf)?;
        let tree_oid = u64::from_le_bytes(buf[48..56].try_into().unwrap());

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
        let toc_offset = 56 + node.table_space_offset as usize;
        let key_area_offset = toc_offset + node.table_space_len as usize;

        // Get key-value locations from TOC
        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            // Read key (OID + XID)
            let key_offset = key_area_offset + kvloc.key_offset as usize;
            if key_offset + 16 > node_buf.len() {
                continue;
            }

            let key_oid =
                u64::from_le_bytes(node_buf[key_offset..key_offset + 8].try_into().unwrap());

            if is_leaf {
                if key_oid == target_oid {
                    // Value is at end of block, working backwards
                    let val_offset =
                        block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;
                    if val_offset + 16 <= node_buf.len() {
                        // Skip flags (4 bytes) and size (4 bytes), get paddr
                        let paddr = u64::from_le_bytes(
                            node_buf[val_offset + 8..val_offset + 16]
                                .try_into()
                                .unwrap(),
                        );
                        return Ok(paddr);
                    }
                }
            } else {
                // Index node - check if we should descend
                if key_oid >= target_oid {
                    let val_offset =
                        block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;
                    if val_offset + 8 <= node_buf.len() {
                        let child_oid = u64::from_le_bytes(
                            node_buf[val_offset..val_offset + 8].try_into().unwrap(),
                        );
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
            let last = kvlocs.last().unwrap();
            let val_offset = block_size as usize - last.val_offset as usize - last.val_len as usize;
            if val_offset + 8 <= node_buf.len() {
                let child_oid =
                    u64::from_le_bytes(node_buf[val_offset..val_offset + 8].try_into().unwrap());
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
            flags: u16::from_le_bytes(buf[32..34].try_into().unwrap()),
            level: u16::from_le_bytes(buf[34..36].try_into().unwrap()),
            nkeys: u32::from_le_bytes(buf[36..40].try_into().unwrap()),
            table_space_offset: u16::from_le_bytes(buf[40..42].try_into().unwrap()),
            table_space_len: u16::from_le_bytes(buf[42..44].try_into().unwrap()),
            free_space_offset: u16::from_le_bytes(buf[44..46].try_into().unwrap()),
            free_space_len: u16::from_le_bytes(buf[46..48].try_into().unwrap()),
            key_free_list_offset: u16::from_le_bytes(buf[48..50].try_into().unwrap()),
            key_free_list_len: u16::from_le_bytes(buf[50..52].try_into().unwrap()),
            val_free_list_offset: u16::from_le_bytes(buf[52..54].try_into().unwrap()),
            val_free_list_len: u16::from_le_bytes(buf[54..56].try_into().unwrap()),
        })
    }

    /// Parse table of contents
    fn parse_toc(buf: &[u8], toc_offset: usize, nkeys: usize) -> Result<Vec<KvLoc>, VfsError> {
        let mut locs = Vec::with_capacity(nkeys);

        for i in 0..nkeys {
            let entry_offset = toc_offset + i * 8; // Each TOC entry is 8 bytes
            if entry_offset + 8 > buf.len() {
                break;
            }

            locs.push(KvLoc {
                key_offset: u16::from_le_bytes(
                    buf[entry_offset..entry_offset + 2].try_into().unwrap(),
                ),
                key_len: u16::from_le_bytes(
                    buf[entry_offset + 2..entry_offset + 4].try_into().unwrap(),
                ),
                val_offset: u16::from_le_bytes(
                    buf[entry_offset + 4..entry_offset + 6].try_into().unwrap(),
                ),
                val_len: u16::from_le_bytes(
                    buf[entry_offset + 6..entry_offset + 8].try_into().unwrap(),
                ),
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

        let magic = u32::from_le_bytes(buf[32..36].try_into().unwrap());
        let vol_index = u32::from_le_bytes(buf[36..40].try_into().unwrap());
        let omap_oid = u64::from_le_bytes(buf[80..88].try_into().unwrap());
        let root_tree_oid = u64::from_le_bytes(buf[88..96].try_into().unwrap());
        let root_tree_type = u32::from_le_bytes(buf[96..100].try_into().unwrap());

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
        let toc_offset = 56 + node.table_space_offset as usize;
        let key_area_offset = toc_offset + node.table_space_len as usize;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let key_offset = key_area_offset + kvloc.key_offset as usize;
            if key_offset + 10 > node_buf.len() {
                continue;
            }

            // Catalog key: obj_id (8) + type (1)
            let obj_id =
                u64::from_le_bytes(node_buf[key_offset..key_offset + 8].try_into().unwrap());
            let rec_type = node_buf[key_offset + 8];

            // Clear high bits that indicate type
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                let val_offset =
                    self.block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;

                if rec_type == J_DIR_REC_TYPE && inode_id == parent_id {
                    // This is a directory record for our parent
                    if let Ok(drec) = self.parse_drec_value(&node_buf[val_offset..]) {
                        entries.push((drec.name, drec.d_type, drec.file_id));
                    }
                }
            } else {
                // Index node - traverse children that might contain our parent
                let val_offset =
                    self.block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;
                if val_offset + 8 <= node_buf.len() {
                    let child_addr = u64::from_le_bytes(
                        node_buf[val_offset..val_offset + 8].try_into().unwrap(),
                    );
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

        let file_id = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        let date_added = u64::from_le_bytes(buf[8..16].try_into().unwrap());
        let flags = u16::from_le_bytes(buf[16..18].try_into().unwrap());
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
        let toc_offset = 56 + node.table_space_offset as usize;
        let key_area_offset = toc_offset + node.table_space_len as usize;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let key_offset = key_area_offset + kvloc.key_offset as usize;
            if key_offset + 10 > node_buf.len() {
                continue;
            }

            let obj_id =
                u64::from_le_bytes(node_buf[key_offset..key_offset + 8].try_into().unwrap());
            let rec_type = node_buf[key_offset + 8];
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                if rec_type == J_INODE_VAL_TYPE && inode_id == target_id {
                    let val_offset = self.block_size as usize
                        - kvloc.val_offset as usize
                        - kvloc.val_len as usize;
                    return self.parse_inode_value(&node_buf[val_offset..]);
                }
            } else {
                let val_offset =
                    self.block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;
                if val_offset + 8 <= node_buf.len() {
                    let child_addr = u64::from_le_bytes(
                        node_buf[val_offset..val_offset + 8].try_into().unwrap(),
                    );
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
            parent_id: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            private_id: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            create_time: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            mod_time: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            change_time: u64::from_le_bytes(buf[32..40].try_into().unwrap()),
            access_time: u64::from_le_bytes(buf[40..48].try_into().unwrap()),
            flags: u64::from_le_bytes(buf[48..56].try_into().unwrap()),
            nchildren: u32::from_le_bytes(buf[56..60].try_into().unwrap()),
            bsd_flags: u32::from_le_bytes(buf[68..72].try_into().unwrap()),
            uid: u32::from_le_bytes(buf[72..76].try_into().unwrap()),
            gid: u32::from_le_bytes(buf[76..80].try_into().unwrap()),
            mode: u16::from_le_bytes(buf[80..82].try_into().unwrap()),
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
        let toc_offset = 56 + node.table_space_offset as usize;
        let key_area_offset = toc_offset + node.table_space_len as usize;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let key_offset = key_area_offset + kvloc.key_offset as usize;
            if key_offset + 10 > node_buf.len() {
                continue;
            }

            let obj_id =
                u64::from_le_bytes(node_buf[key_offset..key_offset + 8].try_into().unwrap());
            let rec_type = node_buf[key_offset + 8];
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                if rec_type == J_DSTREAM_TYPE && inode_id == target_id {
                    let val_offset = self.block_size as usize
                        - kvloc.val_offset as usize
                        - kvloc.val_len as usize;
                    if val_offset + 24 <= node_buf.len() {
                        let size = u64::from_le_bytes(
                            node_buf[val_offset..val_offset + 8].try_into().unwrap(),
                        );
                        return Ok(size);
                    }
                }
            } else {
                let val_offset =
                    self.block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;
                if val_offset + 8 <= node_buf.len() {
                    let child_addr = u64::from_le_bytes(
                        node_buf[val_offset..val_offset + 8].try_into().unwrap(),
                    );
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
        if offset >= file_size {
            return Ok(Vec::new());
        }

        // Clamp read length to file boundary
        let read_end = std::cmp::min(offset + size as u64, file_size);
        let total_to_read = (read_end - offset) as usize;

        // Find file extents from catalog tree
        let root_block = self.read_block(self.volume.root_tree_oid)?;
        let mut extents = Vec::new();
        self.find_file_extents(&root_block, inode_id, &mut extents)?;

        // Sort extents by logical offset
        extents.sort_by_key(|e| e.0);

        // Read data from extents, mapping logical offset to physical blocks
        let mut result = vec![0u8; total_to_read];
        let mut bytes_filled = 0usize;

        for &(extent_logical_offset, extent_phys_block, extent_length) in &extents {
            let extent_end = extent_logical_offset + extent_length;

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
            let read_end_in_extent = std::cmp::min(extent_length, read_end - extent_logical_offset);

            let bytes_from_extent = (read_end_in_extent - read_start_in_extent) as usize;
            if bytes_from_extent == 0 {
                continue;
            }

            // Calculate which physical blocks to read
            let phys_byte_offset =
                self.offset + (extent_phys_block * self.block_size as u64) + read_start_in_extent;

            let mut extent_buf = vec![0u8; bytes_from_extent];
            self.device
                .read_at(phys_byte_offset, &mut extent_buf)
                .map_err(|e| VfsError::IoError(e.to_string()))?;

            // Copy into result at the right position
            let dest_offset = if extent_logical_offset > offset {
                (extent_logical_offset - offset) as usize
            } else {
                0
            };

            let copy_len = std::cmp::min(bytes_from_extent, total_to_read - dest_offset);
            if dest_offset + copy_len <= result.len() {
                result[dest_offset..dest_offset + copy_len]
                    .copy_from_slice(&extent_buf[..copy_len]);
                bytes_filled += copy_len;
            }
        }

        // If no extents matched, the file may be inline or sparse
        if bytes_filled == 0 && total_to_read > 0 {
            return Err(VfsError::Internal(format!(
                "No file extents found for inode {} (found {} extents total)",
                inode_id,
                extents.len()
            )));
        }

        result.truncate(std::cmp::min(total_to_read, bytes_filled));
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
        let toc_offset = 56 + node.table_space_offset as usize;
        let key_area_offset = toc_offset + node.table_space_len as usize;

        let kvlocs = Self::parse_toc(node_buf, toc_offset, node.nkeys as usize)?;

        for kvloc in &kvlocs {
            let key_offset = key_area_offset + kvloc.key_offset as usize;
            if key_offset + 16 > node_buf.len() {
                continue;
            }

            let obj_id =
                u64::from_le_bytes(node_buf[key_offset..key_offset + 8].try_into().unwrap());
            let rec_type = node_buf[key_offset + 8];
            let inode_id = obj_id & 0x0FFFFFFFFFFFFFFF;

            if is_leaf {
                // File extent key: [obj_id(8)] [type(1)] [pad(3)] [logical_offset(8)]
                if rec_type == J_FILE_EXTENT_TYPE && inode_id == target_id {
                    // Parse logical offset from key (offset 12, 8 bytes)
                    let logical_offset = if key_offset + 20 <= node_buf.len() {
                        u64::from_le_bytes(
                            node_buf[key_offset + 12..key_offset + 20]
                                .try_into()
                                .unwrap(),
                        )
                    } else {
                        0
                    };

                    // Parse extent value: [flags(8)] [phys_block_num(8)] [length(8)]
                    let val_offset = self.block_size as usize
                        - kvloc.val_offset as usize
                        - kvloc.val_len as usize;
                    if val_offset + 24 <= node_buf.len() {
                        // Skip 8-byte flags field
                        let phys_block = u64::from_le_bytes(
                            node_buf[val_offset + 8..val_offset + 16]
                                .try_into()
                                .unwrap(),
                        );
                        let length = u64::from_le_bytes(
                            node_buf[val_offset + 16..val_offset + 24]
                                .try_into()
                                .unwrap(),
                        );
                        extents.push((logical_offset, phys_block, length));
                    }
                }
            } else {
                // Internal node - recurse into child
                let val_offset =
                    self.block_size as usize - kvloc.val_offset as usize - kvloc.val_len as usize;
                if val_offset + 8 <= node_buf.len() {
                    let child_addr = u64::from_le_bytes(
                        node_buf[val_offset..val_offset + 8].try_into().unwrap(),
                    );
                    if let Ok(child_buf) = self.read_block(child_addr) {
                        let _ = self.find_file_extents(&child_buf, target_id, extents);
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
}
