// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Allow dead code for EXT binary structures - many fields are parsed but not yet used
#![allow(dead_code)]

//! # EXT2/3/4 Filesystem Driver
//!
//! Implements EXT2, EXT3, and EXT4 filesystem access for Linux partitions.
//!
//! ## EXT Filesystem Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Boot Block (1024 bytes, unused by filesystem)              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Superblock (1024 bytes at offset 1024)                     │
//! │  - Magic: 0xEF53 at offset 0x38                             │
//! │  - Block size, inode size, etc.                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Block Group 0                                              │
//! │  ├── Block Group Descriptor Table                           │
//! │  ├── Block Bitmap                                           │
//! │  ├── Inode Bitmap                                           │
//! │  ├── Inode Table                                            │
//! │  └── Data Blocks                                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Block Group 1...N (same structure)                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Constants
//! - EXT_SUPER_MAGIC: 0xEF53
//! - Superblock offset: 1024 bytes
//! - Root inode: 2
//! - Default block size: 1024, 2048, or 4096 bytes

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::common::vfs::{VfsError, FileAttr, DirEntry, normalize_path};
use super::traits::{FilesystemDriver, FilesystemInfo, FilesystemType, SeekableBlockDevice};

// =============================================================================
// Constants
// =============================================================================

/// EXT superblock magic number
const EXT_SUPER_MAGIC: u16 = 0xEF53;

/// Superblock offset from partition start
const SUPERBLOCK_OFFSET: u64 = 1024;

/// Superblock size
const SUPERBLOCK_SIZE: usize = 1024;

/// Root directory inode number
const ROOT_INODE: u32 = 2;

// Feature flags for detecting ext3/ext4
const EXT3_FEATURE_COMPAT_HAS_JOURNAL: u32 = 0x0004;
const EXT4_FEATURE_INCOMPAT_EXTENTS: u32 = 0x0040;
const EXT4_FEATURE_INCOMPAT_64BIT: u32 = 0x0080;
const EXT4_FEATURE_INCOMPAT_FLEX_BG: u32 = 0x0200;

// Inode mode flags
const S_IFMT: u16 = 0o170000;  // File type mask
const S_IFDIR: u16 = 0o040000; // Directory
const S_IFREG: u16 = 0o100000; // Regular file
const S_IFLNK: u16 = 0o120000; // Symbolic link

// Directory entry file types
const EXT_FT_UNKNOWN: u8 = 0;
const EXT_FT_REG_FILE: u8 = 1;
const EXT_FT_DIR: u8 = 2;
const EXT_FT_SYMLINK: u8 = 7;

// =============================================================================
// On-Disk Structures
// =============================================================================

/// EXT Superblock (partial - key fields only)
#[derive(Debug, Clone)]
struct ExtSuperblock {
    /// Total inode count
    inodes_count: u32,
    /// Total block count
    blocks_count: u64,
    /// Free block count
    free_blocks_count: u64,
    /// Free inode count
    free_inodes_count: u32,
    /// First data block (0 for large blocks, 1 for 1K blocks)
    first_data_block: u32,
    /// Block size = 1024 << log_block_size
    log_block_size: u32,
    /// Blocks per group
    blocks_per_group: u32,
    /// Inodes per group
    inodes_per_group: u32,
    /// Magic number (should be 0xEF53)
    magic: u16,
    /// Filesystem state
    state: u16,
    /// Revision level
    rev_level: u32,
    /// Inode size (for rev >= 1)
    inode_size: u16,
    /// Block group number of this superblock
    block_group_nr: u16,
    /// Compatible feature set
    feature_compat: u32,
    /// Incompatible feature set
    feature_incompat: u32,
    /// Read-only compatible feature set
    feature_ro_compat: u32,
    /// Volume label
    volume_name: String,
    /// Descriptor size (for 64-bit)
    desc_size: u16,
}

/// Block Group Descriptor
#[derive(Debug, Clone)]
struct ExtBlockGroupDesc {
    /// Block bitmap block
    block_bitmap: u64,
    /// Inode bitmap block
    inode_bitmap: u64,
    /// Inode table start block
    inode_table: u64,
    /// Free blocks count
    free_blocks_count: u32,
    /// Free inodes count
    free_inodes_count: u32,
    /// Used directories count
    used_dirs_count: u32,
}

