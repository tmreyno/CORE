// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Virtual Filesystem commands for mounting and browsing disk images.

use tracing::debug;

use crate::common::vfs::VirtualFileSystem;
use crate::ewf;
use crate::raw;

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

/// Mount a disk image (E01/Raw) and return partition information
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
                    (parts, mode.to_string(), disk_size)
                }
                Err(_) => {
                    match raw::vfs::RawVfs::open(&containerPath) {
                        Ok(_vfs) => (Vec::new(), "physical".to_string(), 0),
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

/// List directory contents in a mounted VFS
#[tauri::command]
pub async fn vfs_list_dir(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] dirPath: String,
) -> Result<Vec<VfsEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type: {}", containerPath));
        };

        let entries = if container_type == "e01" {
            let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                .map_err(|e| format!("Failed to open E01: {:?}", e))?;

            let dir_entries = vfs
                .readdir(&dirPath)
                .map_err(|e| format!("Failed to read directory: {:?}", e))?;

            dir_entries
                .into_iter()
                .map(|e| {
                    let full_path = if dirPath == "/" {
                        format!("/{}", e.name)
                    } else {
                        format!("{}/{}", dirPath, e.name)
                    };

                    let size = vfs.getattr(&full_path).map(|a| a.size).unwrap_or(0);

                    VfsEntry {
                        name: e.name,
                        path: full_path,
                        is_dir: e.is_directory,
                        size,
                        file_type: None,
                    }
                })
                .collect()
        } else {
            let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                .map_err(|e| format!("Failed to open raw: {:?}", e))?;

            let dir_entries = vfs
                .readdir(&dirPath)
                .map_err(|e| format!("Failed to read directory: {:?}", e))?;

            dir_entries
                .into_iter()
                .map(|e| {
                    let full_path = if dirPath == "/" {
                        format!("/{}", e.name)
                    } else {
                        format!("{}/{}", dirPath, e.name)
                    };

                    let size = vfs.getattr(&full_path).map(|a| a.size).unwrap_or(0);

                    VfsEntry {
                        name: e.name,
                        path: full_path,
                        is_dir: e.is_directory,
                        size,
                        file_type: None,
                    }
                })
                .collect()
        };

        Ok(entries)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read file content from a mounted VFS
#[tauri::command]
pub async fn vfs_read_file(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] filePath: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type: {}", containerPath));
        };

        if container_type == "e01" {
            let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                .map_err(|e| format!("Failed to open E01: {:?}", e))?;
            vfs.read(&filePath, offset, length)
                .map_err(|e| format!("Failed to read file: {:?}", e))
        } else {
            let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                .map_err(|e| format!("Failed to open raw: {:?}", e))?;
            vfs.read(&filePath, offset, length)
                .map_err(|e| format!("Failed to read file: {:?}", e))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
