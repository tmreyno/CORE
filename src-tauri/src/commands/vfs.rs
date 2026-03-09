// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Virtual Filesystem commands for mounting and browsing disk images.
//!
//! ## VFS Handle Pool
//!
//! A global handle pool (`VFS_POOL`) caches opened VFS instances so that
//! `vfs_list_dir` and `vfs_read_file` do not re-open (and re-parse all
//! segment headers of) the container on every call. Handles are inserted
//! on first use and can be explicitly evicted with `vfs_close_container`.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use parking_lot::RwLock as PLRwLock;
use tracing::debug;

use crate::common::vfs::{DirEntry, FileAttr, VirtualFileSystem};
use crate::ewf;
use crate::raw;

// =============================================================================
// VFS Handle Pool
// =============================================================================

/// Maximum number of cached VFS handles before LRU eviction
const VFS_POOL_MAX_ENTRIES: usize = 32;

/// Type-erased VFS wrapper for the handle pool
enum PooledVfsKind {
    Ewf(ewf::vfs::EwfVfs),
    Raw(raw::vfs::RawVfs),
}

/// Cached VFS handle with directory/attribute metadata caches
struct PooledVfs {
    inner: PooledVfsKind,
    /// Container type tag for diagnostics / logging
    #[allow(dead_code)]
    container_type: &'static str,
    /// Cached file attributes (path → FileAttr)
    attr_cache: PLRwLock<HashMap<String, FileAttr>>,
    /// Cached directory listings (path → Vec<DirEntry>)
    dir_cache: PLRwLock<HashMap<String, Vec<DirEntry>>>,
    /// Monotonically increasing access counter for LRU eviction
    access_count: std::sync::atomic::AtomicU64,
}