/// Inode structure
#[derive(Debug, Clone)]
struct ExtInode {
    /// File mode (type and permissions)
    mode: u16,
    /// Owner UID
    uid: u16,
    /// Size in bytes (lower 32 bits)
    size_lo: u32,
    /// Access time
    atime: u32,
    /// Creation time
    ctime: u32,
    /// Modification time
    mtime: u32,
    /// Deletion time
    dtime: u32,
    /// Group ID
    gid: u16,
    /// Link count
    links_count: u16,
    /// Block count (in 512-byte units)
    blocks: u32,
    /// File flags
    flags: u32,
    /// Block pointers (12 direct + 3 indirect)
    block: [u32; 15],
    /// Size high (for regular files in rev 1)
    size_hi: u32,
}

impl ExtInode {
    fn size(&self) -> u64 {
        ((self.size_hi as u64) << 32) | (self.size_lo as u64)
    }
    
    fn is_directory(&self) -> bool {
        (self.mode & S_IFMT) == S_IFDIR
    }
    
    fn is_regular_file(&self) -> bool {
        (self.mode & S_IFMT) == S_IFREG
    }
    
    fn is_symlink(&self) -> bool {
        (self.mode & S_IFMT) == S_IFLNK
    }
}

/// Directory entry
#[derive(Debug, Clone)]
struct ExtDirEntry {
    /// Inode number
    inode: u32,
    /// Record length
    rec_len: u16,
    /// Name length
    name_len: u8,
    /// File type
    file_type: u8,
    /// File name
    name: String,
}

// =============================================================================
// EXT Driver
// =============================================================================

/// EXT2/3/4 filesystem driver
pub struct ExtDriver {
    /// Filesystem info
    info: FilesystemInfo,
    /// Block device
    device: Arc<dyn SeekableBlockDevice>,
    /// Partition offset
    offset: u64,
    /// Superblock
    superblock: ExtSuperblock,
    /// Block size in bytes
    block_size: u32,
    /// Inode size
    inode_size: u16,
    /// Block group descriptors
    group_descs: Vec<ExtBlockGroupDesc>,
    /// Directory cache: inode -> entries
    dir_cache: RwLock<HashMap<u32, Vec<ExtDirEntry>>>,
}

impl ExtDriver {
    /// Create a new EXT driver
    pub fn new(
        device: Box<dyn SeekableBlockDevice>,
        offset: u64,
        size: u64,
    ) -> Result<Self, VfsError> {
        let device: Arc<dyn SeekableBlockDevice> = Arc::from(device);

        // Read superblock
        let mut sb_buf = vec![0u8; SUPERBLOCK_SIZE];
        device
            .read_at(offset + SUPERBLOCK_OFFSET, &mut sb_buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        let superblock = Self::parse_superblock(&sb_buf)?;

        // Validate magic
        if superblock.magic != EXT_SUPER_MAGIC {
            return Err(VfsError::Internal(format!(
                "Invalid EXT magic: 0x{:04X}",
                superblock.magic
            )));
        }

        let block_size = 1024u32 << superblock.log_block_size;
        let inode_size = if superblock.rev_level >= 1 {
            superblock.inode_size
        } else {
            128 // Default for rev 0
        };

        // Determine filesystem type
        let fs_type = Self::detect_ext_version(&superblock);

        tracing::debug!(
            "EXT superblock: blocks={}, inodes={}, block_size={}, inode_size={}, type={:?}",
            superblock.blocks_count,
            superblock.inodes_count,
            block_size,
            inode_size,
            fs_type
        );

        // Read block group descriptors
        let group_descs = Self::read_group_descriptors(&device, offset, &superblock, block_size)?;

        let total_size = superblock.blocks_count * block_size as u64;
        let free_space = superblock.free_blocks_count * block_size as u64;

        let label = if superblock.volume_name.is_empty() {
            None
        } else {
            Some(superblock.volume_name.clone())
        };

        let info = FilesystemInfo {
            fs_type,
            label,
            total_size: total_size.min(size),
            free_space: Some(free_space),
            cluster_size: block_size,
        };

        Ok(Self {
            info,
            device,
            offset,
            superblock,
            block_size,
            inode_size,
            group_descs,
            dir_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Parse superblock from buffer
    fn parse_superblock(buf: &[u8]) -> Result<ExtSuperblock, VfsError> {
        if buf.len() < SUPERBLOCK_SIZE {
            return Err(VfsError::IoError("Buffer too small for superblock".into()));
        }

        let inodes_count = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let blocks_count_lo = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let free_blocks_count_lo = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let free_inodes_count = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let first_data_block = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        let log_block_size = u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]);
        let blocks_per_group = u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]);
        let inodes_per_group = u32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]]);
        let magic = u16::from_le_bytes([buf[56], buf[57]]);
        let state = u16::from_le_bytes([buf[58], buf[59]]);
        let rev_level = u32::from_le_bytes([buf[76], buf[77], buf[78], buf[79]]);
        
        // Rev 1+ fields
        let inode_size = if rev_level >= 1 {
            u16::from_le_bytes([buf[88], buf[89]])
        } else {
            128
        };
        let block_group_nr = u16::from_le_bytes([buf[90], buf[91]]);
        
        let feature_compat = u32::from_le_bytes([buf[92], buf[93], buf[94], buf[95]]);
        let feature_incompat = u32::from_le_bytes([buf[96], buf[97], buf[98], buf[99]]);
        let feature_ro_compat = u32::from_le_bytes([buf[100], buf[101], buf[102], buf[103]]);

        // Volume name (16 bytes at offset 120)
        let volume_name = String::from_utf8_lossy(&buf[120..136])
            .trim_end_matches('\0')
            .to_string();

        // 64-bit descriptor size (if feature enabled)
        let desc_size = if (feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 {
            u16::from_le_bytes([buf[254], buf[255]]).max(32)
        } else {
            32
        };

        // High 32 bits for 64-bit mode
        let blocks_count_hi = if (feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 {
            u32::from_le_bytes([buf[336], buf[337], buf[338], buf[339]])
        } else {
            0
        };
        let free_blocks_hi = if (feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 {
            u32::from_le_bytes([buf[340], buf[341], buf[342], buf[343]])
        } else {
            0
        };

        let blocks_count = ((blocks_count_hi as u64) << 32) | (blocks_count_lo as u64);
        let free_blocks_count = ((free_blocks_hi as u64) << 32) | (free_blocks_count_lo as u64);

        Ok(ExtSuperblock {
            inodes_count,
            blocks_count,
            free_blocks_count,
            free_inodes_count,
            first_data_block,
            log_block_size,
            blocks_per_group,
            inodes_per_group,
            magic,
            state,
            rev_level,
            inode_size,
            block_group_nr,
            feature_compat,
            feature_incompat,
            feature_ro_compat,
            volume_name,
            desc_size,
        })
    }

    /// Detect EXT version based on feature flags
    fn detect_ext_version(sb: &ExtSuperblock) -> FilesystemType {
        if (sb.feature_incompat & EXT4_FEATURE_INCOMPAT_EXTENTS) != 0
            || (sb.feature_incompat & EXT4_FEATURE_INCOMPAT_FLEX_BG) != 0
        {
            FilesystemType::Ext4
        } else if (sb.feature_compat & EXT3_FEATURE_COMPAT_HAS_JOURNAL) != 0 {
            FilesystemType::Ext3
        } else {
            FilesystemType::Ext2
        }
    }

    /// Read block group descriptors
    fn read_group_descriptors(
        device: &Arc<dyn SeekableBlockDevice>,
        offset: u64,
        sb: &ExtSuperblock,
        block_size: u32,
    ) -> Result<Vec<ExtBlockGroupDesc>, VfsError> {
        // Validate superblock fields to prevent divide by zero
        if sb.blocks_per_group == 0 {
            return Err(VfsError::Internal("Invalid ext superblock: blocks_per_group is 0".to_string()));
        }
        if block_size == 0 {
            return Err(VfsError::Internal("Invalid ext superblock: block_size is 0".to_string()));
        }
        
        // Calculate number of block groups
        let num_groups = sb.blocks_count.div_ceil(sb.blocks_per_group as u64) as usize;

        // Block group descriptor table is in the block after superblock
        // For 1K block size, superblock is in block 1, so BGD is in block 2
        // For larger block sizes, superblock is in block 0, BGD is in block 1
        let bgd_block = if block_size == 1024 { 2 } else { 1 };
        let bgd_offset = offset + (bgd_block * block_size) as u64;

        let desc_size = sb.desc_size as usize;
        let mut buf = vec![0u8; num_groups * desc_size];
        device
            .read_at(bgd_offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        let is_64bit = (sb.feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0;
        let mut descs = Vec::with_capacity(num_groups);

        for i in 0..num_groups {
            let d = &buf[i * desc_size..(i + 1) * desc_size];
            
            let block_bitmap_lo = u32::from_le_bytes([d[0], d[1], d[2], d[3]]);
            let inode_bitmap_lo = u32::from_le_bytes([d[4], d[5], d[6], d[7]]);
            let inode_table_lo = u32::from_le_bytes([d[8], d[9], d[10], d[11]]);
            let free_blocks_count_lo = u16::from_le_bytes([d[12], d[13]]) as u32;
            let free_inodes_count_lo = u16::from_le_bytes([d[14], d[15]]) as u32;
            let used_dirs_count_lo = u16::from_le_bytes([d[16], d[17]]) as u32;

            let (block_bitmap, inode_bitmap, inode_table) = if is_64bit && desc_size >= 64 {
                let bb_hi = u32::from_le_bytes([d[32], d[33], d[34], d[35]]);
                let ib_hi = u32::from_le_bytes([d[36], d[37], d[38], d[39]]);
                let it_hi = u32::from_le_bytes([d[40], d[41], d[42], d[43]]);
                (
                    ((bb_hi as u64) << 32) | (block_bitmap_lo as u64),
                    ((ib_hi as u64) << 32) | (inode_bitmap_lo as u64),
                    ((it_hi as u64) << 32) | (inode_table_lo as u64),
                )
            } else {
                (
                    block_bitmap_lo as u64,
                    inode_bitmap_lo as u64,
                    inode_table_lo as u64,
                )
            };

            descs.push(ExtBlockGroupDesc {
                block_bitmap,
                inode_bitmap,
                inode_table,
                free_blocks_count: free_blocks_count_lo,
                free_inodes_count: free_inodes_count_lo,
                used_dirs_count: used_dirs_count_lo,
            });
        }

        tracing::debug!("Read {} block group descriptors", descs.len());
        Ok(descs)
    }

    /// Read an inode by number
    fn read_inode(&self, inode_num: u32) -> Result<ExtInode, VfsError> {
        if inode_num == 0 {
            return Err(VfsError::Internal("Invalid inode number 0".into()));
        }

        // Calculate which block group the inode is in
        let inode_index = inode_num - 1; // Inodes are 1-indexed
        let group = (inode_index / self.superblock.inodes_per_group) as usize;
        let index_in_group = inode_index % self.superblock.inodes_per_group;

        if group >= self.group_descs.len() {
            return Err(VfsError::Internal(format!(
                "Block group {} out of range",
                group
            )));
        }

        let inode_table_block = self.group_descs[group].inode_table;
        let inode_offset = self.offset
            + inode_table_block * self.block_size as u64
            + index_in_group as u64 * self.inode_size as u64;

        let mut buf = vec![0u8; self.inode_size as usize];
        self.device
            .read_at(inode_offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        Self::parse_inode(&buf)
    }

    /// Parse inode from buffer
    fn parse_inode(buf: &[u8]) -> Result<ExtInode, VfsError> {
        if buf.len() < 128 {
            return Err(VfsError::IoError("Buffer too small for inode".into()));
        }

        let mode = u16::from_le_bytes([buf[0], buf[1]]);
        let uid = u16::from_le_bytes([buf[2], buf[3]]);
        let size_lo = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let atime = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let ctime = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let mtime = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let dtime = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        let gid = u16::from_le_bytes([buf[24], buf[25]]);
        let links_count = u16::from_le_bytes([buf[26], buf[27]]);
        let blocks = u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);
        let flags = u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]);

        // Block pointers start at offset 40
        let mut block = [0u32; 15];
        for (i, block_ptr) in block.iter_mut().enumerate() {
            let off = 40 + i * 4;
            *block_ptr = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        }

        // Size high (for regular files)
        let size_hi = u32::from_le_bytes([buf[108], buf[109], buf[110], buf[111]]);

        Ok(ExtInode {
            mode,
            uid,
            size_lo,
            atime,
            ctime,
            mtime,
            dtime,
            gid,
            links_count,
            blocks,
            flags,
            block,
            size_hi,
        })
    }

    /// Read data blocks for an inode (direct blocks only for now)
    fn read_inode_data(&self, inode: &ExtInode, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let file_size = inode.size();
        if offset >= file_size {
            return Ok(Vec::new());
        }

        let actual_size = std::cmp::min(size as u64, file_size - offset) as usize;
        let mut result = vec![0u8; actual_size];
        let mut bytes_read = 0usize;
        let mut current_offset = offset;

        while bytes_read < actual_size {
            let block_index = (current_offset / self.block_size as u64) as usize;
            let offset_in_block = (current_offset % self.block_size as u64) as usize;
            let bytes_remaining = actual_size - bytes_read;
            let bytes_to_read = std::cmp::min(bytes_remaining, self.block_size as usize - offset_in_block);

            let block_num = self.get_block_number(inode, block_index)?;
            if block_num == 0 {
                // Sparse block (hole) - fill with zeros
                // Already zeroed in result
                bytes_read += bytes_to_read;
                current_offset += bytes_to_read as u64;
                continue;
            }

            let block_offset = self.offset + block_num as u64 * self.block_size as u64 + offset_in_block as u64;
            self.device
                .read_at(block_offset, &mut result[bytes_read..bytes_read + bytes_to_read])
                .map_err(|e| VfsError::IoError(e.to_string()))?;

            bytes_read += bytes_to_read;
            current_offset += bytes_to_read as u64;
        }

        Ok(result)
    }

    /// Get block number for a logical block index
    fn get_block_number(&self, inode: &ExtInode, index: usize) -> Result<u32, VfsError> {
        let ptrs_per_block = (self.block_size / 4) as usize;

        // Direct blocks (0-11)
        if index < 12 {
            return Ok(inode.block[index]);
        }

        // Single indirect (block 12)
        let index = index - 12;
        if index < ptrs_per_block {
            let indirect_block = inode.block[12];
            if indirect_block == 0 {
                return Ok(0);
            }
            return self.read_indirect_block(indirect_block, index);
        }

        // Double indirect (block 13)
        let index = index - ptrs_per_block;
        if index < ptrs_per_block * ptrs_per_block {
            let dind_block = inode.block[13];
            if dind_block == 0 {
                return Ok(0);
            }
            let ind_index = index / ptrs_per_block;
            let ptr_index = index % ptrs_per_block;
            let ind_block = self.read_indirect_block(dind_block, ind_index)?;
            if ind_block == 0 {
                return Ok(0);
            }
            return self.read_indirect_block(ind_block, ptr_index);
        }

        // Triple indirect (block 14) - very large files
        let index = index - ptrs_per_block * ptrs_per_block;
        let tind_block = inode.block[14];
        if tind_block == 0 {
            return Ok(0);
        }
        let dind_index = index / (ptrs_per_block * ptrs_per_block);
        let remaining = index % (ptrs_per_block * ptrs_per_block);
        let ind_index = remaining / ptrs_per_block;
        let ptr_index = remaining % ptrs_per_block;

        let dind_block = self.read_indirect_block(tind_block, dind_index)?;
        if dind_block == 0 {
            return Ok(0);
        }
        let ind_block = self.read_indirect_block(dind_block, ind_index)?;
        if ind_block == 0 {
            return Ok(0);
        }
        self.read_indirect_block(ind_block, ptr_index)
    }

    /// Read a block pointer from an indirect block
    fn read_indirect_block(&self, block_num: u32, index: usize) -> Result<u32, VfsError> {
        let block_offset = self.offset + block_num as u64 * self.block_size as u64;
        let ptr_offset = block_offset + (index * 4) as u64;
        
        let mut buf = [0u8; 4];
        self.device
            .read_at(ptr_offset, &mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        Ok(u32::from_le_bytes(buf))
    }

    /// Read directory entries from an inode
    fn read_directory(&self, inode_num: u32) -> Result<Vec<ExtDirEntry>, VfsError> {
        // Check cache first
        if let Some(entries) = self.dir_cache.read().get(&inode_num) {
            return Ok(entries.clone());
        }

        let inode = self.read_inode(inode_num)?;
        if !inode.is_directory() {
            return Err(VfsError::NotADirectory(format!("inode {}", inode_num)));
        }

        // Read all directory data
        let dir_size = inode.size() as usize;
        let data = self.read_inode_data(&inode, 0, dir_size)?;

        let mut entries = Vec::new();
        let mut offset = 0usize;

        while offset + 8 <= data.len() {
            let entry_inode = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            ]);
            let rec_len = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);
            let name_len = data[offset + 6];
            let file_type = data[offset + 7];

            if rec_len == 0 {
                break; // Prevent infinite loop
            }

            if entry_inode != 0 && name_len > 0 {
                let name_end = offset + 8 + name_len as usize;
                if name_end <= data.len() {
                    let name = String::from_utf8_lossy(&data[offset + 8..name_end]).to_string();
                    entries.push(ExtDirEntry {
                        inode: entry_inode,
                        rec_len,
                        name_len,
                        file_type,
                        name,
                    });
                }
            }

            offset += rec_len as usize;
        }

        // Update cache
        self.dir_cache.write().insert(inode_num, entries.clone());

        Ok(entries)
    }

    /// Resolve a path to an inode number
    fn resolve_path(&self, path: &str) -> Result<u32, VfsError> {
        let normalized = normalize_path(path);
        if normalized == "/" {
            return Ok(ROOT_INODE);
        }

        let parts: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current_inode = ROOT_INODE;

        for part in parts {
            let entries = self.read_directory(current_inode)?;
            let entry = entries
                .iter()
                .find(|e| e.name == part)
                .ok_or_else(|| VfsError::NotFound(format!("{} not found in directory", part)))?;
            current_inode = entry.inode;
        }

        Ok(current_inode)
    }
}