impl PooledVfs {
    fn new_ewf(vfs: ewf::vfs::EwfVfs) -> Self {
        Self {
            inner: PooledVfsKind::Ewf(vfs),
            container_type: "e01",
            attr_cache: PLRwLock::new(HashMap::new()),
            dir_cache: PLRwLock::new(HashMap::new()),
            access_count: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn new_raw(vfs: raw::vfs::RawVfs) -> Self {
        Self {
            inner: PooledVfsKind::Raw(vfs),
            container_type: "raw",
            attr_cache: PLRwLock::new(HashMap::new()),
            dir_cache: PLRwLock::new(HashMap::new()),
            access_count: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn touch(&self) {
        self.access_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Cached getattr — returns from cache or queries VFS and caches result
    fn getattr_cached(&self, path: &str) -> Result<FileAttr, String> {
        let norm = crate::common::vfs::normalize_path(path);
        if let Some(attr) = self.attr_cache.read().get(&norm) {
            return Ok(attr.clone());
        }
        let attr = match &self.inner {
            PooledVfsKind::Ewf(v) => v.getattr(&norm),
            PooledVfsKind::Raw(v) => v.getattr(&norm),
        }
        .map_err(|e| format!("{:?}", e))?;
        self.attr_cache.write().insert(norm, attr.clone());
        Ok(attr)
    }

    /// Cached readdir — returns from cache or queries VFS and caches result
    fn readdir_cached(&self, path: &str) -> Result<Vec<DirEntry>, String> {
        let norm = crate::common::vfs::normalize_path(path);
        if let Some(entries) = self.dir_cache.read().get(&norm) {
            return Ok(entries.clone());
        }
        let entries = match &self.inner {
            PooledVfsKind::Ewf(v) => v.readdir(&norm),
            PooledVfsKind::Raw(v) => v.readdir(&norm),
        }
        .map_err(|e| format!("{:?}", e))?;
        self.dir_cache.write().insert(norm, entries.clone());
        Ok(entries)
    }

    /// Read file data (not cached — too large)
    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, String> {
        match &self.inner {
            PooledVfsKind::Ewf(v) => v.read(path, offset, size),
            PooledVfsKind::Raw(v) => v.read(path, offset, size),
        }
        .map_err(|e| format!("{:?}", e))
    }
}

/// Global VFS handle pool — avoids re-opening containers on every call
static VFS_POOL: LazyLock<PLRwLock<HashMap<String, Arc<PooledVfs>>>> =
    LazyLock::new(|| PLRwLock::new(HashMap::with_capacity(16)));

/// Get or open a VFS handle for the given container path
fn get_or_open_vfs(container_path: &str) -> Result<Arc<PooledVfs>, String> {
    // Fast path: check pool with read lock
    {
        let pool = VFS_POOL.read();
        if let Some(handle) = pool.get(container_path) {
            handle.touch();
            return Ok(Arc::clone(handle));
        }
    }

    // Slow path: open the container and insert into pool
    let container_type = if ewf::is_ewf(container_path).unwrap_or(false) {
        "e01"
    } else if raw::is_raw(container_path).unwrap_or(false) {
        "raw"
    } else {
        return Err(format!("Unsupported container type: {}", container_path));
    };

    debug!(
        "[vfs_pool] Opening {} container: {}",
        container_type, container_path
    );

    let pooled = if container_type == "e01" {
        let vfs = ewf::vfs::EwfVfs::open(container_path)
            .map_err(|e| format!("Failed to open E01: {:?}", e))?;
        PooledVfs::new_ewf(vfs)
    } else {
        let vfs = raw::vfs::RawVfs::open_filesystem(container_path)
            .or_else(|_| raw::vfs::RawVfs::open(container_path))
            .map_err(|e| format!("Failed to open raw: {:?}", e))?;
        PooledVfs::new_raw(vfs)
    };

    let handle = Arc::new(pooled);

    // Insert into pool, evicting LRU if at capacity
    {
        let mut pool = VFS_POOL.write();
        if pool.len() >= VFS_POOL_MAX_ENTRIES {
            evict_lru_vfs(&mut pool);
        }
        pool.insert(container_path.to_string(), Arc::clone(&handle));
    }

    debug!(
        "[vfs_pool] Cached handle for {} (pool size: {})",
        container_path,
        VFS_POOL.read().len()
    );

    Ok(handle)
}

/// Evict the least-recently-used VFS handle from the pool
fn evict_lru_vfs(pool: &mut HashMap<String, Arc<PooledVfs>>) {
    if let Some((key, _)) = pool
        .iter()
        .min_by_key(|(_, v)| v.access_count.load(std::sync::atomic::Ordering::Relaxed))
        .map(|(k, v)| (k.clone(), v.clone()))
    {
        debug!("[vfs_pool] Evicting LRU handle: {}", key);
        pool.remove(&key);
    }
}

/// VFS entry returned to the frontend
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsEntry {
    /// Entry name
    pub name: String,
    /// Full path within the VFS
    pub path: String,
    /// Is this a directory?
    pub is_dir: bool,
    /// File size (0 for directories)
    pub size: u64,
    /// File type hint (from extension or magic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
}

/// Partition information for mounted disk images
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsPartitionInfo {
    /// Partition number (1-based)
    pub number: u32,
    /// Mount name (e.g., "Partition1_NTFS")
    pub mount_name: String,
    /// Filesystem type (NTFS, FAT32, etc.)
    pub fs_type: String,
    /// Partition size in bytes
    pub size: u64,
    /// Start offset in the disk image
    pub start_offset: u64,
}

/// Information about a mounted disk image
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsMountInfo {
    /// Container path
    pub container_path: String,
    /// Container type (e01, raw, etc.)
    pub container_type: String,
    /// Total disk size
    pub disk_size: u64,
    /// Detected partitions
    pub partitions: Vec<VfsPartitionInfo>,
    /// Mount mode (physical or filesystem)
    pub mode: String,
}

/// Mount a disk image (E01/Raw) and return partition information.
///
/// This also pre-populates the VFS handle pool so that subsequent
/// `vfs_list_dir` / `vfs_read_file` calls reuse the opened handle.
#[tauri::command]
pub async fn vfs_mount_image(
    #[allow(non_snake_case)] containerPath: String,
) -> Result<VfsMountInfo, String> {
    debug!("[vfs_mount_image] Starting mount for: {}", containerPath);

    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type for VFS: {}", containerPath));
        };

        debug!("[vfs_mount_image] Container type: {}", container_type);

        // Try to mount with filesystem mode first
        let (partitions, mode, disk_size) = if container_type == "e01" {
            debug!("[vfs_mount_image] Opening E01 in filesystem mode...");
            match ewf::vfs::EwfVfs::open_filesystem(&containerPath) {
                Ok(vfs) => {
                    debug!("[vfs_mount_image] E01 opened successfully in filesystem mode");

                    // Get disk size from the VFS
                    let disk_size = vfs.disk_size().unwrap_or(0);
                    debug!("[vfs_mount_image] Disk size from vfs.disk_size(): {} bytes", disk_size);

                    // Get partitions from readdir
                    let root_entries = vfs.readdir("/").unwrap_or_default();
                    debug!("[vfs_mount_image] Root entries count: {}", root_entries.len());
                    for entry in &root_entries {
                        debug!("[vfs_mount_image] Root entry: {} (is_dir: {})", entry.name, entry.is_directory);
                    }

                    let parts: Vec<VfsPartitionInfo> = root_entries
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.is_directory)
                        .map(|(idx, e)| {
                            // Extract fs type from name (e.g., "Partition1_NTFS" -> "NTFS")
                            let fs_type = e.name.split('_').next_back().unwrap_or("Unknown").to_string();
                            // Get partition size from the mounted partition info
                            let part_size = vfs.get_partition_size(&e.name).unwrap_or(0);
                            debug!("[vfs_mount_image] Partition {}: {} size={}", idx + 1, e.name, part_size);
                            VfsPartitionInfo {
                                number: (idx + 1) as u32,
                                mount_name: e.name.clone(),
                                fs_type,
                                size: part_size,
                                start_offset: 0,
                            }
                        })
                        .collect();

                    let mode = if parts.is_empty() { "physical" } else { "filesystem" };
                    debug!("[vfs_mount_image] Mode: {}, Partitions: {}, Disk size: {}", mode, parts.len(), disk_size);

                    // Store in VFS pool for reuse by vfs_list_dir / vfs_read_file
                    {
                        let mut pool = VFS_POOL.write();
                        if pool.len() >= VFS_POOL_MAX_ENTRIES {
                            evict_lru_vfs(&mut pool);
                        }
                        pool.insert(containerPath.clone(), Arc::new(PooledVfs::new_ewf(vfs)));
                    }

                    (parts, mode.to_string(), disk_size)
                }
                Err(e) => {
                    debug!("[vfs_mount_image] Filesystem mode failed: {:?}, falling back to physical mode", e);
                    // Fall back to physical mode
                    match ewf::vfs::EwfVfs::open_physical(&containerPath) {
                        Ok(vfs) => {
                            debug!("[vfs_mount_image] Physical mode opened");
                            // Get disk size from disk_size() method
                            let disk_size = vfs.disk_size().unwrap_or(0);
                            debug!("[vfs_mount_image] Physical mode disk size: {} bytes", disk_size);

                            // Store in VFS pool
                            {
                                let mut pool = VFS_POOL.write();
                                if pool.len() >= VFS_POOL_MAX_ENTRIES {
                                    evict_lru_vfs(&mut pool);
                                }
                                pool.insert(containerPath.clone(), Arc::new(PooledVfs::new_ewf(vfs)));
                            }

                            (Vec::new(), "physical".to_string(), disk_size)
                        }
                        Err(e) => return Err(format!("Failed to mount E01: {:?}", e)),
                    }
                }
            }
        } else {
            // Raw image
            match raw::vfs::RawVfs::open_filesystem(&containerPath) {
                Ok(vfs) => {
                    let parts: Vec<VfsPartitionInfo> = vfs.readdir("/")
                        .unwrap_or_default()
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.is_directory)
                        .map(|(idx, e)| {
                            let fs_type = e.name.split('_').next_back().unwrap_or("Unknown").to_string();
                            VfsPartitionInfo {
                                number: (idx + 1) as u32,
                                mount_name: e.name.clone(),
                                fs_type,
                                size: 0,
                                start_offset: 0,
                            }
                        })
                        .collect();

                    let mode = if parts.is_empty() { "physical" } else { "filesystem" };
                    let disk_size = vfs.getattr("/")
                        .map(|a| a.size)
                        .unwrap_or(0);

                    // Store in VFS pool
                    {
                        let mut pool = VFS_POOL.write();
                        if pool.len() >= VFS_POOL_MAX_ENTRIES {
                            evict_lru_vfs(&mut pool);
                        }
                        pool.insert(containerPath.clone(), Arc::new(PooledVfs::new_raw(vfs)));
                    }

                    (parts, mode.to_string(), disk_size)
                }
                Err(_) => {
                    match raw::vfs::RawVfs::open(&containerPath) {
                        Ok(vfs) => {
                            // Store in VFS pool
                            {
                                let mut pool = VFS_POOL.write();
                                if pool.len() >= VFS_POOL_MAX_ENTRIES {
                                    evict_lru_vfs(&mut pool);
                                }
                                pool.insert(containerPath.clone(), Arc::new(PooledVfs::new_raw(vfs)));
                            }
                            (Vec::new(), "physical".to_string(), 0)
                        }
                        Err(e) => return Err(format!("Failed to mount raw image: {:?}", e)),
                    }
                }
            }
        };

        Ok(VfsMountInfo {
            container_path: containerPath,
            container_type: container_type.to_string(),
            disk_size,
            partitions,
            mode,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// List directory contents in a mounted VFS.
///
/// Uses the VFS handle pool to avoid re-opening the container on every call.
#[tauri::command]
pub async fn vfs_list_dir(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] dirPath: String,
) -> Result<Vec<VfsEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let handle = get_or_open_vfs(&containerPath)?;

        let dir_entries = handle.readdir_cached(&dirPath)?;

        let entries: Vec<VfsEntry> = dir_entries
            .into_iter()
            .map(|e| {
                let full_path = if dirPath == "/" {
                    format!("/{}", e.name)
                } else {
                    format!("{}/{}", dirPath, e.name)
                };

                let size = handle
                    .getattr_cached(&full_path)
                    .map(|a| a.size)
                    .unwrap_or(0);

                VfsEntry {
                    name: e.name,
                    path: full_path,
                    is_dir: e.is_directory,
                    size,
                    file_type: None,
                }
            })
            .collect();

        Ok(entries)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read file content from a mounted VFS.
///
/// Uses the VFS handle pool to avoid re-opening the container on every call.
#[tauri::command]
pub async fn vfs_read_file(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] filePath: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let handle = get_or_open_vfs(&containerPath)?;
        handle.read(&filePath, offset, length)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Close a VFS handle and remove it from the pool.
///
/// Call this when the user closes a container tab or unloads evidence.
#[tauri::command]
pub async fn vfs_close_container(
    #[allow(non_snake_case)] containerPath: String,
) -> Result<(), String> {
    let removed = VFS_POOL.write().remove(&containerPath).is_some();
    debug!(
        "[vfs_close_container] {} (was_cached: {})",
        containerPath, removed
    );
    Ok(())
}

/// Clear all cached VFS handles. Used on project close or cache cleanup.
#[tauri::command]
pub async fn vfs_clear_pool() -> Result<usize, String> {
    let mut pool = VFS_POOL.write();
    let count = pool.len();
    pool.clear();
    debug!("[vfs_clear_pool] Cleared {} cached VFS handles", count);
    Ok(count)
}