impl FilesystemDriver for ExtDriver {
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
                inode: ROOT_INODE as u64,
                ..Default::default()
            });
        }

        let inode_num = self.resolve_path(&normalized)?;
        let inode = self.read_inode(inode_num)?;

        Ok(FileAttr {
            size: inode.size(),
            is_directory: inode.is_directory(),
            permissions: (inode.mode & 0o7777),
            nlink: inode.links_count as u32,
            inode: inode_num as u64,
            modified: Some(inode.mtime as i64 * 1_000_000_000), // Convert seconds to nanoseconds
            created: Some(inode.ctime as i64 * 1_000_000_000),
            accessed: Some(inode.atime as i64 * 1_000_000_000),
            uid: inode.uid as u32,
            gid: inode.gid as u32,
        })
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);
        let inode_num = self.resolve_path(&normalized)?;
        
        let entries = self.read_directory(inode_num)?;
        
        let mut result = Vec::new();
        for entry in entries {
            // Skip . and ..
            if entry.name == "." || entry.name == ".." {
                continue;
            }

            let is_directory = match entry.file_type {
                EXT_FT_DIR => true,
                EXT_FT_REG_FILE | EXT_FT_SYMLINK => false,
                EXT_FT_UNKNOWN => {
                    // Need to read inode to determine type
                    if let Ok(inode) = self.read_inode(entry.inode) {
                        inode.is_directory()
                    } else {
                        false
                    }
                }
                _ => false,
            };

            // DT_DIR = 4, DT_REG = 8
            let file_type = if is_directory { 4u8 } else { 8u8 };

            result.push(DirEntry {
                name: entry.name,
                is_directory,
                inode: entry.inode as u64,
                file_type,
            });
        }

        Ok(result)
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);
        let inode_num = self.resolve_path(&normalized)?;
        let inode = self.read_inode(inode_num)?;

        if inode.is_directory() {
            return Err(VfsError::NotAFile(normalized));
        }

        self.read_inode_data(&inode, offset, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext_magic() {
        assert_eq!(EXT_SUPER_MAGIC, 0xEF53);
    }

    #[test]
    fn test_inode_type_detection() {
        let mut inode = ExtInode {
            mode: S_IFDIR | 0o755,
            uid: 0,
            size_lo: 0,
            atime: 0,
            ctime: 0,
            mtime: 0,
            dtime: 0,
            gid: 0,
            links_count: 2,
            blocks: 0,
            flags: 0,
            block: [0; 15],
            size_hi: 0,
        };
        assert!(inode.is_directory());
        assert!(!inode.is_regular_file());

        inode.mode = S_IFREG | 0o644;
        assert!(!inode.is_directory());
        assert!(inode.is_regular_file());
    }
}
